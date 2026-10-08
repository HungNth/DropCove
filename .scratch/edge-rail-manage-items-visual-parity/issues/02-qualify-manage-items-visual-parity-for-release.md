# 02: Qualify Manage Items visual parity for release

**What to build:** Produce same-payload installed Release evidence that the completed Edge Rail Manage Items presentation matches the Drop Shelf popup across supported themes, rail edges, input modes, accessibility states, scrolling, lifecycle, drag, motion, and established performance gates, then update user-facing documentation without adding permanent visual-test infrastructure.

**Blocked by:** 01: Match Edge Rail Manage Items presentation to Drop Shelf.

**Status:** ready-for-agent

**Parent specification:** [Edge Rail Manage Items Visual Parity](../spec.md)

- [ ] A clean self-contained Release build and publish completes without new warnings or errors, and the exact executable and application payload fingerprints are recorded for every qualification artifact.
- [ ] The complete existing automated suite passes on the qualified source state without weakening lifecycle, persistence, sizing, placement, drag, visual-resource, settings, or native-interop coverage.
- [ ] One isolated profile contains the approved representative Shelf Batch: image, non-image file, folder, long name, long Path Reference, Pinned and Temporary Items, and enough Shelf Items to require vertical scrolling.
- [ ] The same Shelf Batch is captured through Manage Items on the Drop Shelf and Edge Rail from the same payload, profile state, theme, and exercised DPI.
- [ ] Human-reviewed Light-mode evidence shows matching solid background, one-pixel boundary, eight-pixel corners, six-pixel padding, flat no-shadow treatment, header composition, item-row geometry, text hierarchy, row separation, and action alignment.
- [ ] Human-reviewed Dark-mode evidence from the same payload proves the same parity and confirms the Edge Rail Flyout does not revert to acrylic-style fill or default elevation.
- [ ] High Contrast is exercised only in an isolated or safely reversible environment and proves legible background/boundary separation, text, mixed Bulk Pinning state, action glyphs, and visible focus without relying only on color; no host theme state is left altered.
- [ ] Screenshots and captured bounds are stored as human-review evidence only; no pixel-baseline, screenshot-regression framework, source-text test, or permanent visual-test hook is added.
- [ ] Left-edge and Right-edge evidence shows unchanged desktop-facing placement, work-area reachability, 320-to-480 logical-pixel width, maximum 480 logical-pixel height, fixed header, vertical scrolling, and disabled horizontal scrolling.
- [ ] UI Automation verifies existing accessible names, enabled and busy state, ToggleState Off/On/Indeterminate, visible focus, Flyout ownership, and logical 24-by-24 Bulk Pinning, Pin/Unpin, and Remove Item targets at the exercised DPI.
- [ ] Pointer evidence covers normal, pointer-over, pressed, disabled, checked, and indeterminate states where applicable without changing hit targets or initiating unintended drag.
- [ ] Keyboard evidence verifies Enter and Space activation, pointer opening without forced focus, keyboard opening with initial Bulk Pinning focus, forward and reverse traversal with wrapping, on-demand off-screen row realization, Escape closure, and originating-control focus restoration.
- [ ] Lifecycle evidence verifies that an open Manage Items Flyout holds the Edge Rail expanded, pending mutations and active drag retain their guards, owner changes and removals follow existing close rules, item projections release on close, and normal collapse timing resumes afterward.
- [ ] Pin/Unpin, Bulk Pinning, and Remove Item remain manager-derived and durable; successful actions refresh the visible surface, failures leave state unchanged through existing handling, and no action modifies source filesystem objects.
- [ ] Drag evidence covers accepted and canceled direct Shelf Item drag-out and confirms Temporary consumption, Pinned retention, cancellation/rejection retention, Missing cleanup, Unavailable retention, and no accidental whole-batch drag from action controls.
- [ ] Ordinary animation evidence shows the existing Flyout open/close transition remains present, and reduced-motion evidence shows the established preference handling remains functional without a new animation contract.
- [ ] Repeated open, scroll, close, rail collapse, and reopen cycles show item visuals reload correctly and do not rely on retained hidden Shelf Item projections or ImageSource state.
- [ ] The canonical 100 Shelf Batch / 1,000 Shelf Item performance protocol remains below the established 160 MB total WorkingSet64 gate, idle CPU remains at or below 0.1%, and shelf-show latency p95 remains at or below 150 ms; UI Automation and performance measurement run separately against the same payload identity.
- [ ] No forced collection, working-set trimming, reduced fixture, alternate memory metric, process restart inside the measurement lifecycle, or automation-contaminated sample is used to manufacture a performance pass.
- [ ] User-facing documentation states that Manage Items now shares one visual language across Drop Shelf and Edge Rail while preserving surface-specific placement, ownership, focus, motion, and lifecycle behavior.
- [ ] Qualification records the Windows build, theme, contrast mode, DPI, rail edge, profile and fixture identity, observed results, failures, environmental limitations, and any unobserved configuration without inferring a pass from source declarations or older builds.
- [ ] Disposable profiles, fixtures, screenshots not retained as evidence, helper processes, and temporary qualification scripts are removed or shut down; no test scaffold, fallback, dependency, telemetry, or generated artifact remains in production code.
- [ ] All repository changes remain uncommitted until the user explicitly authorizes a Git commit.

## Testing seam

Use the installed self-contained Release with an isolated profile as the single authoritative acceptance seam for rendered WinUI parity, default-shadow suppression, theme behavior, geometry, input, accessibility, no-activate behavior, motion, drag, and resource lifetime. Keep UI Automation and visual evidence separate from canonical performance measurement while using the same payload fingerprint. The existing automated suite remains regression coverage rather than proof of rendered styling.

## Demo path

Launch the qualified Release with the representative mixed Shelf Batch. In Light and Dark modes, open Manage Items on the Drop Shelf and then on Left and Right Edge Rails, showing matched flat chrome, header, rows, scrolling, pointer states, keyboard traversal, Escape focus restoration, Pin/Unpin, Bulk Pinning, Remove Item, and one accepted and canceled Shelf Item drag. Replay the accessible-state checks, isolated High Contrast proof, reduced-motion path, repeated open/close resource cycle, and canonical performance gate from the same payload identity.
