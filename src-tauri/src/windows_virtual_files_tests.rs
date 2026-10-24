use super::*;
use std::{io::Cursor, sync::atomic::{AtomicUsize, Ordering}};
use windows::{core::{implement, Ref, HRESULT, BOOL}, Win32::{Foundation::{DATA_S_SAMEFORMATETC, DV_E_FORMATETC, E_NOTIMPL, E_POINTER, S_OK}, System::{Com::{IDataObject_Impl, IAdviseSink, IEnumFORMATETC, IEnumSTATDATA, STGMEDIUM, STGMEDIUM_0}, Memory::{GlobalAlloc, GMEM_MOVEABLE}}}};

#[derive(Default)]
#[implement(IDataObject)]
struct VirtualSource {
    entries: Vec<(String, Vec<u8>)>,
    reads: std::sync::Arc<AtomicUsize>,
    stream_paths: Vec<PathBuf>,
    source_url_w: Option<String>,
    source_url_a: Option<String>,
    failing_contents_index: Option<i32>,
}

impl VirtualSource {
    fn payload(&self, request: &FORMATETC) -> Option<Vec<u8>> {
        if request.cfFormat == format(CFSTR_FILEDESCRIPTORW, -1, TYMED_HGLOBAL.0 as u32).cfFormat {
            let offset = offset_of!(FILEGROUPDESCRIPTORW, fgd);
            let mut bytes = vec![0u8; offset + self.entries.len() * size_of::<FILEDESCRIPTORW>()];
            bytes[..4].copy_from_slice(&(self.entries.len() as u32).to_ne_bytes());
            for (index, (name, _)) in self.entries.iter().enumerate() {
                let mut descriptor = FILEDESCRIPTORW::default();
                descriptor.dwFlags = 0x40;
                let mut filename = [0u16; 260];
                for (slot, unit) in filename.iter_mut().take(259).zip(name.encode_utf16()) { *slot = unit; }
                descriptor.cFileName = filename;
                // SAFETY: Destination allocation has one checked native descriptor slot per entry.
                unsafe { bytes.as_mut_ptr().add(offset + index * size_of::<FILEDESCRIPTORW>()).cast::<FILEDESCRIPTORW>().write_unaligned(descriptor); }
            }
            Some(bytes)
        } else if request.cfFormat == format(CFSTR_FILECONTENTS, 0, TYMED_HGLOBAL.0 as u32).cfFormat {
            if self.failing_contents_index == Some(request.lindex) {
                return None;
            }
            self.entries.get(usize::try_from(request.lindex).ok()?).map(|(_, bytes)| bytes.clone())
        } else if request.cfFormat == format(CFSTR_INETURLW, -1, TYMED_HGLOBAL.0 as u32).cfFormat {
            self.source_url_w.as_ref().map(|url| {
                let mut wide: Vec<u16> = url.encode_utf16().collect();
                wide.push(0);
                wide.into_iter().flat_map(|u| u.to_ne_bytes()).collect::<Vec<u8>>()
            })
        } else if request.cfFormat == format(CFSTR_INETURLA, -1, TYMED_HGLOBAL.0 as u32).cfFormat {
            self.source_url_a.as_ref().map(|url| {
                let mut bytes = url.as_bytes().to_vec();
                bytes.push(0);
                bytes
            })
        } else { None }
    }
}

