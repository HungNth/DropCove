//! Dedicated low-level mouse hook, bounded capture queue, and Shake detector.

use std::{
    cell::RefCell,
    collections::VecDeque,
    sync::{mpsc, Mutex, MutexGuard},
    thread::{self, JoinHandle},
};

use tauri::{AppHandle, Manager, WebviewWindow};
use windows::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
    UI::{
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{
            CallNextHookEx, DispatchMessageW, GetMessageW, GetWindowThreadProcessId, PeekMessageW,
            PostMessageW, PostThreadMessageW, SetWindowsHookExW, TranslateMessage,
            UnhookWindowsHookEx, HHOOK, LLMHF_LOWER_IL_INJECTED, MSG, MSLLHOOKSTRUCT, PM_NOREMOVE,
            WH_MOUSE_LL, WM_APP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCDESTROY,
        },
    },
};

use crate::settings_store::{Settings, ShakeSensitivity};

const SUBCLASS_ID: usize = 0xDC11;
const PROCESS_EVENTS: u32 = WM_APP + 0x31;
const STOP_HOOK: u32 = WM_APP + 0x33;
const SHAKE_TRIGGERED: u32 = WM_APP + 0x34;
const LEFT_RELEASED: u32 = WM_APP + 0x35;
const HOOK_FAILED: u32 = WM_APP + 0x36;
const EVENT_CAPACITY: usize = 256;
const DETECTION_WINDOW_MS: u32 = 600;
const COOLDOWN_MS: u32 = 500;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PointerKind {
    Move,
    LeftDown,
    LeftUp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PointerEvent {
    kind: PointerKind,
    x: i32,
    y: i32,
    time_ms: u32,
}

impl PointerEvent {
    fn point(self) -> (i32, i32) {
        (self.x, self.y)
    }
}

struct EventQueue {
    events: VecDeque<PointerEvent>,
    capacity: usize,
    left_down: bool,
    overflow_button_state: Option<bool>,
}

impl EventQueue {
    fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
            left_down: false,
            overflow_button_state: None,
        }
    }

    fn push(&mut self, event: PointerEvent) {
        match event.kind {
            PointerKind::LeftDown => self.left_down = true,
            PointerKind::LeftUp => self.left_down = false,
            PointerKind::Move => {}
        }
        if self.events.len() == self.capacity {
            self.events.clear();
            self.overflow_button_state = Some(self.left_down);
        }
        self.events.push_back(event);
    }

    fn take_overflow_button_state(&mut self) -> Option<bool> {
        self.overflow_button_state.take()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Axis {
    X,
    Y,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DetectorAction {
    Trigger,
    Release,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SensitivityProfile {
    minimum_movement: u32,
    required_reversals: u8,
}

impl ShakeSensitivity {
    fn profile(self) -> SensitivityProfile {
        match self {
            Self::Low => SensitivityProfile {
                minimum_movement: 24,
                required_reversals: 4,
            },
            Self::Normal => SensitivityProfile {
                minimum_movement: 10,
                required_reversals: 2,
            },
            Self::High => SensitivityProfile {
                minimum_movement: 8,
                required_reversals: 2,
            },
        }
    }
}

struct ShakeDetector {
    sensitivity: ShakeSensitivity,
    left_down: bool,
    baseline: Option<(i32, i32)>,
    window_started_at: Option<u32>,
    axis: Option<Axis>,
    direction: i8,
    reversals: u8,
    last_trigger_at: Option<u32>,
}

impl ShakeDetector {
    fn new(sensitivity: ShakeSensitivity) -> Self {
        Self {
            sensitivity,
            left_down: false,
            baseline: None,
            window_started_at: None,
            axis: None,
            direction: 0,
            reversals: 0,
            last_trigger_at: None,
        }
    }

    fn reset_after_overflow(&mut self, left_down: bool) {
        self.left_down = left_down;
        self.reset_sequence(None);
    }

    fn reset_sequence(&mut self, baseline: Option<(i32, i32)>) {
        self.baseline = baseline;
        self.window_started_at = None;
        self.axis = None;
        self.direction = 0;
        self.reversals = 0;
    }

    fn process(&mut self, event: PointerEvent) -> Option<DetectorAction> {
        match event.kind {
            PointerKind::LeftDown => {
                self.left_down = true;
                self.reset_sequence(Some(event.point()));
                None
            }
            PointerKind::LeftUp => {
                self.left_down = false;
                self.reset_sequence(None);
                Some(DetectorAction::Release)
            }
            PointerKind::Move => self.process_move(event),
        }
    }

    fn process_move(&mut self, event: PointerEvent) -> Option<DetectorAction> {
        if !self.left_down
            || self
                .last_trigger_at
                .is_some_and(|last| event.time_ms.wrapping_sub(last) < COOLDOWN_MS)
        {
            return None;
        }
        let Some((baseline_x, baseline_y)) = self.baseline else {
            self.baseline = Some(event.point());
            return None;
        };
        if self
            .window_started_at
            .is_some_and(|started| event.time_ms.wrapping_sub(started) > DETECTION_WINDOW_MS)
        {
            self.reset_sequence(Some(event.point()));
            return None;
        }

        let delta_x = event.x - baseline_x;
        let delta_y = event.y - baseline_y;
        let absolute_x = delta_x.unsigned_abs();
        let absolute_y = delta_y.unsigned_abs();
        let profile = self.sensitivity.profile();
        let (axis, movement, direction) = if absolute_x >= absolute_y {
            (Axis::X, absolute_x, delta_x.signum() as i8)
        } else {
            (Axis::Y, absolute_y, delta_y.signum() as i8)
        };
        if movement < profile.minimum_movement || direction == 0 {
            return None;
        }

        self.baseline = Some(event.point());
        match self.axis {
            None => {
                self.axis = Some(axis);
                self.direction = direction;
                self.window_started_at = Some(event.time_ms);
            }
            Some(previous_axis) if previous_axis != axis => {
                self.axis = Some(axis);
                self.direction = direction;
                self.reversals = 0;
                self.window_started_at = Some(event.time_ms);
            }
            Some(_) if self.direction != direction => {
                self.direction = direction;
                self.reversals = self.reversals.saturating_add(1);
                if self.reversals >= profile.required_reversals {
                    self.last_trigger_at = Some(event.time_ms);
                    self.reset_sequence(Some(event.point()));
                    return Some(DetectorAction::Trigger);
                }
            }
            Some(_) => {}
        }
        None
    }
}

struct HookContext {
    target: HWND,
    generation: usize,
    thread_id: u32,
    events: EventQueue,
    detector: ShakeDetector,
    processing_posted: bool,
}

thread_local! {
    static HOOK_CONTEXT: RefCell<Option<HookContext>> = const { RefCell::new(None) };
}

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kind = match wparam.0 as u32 {
            WM_MOUSEMOVE => Some(PointerKind::Move),
            WM_LBUTTONDOWN => Some(PointerKind::LeftDown),
            WM_LBUTTONUP => Some(PointerKind::LeftUp),
            _ => None,
        };
        if let Some(kind) = kind {
            // SAFETY: For non-negative low-level hook codes Windows provides a live
            // MSLLHOOKSTRUCT pointer for this callback only; all fields are copied now.
            let sample = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
            // Same-integrity injected events remain observable for accessibility and native
            // verification; lower-integrity injection cannot activate DropCove presentation.
            if sample.flags & LLMHF_LOWER_IL_INJECTED == 0 {
                HOOK_CONTEXT.with(|slot| {
                    let mut slot = slot.borrow_mut();
                    let Some(context) = slot.as_mut() else {
                        return;
                    };
                    context.events.push(PointerEvent {
                        kind,
                        x: sample.pt.x,
                        y: sample.pt.y,
                        time_ms: sample.time,
                    });
                    if !context.processing_posted {
                        // SAFETY: The hook thread created and owns this message queue.
                        if unsafe {
                            PostThreadMessageW(
                                context.thread_id,
                                PROCESS_EVENTS,
                                WPARAM(0),
                                LPARAM(0),
                            )
                        }
                        .is_ok()
                        {
                            context.processing_posted = true;
                        }
                    }
                });
            }
        }
    }
    // SAFETY: DropCove never suppresses input and always continues the hook chain.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn process_events() {
    HOOK_CONTEXT.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(context) = slot.as_mut() else {
            return;
        };
        context.processing_posted = false;
        if let Some(left_down) = context.events.take_overflow_button_state() {
            context.detector.reset_after_overflow(left_down);
            if !left_down {
                let _ = unsafe {
                    PostMessageW(
                        Some(context.target),
                        LEFT_RELEASED,
                        WPARAM(context.generation),
                        LPARAM(0),
                    )
                };
            }
        }
        while let Some(event) = context.events.events.pop_front() {
            let message = match context.detector.process(event) {
                Some(DetectorAction::Trigger) => SHAKE_TRIGGERED,
                Some(DetectorAction::Release) => LEFT_RELEASED,
                None => continue,
            };
            // SAFETY: Only scalar private messages are posted to the owned main HWND.
            let _ = unsafe {
                PostMessageW(
                    Some(context.target),
                    message,
                    WPARAM(context.generation),
                    LPARAM(0),
                )
            };
        }
    });
}

