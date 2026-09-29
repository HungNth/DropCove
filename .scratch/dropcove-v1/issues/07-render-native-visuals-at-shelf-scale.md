# 07: Render native visuals at shelf scale

**What to build:** Make large shelves readable and responsive with native icons, lazy image thumbnails, and virtualization while keeping derived content out of DropCove persistence.

**Blocked by:** 03: Drop filesystem paths into Shelf Batches.

**Status:** in-progress

- [x] Files and folders use native Windows icons appropriate to their current path and type.
- [x] Image Shelf Items request Windows Shell thumbnails lazily for visible content.
- [x] Thumbnail failure or unavailability falls back to the native file icon without blocking interaction.
- [x] DropCove does not maintain a persistent thumbnail-byte cache or store thumbnail bytes in SQLite.
- [x] Thumbnail acquisition is cancellable or discardable when an item leaves the visible range and does not block the UI thread.
- [x] The unified shelf uses virtualization or equivalent bounded realization for at least 100 Shelf Batches and 1,000 Shelf Items.
- [ ] Scrolling, pinning, removal, and drag initiation remain responsive at the required scale.
- [x] Automated tests cover fallback and lifecycle behavior at the highest non-Shell seam; installed unpackaged-EXE smoke/performance checks exercise real Shell thumbnails and scale.

- 2026-09-29: Added a non-Shell `ShelfVisualCoordinator<TVisual>` seam with tests for native-icon and image-thumbnail selection, independent request timing, fallback after thumbnail unavailability, uncached failures, and cancellation propagation.
- 2026-09-29: Added real Windows Shell icon extraction through `SHGetFileInfo`/GDI in `DropCove.Native`; image previews use `StorageItem.GetThumbnailAsync(ThumbnailMode.PicturesView)` independently of native icon extraction, with the native icon retained as fallback.
- 2026-09-29: Replaced eager `ItemsControl` realization and `Task.WhenAll` visual loading with nested `ItemsRepeater`/`StackLayout` virtualization. Element prepared/clearing events retain only visible request lifetimes; no thumbnail or icon byte cache is persisted.
- 2026-09-29: Release build passed with zero warnings/errors and the five Ticket 07 coordinator tests passed. The native icon smoke returned 32×32 BGRA images for a real file, folder, and image.
- 2026-09-29: Seeded the actual Release EXE with 100 real SQLite Shelf Batches and 1,000 Shelf Items. UI Automation observed a `180 × 348` shelf with `+96 batches`, five visible batch groups, and expansion exposing `SmokeFile.txt`, `SmokeFolder`, and `SmokeImage.png` while the remaining batches stayed virtualized.
- 2026-09-29: The installed Release EXE smoke then confirmed real visual results through UI Automation: `SmokeFile.txt native icon`, `SmokeFolder native icon`, and `SmokeImage.png thumbnail`. The 100-batch/1,000-item shelf was scrolled to its end (the top image visual left the realized tree), then returned to the top; pinning `SmokeFile.txt` updated the card to `1 pinned`, and removing `SmokeFolder` removed its item row.
- 2026-09-29: Drag initiation was attempted from a visible `Bulk01.txt` Shelf Item toward Explorer, but the destination did not accept the OLE drop and the source remained. Criterion 15 therefore remains open; no drag responsiveness claim is made. The original shelf database was restored and no `DropCove-T7-*` smoke artifacts remain.