#[allow(non_snake_case)]
impl IDataObject_Impl for VirtualSource_Impl {
    fn GetData(&self, request: *const FORMATETC) -> windows::core::Result<STGMEDIUM> {
        // SAFETY: COM supplies a readable FORMATETC, checked for null before dereference.
        let request = unsafe { request.as_ref() }.ok_or_else(|| windows::core::Error::from_hresult(E_POINTER))?;
        let bytes = self.payload(request).ok_or_else(|| windows::core::Error::from_hresult(DV_E_FORMATETC))?;
        if request.cfFormat == format(CFSTR_FILECONTENTS, 0, TYMED_HGLOBAL.0 as u32).cfFormat {
            self.reads.fetch_add(1, Ordering::SeqCst);
            if let Some(path) = usize::try_from(request.lindex).ok().and_then(|index| self.stream_paths.get(index)) {
                let path = windows::core::HSTRING::from(path.as_os_str());
                // SAFETY: The owned fixture path stays live during creation; medium owns the stream.
                let stream = unsafe { windows::Win32::UI::Shell::SHCreateStreamOnFileEx(PCWSTR(path.as_ptr()), 0, 0, false, None) }?;
                return Ok(STGMEDIUM { tymed: TYMED_ISTREAM.0 as u32, u: STGMEDIUM_0 { pstm: std::mem::ManuallyDrop::new(Some(stream)) }, pUnkForRelease: std::mem::ManuallyDrop::new(None) });
            }
            if request.lindex % 2 == 0 {
                // SAFETY: SHCreateMemStream copies the initialized fixture bytes into an owned stream.
                let stream = unsafe { windows::Win32::UI::Shell::SHCreateMemStream(Some(&bytes)) }.unwrap();
                return Ok(STGMEDIUM {
                    tymed: TYMED_ISTREAM.0 as u32,
                    u: STGMEDIUM_0 { pstm: std::mem::ManuallyDrop::new(Some(stream)) },
                    pUnkForRelease: std::mem::ManuallyDrop::new(None),
                });
            }
        }
        // SAFETY: Allocation is initialized completely and ownership transfers to ReleaseStgMedium.
        unsafe {
            let handle = GlobalAlloc(GMEM_MOVEABLE, bytes.len())?;
            let memory = GlobalLock(handle);
            assert!(!memory.is_null());
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), memory.cast::<u8>(), bytes.len());
            let _ = GlobalUnlock(handle);
            Ok(STGMEDIUM { tymed: TYMED_HGLOBAL.0 as u32, u: STGMEDIUM_0 { hGlobal: handle }, pUnkForRelease: std::mem::ManuallyDrop::new(None) })
        }
    }
    fn GetDataHere(&self, _: *const FORMATETC, _: *mut STGMEDIUM) -> windows::core::Result<()> { Err(E_NOTIMPL.into()) }
    fn QueryGetData(&self, request: *const FORMATETC) -> HRESULT {
        let Some(request) = (unsafe { request.as_ref() }) else { return E_POINTER; };
        if self.payload(request).is_some() { S_OK } else { DV_E_FORMATETC }
    }
    fn GetCanonicalFormatEtc(&self, _: *const FORMATETC, out: *mut FORMATETC) -> HRESULT {
        if let Some(out) = unsafe { out.as_mut() } { out.ptd = std::ptr::null_mut(); }
        DATA_S_SAMEFORMATETC
    }
    fn SetData(&self, _: *const FORMATETC, _: *const STGMEDIUM, _: BOOL) -> windows::core::Result<()> { Err(E_NOTIMPL.into()) }
    fn EnumFormatEtc(&self, _: u32) -> windows::core::Result<IEnumFORMATETC> { Err(E_NOTIMPL.into()) }
    fn DAdvise(&self, _: *const FORMATETC, _: u32, _: Ref<'_, IAdviseSink>) -> windows::core::Result<u32> { Err(E_NOTIMPL.into()) }
    fn DUnadvise(&self, _: u32) -> windows::core::Result<()> { Err(E_NOTIMPL.into()) }
    fn EnumDAdvise(&self) -> windows::core::Result<IEnumSTATDATA> { Err(E_NOTIMPL.into()) }
}

fn png() -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(2, 3).write_to(&mut output, image::ImageFormat::Png).unwrap();
    output.into_inner()
}

fn jpeg() -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 3).write_to(&mut output, image::ImageFormat::Jpeg).unwrap();
    output.into_inner()
}

