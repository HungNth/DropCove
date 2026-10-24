# 04: Drop Shelf accessible multiselectable grid

**What to build:** Complete Drop Shelf Batch Selection for keyboard and assistive technology. Expose a valid multiselectable grid around cards that contain independent actions, provide stable roving focus and responsive arrow navigation across virtualized rows, mirror the approved pointer selection model with Space modifiers and Ctrl+A, recover focus after mutations, announce selection counts politely, and preserve action-control, appearance, and reduced-motion behavior.

**Blocked by:** 02: Occurrence-safe mixed batch-set drag core.

**Status:** resolved

**Parent specification:** [DropCove 0.0.25 — Multi-Batch Selection and Drag-Out](../spec.md)

**Testing seam:** Actual Drop Shelf/WebView2 accessibility surface with real keyboard input and Windows accessibility inspection or a screen reader, ordinary and >50-batch responsive grids, item/batch pruning, mixed availability confirmation, and supporting Vue DOM interaction/accessibility regressions.

**Demo path:** Open the Drop Shelf through the global hotkey, traverse a multi-column collection with arrows/Home/End, form and adjust selection with Space variants and Ctrl+A, operate Pin/Manage/Remove controls, prune the focused batch, cancel a confirmation with Escape, and inspect selected states plus polite count announcements.

- [x] The Drop Shelf batch collection exposes `role="grid"` and `aria-multiselectable="true"` with an accessible name.
- [x] Each selectable Shelf Batch row exposes explicit `aria-selected="true"` or `"false"`; no selected state is inferred only from color.
- [x] Batch drag regions participate in one roving-focus model while Pin, Bulk Pinning, Manage Items, and Remove remain independent native controls.
- [x] Only the current batch drag region participates in the grid's primary tab stop; established card actions remain reachable in a predictable native Tab sequence.
- [x] Left/Right moves to the preceding/following batch in displayed row-major order.
- [x] Up/Down moves by the current responsive column count and clamps predictably when the target row is incomplete.
- [x] `Home` and `End` move to the first and last Shelf Batch across the complete collection.
- [x] Keyboard navigation scrolls virtualized targets into view and mounts the focused identity without losing selection.
- [x] Plain `Space` selects only the focused batch and sets the selection anchor.
- [x] `Ctrl+Space` toggles the focused batch and sets the anchor even when toggling it off.
- [x] `Shift+Space` replaces selection with the inclusive anchor-to-focused range and leaves a valid anchor unchanged.
- [x] `Ctrl+Shift+Space` follows the same replacement-range behavior; no additive range mode is introduced.
- [x] With no valid anchor, `Shift+Space` selects only the focused batch and establishes it as anchor.
- [x] `Ctrl+A` while the batch grid owns focus selects every Shelf Batch, including virtualized off-screen batches, without intercepting unrelated application controls or changing a valid anchor; if no anchor exists, it remains absent.
- [x] `Escape` cancels an active availability confirmation/prepared token first, then closes Manage Items, then clears Batch Selection, then dismisses the Drop Shelf on a later press.
- [x] Starting batch-set preparation from a keyboard-selected set closes Manage Items and preserves the authoritative selection until completion or a clear trigger.
- [x] Keyboard selection itself does not attempt keyboard-only native OLE destination drag; the documented global hotkey remains the path from Edge Rail to the keyboard-accessible Drop Shelf.
- [x] Pointer click on a Drop Shelf card drag region also establishes grid focus on that region without changing the approved click-versus-drag threshold behavior.
- [x] Activating Pin, Bulk Pinning, Manage Items, or Remove retains focus on the activated control while it exists.
- [x] When the focused batch is removed or pruned, focus moves to the batch now at the same displayed index, otherwise the previous batch.
- [x] When the final batch disappears, focus moves to the always-visible More action rather than the document body or a removed node.
- [x] Focus recovery remains correct when the removed/focused identity is outside the currently mounted virtual window.
- [x] Every selection change produces one coalesced polite announcement such as `3 batches, 8 items selected`; clearing produces `Selection cleared`.
- [x] Announcement counts use current selected Shelf Batch identities and Shelf Item occurrences after authoritative mutations.
- [x] Repeated rapid keyboard or pointer changes do not produce stale or overlapping count announcements.
- [x] Selected, focused, hovered, and disabled states remain visually distinguishable in Dark, Light, and Windows Contrast Themes.
- [x] Focus uses the dedicated focus resource and does not restore the removed Drop Shelf resize-hover border highlight.
- [x] Non-essential selected/focus transitions respect the existing Windows animation and reduced-motion policy.
- [x] Full paths and drag instructions remain available through tooltips and accessibility text without adding visible drag-hint subtitles.
- [x] Existing Manage Items focus cycle, Clear confirmation focus trap, Utility Menu navigation, Settings controls, and native action names remain unchanged.
- [x] Permanent frontend tests exercise real component interaction and accessibility attributes rather than source text, CSS class strings, or implementation-owned collection shapes.
- [x] Virtualization tests cover focus and range movement across mount boundaries, responsive column-count changes, removal, and collection sizes above 50.
- [x] A real installed accessibility smoke records grid/multiselectable/selected state, focus movement, Space variants, Ctrl+A, Escape ordering, focus recovery, and live announcements through Windows accessibility tooling.
- [x] Version metadata remains `0.0.24` and no Git commit is created without separate explicit user authorization.

