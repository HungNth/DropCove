# 12: Drop into and scroll the Edge Rail

**What to build:** Complete the Edge Rail as a direct workspace by accepting new file/folder drops, expanding deliberately during drag, and remaining usable with more batches than fit vertically.

**Blocked by:** 11: Drag Shelf Items and Shelf Batches from the Edge Rail.

**Status:** ready-for-agent

- [ ] Drag-enter expands the rail after 200 ms without activating DropCove and cancels expansion when the pointer leaves before the delay.
- [ ] Drag-leave collapses the rail after 300 ms unless a flyout remains open.
- [ ] Supported file/folder paths dropped on the rail create Shelf Batches through the same application behavior seam as the Drop Shelf.
- [ ] Deduplication, mixed-payload skipping, source order, persistence, and Missing/Unavailable semantics remain identical to Drop Shelf drag-in.
- [ ] Rail height remains bounded relative to the monitor and excess Shelf Batches are reachable through natural mouse-wheel scrolling.
- [ ] The scrollbar is unobtrusive and does not interfere with direct drag initiation.
- [ ] Packaged smoke scenarios cover rail drag-in, canceled delayed expansion, multi-monitor/DPI behavior, long lists, and persisted batches.
- [ ] Completion leaves the full Edge Rail stage demoable without shake-to-open.