//! Surface-owned Windows Shell visuals; failures are silent presentation fallbacks.
use std::{collections::{HashMap, VecDeque}, path::{Path, PathBuf}, sync::{Arc, Condvar, Mutex, MutexGuard, mpsc}, thread::JoinHandle};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use windows::{core::HSTRING, Win32::{Foundation::SIZE, Graphics::Gdi::{DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HGDIOBJ}, System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED}, UI::{Shell::{IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_ICONONLY, SIIGBF_RESIZETOFIT}, WindowsAndMessaging::IsWindowVisible}}};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate) enum SurfaceId { Shelf, ManageItems, EdgeRail }
impl SurfaceId {
    pub(crate) fn from_window_label(label: &str) -> Option<Self> {
        match label { "main" => Some(Self::Shelf), "manage-items" => Some(Self::ManageItems), "edge-rail" => Some(Self::EdgeRail), _ => None }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VisualRole { ImageThumbnail, ShellIcon }
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum VisualOutcome { Success(Vec<u8>), Fallback }
#[derive(Clone, Serialize)]
pub(crate) struct GenerationChangedPayload { pub(crate) surface: SurfaceId, pub(crate) generation: u64 }
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key { surface: SurfaceId, generation: u64, item: String, size: u32 }
struct Job { key: Key, path: PathBuf, role: VisualRole, responders: Vec<mpsc::Sender<VisualOutcome>>, overscan: bool }
#[derive(Default)]
struct Queue { generations: HashMap<SurfaceId,u64>, pending: VecDeque<Job>, active: HashMap<Key,Vec<mpsc::Sender<VisualOutcome>>>, stopping: bool }
struct Shared { queue: Mutex<Queue>, available: Condvar }
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_,T> { mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner) }
pub(crate) trait ShellVisualExtractor: Send + Sync + 'static {
    fn extract(&self, path: &Path, role: VisualRole, size: u32) -> VisualOutcome;
}
struct WindowsShellExtractor;
struct Bitmap(HBITMAP);
impl Drop for Bitmap { fn drop(&mut self) {
    // SAFETY: This guard exclusively owns the bitmap returned by GetImage.
    unsafe { let _ = DeleteObject(HGDIOBJ(self.0.0)); }
} }
impl ShellVisualExtractor for WindowsShellExtractor {
    fn extract(&self, path: &Path, role: VisualRole, size: u32) -> VisualOutcome {
        let name = HSTRING::from(path.as_os_str());
        // SAFETY: Workers have initialized COM; name owns a valid UTF-16 parsing name.
        let factory: IShellItemImageFactory = match unsafe { SHCreateItemFromParsingName(&name,None) } { Ok(value)=>value, Err(_)=>return VisualOutcome::Fallback };
        let flags = match role { VisualRole::ImageThumbnail=>SIIGBF_RESIZETOFIT, VisualRole::ShellIcon=>SIIGBF_ICONONLY | SIIGBF_RESIZETOFIT };
        // SAFETY: Request dimensions are validated and bounded at the command interface.
        let bitmap = match unsafe { factory.GetImage(SIZE{cx:size as i32,cy:size as i32},flags) } { Ok(value)=>Bitmap(value), Err(_)=>return VisualOutcome::Fallback };
        match encode_bitmap(bitmap.0,size) { Some(bytes)=>VisualOutcome::Success(bytes),None=>VisualOutcome::Fallback }
    }
}
fn encode_bitmap(bitmap: HBITMAP, bound:u32) -> Option<Vec<u8>> {
    let mut description = BITMAP::default();
    // SAFETY: Writable BITMAP buffer has the documented size; bitmap remains owned by its guard.
    if unsafe { GetObjectW(HGDIOBJ(bitmap.0),std::mem::size_of::<BITMAP>() as i32,Some((&mut description as *mut BITMAP).cast())) } == 0 { return None; }
    let width=u32::try_from(description.bmWidth).ok()?;
    let height=u32::try_from(description.bmHeight).ok()?;
    if width==0 || height==0 || width>bound || height>bound { return None; }
    let bytes=width.checked_mul(height)?.checked_mul(4)?;
    let mut info=BITMAPINFO{bmiHeader:BITMAPINFOHEADER{biSize:std::mem::size_of::<BITMAPINFOHEADER>() as u32,biWidth:width as i32,biHeight:height as i32,biPlanes:1,biBitCount:32,biCompression:BI_RGB.0,biSizeImage:bytes,..Default::default()},..Default::default()};
    let mut output=Vec::with_capacity(54+bytes as usize);
    output.extend_from_slice(b"BM"); output.extend_from_slice(&(54+bytes).to_le_bytes());output.extend_from_slice(&[0;4]);output.extend_from_slice(&54u32.to_le_bytes());
    output.extend_from_slice(&40u32.to_le_bytes());output.extend_from_slice(&width.to_le_bytes());output.extend_from_slice(&height.to_le_bytes());output.extend_from_slice(&1u16.to_le_bytes());output.extend_from_slice(&32u16.to_le_bytes());output.extend_from_slice(&0u32.to_le_bytes());output.extend_from_slice(&bytes.to_le_bytes());output.extend_from_slice(&[0;16]);
    output.resize(54+bytes as usize,0);
    // SAFETY: DC is paired with ReleaseDC; buffer fits the checked 32-bit DIB dimensions.
    let dc=unsafe{GetDC(None)};
    if dc.is_invalid(){return None;}
    let lines=unsafe{GetDIBits(dc,bitmap,0,height,Some(output[54..].as_mut_ptr().cast()),&mut info,DIB_RGB_COLORS)};
    unsafe{let _=ReleaseDC(None,dc);}
    if lines!=height as i32{return None;}
    Some(output)
}