## Verification evidence — 2026-10-24

- Real Windows WebView2 keyboard smoke selected the complete virtualized range with Home, Space, End, and Shift+Space; footer and collection row count both reported 58 batches, with focus on the last mounted batch.
- Actual accessibility tree exposed `grid` named `Shelf batches`, `multiselectable=true`, selectable `row` state, focusable `gridcell` drag regions, separate action cells, and the existing polite status channel.
- Real removal of a focused action's batch moved focus to the nearest surviving batch drag region. Pinned Managed File and external-reference mutations preserved the selected identity/count model.
- Frontend suite passed **23/23**, including Ctrl+A preserving a valid anchor, anchor removal/fallback, virtualized range identity, direct/armed confirmation dispatch, late preparation cancellation, and actual Shelf-composable Escape ordering. The Escape regression also proves an armed token is canceled before clearing selection.
- Review fixed focus/feedback details: outside-click popup dismissal no longer prevents native action-button focus; action completion restores an existing control after its temporary disabled state; armed-token cancellation clears its Ready status. These fixes passed the integrated Vue typecheck/production build and frontend suite.
- Final installed-candidate appearance/accessibility repetition and physical tooling limits are recorded in Ticket 05. No Edge Rail keyboard or live-region ownership was added.

- Installed behavior proof uses the owned NSIS `0.0.24` installation under the isolated smoke directory, embedded production assets at `http://tauri.localhost/`, and WebView2's real accessibility tree through CDP Accessibility inspection. The installed tree exposed selected rows and a multiselectable grid; native action-button focus was retained after Pin completion. Spoken screen-reader audio was not captured and is not claimed.
- Permanent card-component tests now execute actual compiled Vue SFCs and assert pointer-down/threshold/release arbitration, singleton versus multi-item batch drag, selected ARIA state, and action isolation. A real virtualization-composable regression covers off-screen focus through responsive column changes and earlier-row removal. Integrated frontend total is **26/26**.
- A fast Escape→new-selection smoke exposed a delayed IPC race. Its regression was observed red before fixing the handler to preserve selection identity across the await, then passed in the full suite. The updated unchanged-version behavior installer is rebuilt before the `0.0.25` gate; selected Contrast focus now uses HighlightText against Highlight.

- Final pre-sync NSIS installer rebuilt successfully with all **26/26 frontend tests** and Vue production build passing. The exact rebuilt installed `0.0.24` candidate proved selected Contrast background `rgb(26,235,255)` with focused outline and text `rgb(0,0,0)`, `focus-visible=true`, and reduced-motion transition `0.00001s`. Dark and Light selected palettes were also observed. Forced-colors media was emulated in the real Windows WebView2; the user's operating-system theme was not changed. The installed polite channel announced `2 batches, 2 items selected` and matched its footer count.