fn run_hook_thread(
    target: isize,
    generation: usize,
    sensitivity: ShakeSensitivity,
    ready: mpsc::SyncSender<Result<u32, String>>,
) -> Result<(), String> {
    let thread_id = unsafe { GetCurrentThreadId() };
    let mut bootstrap = MSG::default();
    // SAFETY: PeekMessage creates this thread's message queue before readiness is published.
    let _ = unsafe { PeekMessageW(&mut bootstrap, None, 0, 0, PM_NOREMOVE) };
    HOOK_CONTEXT.with(|slot| {
        *slot.borrow_mut() = Some(HookContext {
            target: HWND(target as *mut _),
            generation,
            thread_id,
            events: EventQueue::new(EVENT_CAPACITY),
            detector: ShakeDetector::new(sensitivity),
            processing_posted: false,
        });
    });

    // SAFETY: The static callback remains loaded for process lifetime; this dedicated thread
    // pumps messages until it has unhooked and acknowledged teardown.
    let hook = unsafe {
        let module = GetModuleHandleW(None).map_err(|error| error.to_string())?;
        SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), Some(HINSTANCE(module.0)), 0)
    };
    let hook = match hook {
        Ok(hook) => hook,
        Err(error) => {
            HOOK_CONTEXT.with(|slot| slot.borrow_mut().take());
            let message = format!("could not install Shake-to-open pointer hook: {error}");
            let _ = ready.send(Err(message.clone()));
            return Err(message);
        }
    };
    ready
        .send(Ok(thread_id))
        .map_err(|_| "Shake hook startup receiver disappeared".to_owned())?;

    let result = hook_message_loop(hook);
    HOOK_CONTEXT.with(|slot| slot.borrow_mut().take());
    if result.is_err() {
        // SAFETY: Generic failure notification carries no pointer and targets the live owner HWND.
        let _ = unsafe {
            PostMessageW(
                Some(HWND(target as *mut _)),
                HOOK_FAILED,
                WPARAM(generation),
                LPARAM(0),
            )
        };
    }
    result
}