struct Profile(PathBuf);
impl Profile {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("dropcove-image-{:032x}", GUID::new().unwrap().to_u128()));
        fs::create_dir_all(&root).unwrap(); Self(root)
    }
}
impl Drop for Profile { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

#[test]
fn native_png_payload_is_eligible_without_opening_content_until_drop() {
    let reads = std::sync::Arc::new(AtomicUsize::new(0));
    let data: IDataObject = VirtualSource { entries: vec![("photo.png".into(), png())], reads: reads.clone(), stream_paths: Vec::new(), ..Default::default() }.into();
    assert_eq!(crate::windows_drop::evaluate_data_payload(&data), crate::windows_drop::EvaluatedPayload::VirtualImages);
    assert_eq!(reads.load(Ordering::SeqCst), 0);
}

#[test]
fn native_png_drop_preserves_bytes_name_restart_and_temporary_copy_cleanup() {
    let profile = Profile::new();
    let database = profile.0.join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let bytes = png();
    let data: IDataObject = VirtualSource { entries: vec![("photo.wrong".into(), bytes.clone())], ..Default::default() }.into();
    let result = extract(&data, state.image_staging_store().unwrap()).unwrap()
        .accept_with_security(&state, |_, _, _| Ok(())).unwrap();
    let snapshot = result.snapshot.unwrap();
    let batch = &snapshot.batches[0];
    let item = &batch.items[0];
    state.complete_item_drag(&item.id, crate::shelf::DragOutcome::Canceled).unwrap();
    assert_eq!(fs::read(&item.path).unwrap(), bytes);
    state.complete_item_drag(&item.id, crate::shelf::DragOutcome::Rejected).unwrap();
    assert_eq!(fs::read(&item.path).unwrap(), bytes);
    assert_eq!(item.name, "photo.png");
    assert_eq!(fs::read(&item.path).unwrap(), bytes);
    assert!(!item.is_pinned);
    drop(state);
    let state = ShelfState::open(&database).unwrap();
    assert_eq!(state.snapshot().unwrap().batches[0].items[0].id, item.id);
    let preparation = state.prepare_batch_drag(std::slice::from_ref(&batch.id)).unwrap();
    let transfer = state.begin_batch_drag(preparation.token.as_deref().unwrap()).unwrap();
    state.complete_batch_drag(&transfer, crate::shelf::DragOutcome::Copy).unwrap();
    assert!(state.snapshot().unwrap().batches.is_empty());
    assert!(!std::path::Path::new(&item.path).exists());
}

#[test]
fn native_payload_does_not_open_descriptor_twenty_one() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let reads = std::sync::Arc::new(AtomicUsize::new(0));
    let data: IDataObject = VirtualSource { entries: (0..21).map(|i| (format!("image{i}.png"), png())).collect(), reads: reads.clone(), ..Default::default() }.into();
    let extracted = extract(&data, state.image_staging_store().unwrap()).unwrap();
    assert_eq!(reads.load(Ordering::SeqCst), 20);
    assert_eq!(extracted.skipped, 1);
    assert_eq!(extracted.entries.len(), 20);
}

#[test]
fn native_png_drop_rejects_truncated_image_instead_of_publishing_header_only_content() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let mut bytes = png();
    bytes.truncate(bytes.len() - 12);
    let data: IDataObject = VirtualSource { entries: vec![("broken.png".into(), bytes)], ..Default::default() }.into();
    let result = extract(&data, state.image_staging_store().unwrap()).unwrap()
        .accept_with_security(&state, |_, _, _| Ok(()));
    let drop_result = result.unwrap();
    assert!(drop_result.snapshot.is_none(), "a PNG without IEND must not become a Shelf Item");
    assert_eq!(drop_result.skipped, 1);
    assert!(drop_result.message.is_some());
    assert!(state.snapshot().unwrap().batches.is_empty());
}

