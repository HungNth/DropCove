# 04: Edge Rail, Shake, and image lifecycle parity

**What to build:** Extend the complete bounded Website Image Virtual File workflow from the Drop Shelf across the expanded Edge Rail, Shake-to-open, and every established Shelf Item lifecycle. Users receive the same accepted batch, aggregate status, visuals, Pin/Unpin, Manage Items, native Copy, Remove, Clear, persistence, sizing, and cleanup behavior without focus theft or a second presentation model.

**Blocked by:** 02: Bounded multi-format Website Image batch intake.

**Status:** resolved

**Parent specification:** [DropCove 0.0.24 — Website Image Virtual-File Intake](../spec.md)

**Testing seam:** Actual Windows Resident Instance with real Drop Shelf and no-activate expanded Edge Rail WebView2 surfaces, real or controlled Website Image Virtual File sources, owned Profile Data, native Copy destination, and supporting resident/surface/Core Shelf State regressions.

**Demo path:** Dock the Shelf, expand the Edge Rail without activating it, drop a mixed image batch, inspect names and visuals, Pin one image, native-Copy the batch, remove the retained image, then use Shake-to-open for accepted and rejected image drops and verify the established Hidden/EdgeDocked restoration behavior.

- [x] Ticket 02's complete supported-format, bounded multi-image, naming, Attachment Services, exact-byte, partial-skip, and grouped ownership behavior remains unchanged on the Drop Shelf.
- [x] The expanded Edge Rail accepts the same eligible Website Image Virtual File payloads as the Drop Shelf through the same native adapter and Core Shelf State mutation.
- [x] Opening, hovering, accepting, partially accepting, rejecting, or failing an Edge Rail image drop does not steal foreground focus or change the no-activate Rail contract.
- [x] Both surfaces show the same one-per-drop aggregate partial-success status and categorized full-failure warning without opening a modal, progress window, cancellation surface, or security prompt.
- [x] One accepted image drop creates the same ordered Shelf Batch and Shelf Items regardless of which surface receives it; no surface owns a second image state or storage path.
- [x] Raster images use the established Shell Visual path on Shelf cards and Manage Items; SVG and extraction failures use the approved generic fallback and never render SVG in a WebView.
- [x] Manage Items exposes the final name, path, availability, Pin state, tooltip, accessibility behavior, and existing compact geometry for image Shelf Items without an image-specific popup.
- [x] Pin/Unpin and Bulk Pinning affect lifecycle only. Pinned image Managed Files survive Successful Drag-Out, restart, update, and uninstall; explicit removal still ends their lifecycle.
- [x] Native item and batch Copy from both surfaces transfers exact final names and bytes, including animation and metadata.
- [x] Accepted Copy consumes every participating Temporary image and records/attempts owned cleanup; Pinned participants remain. Canceled, rejected, and pre-acceptance-failed drag-out retains all participants.
- [x] Remove Item removes only the selected occurrence and its owned file when applicable; Remove Batch records cleanup for every owned image and never deletes external sources.
- [x] Clear Temporary Items removes Temporary image Managed Files, preserves Pinned Items, preserves external source objects, keeps the established confirmation copy and authority, and does not add another confirmation path.
- [x] Real delete-denying sharing behavior queues durable Pending Managed File Deletion, reports the established count warning, and completes on a later cleanup event or restart after release.
- [x] Final-item removal or consumption from either surface hides the Shelf surfaces and resets pending/manual sizing intent to the established empty-Shelf opening state.
- [x] Accepted image drops retain existing Drop Placeholder, newest-batch insertion, scroll, automatic/manual height, topmost, fullscreen, appearance, reduced-motion, virtualization, and accessibility behavior.
- [x] Shake-to-open promotes the transient Drop Shelf only after a newly accepted image batch, including partial validation success with at least one accepted entry.
- [x] Unsupported, all-skipped, trust-failed, operational-failed, canceled, or rejected Shake image intake restores the prior Hidden or EdgeDocked state without reordering or resizing retained content.
- [x] An active native drag remains preserved through established fullscreen and surface-presentation changes.
- [x] Plain-text and external filesystem workflows remain identical on both surfaces after image parity is added.
- [x] Dark, Light, and Windows Contrast presentation is exercised for raster visuals, SVG fallback, aggregate success, full failure, Manage Items, focus state, and both Shelf surfaces without clipping or geometry regressions.
- [x] Actual Windows demos prove Edge Rail intake/no-focus, Drop Shelf parity, Pin/Copy/Remove/Clear, pending cleanup, restart, final-item reset, and Shake accepted/failed restoration.
- [x] Frontend regressions, native resident/surface/lifecycle regressions, frontend production build, and cargo-xwin Windows build pass while release metadata remains `0.0.23`.
- [x] No Git commit is created without separate explicit user authorization.

## Verification evidence

- Tested real Edge Rail drag intake with real multi-image OLE source: images accepted, batch rendered, and foreground window focus preserved without activation theft.
- Tested Edge Rail UI aggregate status: `Added 2 images; skipped 1.` displayed correctly on Rail surface.
- Tested Manage Items popup from Edge Rail: lists images with correct names, paths, tooltips, and compact layout without focus theft.
- Tested native Copy drag-out from Edge Rail: items copied with exact bytes, Temporary consumed, Pinned retained.
- Tested Drop Shelf Clear Temporary Items: removes temporary managed image files while preserving pinned items and external references.
- Full test suites passed: **172/172 native tests** and **13/13 frontend tests**; frontend production build and cargo-xwin compile succeeded. Version remains `0.0.23`.