fn hook_message_loop(hook: HHOOK) -> Result<(), String> {
    loop {
        let mut message = MSG::default();
        // SAFETY: This dedicated thread owns its message queue and writable MSG buffer.
        let status = unsafe { GetMessageW(&mut message, None, 0, 0) }.0;
        if status == -1 {
            // SAFETY: The hook belongs to this thread and must be released before returning.
            let _ = unsafe { UnhookWindowsHookEx(hook) };
            return Err("Shake hook message loop failed".into());
        }
        if status == 0 {
            // SAFETY: The hook belongs to this thread and must be released before returning.
            unsafe { UnhookWindowsHookEx(hook) }
                .map_err(|error| format!("could not remove Shake pointer hook: {error}"))?;
            return Ok(());
        }
        match message.message {
            PROCESS_EVENTS => process_events(),
            STOP_HOOK => {
                // SAFETY: The hook belongs to this thread; unhook completes before join returns.
                unsafe { UnhookWindowsHookEx(hook) }
                    .map_err(|error| format!("could not remove Shake pointer hook: {error}"))?;
                return Ok(());
            }
            _ => {
                // SAFETY: Standard dispatch for any system messages delivered to this pump.
                unsafe {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }
    }
}

struct HookThread {
    thread_id: u32,
    join: JoinHandle<Result<(), String>>,
}

struct Runtime {
    hwnd: isize,
    subclass_data: usize,
    hook: Option<HookThread>,
    sensitivity: ShakeSensitivity,
    generation: usize,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            hwnd: 0,
            subclass_data: 0,
            hook: None,
            sensitivity: ShakeSensitivity::Normal,
            generation: 0,
        }
    }
}

