# 01: Compact Drop Shelf Surface

**What to build:** Refine the Drop Shelf into a compact borderless surface whose rounded background and border align flush with the transparent window frame. Remove the persistent title/subtitle header, reduce the empty state to a downward icon and `Drop here`, and expose the existing confirmed Clear Temporary Items workflow through a conditional bottom-left trash icon. Establish the dependency-free inline-SVG visual primitive used by later compact controls without changing Shelf behavior.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.10 — Compact Drop Shelf and Edge Rail UI Refinement](../spec.md)

## Acceptance criteria

- [x] All four Drop Shelf corners render as one continuous rounded surface with no transparent interior gap, opaque outer-corner artifact, doubled border, or mismatched background.
- [x] The frame treatment remains correct in Dark, Light, and Windows Contrast presentation and during normal, drag-over, resize-hover, status, confirmation, and disabled-control states.
- [x] The `DropCove` title and `References, not copies` subtitle are absent in every empty and populated Drop Shelf state.
- [x] The empty state contains only a downward icon and the exact visible text `Drop here`; the prior explanatory lines are removed.
- [x] Clear Temporary Items is represented by a trash icon at the bottom-left and is rendered only while at least one Temporary Item exists.
- [x] Activating the trash icon opens the existing confirmation. Cancel, Escape, persistence failure, mixed Temporary/Pinned clearing, and final-item clearing preserve the established lifecycle and source-file safety contracts.
- [x] The icon button uses a dependency-free inline SVG inheriting `currentColor`, a native `title` tooltip, an action-specific accessible name, pointer hover treatment, visible `focus-visible` treatment, disabled state, and Windows Contrast system colors.
- [x] A small shared icon visual primitive may own SVG sizing and decorative behavior only. Command invocation, disabled rules, accessible state, and action semantics remain at the call site.
- [x] Existing Drop Shelf resize hit testing, eight-direction cursors, preferred/manual sizing intent, responsive grid, drag/drop, status, and keyboard behavior remain unchanged.
- [x] Presentation fixes remain in the existing frontend surface composition unless the running Windows application proves a native frame/clipping change is required.
- [x] Package, crate, Tauri, PE, and installer metadata remain `0.0.9`.

## Testing seam

Use the actual Drop Shelf in the running Windows Resident Instance. Inspect real transparent-window corners and operate the empty, populated, drag-over, resize, status, and Clear confirmation states with pointer and keyboard in Dark, Light, and Windows Contrast presentation. Run the existing frontend production build; do not add source-text, CSS-selector, SVG-path, or component-framework tests.

## Demo path

Open an empty Drop Shelf and show the flush four-corner frame plus the `↓` / `Drop here` prompt. Add mixed Temporary and Pinned references, show the bottom-left trash icon, cancel once, confirm once, and prove Pinned references and source files remain. Exercise resize hover and keyboard focus in all three presentation modes.

## Non-goals

Shelf state changes, persistence changes, new confirmation content, Manage Items controls, Shelf Batch action conversion, Edge Rail refinement, Settings changes, custom tooltip infrastructure, version synchronization, packaging, or Git commit creation.

## Answer

- Implemented compact empty/header-free Shelf, shared decorative SVG, transparent root canvas, no native Shelf shadow, and conditional bottom-left Clear icon. Real empty-state smoke observed `Drop here`, no header, and transparent body; final candidate desktop capture inspected actual corners.
- `pnpm build` passed. Real Shelf Clear opened the existing confirmation; Cancel retained all 65 batches. Confirmed Rail Clear retained exactly the three expected Pinned IDs, removed only Temporary references, and hid the trash control. All 75 fixture source bytes remained unchanged.
- Dark/Light/forced-colors rendering was inspected through native WebView2 media emulation. Actual Windows Contrast/theme matrix, every resize/error/disabled state, and full UI Automation acceptance remain unchecked rather than inferred.
- Metadata remained 0.0.9 during implementation; Ticket 04 produced the 0.0.10 acceptance candidate. Final code review found no remaining source defect. No commit created.