#[test]
fn native_image_stream_accepts_exact_byte_limit_and_skips_one_byte_over() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let source = profile.0.join("source.png");
    fs::write(&source, png()).unwrap();
    let file = fs::OpenOptions::new().write(true).open(&source).unwrap();
    file.set_len(MAX_IMAGE_BYTES).unwrap();
    let data: IDataObject = VirtualSource { entries: vec![("bounded.png".into(), Vec::new())], stream_paths: vec![source.clone()], ..Default::default() }.into();
    let result = extract(&data, state.image_staging_store().unwrap()).unwrap().accept_with_security(&state, |_, _, _| Ok(())).unwrap();
    let retained = result.snapshot.unwrap();
    assert_eq!(fs::metadata(&retained.batches[0].items[0].path).unwrap().len(), MAX_IMAGE_BYTES);
    file.set_len(MAX_IMAGE_BYTES + 1).unwrap();
    let over = extract(&data, state.image_staging_store().unwrap()).unwrap();
    assert_eq!(over.skipped, 1);
    assert!(over.entries.is_empty());
    assert_eq!(state.snapshot().unwrap().batches[0].id, retained.batches[0].id);
    assert_eq!(fs::read_dir(profile.0.join("staging")).unwrap().count(), 0);
    assert_eq!(fs::metadata(&source).unwrap().len(), MAX_IMAGE_BYTES + 1);
}

#[test]
fn native_image_fallback_names_are_distinct_across_drops_in_the_same_second() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let data: IDataObject = VirtualSource { entries: vec![("CON.exe".into(), png())], ..Default::default() }.into();
    let mut first = extract(&data, state.image_staging_store().unwrap()).unwrap();
    first.timestamp = "2026-10-23 12-34-56".into();
    first.accept_with_security(&state, |_, _, _| Ok(())).unwrap();
    let mut second = extract(&data, state.image_staging_store().unwrap()).unwrap();
    second.timestamp = "2026-10-23 12-34-56".into();
    let snapshot = second.accept_with_security(&state, |_, _, _| Ok(())).unwrap().snapshot.unwrap();
    assert_eq!(snapshot.batches[0].items[0].name, "Dropped image 2026-10-23 12-34-56-2.png");
    assert_eq!(snapshot.batches[1].items[0].name, "Dropped image 2026-10-23 12-34-56.png");
}

#[test]
fn native_security_rejection_cleans_staging_and_preserves_external_source() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let external = profile.0.join("external.png");
    fs::write(&external, b"external source bytes").unwrap();
    let existing = state.accept_drop(vec![Some(external.display().to_string())]).unwrap().snapshot.unwrap();
    let data: IDataObject = VirtualSource { entries: vec![("blocked.png".into(), png())], ..Default::default() }.into();
    let result = extract(&data, state.image_staging_store().unwrap()).unwrap()
        .accept_with_security(&state, |_, _, _| Err("blocked by trust policy".into()));
    assert!(result.unwrap_err().contains("Attachment Services"));
    assert_eq!(state.snapshot().unwrap().batches[0].id, existing.batches[0].id);
    assert_eq!(fs::read(&external).unwrap(), b"external source bytes");
    assert_eq!(fs::read_dir(profile.0.join("staging")).unwrap().count(), 0);
    assert_eq!(fs::read_dir(profile.0.join("managed")).unwrap().count(), 0);
}