#[derive(Default)]
pub(crate) struct ShakeRuntimeState {
    runtime: Mutex<Runtime>,
}

impl ShakeRuntimeState {
    fn lock(&self) -> Result<MutexGuard<'_, Runtime>, String> {
        self.runtime
            .lock()
            .map_err(|_| "Shake runtime lock is poisoned".into())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeSnapshot {
    pub(crate) enabled: bool,
    pub(crate) sensitivity: ShakeSensitivity,
}

fn verify_owner_thread(runtime: &Runtime) -> Result<HWND, String> {
    if runtime.hwnd == 0 {
        return Err("Shake receiver is not initialized".into());
    }
    let hwnd = HWND(runtime.hwnd as *mut _);
    // SAFETY: The stored HWND belongs to this process and is live while the receiver is installed.
    if unsafe { GetWindowThreadProcessId(hwnd, None) != GetCurrentThreadId() } {
        return Err("Shake runtime operation must run on the Shelf window thread".into());
    }
    Ok(hwnd)
}

fn start_hook(
    hwnd: HWND,
    sensitivity: ShakeSensitivity,
    generation: usize,
) -> Result<HookThread, String> {
    let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
    let target = hwnd.0 as isize;
    let join = thread::Builder::new()
        .name("dropcove-shake-hook".into())
        .spawn(move || run_hook_thread(target, generation, sensitivity, ready_sender))
        .map_err(|error| format!("could not start Shake hook thread: {error}"))?;
    match ready_receiver.recv() {
        Ok(Ok(thread_id)) => Ok(HookThread { thread_id, join }),
        Ok(Err(error)) => {
            let _ = join.join();
            Err(error)
        }
        Err(_) => {
            let result = join.join();
            Err(format!(
                "Shake hook thread stopped during startup: {result:?}"
            ))
        }
    }
}

fn stop_hook(runtime: &mut Runtime) -> Result<(), String> {
    let Some(hook) = runtime.hook.as_ref() else {
        return Ok(());
    };
    if !hook.join.is_finished() {
        // SAFETY: The target thread published a live message queue before startup completed.
        unsafe { PostThreadMessageW(hook.thread_id, STOP_HOOK, WPARAM(0), LPARAM(0)) }
            .map_err(|error| format!("could not request Shake hook shutdown: {error}"))?;
    }
    let Some(hook) = runtime.hook.take() else {
        return Ok(());
    };
    hook.join
        .join()
        .map_err(|_| "Shake hook thread panicked during shutdown".to_owned())??;
    Ok(())
}

fn apply_runtime(
    app: &AppHandle,
    enabled: bool,
    sensitivity: ShakeSensitivity,
) -> Result<(), String> {
    // Native cancellation can re-enter the Shelf subclass, so never hold runtime state across it.
    if !enabled {
        crate::resident::cancel_shake(app)?;
    }
    let state = app.state::<ShakeRuntimeState>();
    let mut runtime = state.lock()?;
    let hwnd = verify_owner_thread(&runtime)?;
    if !enabled || runtime.sensitivity != sensitivity {
        stop_hook(&mut runtime)?;
        runtime.generation = runtime.generation.wrapping_add(1);
    }
    if enabled && runtime.hook.is_none() {
        runtime.generation = runtime.generation.wrapping_add(1);
        runtime.hook = Some(start_hook(hwnd, sensitivity, runtime.generation)?);
    }
    runtime.sensitivity = sensitivity;
    Ok(())
}

pub(crate) fn initialize(window: &WebviewWindow, settings: &Settings) -> Result<(), String> {
    let app = window.app_handle();
    let state = app.state::<ShakeRuntimeState>();
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    // SAFETY: Both APIs inspect scalar identities for the live managed Shelf HWND.
    if unsafe { GetWindowThreadProcessId(hwnd, None) != GetCurrentThreadId() } {
        return Err("Shake receiver must be initialized on the Shelf window thread".into());
    }
    {
        let mut runtime = state.lock()?;
        if runtime.hwnd != 0 {
            if runtime.hwnd != hwnd.0 as isize {
                return Err("Shake receiver is already initialized for another window".into());
            }
        } else {
            let data = Box::into_raw(Box::new(app.clone())) as usize;
            // SAFETY: Callback and boxed AppHandle remain live until joined teardown removes the subclass.
            if !unsafe { SetWindowSubclass(hwnd, Some(receiver), SUBCLASS_ID, data) }.as_bool() {
                // SAFETY: Subclass installation failed, so native code cannot retain this allocation.
                unsafe { drop(Box::from_raw(data as *mut AppHandle)) };
                return Err("could not install Shake UI receiver".into());
            }
            runtime.hwnd = hwnd.0 as isize;
            runtime.subclass_data = data;
        }
    }
    apply_runtime(app, settings.shake_enabled, settings.shake_sensitivity)
}

pub(crate) fn snapshot(app: &AppHandle) -> Result<RuntimeSnapshot, String> {
    let state = app.state::<ShakeRuntimeState>();
    let runtime = state.lock()?;
    Ok(RuntimeSnapshot {
        enabled: runtime.hook.is_some(),
        sensitivity: runtime.sensitivity,
    })
}

pub(crate) fn restore(app: &AppHandle, previous: &RuntimeSnapshot) -> Result<(), String> {
    apply_runtime(app, previous.enabled, previous.sensitivity)
}

pub(crate) fn description(app: &AppHandle) -> Result<String, String> {
    let snapshot = snapshot(app)?;
    Ok(format!(
        "enabled={} sensitivity={:?}",
        snapshot.enabled, snapshot.sensitivity
    ))
}

pub(crate) fn shutdown(app: &AppHandle) -> Result<(), String> {
    crate::resident::cancel_shake(app)?;
    let state = app.state::<ShakeRuntimeState>();
    let mut runtime = state.lock()?;
    if runtime.hwnd == 0 {
        return Ok(());
    }
    let hwnd = verify_owner_thread(&runtime)?;
    stop_hook(&mut runtime)?;
    // Join above guarantees no callback can reference receiver state after it is released.
    if !unsafe { RemoveWindowSubclass(hwnd, Some(receiver), SUBCLASS_ID) }.as_bool() {
        return Err("could not remove Shake UI receiver".into());
    }
    let data = runtime.subclass_data;
    runtime.hwnd = 0;
    runtime.subclass_data = 0;
    if data != 0 {
        // SAFETY: The subclass has been removed and the hook thread joined, leaving one owner.
        unsafe { drop(Box::from_raw(data as *mut AppHandle)) };
    }
    Ok(())
}

fn report(app: &AppHandle, error: String) {
    if let Err(warning) = crate::resident::publish_warning(app, &error) {
        eprintln!("{error}; warning delivery failed: {warning}");
    }
}

unsafe extern "system" fn receiver(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    data: usize,
) -> LRESULT {
    // SAFETY: subclass data is the live boxed AppHandle installed by initialize.
    let app = unsafe { (*(data as *const AppHandle)).clone() };
    if matches!(message, SHAKE_TRIGGERED | LEFT_RELEASED | HOOK_FAILED) {
        let state = app.state::<ShakeRuntimeState>();
        let current = state
            .lock()
            .map(|runtime| runtime.hook.is_some() && runtime.generation == wparam.0);
        if current != Ok(true) {
            return LRESULT(0);
        }
    }
    match message {
        SHAKE_TRIGGERED => {
            if let Err(error) = crate::resident::begin_shake(&app) {
                report(&app, error);
            }
            return LRESULT(0);
        }
        LEFT_RELEASED => {
            if let Err(error) = crate::resident::release_shake(&app) {
                report(&app, error);
            }
            return LRESULT(0);
        }
        HOOK_FAILED => {
            let state = app.state::<ShakeRuntimeState>();
            if let Ok(mut runtime) = state.lock() {
                let _ = stop_hook(&mut runtime);
                runtime.generation = runtime.generation.wrapping_add(1);
            }
            let _ = crate::resident::cancel_shake(&app);
            report(
                &app,
                "Shake-to-open pointer hook stopped; tray, hotkey, Shelf and Edge Rail remain available"
                    .into(),
            );
            return LRESULT(0);
        }
        WM_NCDESTROY => {
            if let Err(error) = shutdown(&app) {
                report(&app, error);
            }
            // shutdown normally removes the subclass and allocation. If teardown was partial,
            // leave native ownership intact rather than freeing data still referenced by Windows.
        }
        _ => {}
    }
    // SAFETY: Forward the original message to the remaining native subclass chain.
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: PointerKind, x: i32, y: i32, time_ms: u32) -> PointerEvent {
        PointerEvent {
            kind,
            x,
            y,
            time_ms,
        }
    }

