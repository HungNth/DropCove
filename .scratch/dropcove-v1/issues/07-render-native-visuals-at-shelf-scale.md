# 07: Render native visuals at shelf scale

**What to build:** Make large shelves readable and responsive with native icons, lazy image thumbnails, and virtualization while keeping derived content out of DropCove persistence.

**Blocked by:** 03: Drop filesystem paths into Shelf Batches.

**Status:** ready-for-agent

- [ ] Files and folders use native Windows icons appropriate to their current path and type.
- [ ] Image Shelf Items request Windows Shell thumbnails lazily for visible content.
- [ ] Thumbnail failure or unavailability falls back to the native file icon without blocking interaction.
- [ ] DropCove does not maintain a persistent thumbnail-byte cache or store thumbnail bytes in SQLite.
- [ ] Thumbnail acquisition is cancellable or discardable when an item leaves the visible range and does not block the UI thread.
- [ ] The unified shelf uses virtualization or equivalent bounded realization for at least 100 Shelf Batches and 1,000 Shelf Items.
- [ ] Scrolling, pinning, removal, and drag initiation remain responsive at the required scale.
- [ ] Automated tests cover fallback and lifecycle behavior at the highest non-Shell seam; packaged smoke/performance checks exercise real Shell thumbnails and scale.