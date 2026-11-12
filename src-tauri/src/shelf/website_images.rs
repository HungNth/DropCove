//! Safe filename/content validation and Windows Attachment Services preparation
//! for Website Image Virtual Files (PNG, JPEG, GIF, WebP, AVIF, SVG, BMP, ICO).

use std::{
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use windows::{
    core::{w, Result as WinResult, HRESULT, PCWSTR},
    Win32::{
        Foundation::{RPC_E_CHANGED_MODE, S_FALSE, S_OK},
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
            CoUninitialize,
        },
        UI::Shell::{AttachmentServices, IAttachmentExecute},
    },
};

pub(crate) const MAX_IMAGE_BYTES: u64 = 100 * 1024 * 1024;
pub(crate) const MAX_DROP_BYTES: u64 = 500 * 1024 * 1024;
pub(crate) const MAX_DESCRIPTORS: usize = 20;
pub(crate) const MAX_DECODED_BYTES: u64 = 512 * 1024 * 1024;

/// Categorized error distinguishing operational filesystem I/O from invalid/unsupported content.
#[derive(Debug)]
pub enum ValidationError {
    Invalid(String),
    Io(io::Error),
}

impl ValidationError {
    pub fn is_io(&self) -> bool {
        matches!(self, Self::Io(_))
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(msg) => write!(f, "invalid image: {msg}"),
            Self::Io(err) => write!(f, "I/O error during image validation: {err}"),
        }
    }
}

impl std::error::Error for ValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Invalid(_) => None,
            Self::Io(err) => Some(err),
        }
    }
}

impl From<io::Error> for ValidationError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Supported image formats for DropCove website image intake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
    Avif,
    Svg,
    Bmp,
    Ico,
}

impl ImageFormat {
    /// Canonical file extension without leading dot.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
            Self::Webp => "webp",
            Self::Avif => "avif",
            Self::Svg => "svg",
            Self::Bmp => "bmp",
            Self::Ico => "ico",
        }
    }
}

/// Returns a safe content-matching source filename, or None when a generated name is required.
pub(crate) fn sanitize_image_name(name: &str, format: ImageFormat) -> Option<String> {
    let basename = name.rsplit(['\\', '/']).next().unwrap_or(name).trim();
    let stem = basename.rsplit_once('.').map_or(basename, |(stem, _)| stem);
    let clean: String = stem.chars().filter(|ch| !ch.is_control() && !":*?\"<>|\\/".contains(*ch)).collect();
    let clean = clean.trim_matches([' ', '.']);
    let first = clean.split('.').next().unwrap_or_default().trim_end().to_ascii_uppercase();
    let reserved = matches!(first.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| first.strip_prefix(prefix).is_some_and(|number| matches!(number, "1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9"|"¹"|"²"|"³")));
    if clean.is_empty() || reserved { return None; }
    let mut units = 0;
    let limit = 240 - format.extension().len() - 1;
    let stem: String = clean.chars().take_while(|ch| {
        units += ch.len_utf16();
        units <= limit
    }).collect();
    let stem = stem.trim_end_matches([' ', '.']);
    (!stem.is_empty()).then(|| format!("{stem}.{}", format.extension()))
}

/// RAII guard for balancing COM initialization.
///
/// Only uninitializes if `CoInitializeEx` returned `S_OK` or `S_FALSE`.
/// Threads where COM was already initialized in another mode (`RPC_E_CHANGED_MODE`)
/// are not uninitialized.
struct ComGuard {
    needs_uninit: bool,
}

impl ComGuard {
    fn new() -> WinResult<Self> {
        // SAFETY: CoInitializeEx is called to ensure an apartment is initialized on the current thread.
        let hr: HRESULT = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if hr == S_OK || hr == S_FALSE {
            Ok(Self { needs_uninit: true })
        } else if hr == RPC_E_CHANGED_MODE {
            // Already initialized in a different threading mode (e.g. MTA); proceed without balancing.
            Ok(Self { needs_uninit: false })
        } else {
            Err(windows::core::Error::from_hresult(hr))
        }
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.needs_uninit {
            // SAFETY: Matches successful CoInitializeEx on this thread.
            unsafe {
                CoUninitialize();
            }
        }
    }
}

