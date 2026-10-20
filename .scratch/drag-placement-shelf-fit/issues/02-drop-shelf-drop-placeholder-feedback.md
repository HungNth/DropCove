# 02: Drop Shelf Drop Placeholder feedback

**What to build:** Replace the Drop Shelf's full-surface drag glow with one precise Drop Placeholder at the prospective newest-first Shelf Batch position. The placeholder must work for empty, populated, scrollable, and virtualized collections while preserving native Copy feedback, Drop Shelf movement guards, resize feedback, intake behavior, and stable hover-time window geometry.

**Parent specification:** [DropCove 0.0.16 — Drag Placement Feedback and Drop Shelf Fit](../spec.md)

**Testing seam:** Actual Windows Resident Instance with the real WebView2 Drop Shelf and native OLE drag behavior, supported by existing storage eligibility regressions.

**Demo path:** Drag supported, unsupported, application-owned, multi-file, and duplicate-only payloads over empty, ordinary, scrollable, and virtualized Drop Shelves; leave, cancel, and complete each relevant path.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] An eligible external storage drag displays exactly one `48`-logical-pixel Drop Placeholder regardless of dragged item count.
- [x] The placeholder occupies visual index zero before the newest retained Shelf Batch and shifts retained cards through the same 4-pixel collection gap.
- [x] An empty Drop Shelf replaces the downward icon and `Drop here` prompt with the placeholder while the eligible drag remains over the surface.
- [x] Populated ordinary and virtualized collections account for the transient visual index without overlap, broken ordering, disabled virtualization, or an unreachable final Shelf Batch.
- [x] The placeholder has a dashed drag-hover border and a background distinct from the Drop Shelf surface using existing Dark, Light, and Windows Contrast resources.
- [x] The placeholder exposes no filename, path, Shell Visual, thumbnail, file/folder type, or item count before acceptance.
- [x] The provisional placeholder does not alter authoritative Shelf Batch count, accessibility collection size, persistence, duplicate comparison, Automatic Shelf Sizing input, or restart state.
- [x] Unsupported payloads and application-owned drag-out display no intake placeholder and retain existing rejection/isolation behavior.
- [x] A valid duplicate-only drag may show the provisional placeholder, but completion remains a silent no-op with no batch, resize, reorder, scroll, or persistence change.
- [x] Drag leave and drop completion remove the placeholder; accepted content appears once at the same newest-first position and rejected or duplicate-only completion restores the prior collection.
- [x] The Drop Shelf outer surface no longer changes to the drag-hover background, border, outline, inset shadow, or glow during storage drag.
- [x] Native Windows Copy cursor feedback remains unchanged.
- [x] The independent resize-hover accent outline remains visible and functional after drag glow removal.
- [x] The existing drag-over behavioral state remains active and continues to block Drop Shelf movement until drag leave or completion.
- [x] Hovering a placeholder does not temporarily resize or reposition the Drop Shelf; accepted content alone applies authoritative sizing.
- [x] Multi-file input still creates at most one Shelf Batch, preserves supported source order, filters unsupported entries, and reports skipped entries through existing behavior.
- [x] Source files and folders remain unchanged by every placeholder, leave, rejection, duplicate, and accepted intake path.
- [x] Real WebView2/OLE smoke covers empty, populated, scrollable, and virtualized collections; supported, unsupported, application-owned, duplicate-only, leave, and accepted paths; Dark, Light, Windows Contrast where available; and reduced-motion presentation.
- [x] The complete frontend production build and relevant native regression suite pass without adding a frontend component-test framework or screenshot snapshot suite.
- [x] The implementation reuses the existing native storage-drop eligibility/event interface and adds no second parser, duplicate preflight, persistence state, dependency, telemetry, or generalized drag-preview framework.

## Answer — observed evidence, 2026-10-20

- Real Windows Forms FileDrop/Copy input displayed one 48-pixel dashed Drop Placeholder on empty and populated Drop Shelves without surface shadow or hover-time resize. Native leave restored Drop here; unsupported text showed no placeholder. Multi-file Copy created one three-item batch.
- Application-owned native drag-out held the Shelf busy without showing an intake placeholder; rejected self-drop retained all three batches. Accepted Copy to the owned Forms destination preserved the Pinned reference and copied exact fixture bytes. Duplicate-only external Shelf drop retained batch count and removed the placeholder.
- Ordinary and virtualized WebView2 collections were exercised. At 66 retained batches, the placeholder shifted cards without overlap, authoritative accessibility set size remained 66, and the virtual final index 65 was reachable with 8 pixels of bottom clearance. Width 524 retained three columns and bounded mounting.
- Dark and Light visual checks passed. Installed forced-colors/reduced-motion emulation showed a dashed border and a system Highlight background distinct from the Rail surface; no physical Windows Contrast or screen-reader claim is made.
- Complete frontend build passed; the final complete native suite passed 113/113. Exact installed production assets with Vite stopped proved the same placeholder, no-glow, Copy intake, and sizing behavior.
- All 80 source fixtures remained byte-for-byte unchanged; no owned fixture outputs were found on the Desktop at cleanup.
