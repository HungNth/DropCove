# 03: Edge Rail Drop Placeholder feedback

**What to build:** Extend the approved Drop Placeholder feedback to the Edge Rail. A valid external storage drag must use the existing native delayed expansion, then show one placeholder before the newest retained Shelf Batch without stealing focus, adding a surface-wide highlight, preview-resizing the Rail, or changing established Rail timing and placement behavior.

**Parent specification:** [DropCove 0.0.16 — Drag Placement Feedback and Drop Shelf Fit](../spec.md)

**Testing seam:** Actual Windows Resident Instance with the real no-activate WebView2 Edge Rail on both docking edges, supported by existing native storage eligibility and Rail timing regressions.

**Demo path:** Dock a non-empty Shelf on each edge, drag supported and unsupported storage payloads over the collapsed Rail, observe delayed expansion and placeholder lifecycle, then leave or accept the drop and verify focus, height, collapse, and retained content behavior.

**Blocked by:** 02: Drop Shelf Drop Placeholder feedback.

**Status:** resolved

- [x] A valid external storage drag over any part of the collapsed Rail retains the existing native `200 ms` expansion delay.
- [x] The frontend observes the existing window-local drag-over state and does not add a second expansion timer, drop parser, or eligibility model.
- [x] After native expansion, exactly one `48`-logical-pixel Drop Placeholder appears before the newest retained Shelf Batch.
- [x] The Edge Rail reuses the Drop Placeholder visual and transient collection-projection contract established by Ticket 02 without introducing a second design or generalized framework.
- [x] Ordinary, scrollable, and virtualized Rail collections shift retained cards without overlap, broken ordering, disabled virtualization, or an unreachable final retained Shelf Batch.
- [x] The placeholder uses a dashed drag-hover border and distinct semantic background in Dark, Light, and Windows Contrast presentation and exposes no filename, thumbnail, Shell Visual, or item count.
- [x] The placeholder remains presentation-only and does not alter Shelf Batch count, persistence, accessibility collection size, Automatic Shelf Sizing, or Edge Rail target-height input.
- [x] Drag hover adds no surface-wide Edge Rail background, border, outline, shadow, or glow.
- [x] Placeholder display and drag entry remain no-activate and do not steal foreground focus from the current application.
- [x] Drag hover does not preview-resize the Edge Rail; accepted content updates height only through the existing retained-batch formula.
- [x] The established Rail height formula, one-through-four minimum tier, bounded five-through-eleven growth, twelve-plus scrolling, and removal no-live-shrink behavior remain unchanged.
- [x] Drag leave before expansion cancels pending expansion through existing native behavior; leave after expansion removes the placeholder and retains existing `300 ms` collapse, re-entry cancellation, and transient holds.
- [x] Unsupported payloads and application-owned drag-out show no intake placeholder and preserve existing rejection/isolation behavior.
- [x] Duplicate-only completion creates no Shelf Batch and does not change Rail height, ordering, scrolling intent, persistence, or placement.
- [x] Accepted multi-file input creates at most one newest-first Shelf Batch, preserves supported source order, and replaces the transient placeholder without duplicate rendering.
- [x] Left and Right docking mirror established corners and controls while presenting the same placeholder geometry and collection behavior.
- [x] Open Shelf placement, configured/remembered monitor precedence, fullscreen policy, Manage Items, Clear Temporary Items, status ownership, source-file safety, and final-item hiding remain unchanged.
- [x] Existing native regressions continue to prove `200 ms` expansion, leave reconciliation, `300 ms` collapse, re-entry cancellation, transient holds, valid Copy eligibility, unsupported input, and application-owned drag exclusion.
- [x] Actual Windows OLE/WebView2 smoke covers both docking edges, collapsed-to-expanded drag entry, leave before and after expansion, accepted and duplicate-only drops, focus preservation, stable hover-time Rail bounds, themes, and virtualized content.
- [x] The complete frontend production build and cargo-xwin-produced Windows regression executable pass without a new frontend test framework, native timer, persistence field, setting, dependency, or compatibility path.

## Answer — observed evidence, 2026-10-20

- Reused the Drop Placeholder visual and the transient leading collection slot; Rail consumes the existing native shelf-drag-over event and unregisters its listener on disposal. No native parser, timer, eligibility model, or state mutation was added.
- Real OLE drag entry expanded both Left and Right Rails and displayed one 48-pixel dashed placeholder before retained batches. The foreground remained the owned Forms source; the Rail applied no surface shadow.
- A large virtualized Rail remained at 280 by 642 with 13 mounted cards during hover. A three-batch Rail remained at 280 by 280 while previewing a fourth visual slot. Accepted intake updated retained state; duplicate-only intake retained three batches and unchanged height.
- Leaving to the owned non-drop source removed the placeholder, returned native None, and the Rail subsequently collapsed to its 16 by 280 handle with unchanged content. Native expansion/collapse, transient-hold, and own-drag eligibility regressions passed in the complete 113/113 suite.
- The exact installed candidate, with Vite stopped, repeated real OLE feedback on both docking edges at minimum height. Production forced-colors/reduced-motion emulation retained a distinct system-color placeholder. The owned profile was returned to Right docking before shutdown.
- Physical high/mixed-DPI, actual Windows Contrast Theme, and fullscreen combinations were not re-exercised; established native policy regressions remain supporting evidence rather than physical proof.