/// Applies mandatory Windows Attachment Services (`IAttachmentExecute::Save`) to a staged file.
///
/// - Runs non-interactively without user prompts.
/// - Sets `SetClientTitle("DropCove")`.
/// - Sets `SetLocalPath` to `path` and `SetFileName` to `filename`.
/// - If `source` is provided with an `http://` or `https://` scheme, passes it to `SetSource`.
/// - If `source` is missing or uses an unapproved scheme, does not call `SetSource`,
///   defaulting to the Attachment Services Restricted Zone.
/// - Calls `Save()`. Fails closed if Attachment Services or antivirus policy rejects the file.
pub(crate) fn secure_image(
    path: &Path,
    source: Option<&str>,
    filename: &str,
) -> Result<(), String> {
    let _com_guard = ComGuard::new().map_err(|e| format!("COM initialization failed: {e}"))?;

    // SAFETY: AttachmentServices is created via standard COM CoCreateInstance.
    let attachment: IAttachmentExecute = unsafe {
        CoCreateInstance(&AttachmentServices, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| format!("failed to instantiate IAttachmentExecute: {e}"))?
    };

    // SAFETY: Method calls on valid IAttachmentExecute interface.
    unsafe {
        attachment
            .SetClientTitle(w!("DropCove"))
            .map_err(|e| format!("SetClientTitle failed: {e}"))?;

        let path_hstring = windows::core::HSTRING::from(path.as_os_str());
        attachment
            .SetLocalPath(PCWSTR(path_hstring.as_ptr()))
            .map_err(|e| format!("SetLocalPath failed: {e}"))?;

        let file_hstring = windows::core::HSTRING::from(filename);
        attachment
            .SetFileName(PCWSTR(file_hstring.as_ptr()))
            .map_err(|e| format!("SetFileName failed: {e}"))?;

        // Only explicit http and https schemes are accepted as provenance.
        if let Some(url) = source {
            let lower = url.to_ascii_lowercase();
            if lower.starts_with("http://") || lower.starts_with("https://") {
                let url_hstring = windows::core::HSTRING::from(url);
                attachment
                    .SetSource(PCWSTR(url_hstring.as_ptr()))
                    .map_err(|e| format!("SetSource failed: {e}"))?;
            }
        }

        attachment
            .Save()
            .map_err(|e| format!("Attachment Services rejected the file: {e}"))?;
    }

    Ok(())
}

/// Validates image content, bounded resources, and formats from a filesystem path.
///
/// Returns `Ok(ImageFormat)` if the file is a valid, bounded instance of one of
/// the 8 approved formats (PNG, JPEG, GIF, WebP, AVIF, SVG, BMP, ICO).
/// Returns `ValidationError::Io` for filesystem errors, or `ValidationError::Invalid`
/// for malformed, unsupported, or over-limit content.
pub(crate) fn validate_image(path: &Path) -> Result<ImageFormat, ValidationError> {
    let file = File::open(path)?;
    let len = file.metadata()?.len();

    if len == 0 {
        return Err(ValidationError::Invalid("image file is empty".to_string()));
    }
    if len > MAX_IMAGE_BYTES {
        return Err(ValidationError::Invalid(format!(
            "image byte length {len} exceeds limit of {MAX_IMAGE_BYTES} bytes"
        )));
    }

    // Inspect first 32 bytes to route to format validator.
    let mut header = [0u8; 32];
    let mut reader = BufReader::new(file);
    let bytes_read = reader.read(&mut header)?;
    let prefix = &header[..bytes_read];

    reader.seek(SeekFrom::Start(0))?;

    if prefix.starts_with(b"\x89PNG\r\n\x1a\n") {
        let mut decoder = png::Decoder::new(&mut reader);
        decoder.set_limits(png::Limits { bytes: MAX_DECODED_BYTES as usize });
        let mut png = decoder.read_info().map_err(png_validation_error)?;
        png.finish().map_err(png_validation_error)?;
        drop(png);
        reader.seek(SeekFrom::Start(0))?;
    }

    // 1. AVIF check (starts with ftyp box containing avif/avis brand)
    if is_avif_prefix(prefix) {
        return validate_avif(reader, len);
    }

    // 2. SVG check (text/xml starting with <?xml or <svg, or comments/spaces)
    if is_svg_prefix(prefix) {
        return validate_svg(reader, len);
    }

    // 3. Raster formats handled via image crate decoder container validation
    validate_raster_image(reader)
}