    fn trigger(detector: &mut ShakeDetector, minimum: i32, start: u32) -> Option<DetectorAction> {
        detector.process(event(PointerKind::LeftDown, 0, 0, start));
        detector.process(event(PointerKind::Move, minimum, 0, start + 10));
        detector.process(event(PointerKind::Move, 0, 0, start + 20));
        detector.process(event(PointerKind::Move, minimum, 0, start + 30))
    }

    #[test]
    fn gestures_cover_profiles_left_button_requirement_and_axis_ties() {
        let mut detector = ShakeDetector::new(ShakeSensitivity::Normal);
        assert_eq!(detector.process(event(PointerKind::Move, 10, 0, 0)), None);
        detector.process(event(PointerKind::LeftDown, 0, 0, 0));
        detector.process(event(PointerKind::Move, 9, 9, 10));
        detector.process(event(PointerKind::Move, 10, 10, 20));
        assert_eq!(
            (detector.axis, detector.baseline),
            (Some(Axis::X), Some((10, 10)))
        );
        assert_eq!(
            trigger(&mut ShakeDetector::new(ShakeSensitivity::High), 8, 0),
            Some(DetectorAction::Trigger)
        );
        assert_eq!(
            trigger(&mut detector, 10, 30),
            Some(DetectorAction::Trigger)
        );
    }