#[test]
fn native_mixed_image_drop_preserves_all_formats_order_duplicates_and_skip_result() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let mut images = Vec::new();
    for (format, extension) in [(image::ImageFormat::Png, "png"), (image::ImageFormat::Jpeg, "jpg"), (image::ImageFormat::Gif, "gif"), (image::ImageFormat::WebP, "webp"), (image::ImageFormat::Bmp, "bmp"), (image::ImageFormat::Ico, "ico")] {
        let mut encoded = Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(2, 3).write_to(&mut encoded, format).unwrap();
        images.push((format!("asset.{extension}"), encoded.into_inner()));
    }
    images.push(("asset.avif".into(), include_bytes!("fixtures/website-image.avif").to_vec()));
    images.push(("asset.svg".into(), br#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert('preserved')</script><image href="https://example.test/source.png"/></svg>"#.to_vec()));
    images.push(images[0].clone());
    let mut entries = images.clone();
    entries.insert(2, ("unsupported.bin".into(), b"not an image".to_vec()));
    let data: IDataObject = VirtualSource { entries, ..Default::default() }.into();
    let result = extract(&data, state.image_staging_store().unwrap()).unwrap().accept_with_security(&state, |_, _, _| Ok(())).unwrap();
    assert_eq!(result.skipped, 1);
    assert_eq!(result.message.as_deref(), Some("Added 9 images; skipped 1."));
    let snapshot = result.snapshot.unwrap();
    assert_eq!(snapshot.batches.len(), 1);
    let items = &snapshot.batches[0].items;
    assert_eq!(items.iter().map(|item| item.name.as_str()).collect::<Vec<_>>(), vec!["asset.png", "asset.jpg", "asset.gif", "asset.webp", "asset.bmp", "asset.ico", "asset.avif", "asset.svg", "asset.png"]);
    for (item, (_, expected)) in items.iter().zip(&images) {
        assert_eq!(fs::read(&item.path).unwrap(), *expected);
    }
    assert_ne!(items[0].path, items[8].path);
    assert_eq!(state.read_managed_text_content(&items[7].id).unwrap(), None);
}

#[test]
fn native_svg_intake_accepts_namespaced_raw_content_and_skips_malformed_xml() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let valid = br#"<s:svg xmlns:s="http://www.w3.org/2000/svg"><s:text>A &amp; B</s:text><s:script>alert('preserved')</s:script></s:svg>"#.to_vec();
    let entries = vec![
        ("attribute.svg".into(), br#"<svg><g value="&unknown;"/></svg>"#.to_vec()),
        ("outside.svg".into(), b"<svg/><![CDATA[not outside root]]>".to_vec()),
        ("duplicate.svg".into(), br#"<svg><rect width="1" width="2"/></svg>"#.to_vec()),
        ("numeric.svg".into(), b"<svg>&#0;</svg>".to_vec()),
        ("valid.svg".into(), valid.clone()),
    ];
    let data: IDataObject = VirtualSource { entries, ..Default::default() }.into();
    let result = extract(&data, state.image_staging_store().unwrap()).unwrap().accept_with_security(&state, |_, _, _| Ok(())).unwrap();
    assert_eq!(result.skipped, 4);
    let snapshot = result.snapshot.unwrap();
    assert_eq!(snapshot.batches[0].items.iter().map(|item| item.name.as_str()).collect::<Vec<_>>(), vec!["valid.svg"]);
    assert_eq!(fs::read(&snapshot.batches[0].items[0].path).unwrap(), valid);
}

#[test]
fn native_total_byte_limit_skips_nonfitting_entry_and_accepts_later_smaller_image() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let large = profile.0.join("large.png");
    let medium = profile.0.join("medium.png");
    for (path, size) in [(&large, MAX_IMAGE_BYTES), (&medium, MAX_IMAGE_BYTES - 1024)] {
        fs::write(path, png()).unwrap();
        fs::OpenOptions::new().write(true).open(path).unwrap().set_len(size).unwrap();
    }
    let mut entries: Vec<_> = (0..6).map(|index| (format!("image{index}.png"), Vec::new())).collect();
    entries.push(("later.png".into(), png()));
    let paths = vec![large.clone(), large.clone(), large.clone(), large.clone(), medium, large];
    let data: IDataObject = VirtualSource { entries, stream_paths: paths, ..Default::default() }.into();
    let result = extract(&data, state.image_staging_store().unwrap()).unwrap().accept_with_security(&state, |_, _, _| Ok(())).unwrap();
    assert_eq!(result.skipped, 1);
    let items = &result.snapshot.unwrap().batches[0].items;
    assert_eq!(items.iter().map(|item| item.name.as_str()).collect::<Vec<_>>(), vec!["image0.png", "image1.png", "image2.png", "image3.png", "image4.png", "later.png"]);
    assert!(items.iter().map(|item| fs::metadata(&item.path).unwrap().len()).sum::<u64>() < MAX_DROP_BYTES);
    let exact: IDataObject = VirtualSource {
        entries: (0..6).map(|index| (format!("exact{index}.png"), Vec::new())).collect(),
        stream_paths: vec![profile.0.join("large.png"); 6],
        ..Default::default()
    }.into();
    let result = extract(&exact, state.image_staging_store().unwrap()).unwrap().accept_with_security(&state, |_, _, _| Ok(())).unwrap();
    assert_eq!(result.skipped, 1);
    let snapshot = result.snapshot.unwrap();
    let sizes: Vec<_> = snapshot.batches[0].items.iter().map(|item| fs::metadata(&item.path).unwrap().len()).collect();
    assert_eq!(sizes, vec![100 * 1024 * 1024; 5]);
    assert_eq!(sizes.iter().sum::<u64>(), 500 * 1024 * 1024);
}

#[test]
fn native_raster_resource_limit_skips_huge_dimensions_without_decoding_a_pixel_buffer() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let mut encoded = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 3).write_to(&mut encoded, image::ImageFormat::Bmp).unwrap();
    let mut huge = encoded.into_inner();
    huge[18..22].copy_from_slice(&(1i32 << 28).to_le_bytes());
    huge[22..26].copy_from_slice(&1i32.to_le_bytes());
    let data: IDataObject = VirtualSource { entries: vec![("huge.bmp".into(), huge), ("safe.png".into(), png())], ..Default::default() }.into();
    let result = extract(&data, state.image_staging_store().unwrap()).unwrap().accept_with_security(&state, |_, _, _| Ok(())).unwrap();
    assert_eq!(result.skipped, 1);
    assert_eq!(result.snapshot.unwrap().batches[0].items[0].name, "safe.png");
}