fn png_validation_error(error: png::DecodingError) -> ValidationError {
    match error {
        png::DecodingError::IoError(error) if error.kind() != std::io::ErrorKind::UnexpectedEof => ValidationError::Io(error),
        other => ValidationError::Invalid(other.to_string()),
    }
}

fn is_avif_prefix(prefix: &[u8]) -> bool {
    if prefix.len() >= 12 && &prefix[4..8] == b"ftyp" {
        let brand = &prefix[8..12];
        brand == b"avif" || brand == b"avis"
    } else {
        false
    }
}

fn is_svg_prefix(prefix: &[u8]) -> bool {
    prefix.strip_prefix(b"\xef\xbb\xbf").unwrap_or(prefix).trim_ascii_start().starts_with(b"<")
}

/// Validates AVIF files using avif_parse for container validation, and checks
/// spatial extents for bounded resource limits.
fn validate_avif<R: Read + Seek>(mut reader: R, file_len: u64) -> Result<ImageFormat, ValidationError> {
    avif_parse::read_avif(&mut reader)
        .map_err(|e| ValidationError::Invalid(format!("invalid AVIF container: {e:?}")))?;

    // Read file bytes to traverse structured ISOBMFF box hierarchy for ispe extents.
    reader.seek(SeekFrom::Start(0))?;
    let mut data = Vec::with_capacity(file_len.min(MAX_IMAGE_BYTES) as usize);
    reader.read_to_end(&mut data)?;

    let extents = find_avif_ispe_extents(&data);
    if extents.is_empty() {
        return Err(ValidationError::Invalid(
            "AVIF missing required image spatial extents (ispe) property".to_string(),
        ));
    }

    for (width, height) in extents {
        if width == 0 || height == 0 {
            return Err(ValidationError::Invalid(format!(
                "invalid AVIF dimensions: {width}x{height}"
            )));
        }
        let worst_bytes = 4u64; // RGBA
        let decoded_bytes = (width as u64)
            .checked_mul(height as u64)
            .and_then(|px| px.checked_mul(worst_bytes))
            .ok_or_else(|| {
                ValidationError::Invalid("AVIF dimension arithmetic overflow".to_string())
            })?;

        if decoded_bytes > MAX_DECODED_BYTES {
            return Err(ValidationError::Invalid(format!(
                "AVIF decoded dimensions {width}x{height} exceed decoded limit of {MAX_DECODED_BYTES} bytes"
            )));
        }
    }

    Ok(ImageFormat::Avif)
}