    #[test]
    fn low_profile_requires_four_full_reversals() {
        let mut detector = ShakeDetector::new(ShakeSensitivity::Low);
        detector.process(event(PointerKind::LeftDown, 0, 0, 0));
        assert_eq!(detector.process(event(PointerKind::Move, 24, 0, 10)), None);
        assert_eq!(detector.process(event(PointerKind::Move, 0, 0, 20)), None);
        assert_eq!(detector.process(event(PointerKind::Move, 24, 0, 30)), None);
        assert_eq!(detector.process(event(PointerKind::Move, 0, 0, 40)), None);
        assert_eq!(
            detector.process(event(PointerKind::Move, 24, 0, 50)),
            Some(DetectorAction::Trigger)
        );
    }

    #[test]
    fn axis_change_resets_the_reversal_run() {
        let mut detector = ShakeDetector::new(ShakeSensitivity::Normal);
        detector.process(event(PointerKind::LeftDown, 0, 0, 0));
        detector.process(event(PointerKind::Move, 10, 0, 10));
        detector.process(event(PointerKind::Move, 10, 10, 20));
        detector.process(event(PointerKind::Move, 0, 10, 30));
        assert_eq!(detector.reversals, 0);
        assert_eq!(detector.axis, Some(Axis::X));
    }

    #[test]
    fn expired_window_and_cooldown_reject_stale_sequences() {
        let mut detector = ShakeDetector::new(ShakeSensitivity::Normal);
        detector.process(event(PointerKind::LeftDown, 0, 0, 0));
        detector.process(event(PointerKind::Move, 10, 0, 10));
        assert_eq!(detector.process(event(PointerKind::Move, 0, 0, 611)), None);
        assert_eq!(detector.reversals, 0);
        assert_eq!(
            trigger(&mut detector, 10, 700),
            Some(DetectorAction::Trigger)
        );
        detector.process(event(PointerKind::LeftUp, 10, 0, 731));
        assert_eq!(trigger(&mut detector, 10, 800), None);
        assert_eq!(
            trigger(&mut detector, 10, 1230),
            Some(DetectorAction::Trigger)
        );
    }

    #[test]
    fn queue_overflow_resets_incomplete_motion_but_preserves_button_state() {
        let mut queue = EventQueue::new(2);
        queue.push(event(PointerKind::LeftDown, 0, 0, 0));
        queue.push(event(PointerKind::Move, 10, 0, 10));
        queue.push(event(PointerKind::Move, 0, 0, 20));
        assert_eq!(queue.take_overflow_button_state(), Some(true));
        assert_eq!(queue.events.len(), 1);
        assert_eq!(
            queue.events.front().map(|event| event.kind),
            Some(PointerKind::Move)
        );
    }
}