#[test]
fn native_mixed_valid_png_and_truncated_gif_skips_truncated_neighbor() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let mut gif_encoded = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 3).write_to(&mut gif_encoded, image::ImageFormat::Gif).unwrap();
    let mut truncated_gif = gif_encoded.into_inner();
    truncated_gif.truncate(truncated_gif.len() / 2);
    let data: IDataObject = VirtualSource {
        entries: vec![
            ("valid.png".into(), png()),
            ("truncated.gif".into(), truncated_gif),
        ],
        ..Default::default()
    }.into();
    let result = extract(&data, state.image_staging_store().unwrap()).unwrap()
        .accept_with_security(&state, |_, _, _| Ok(())).unwrap();
    assert_eq!(result.skipped, 1);
    let snapshot = result.snapshot.unwrap();
    assert_eq!(snapshot.batches.len(), 1);
    let items = &snapshot.batches[0].items;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "valid.png");
}

#[test]
fn native_second_image_trust_rejection_cleans_all_staging_and_preserves_prior_shelf() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let external = profile.0.join("prior.png");
    fs::write(&external, png()).unwrap();
    let prior = state.accept_drop(vec![Some(external.display().to_string())]).unwrap().snapshot.unwrap();

    let data: IDataObject = VirtualSource {
        entries: vec![
            ("first.png".into(), png()),
            ("second.png".into(), png()),
        ],
        ..Default::default()
    }.into();
    let extracted = extract(&data, state.image_staging_store().unwrap()).unwrap();
    assert_eq!(extracted.entries.len(), 2);
    assert_eq!(fs::read_dir(profile.0.join("staging")).unwrap().count(), 2);

    let result = extracted.accept_with_security(&state, |_path, _source, filename| {
        if filename.starts_with("second") {
            Err("antivirus policy blocked second image".into())
        } else {
            Ok(())
        }
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Attachment Services"));

    assert_eq!(fs::read_dir(profile.0.join("staging")).unwrap().count(), 0);
    assert_eq!(fs::read_dir(profile.0.join("managed")).unwrap().count(), 0);
    let current = state.snapshot().unwrap();
    assert_eq!(current.batches.len(), prior.batches.len());
    assert_eq!(current.batches[0].id, prior.batches[0].id);
}

#[test]
fn native_postscan_deletion_rejects_drop_and_cleans_all_staging() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let data: IDataObject = VirtualSource {
        entries: vec![
            ("first.png".into(), png()),
            ("second.png".into(), png()),
        ],
        ..Default::default()
    }.into();
    let extracted = extract(&data, state.image_staging_store().unwrap()).unwrap();
    let result = extracted.accept_with_security(&state, |path, _source, filename| {
        if filename.starts_with("second") {
            let _ = fs::remove_file(path);
        }
        Ok(())
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Attachment Services"));
    assert_eq!(fs::read_dir(profile.0.join("staging")).unwrap().count(), 0);
    assert!(state.snapshot().unwrap().batches.is_empty());
}

#[test]
fn native_trust_mutation_to_supported_jpeg_preserves_final_bytes_and_repairs_extension() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let jpeg_bytes = jpeg();
    let data: IDataObject = VirtualSource {
        entries: vec![("photo.png".into(), png())],
        ..Default::default()
    }.into();
    let extracted = extract(&data, state.image_staging_store().unwrap()).unwrap();
    let mutated = jpeg_bytes.clone();
    let result = extracted.accept_with_security(&state, |path, _source, _filename| {
        fs::write(path, &mutated).map_err(|e| e.to_string())
    }).unwrap();

    let snapshot = result.snapshot.unwrap();
    assert_eq!(snapshot.batches.len(), 1);
    let item = &snapshot.batches[0].items[0];
    assert_eq!(item.name, "photo.jpg");
    assert_eq!(fs::read(&item.path).unwrap(), jpeg_bytes);
    assert_eq!(fs::read_dir(profile.0.join("staging")).unwrap().count(), 0);
}

#[test]
fn native_trust_invalid_and_over_limit_mutation_rejects_and_cleans_staging() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();

    // Invalid mutation to corrupt bytes
    let data_invalid: IDataObject = VirtualSource {
        entries: vec![("corrupt.png".into(), png())],
        ..Default::default()
    }.into();
    let extracted = extract(&data_invalid, state.image_staging_store().unwrap()).unwrap();
    let result_invalid = extracted.accept_with_security(&state, |path, _source, _filename| {
        fs::write(path, b"corrupted garbage non-image bytes").map_err(|e| e.to_string())
    });
    assert!(result_invalid.is_err());
    assert!(result_invalid.unwrap_err().contains("Attachment Services produced invalid content"));
    assert_eq!(fs::read_dir(profile.0.join("staging")).unwrap().count(), 0);
    assert!(state.snapshot().unwrap().batches.is_empty());

    // Over-limit mutation (> MAX_IMAGE_BYTES)
    let data_overlimit: IDataObject = VirtualSource {
        entries: vec![("overlimit.png".into(), png())],
        ..Default::default()
    }.into();
    let extracted = extract(&data_overlimit, state.image_staging_store().unwrap()).unwrap();
    let result_overlimit = extracted.accept_with_security(&state, |path, _source, _filename| {
        let file = fs::OpenOptions::new().write(true).open(path).map_err(|e| e.to_string())?;
        file.set_len(MAX_IMAGE_BYTES + 1024).map_err(|e| e.to_string())
    });
    assert!(result_overlimit.is_err());
    assert!(result_overlimit.unwrap_err().contains("over-limit content"));
    assert_eq!(fs::read_dir(profile.0.join("staging")).unwrap().count(), 0);
    assert!(state.snapshot().unwrap().batches.is_empty());
}