/// Structured traversal of ISOBMFF top-level boxes -> meta -> iprp -> ipco -> ispe.
fn find_avif_ispe_extents(data: &[u8]) -> Vec<(u32, u32)> {
    let mut extents = Vec::new();
    for (box_type, start, end) in parse_isobmff_boxes(data, 0, data.len()) {
        if box_type == *b"meta" {
            // meta is a FullBox: 4 bytes version (1) + flags (3)
            let meta_body = start.saturating_add(4);
            for (m_type, m_start, m_end) in parse_isobmff_boxes(data, meta_body, end) {
                if m_type == *b"iprp" {
                    for (p_type, p_start, p_end) in parse_isobmff_boxes(data, m_start, m_end) {
                        if p_type == *b"ipco" {
                            for (c_type, c_start, c_end) in parse_isobmff_boxes(data, p_start, p_end) {
                                if c_type == *b"ispe" && c_start.saturating_add(12) <= c_end {
                                    // ispe: 4 bytes version+flags, 4 bytes width, 4 bytes height
                                    if let (Ok(w), Ok(h)) = (
                                        data[c_start + 4..c_start + 8].try_into(),
                                        data[c_start + 8..c_start + 12].try_into(),
                                    ) {
                                        extents.push((u32::from_be_bytes(w), u32::from_be_bytes(h)));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    extents
}

fn parse_isobmff_boxes(data: &[u8], start: usize, end: usize) -> Vec<([u8; 4], usize, usize)> {
    let mut boxes = Vec::new();
    let mut offset = start;
    while offset + 8 <= end {
        let size_bytes: [u8; 4] = match data[offset..offset + 4].try_into() {
            Ok(b) => b,
            Err(_) => break,
        };
        let size = u32::from_be_bytes(size_bytes) as usize;
        let box_type: [u8; 4] = match data[offset + 4..offset + 8].try_into() {
            Ok(b) => b,
            Err(_) => break,
        };

        let (header_len, box_len) = if size == 0 {
            (8, end.saturating_sub(offset))
        } else if size == 1 {
            if offset + 16 > end {
                break;
            }
            let large_size = match data[offset + 8..offset + 16].try_into() {
                Ok(b) => u64::from_be_bytes(b) as usize,
                Err(_) => break,
            };
            (16, large_size)
        } else {
            (8, size)
        };

        if box_len < header_len || offset.saturating_add(box_len) > end {
            break;
        }
        boxes.push((box_type, offset + header_len, offset + box_len));
        offset += box_len;
    }
    boxes
}

fn legal_xml_character(ch: char) -> bool {
    matches!(ch as u32, 0x9 | 0xa | 0xd | 0x20..=0xd7ff | 0xe000..=0xfffd | 0x10000..=0x10ffff)
}

/// Validates XML without expanding document-defined entities or changing raw SVG bytes.
fn validate_svg<R: BufRead>(reader: R, _file_len: u64) -> Result<ImageFormat, ValidationError> {
    use quick_xml::{events::Event, reader::Reader, XmlVersion};
    let mut reader = Reader::from_reader(reader);
    let mut depth = 0usize;
    let mut saw_root = false;
    let mut saw_declaration = false;
    let mut buffer = Vec::new();
    loop {
        let event = reader.read_event_into(&mut buffer).map_err(|error| match error {
            quick_xml::Error::Io(error) => ValidationError::Io(io::Error::new(error.kind(), error.to_string())),
            other => ValidationError::Invalid(other.to_string()),
        })?;
        let empty = matches!(&event, Event::Empty(_));
        match event {
            Event::Start(element) | Event::Empty(element) => {
                if depth == 0 {
                    if saw_root || element.local_name().as_ref() != "svg" {
                        return Err(ValidationError::Invalid("expected one SVG root element".into()));
                    }
                    saw_root = true;
                }
                for attribute in element.attributes() {
                    let attribute = attribute.map_err(|e| ValidationError::Invalid(e.to_string()))?;
                    let value = attribute.normalized_value(XmlVersion::Implicit1_0).map_err(|e| ValidationError::Invalid(e.to_string()))?;
                    if !value.chars().all(legal_xml_character) {
                        return Err(ValidationError::Invalid("illegal XML attribute character".into()));
                    }
                }
                if !empty { depth += 1; }
            }
            Event::End(_) => {
                depth = depth.checked_sub(1).ok_or_else(|| ValidationError::Invalid("unexpected XML closing tag".into()))?;
            }
            Event::DocType(_) => return Err(ValidationError::Invalid("SVG document types and custom entities are unsupported".into())),
            Event::GeneralRef(reference) => {
                let character = reference.resolve_char_ref().map_err(|e| ValidationError::Invalid(e.to_string()))?;
                if depth == 0 || character.is_some_and(|ch| !legal_xml_character(ch))
                    || (character.is_none() && !matches!(reference.as_ref(), "amp" | "lt" | "gt" | "quot" | "apos")) {
                    return Err(ValidationError::Invalid("invalid XML entity reference".into()));
                }
            }
            Event::Text(text) => {
                if !text.as_ref().chars().all(legal_xml_character)
                    || (depth == 0 && !text.as_ref().chars().all(|ch| matches!(ch, ' ' | '\t' | '\r' | '\n'))) {
                    return Err(ValidationError::Invalid("invalid XML character data".into()));
                }
            }
            Event::CData(text) => {
                if depth == 0 || !text.as_ref().chars().all(legal_xml_character) {
                    return Err(ValidationError::Invalid("invalid XML CDATA".into()));
                }
            }
            Event::Decl(_) => {
                if saw_root || saw_declaration { return Err(ValidationError::Invalid("misplaced XML declaration".into())); }
                saw_declaration = true;
            }
            Event::PI(_) | Event::Comment(_) => {},
            Event::Eof => break,
        }
        buffer.clear();
    }
    if !saw_root || depth != 0 { return Err(ValidationError::Invalid("incomplete SVG document".into())); }
    Ok(ImageFormat::Svg)
}

/// Validates raster image formats (PNG, JPEG, GIF, WebP, BMP, ICO)
/// using image::ImageReader with strict decoding allocation limits.
fn validate_raster_image<R: BufRead + Seek>(reader: R) -> Result<ImageFormat, ValidationError> {
    use image::{ImageDecoder, ImageFormat as ImgFmt, ImageReader, Limits};

    let mut limits = Limits::no_limits();
    limits.max_alloc = Some(MAX_DECODED_BYTES);

    let mut img_reader = ImageReader::new(reader)
        .with_guessed_format()
        .map_err(ValidationError::Io)?;

    img_reader.limits(limits);

    let format = img_reader
        .format()
        .ok_or_else(|| ValidationError::Invalid("unrecognized image format".to_string()))?;

    let approved_format = match format {
        ImgFmt::Png => ImageFormat::Png,
        ImgFmt::Jpeg => ImageFormat::Jpeg,
        ImgFmt::Gif => ImageFormat::Gif,
        ImgFmt::WebP => ImageFormat::Webp,
        ImgFmt::Bmp => ImageFormat::Bmp,
        ImgFmt::Ico => ImageFormat::Ico,
        other => {
            return Err(ValidationError::Invalid(format!(
                "unsupported image format: {other:?}"
            )));
        }
    };

    // Obtain decoder to validate container headers and dimensions without allocating pixel buffers.
    let decoder = img_reader.into_decoder().map_err(|e| match e {
        image::ImageError::IoError(io_err) => ValidationError::Io(io_err),
        other => ValidationError::Invalid(format!(
            "failed to initialize {approved_format:?} decoder: {other}"
        )),
    })?;

    let (width, height) = decoder.dimensions();
    if width == 0 || height == 0 {
        return Err(ValidationError::Invalid(format!(
            "invalid image dimensions: {width}x{height}"
        )));
    }

    let total_bytes = decoder.total_bytes();
    if total_bytes > MAX_DECODED_BYTES {
        return Err(ValidationError::Invalid(format!(
            "image decoded size {total_bytes} exceeds maximum allowed {MAX_DECODED_BYTES} bytes"
        )));
    }

    let bytes_per_pixel = decoder.color_type().bytes_per_pixel() as u64;
    let worst_case_bytes = (width as u64)
        .checked_mul(height as u64)
        .and_then(|px| px.checked_mul(bytes_per_pixel.max(4)))
        .ok_or_else(|| {
            ValidationError::Invalid("image dimension calculation overflow".to_string())
        })?;

    if worst_case_bytes > MAX_DECODED_BYTES {
        return Err(ValidationError::Invalid(format!(
            "image worst-case decoded size {worst_case_bytes} exceeds maximum allowed {MAX_DECODED_BYTES} bytes"
        )));
    }

    Ok(approved_format)
}