pub(crate) struct ShellVisualService { shared:Arc<Shared>, workers:Vec<JoinHandle<()>> }
impl ShellVisualService {
    pub(crate) fn new() -> Self { Self::with_extractor(Arc::new(WindowsShellExtractor)) }
    fn with_extractor(extractor:Arc<dyn ShellVisualExtractor>) -> Self {
        let shared=Arc::new(Shared{queue:Mutex::new(Queue::default()),available:Condvar::new()});
        let mut workers=Vec::with_capacity(2);
        for number in 0..2 {
            let state=Arc::clone(&shared);let source=Arc::clone(&extractor);
            if let Ok(worker)=std::thread::Builder::new().name(format!("dropcove-shell-{number}")).spawn(move||{
                // SAFETY: Each dedicated worker owns one balanced STA apartment.
                if unsafe{CoInitializeEx(None,COINIT_APARTMENTTHREADED)}.is_err(){return;}
                loop {
                    let job={let mut queue=lock(&state.queue);
                        while queue.pending.is_empty() && !queue.stopping { queue=state.available.wait(queue).unwrap_or_else(std::sync::PoisonError::into_inner); }
                        if queue.stopping{break;}
                        let Some(job)=queue.pending.pop_front() else{continue;};
                        queue.active.insert(job.key.clone(),job.responders.clone());job
                    };
                    let outcome=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||source.extract(&job.path,job.role,job.key.size))).unwrap_or(VisualOutcome::Fallback);
                    let mut queue=lock(&state.queue);
                    if let Some(responders)=queue.active.remove(&job.key){
                        let valid=queue.generations.get(&job.key.surface).copied().unwrap_or(0)==job.key.generation;
                        let result=if valid{outcome}else{VisualOutcome::Fallback};
                        for responder in responders{let _=responder.send(result.clone());}
                    }
                }
                unsafe{CoUninitialize();}
            }) {workers.push(worker);}
        }
        Self{shared,workers}
    }
    pub(crate) fn current_generation(&self,surface:SurfaceId)->u64{lock(&self.shared.queue).generations.get(&surface).copied().unwrap_or(0)}
    pub(crate) fn advance_generation(&self,surface:SurfaceId)->u64{
        let mut queue=lock(&self.shared.queue);let generation=queue.generations.entry(surface).or_insert(0);*generation=generation.wrapping_add(1);let next=*generation;
        queue.pending.retain(|job|{if job.key.surface==surface{for responder in &job.responders{let _=responder.send(VisualOutcome::Fallback);}false}else{true}});
        queue.active.retain(|key,responders|{if key.surface==surface{for responder in responders{let _=responder.send(VisualOutcome::Fallback);}false}else{true}});
        next
    }
    fn cancel(&self,key:&Key){
        let mut queue=lock(&self.shared.queue);
        queue.pending.retain(|job|{if &job.key==key{for responder in &job.responders{let _=responder.send(VisualOutcome::Fallback);}false}else{true}});
        if let Some(responders)=queue.active.remove(key){for responder in responders{let _=responder.send(VisualOutcome::Fallback);}}
    }
    fn request(&self,key:Key,path:PathBuf,role:VisualRole,overscan:bool)->VisualOutcome{
        let (sender,receiver)=mpsc::channel();
        {let mut queue=lock(&self.shared.queue);
            if queue.stopping || self.workers.iter().all(JoinHandle::is_finished) || queue.generations.get(&key.surface).copied().unwrap_or(0)!=key.generation{return VisualOutcome::Fallback;}
            if let Some(job)=queue.pending.iter_mut().find(|job|job.key==key){job.responders.push(sender);}
            else if let Some(responders)=queue.active.get_mut(&key){responders.push(sender);}
            else {
                if queue.pending.len()>=128{return VisualOutcome::Fallback;}
                let job=Job{key,path,role,responders:vec![sender],overscan};
                if overscan{queue.pending.push_back(job);}else{let index=queue.pending.iter().position(|job|job.overscan).unwrap_or(queue.pending.len());queue.pending.insert(index,job);}
                self.shared.available.notify_one();
            }
        }
        receiver.recv().unwrap_or(VisualOutcome::Fallback)
    }
}
impl Drop for ShellVisualService {fn drop(&mut self){
    {let mut queue=lock(&self.shared.queue);queue.stopping=true;queue.pending.clear();queue.active.clear();}
    self.shared.available.notify_all();for worker in self.workers.drain(..){let _=worker.join();}
}}
pub(crate) struct ShellVisualState(pub(crate) Arc<ShellVisualService>);
impl ShellVisualState{pub(crate) fn new()->Self{Self(Arc::new(ShellVisualService::new()))}}
fn visual_size(size:Option<u32>,scale:Option<f64>)->Option<u32>{let size=size.unwrap_or(32);let scale=scale.unwrap_or(1.0);if !matches!(size,24|32)||!scale.is_finite()||!(0.5..=8.0).contains(&scale){return None;}Some((size as f64*scale).round() as u32)}
#[tauri::command]
pub(crate) async fn get_shelf_visual(window:tauri::WebviewWindow,app:tauri::AppHandle,shelf_state:tauri::State<'_,crate::shelf::ShelfState>,visual_state:tauri::State<'_,ShellVisualState>,item_id:String,generation:u64,size:Option<u32>,scale_factor:Option<f64>,overscan:Option<bool>)->Result<tauri::ipc::Response,String>{
    let empty=||tauri::ipc::Response::new(Vec::new());
    let Some(surface)=SurfaceId::from_window_label(window.label()) else{return Ok(empty());};
    let Some(size)=visual_size(size,scale_factor) else{return Ok(empty());};
    let hwnd=window.hwnd().map_err(|error|error.to_string())?;
    // SAFETY: Tauri owns the caller HWND for the command's lifetime.
    if !unsafe{IsWindowVisible(hwnd)}.as_bool(){return Ok(empty());}
    if surface==SurfaceId::EdgeRail && !app.state::<crate::resident::ResidentState>().rail_presentation()?.expanded{return Ok(empty());}
    let service=Arc::clone(&visual_state.0);let state=shelf_state.inner().clone();
    let outcome=tauri::async_runtime::spawn_blocking(move||{
        let target=state.item_visual_target(&item_id).ok().flatten();
        let Some((path,folder))=target else{return VisualOutcome::Fallback;};
        if surface==SurfaceId::ManageItems {
            let Ok(Some(batch_id))=crate::manage_items::active_batch(&app) else{return VisualOutcome::Fallback;};
            let Ok(batch)=state.management_batch(&batch_id) else{return VisualOutcome::Fallback;};
            if !batch.items.iter().any(|item|item.id==item_id){return VisualOutcome::Fallback;}
        }
        let path=PathBuf::from(path);
        let image=!folder && path.extension().and_then(|ext|ext.to_str()).is_some_and(|ext|["png","jpg","jpeg","bmp","gif","webp","ico","tiff","tif","avif"].iter().any(|format|ext.eq_ignore_ascii_case(format)));
        service.request(Key{surface,generation,item:item_id,size},path,if image{VisualRole::ImageThumbnail}else{VisualRole::ShellIcon},overscan.unwrap_or(false))
    }).await.map_err(|error|error.to_string())?;
    Ok(match outcome{VisualOutcome::Success(bytes)=>tauri::ipc::Response::new(bytes),VisualOutcome::Fallback=>empty()})
}
#[tauri::command]
pub(crate) fn get_surface_generation(window:tauri::WebviewWindow,visual_state:tauri::State<'_,ShellVisualState>)->Result<u64,String>{let surface=SurfaceId::from_window_label(window.label()).ok_or("unauthorized visual caller")?;Ok(visual_state.0.current_generation(surface))}
#[tauri::command]
pub(crate) fn invalidate_surface_visuals(window:tauri::WebviewWindow,app:tauri::AppHandle,visual_state:tauri::State<'_,ShellVisualState>)->Result<u64,String>{let surface=SurfaceId::from_window_label(window.label()).ok_or("unauthorized visual caller")?;let generation=visual_state.0.advance_generation(surface);let _=app.emit("visual-generation-changed",GenerationChangedPayload{surface,generation});Ok(generation)}
#[tauri::command]
pub(crate) fn cancel_shelf_visual(window:tauri::WebviewWindow,visual_state:tauri::State<'_,ShellVisualState>,item_id:String,generation:u64,size:Option<u32>,scale_factor:Option<f64>)->Result<(),String>{let surface=SurfaceId::from_window_label(window.label()).ok_or("unauthorized visual caller")?;if let Some(size)=visual_size(size,scale_factor){visual_state.0.cancel(&Key{surface,generation,item:item_id,size});}Ok(())}

#[cfg(test)]
mod tests {
    use super::*;
    struct Held { entered:mpsc::Sender<()>, release:Mutex<mpsc::Receiver<()>>, calls:std::sync::atomic::AtomicUsize }
    impl ShellVisualExtractor for Held {fn extract(&self,_:&Path,_:VisualRole,_:u32)->VisualOutcome{self.calls.fetch_add(1,std::sync::atomic::Ordering::SeqCst);let _=self.entered.send(());let _=lock(&self.release).recv();VisualOutcome::Success(vec![1,2,3])}}
    #[test]
    fn closing_surface_cancels_in_flight_result_before_extraction_finishes(){
        let (entered_tx,entered)=mpsc::channel();let (release,release_rx)=mpsc::channel();
        let source=Arc::new(Held{entered:entered_tx,release:Mutex::new(release_rx),calls:std::sync::atomic::AtomicUsize::new(0)});
        let service=Arc::new(ShellVisualService::with_extractor(source));let worker=Arc::clone(&service);
        let request=std::thread::spawn(move||worker.request(Key{surface:SurfaceId::Shelf,generation:0,item:"a".into(),size:32},PathBuf::from("a"),VisualRole::ShellIcon,false));
        entered.recv_timeout(std::time::Duration::from_secs(3)).unwrap();
        service.advance_generation(SurfaceId::Shelf);
        assert_eq!(request.join().unwrap(),VisualOutcome::Fallback);
        release.send(()).unwrap();
    }
    #[test]
    fn removed_item_drops_queued_work_without_calling_extractor(){
        let (entered_tx,entered)=mpsc::channel();let (release,release_rx)=mpsc::channel();
        let source=Arc::new(Held{entered:entered_tx,release:Mutex::new(release_rx),calls:std::sync::atomic::AtomicUsize::new(0)});
        let service=Arc::new(ShellVisualService::with_extractor(Arc::clone(&source) as Arc<dyn ShellVisualExtractor>));
        let mut requests=Vec::new();
        for item in ["a","b"]{let clone=Arc::clone(&service);let item=item.to_string();requests.push(std::thread::spawn(move||clone.request(Key{surface:SurfaceId::Shelf,generation:0,item:item.clone(),size:32},PathBuf::from(item),VisualRole::ShellIcon,false)));entered.recv_timeout(std::time::Duration::from_secs(3)).unwrap();}
        let key=Key{surface:SurfaceId::Shelf,generation:0,item:"queued".into(),size:32};
        let (sender,receiver)=mpsc::channel();lock(&service.shared.queue).pending.push_back(Job{key:key.clone(),path:PathBuf::from("queued"),role:VisualRole::ShellIcon,responders:vec![sender],overscan:true});
        service.cancel(&key);assert_eq!(receiver.recv().unwrap(),VisualOutcome::Fallback);
        release.send(()).unwrap();release.send(()).unwrap();for request in requests{let _=request.join().unwrap();}
        assert_eq!(source.calls.load(std::sync::atomic::Ordering::SeqCst),2);
    }
}