#[test]
fn native_missing_content_index_fails_io_and_cleans_all_staging() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let data: IDataObject = VirtualSource {
        entries: vec![
            ("first.png".into(), png()),
            ("missing.png".into(), png()),
        ],
        failing_contents_index: Some(1),
        ..Default::default()
    }.into();
    let result = extract(&data, state.image_staging_store().unwrap());
    assert!(matches!(&result, Err(error) if error.contains("Image I/O failed")));
    assert_eq!(fs::read_dir(profile.0.join("staging")).unwrap().count(), 0);
    assert!(state.snapshot().unwrap().batches.is_empty());
}

#[test]
fn native_source_url_provenance_handles_wide_ansi_and_unsupported_schemes() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();

    // Wide CFSTR_INETURLW https
    let data_w: IDataObject = VirtualSource {
        entries: vec![("wide.png".into(), png())],
        source_url_w: Some("https://example.test/assets/wide.png".into()),
        ..Default::default()
    }.into();
    let mut captured_w = None;
    let extracted = extract(&data_w, state.image_staging_store().unwrap()).unwrap();
    extracted.accept_with_security(&state, |_path, source, _filename| {
        captured_w = source.map(String::from);
        Ok(())
    }).unwrap();
    assert_eq!(captured_w.as_deref(), Some("https://example.test/assets/wide.png"));

    // ANSI CFSTR_INETURLA http
    let data_a: IDataObject = VirtualSource {
        entries: vec![("ansi.png".into(), png())],
        source_url_a: Some("http://example.test/assets/ansi.png".into()),
        ..Default::default()
    }.into();
    let mut captured_a = None;
    let extracted = extract(&data_a, state.image_staging_store().unwrap()).unwrap();
    extracted.accept_with_security(&state, |_path, source, _filename| {
        captured_a = source.map(String::from);
        Ok(())
    }).unwrap();
    assert_eq!(captured_a.as_deref(), Some("http://example.test/assets/ansi.png"));

    // Unsupported scheme javascript: falls back to None (Restricted Zone)
    let data_unsupported: IDataObject = VirtualSource {
        entries: vec![("script.png".into(), png())],
        source_url_w: Some("javascript:alert(1)".into()),
        ..Default::default()
    }.into();
    let mut captured_unsupported = Some("initial".to_string());
    let extracted = extract(&data_unsupported, state.image_staging_store().unwrap()).unwrap();
    extracted.accept_with_security(&state, |_path, source, _filename| {
        captured_unsupported = source.map(String::from);
        Ok(())
    }).unwrap();
    assert_eq!(captured_unsupported, None);

    // Unsupported scheme file: falls back to None (Restricted Zone)
    let data_file: IDataObject = VirtualSource {
        entries: vec![("local.png".into(), png())],
        source_url_w: Some("file:///C:/test.png".into()),
        ..Default::default()
    }.into();
    let mut captured_file = Some("initial".to_string());
    let extracted = extract(&data_file, state.image_staging_store().unwrap()).unwrap();
    extracted.accept_with_security(&state, |_path, source, _filename| {
        captured_file = source.map(String::from);
        Ok(())
    }).unwrap();
    assert_eq!(captured_file, None);
}

#[test]
fn native_unterminated_descriptor_name_still_intakes_image_with_fallback_naming() {
    let profile = Profile::new();
    let state = ShelfState::open(profile.0.join("shelf.db")).unwrap();
    let data: IDataObject = VirtualSource { entries: vec![("".into(), png())], ..Default::default() }.into();
    let result = extract(&data, state.image_staging_store().unwrap()).unwrap()
        .accept_with_security(&state, |_, _, _| Ok(())).unwrap();
    assert_eq!(result.skipped, 0);
    let snapshot = result.snapshot.unwrap();
    assert_eq!(snapshot.batches.len(), 1);
    assert!(snapshot.batches[0].items[0].name.starts_with("Dropped image "));
    assert!(snapshot.batches[0].items[0].name.ends_with(".png"));
}
