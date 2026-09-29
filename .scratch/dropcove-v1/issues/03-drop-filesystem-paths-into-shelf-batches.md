# 03: Drop filesystem paths into Shelf Batches

**What to build:** Let users drag real files and folders from Explorer or Desktop into the unified Drop Shelf and see one ordered Shelf Batch for each accepted drop operation.

**Blocked by:** 02: Summon and manage the resident Drop Shelf.

**Status:** ready-for-agent

- [x] File and folder paths from Explorer and Desktop are accepted into the Drop Shelf without copying or changing the source filesystem objects.
- [x] Each accepted drop creates one Shelf Batch containing Shelf Items in source order, and newest Shelf Batches appear first.
- [x] Repeated paths within one drop are deduplicated while the same path dropped later creates an independent Shelf Item in a new Shelf Batch.
- [x] Local, removable, UNC/network, and cloud-backed filesystem paths are accepted when a usable path is supplied.
- [x] Stream-only virtual items are rejected and never materialized into DropCove storage.
- [x] Mixed payloads accept supported paths and visibly report the number of skipped unsupported items; payloads with no supported path create no Shelf Batch.
- [x] Every new Shelf Item is temporary by default and displays its name, type, and a usable native file/folder icon.
- [x] The bounded unified Drop Shelf renders newest-first Shelf Batch summary cards in one vertical column; single-item batches show the native icon and truncated name, while multi-item batches use a bounded stacked visual and item count.
- [x] The logical-pixel window size is `180 × 180` for zero or one Shelf Batch, `180 × 236` for two, `180 × 292` for three, and `180 × 348` for four or more. Beyond four visible batches, the same window scrolls vertically and shows `+N batches`; it does not introduce Compact or Expanded modes.
- [x] The full content surface accepts drops. Empty state shows a drop prompt; populated state uses a drag-over overlay. Accepting a batch returns the list to its newest-first position, resizes immediately with its top edge anchored, and remains clamped to the monitor work area.
- [x] Application-seam tests cover batching, source order, deduplication boundaries, newest-first ordering, and mixed-payload outcomes.
- [x] An installed-EXE smoke scenario demonstrates Explorer/Desktop drag-in, progressive bounded sizing, overflow scrolling, and newest-first rendering through the visible shelf.

## Comments

- 2026-09-29: Three Release application-seam tests pass for source order, within-drop deduplication, repeated paths across batches, mixed unsupported input, UNC paths, and unsupported-only drops.
- 2026-09-29: Real Explorer drags produced the approved size sequence `180 × 180`, `180 × 236`, `180 × 292`, `180 × 348`, then bounded `180 × 348`. UI Automation confirmed newest-first item order, `+1 batch`, a vertically scrollable viewport, and successful scrolling to 100%.
- 2026-09-29: Installed-EXE smoke accepted one real folder and four real files from Explorer, produced all approved growth steps, showed `+1 batch`, preserved newest-first order, and scrolled to 100%. Running-process uninstall then removed the process, install directory, and startup registration.
- 2026-09-29: A file dragged directly from inside an Explorer ZIP produced no Shelf Batch, retained the empty `180 × 180` shelf, and displayed `No supported filesystem paths were found.`
- 2026-09-29: Removable/cloud-backed hardware paths and a mixed Explorer payload were not available in the smoke environment. Their acceptance boxes remain open; application-seam coverage alone does not prove installed shell behavior.
- 2026-09-29: One installed Explorer drop containing two selected files created one `2 items` Shelf Batch with the source names `a.txt, b.txt`.
- 2026-09-29: Drag feedback and drop acceptance now advertise `Copy` only when the source allows `DataPackageOperation.Copy`. The rebuilt installed EXE accepted and rendered a real Explorer Copy drop.
- 2026-09-29: User completed the remaining removable/cloud-backed and mixed-payload checks and confirmed all Ticket 03 acceptance criteria.