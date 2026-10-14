# 02: Single-Item Shell Visual Tracer

**What to build:** Replace the placeholder glyph on a single-item Shelf Batch with one complete asynchronous Windows Shell Visual path. An image file shows a bounded thumbnail; a non-image file or folder shows its Shell icon; unsupported, unavailable, denied, saturated, or failed work retains an immediate generic fallback without blocking or warning the user.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.7 — Presentation and Accessibility Conformance](../spec.md)

## Acceptance criteria

- [ ] A single-item card immediately shows a generic file or folder fallback, then replaces it with the matching Windows Shell Visual when extraction succeeds.
- [ ] Image thumbnails fit a `32 × 32` logical-pixel box at the target monitor DPI, preserve aspect ratio, and are never cropped or distorted.
- [ ] Non-image files and folders use Shell icons; video, PDF, document, and folder-content thumbnails are not requested.
- [ ] Shell work runs away from presentation and persistence work through two COM STA workers and a bounded queue of 128 requests.
- [ ] Queue saturation, extraction failure, unavailable paths, denied access, or worker failure keeps the fallback and does not emit a status message or disable any Shelf action.
- [ ] Visual bytes cross the frontend seam as binary data rather than base64 text; replacing or disposing the visual revokes its browser resource.
- [ ] Native bitmap/icon ownership is released exactly once on success, fallback, cancellation, stale completion, and error paths.
- [ ] Hiding and reopening the Drop Shelf invalidates the prior surface generation; a late result cannot appear in the reopened Shelf.
- [ ] Source files and Core Shelf State are not modified, and no application-level thumbnail cache or persisted visual data is added.

## Testing seam

Use the approved Shell Visual module interface with a deterministic extractor adapter for delay, fallback, cancellation, stale completion, saturation, and failure. Verify the tracer through the running Drop Shelf, then use the real Windows adapter with temporary image, non-image, folder, unavailable, and denied paths.

## Demo path

Open a Shelf containing one image, one ordinary file, and one folder in separate batches. Observe immediate fallbacks and eventual Shell visuals, hide/reopen during a delayed request, and confirm all pin, remove, drag-out, resize, and dismissal actions remain responsive.

## Non-goals

Multi-item stacked previews, Manage Items visuals, Edge Rail visuals, collection virtualization, cross-session visual caching, or non-image content previews.

## Answer

Delivered in 0.0.7:
- Native Shell Visual pipeline in `src-tauri/src/windows_visuals.rs` with 2 COM STA background workers, 128-request queue, and DPI-scaled 32px aspect-fit extraction using `IShellItemImageFactory::GetImage`.
- Immediate generic fallback (`▰`/`▤`) replaced asynchronously by 32bpp BMP byte buffer transfer via binary IPC and frontend Blob URL creation.
- Native handle guard (`DeleteObject`) and generational cancellation ensuring deterministic cleanup on hide/replacement.
- Verified live on running instance with real BMP image thumbnail and TXT file icon extraction.
