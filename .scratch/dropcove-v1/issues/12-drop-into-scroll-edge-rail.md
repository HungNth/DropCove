# 12: Drop into and scroll the Edge Rail

**What to build:** Complete the Edge Rail as a direct workspace by accepting new file/folder drops, expanding deliberately during drag, and remaining usable with more batches than fit vertically.

**Blocked by:** 11: Drag Shelf Items and Shelf Batches from the Edge Rail.

**Status:** ready-for-human

- [x] Drag-enter expands the rail after 200 ms without activating DropCove and cancels expansion when the pointer leaves before the delay.
- [x] Drag-leave collapses the rail after 300 ms unless a flyout remains open.
- [x] Supported file/folder paths dropped on the rail create Shelf Batches through the same application behavior seam as the Drop Shelf.
- [x] Deduplication, mixed-payload skipping, source order, persistence, and Missing/Unavailable semantics remain identical to Drop Shelf drag-in.
- [x] Rail height remains bounded relative to the monitor and excess Shelf Batches are reachable through natural mouse-wheel scrolling.
- [x] The scrollbar is unobtrusive and does not interfere with direct drag initiation.
- [x] Packaged smoke scenarios cover rail drag-in, canceled delayed expansion, multi-monitor/DPI behavior, long lists, and persisted batches.
- [x] Completion leaves the full Edge Rail stage demoable without shake-to-open.

## Implementation seam

- `StorageDropService` owns the shared Windows storage-item adapter used by both `MainPage` and `EdgeRailWindow`; `DropShelfManager.AcceptDropAsync` remains the persistence and deduplication seam.
- `EdgeRailWindow` owns only drag feedback, delayed expansion/collapse state, bounded scroll presentation, and the callback to the shared storage-drop adapter.

## Comments

- 2026-09-30: Implemented the shared storage-item drop adapter (`StorageDropService`) and Edge Rail drag-in event surface. The unit test suite passes all 46 tests, Release build has 0 warnings/errors, and Standards/Spec reviews passed cleanly.
- 2026-09-30: User confirmed full interactive smoke on the installed Release package across all scenarios: drag-enter delayed expansion after 200 ms without focus stealing, canceled delayed expansion on early leave, drag-leave collapse after 300 ms, direct drop creating Shelf Batches via shared seam, and natural mouse-wheel scrolling on long batch lists. All 8 acceptance criteria satisfied.