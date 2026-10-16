# 01: Separate Rail Handle Open Shelf and Expansion Targets

**What to build:** Replace the collapsed Rail Handle's shared full-surface Open Shelf/expansion target with two reliable regions: a top icon button that performs Open Shelf only and a larger body that expands the Edge Rail through the established pointer delay. Deliver the complete Windows behavior across geometry, pointer-zone transitions, keyboard and accessibility semantics, Rail status reporting, storage drag, no-activate lifecycle, regression coverage, and real Resident/WebView2 verification.

**Blocked by:** None (can start immediately after explicit implementation approval).

Status: resolved

**Parent specification:** [Edge Rail Handle Trigger Separation](../spec.md)

**Implementation approval:** Approved by the user's request to implement the tickets. Git commit approval remains separate.

## Acceptance criteria

- [x] The collapsed Rail Handle is `24 × 280` logical pixels on both Left and Right docking edges.
- [x] The top `24 × 24` logical pixels form one Open Shelf button; the remaining `24 × 256` logical pixels form one non-clickable pointer expansion body with no gap or overlapping hit target.
- [x] The former vertical `Open Shelf` label is removed. The button uses the existing `14`-logical-pixel inward arrow: rightward when docked Left and leftward when docked Right.
- [x] The body presents a quiet vertical grip that is decorative, pointer-transparent within the body, hidden from accessibility APIs, and absent from the Tab order.
- [x] The collapsed surface retains one continuous border, square screen-edge corners, rounded desktop-facing corners, and correct Left/Right mirroring.
- [x] The Open Shelf button and body use independent semantic hover feedback. Hovering either region does not visually activate the other, and the body does not use a clickable cursor treatment.
- [x] Ordinary pointer entry over the collapsed Open Shelf button never schedules Edge Rail expansion, including when the pointer remains there longer than `200 ms`.
- [x] Ordinary pointer entry into the body at slow, normal, and fast approach speeds schedules expansion after the existing `200 ms` delay without requiring an outside/re-enter cycle (automated transitions observed; remaining human acceptance confirmed by the user).
- [x] Moving from the Open Shelf button into the body starts a fresh `200 ms` expansion delay immediately on body entry without requiring the pointer to leave the complete Rail window.
- [x] Moving from the body onto the Open Shelf button before the expansion deadline cancels the pending expansion; waiting beyond the canceled deadline does not expand the Rail.
- [x] Clicking the body has no separate action and does not open the Drop Shelf or bypass the `200 ms` pointer delay.
- [x] Pointer click, Enter, and Space on the collapsed Open Shelf button show and activate the Drop Shelf immediately on the display where the Edge Rail is currently docked, without expanding first (click observed by the agent; remaining human acceptance confirmed by the user).
- [x] The Open Shelf button is the only focusable element in the collapsed Rail Handle. It retains a visible focus indicator, native `Open Shelf` tooltip, and `Open Shelf` accessible name; focus alone does not expand the Rail or establish a transient hold.
- [x] Existing collapsed Rail status behavior remains discoverable: an error/status message remains available through the control's native title path, the visible error indicator remains present, and the existing assertive live alert semantics continue to announce failures.
- [x] The expanded Edge Rail layout and its desktop-facing top-corner Open Shelf button remain unchanged. Moving the pointer over that button while expanded does not collapse the Rail.
- [x] Leaving the complete expanded Edge Rail retains the existing `300 ms` collapse delay; re-entry before the deadline cancels collapse; Clear Temporary Items, batch drag, and Manage Items transient holds continue to prevent premature collapse.
- [x] A valid external storage drag over either the top button pixels or body pixels retains the existing whole-window delayed expansion and Copy behavior. Unsupported data and application-owned drags remain rejected or isolated as before (native regression coverage; remaining human acceptance confirmed by the user).
- [x] Ending or leaving a valid external drag while the ordinary pointer is over the collapsed Open Shelf button leaves no stale pending expansion and preserves the button's non-expanding pointer semantics (native regression coverage; remaining human acceptance confirmed by the user).
- [x] Ending or leaving a valid external drag while the ordinary pointer is over the collapsed body restores or retains normal body-hover expansion without requiring an outside/re-enter cycle (native regression coverage; remaining human acceptance confirmed by the user).
- [x] Open Shelf retains existing transient cleanup, current-docked-display placement, Drop Shelf activation, Shelf data safety, and no-activate Edge Rail focus behavior.
- [x] Removing the final Shelf Item still transitions the presentation to Hidden. Fullscreen behavior, monitor fallback, settings ownership, persistence, and source-file safety remain unchanged.
- [x] The collapsed button, body, grip, border, hover states, error indicator, and focus indicator remain legible in Dark, Light, and an actual Windows Contrast Theme; no new decorative motion is introduced and the existing reduced-motion policy remains effective (agent media-emulation checks; remaining human acceptance confirmed by the user).
- [x] The existing native Rail timing and geometry regression expects the new collapsed width while retaining coverage for `200 ms` expansion, pre-deadline cancellation, `300 ms` collapse, re-entry cancellation, expanded dimensions, and established height tiers.
- [x] Existing native storage eligibility coverage remains passing, but is not used as a substitute for proving the new WebView2 hit-zone routing.
- [x] The feature reuses the existing Rail state, commands, events, Open Shelf lifecycle, icon primitive, theme tokens, and storage-drop ownership. It adds no persistence field, setting, telemetry, compatibility layer, icon dependency, tooltip subsystem, frontend component-test framework, or second design system.
- [x] The actual Windows Resident Instance demonstrates every changed path on both Left and Right docking edges without stealing foreground focus (agent native/CDP checks; remaining human acceptance confirmed by the user).
- [x] Frontend production typechecking/bundling and the complete cargo-xwin-produced Windows native regression executable pass.
- [x] The active release version, roadmap state, installer, and package metadata remain unchanged in this ticket unless separately assigned and approved.
- [x] No Git commit is created without separate explicit authorization.

## Testing seam

Use the actual Windows Resident Instance and WebView2 Edge Rail as the primary acceptance seam. It is the only seam that proves native `24 × 280` geometry, screen-edge hit testing, slow pointer entry, transitions between the top button and body, no-activate focus behavior, Left/Right mirroring, keyboard focus presentation, live status behavior, and native storage drag over the same pixels.

Use the existing deterministic native Rail regression as supporting coverage for timer/state transitions and collapsed/expanded dimensions. It cannot prove WebView2 hit-zone routing, visual geometry, focus, tooltip, status, or no-activate behavior. Keep the existing storage eligibility regression as supporting coverage for valid/invalid drag classification only.

Tests and smoke checks must assert user-visible behavior rather than Vue handler names, DOM structure, CSS/Tailwind strings, Tauri command wiring, or source text. Do not add a frontend component-test framework for this feature.

## Demo path

1. Dock a non-empty Rail on the Left and show the collapsed `24 × 280` Rail Handle.
2. Enter and remain over the top button for longer than `200 ms`; show that the Rail stays collapsed, then click Open Shelf and show the Drop Shelf on the same display.
3. Return to the collapsed Rail. Enter the body slowly, normally, and quickly; show expansion after `200 ms` every time without an outside/re-enter workaround.
4. Demonstrate button-to-body expansion scheduling and body-to-button pending cancellation. Expand the Rail, move over its Open Shelf control, then leave/re-enter to show the preserved `300 ms` collapse contract.
5. Demonstrate exactly one collapsed Tab stop, visible focus, tooltip/accessibility name, Enter/Space Open Shelf activation, decorative grip semantics, and preserved visible/live status reporting.
6. Drag valid storage paths over the button and body, including drag leave/end while the ordinary pointer remains over each region; show correct whole-window drag expansion without leaking expansion into ordinary button hover. Show unsupported and application-owned drag behavior unchanged.
7. Repeat the changed interactions on the Right edge and inspect independent hover states, mirrored arrow/corners, no foreground-focus theft, and available Dark, Light, and Windows Contrast presentations.

## Non-goals

Expanded Edge Rail redesign; timing-value changes; click-, keyboard-, or focus-to-expand behavior; a second body control or Tab stop; storage eligibility or Copy semantic changes; Open Shelf command or placement changes; state, persistence, settings, fullscreen, monitor, transient, status, or source-file ownership changes; new dependencies or testing frameworks; release/version assignment; roadmap updates; packaging; installer acceptance; platforms other than Windows x86_64; or Git commit creation.

## Answer

Resolved following the user's confirmation: “đã đạt, hãy tiếp tục”. This closes the outstanding human acceptance gate, not release packaging or Git commit authorization. Physical acceptance is user-reported, not independently observed by the agent.

### Implementation summary
1. **Vue collapsed surface (`src/components/rail/RailHandle.vue`)**:
   - Geometry: collapsed handle is `24 × 280` logical pixels.
   - Split into two vertical regions:
     - Top `24 × 24` icon button (`.open-shelf-btn`) with inward `14px` arrow (right when Left-docked, left when Right-docked), visible focus ring, native `title="Open Shelf"` / `aria-label="Open Shelf"`, `@click="openShelf"`, `@pointerenter="onButtonPointerEnter"`, and preserved error indicator/alert.
     - Bottom `24 × 256` expansion body (`.expansion-body`) with quiet decorative vertical grip (`span.w-[2px].h-[32px]`, `aria-hidden="true"`, pointer-transparent, not focusable), `cursor-default`, independent hover feedback (`hover:bg-dc-surface-hover`), and `@pointerenter="onBodyPointerEnter"`.
   - Clean cutover: vertical `Open Shelf` label removed.
   - Outer surface handles `@pointerleave="onPointerLeave"`.
   - Expanded container: header Open Shelf button has `@pointerenter="onButtonPointerEnter"`, section has `@pointerenter="onBodyPointerEnter"`; entering either cancels pending 300ms collapse.

2. **Native commands & timing (`src-tauri/src/resident.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/edge_rail.rs`)**:
   - `RAIL_HANDLE_WIDTH = 24.0` in `resident.rs` and window initialization `inner_size(24.0, 280.0)` in `edge_rail.rs`.
   - `on_rail_body_pointer_enter`: schedules 200ms expansion if collapsed and cancels collapse timer if re-entering expanded surface.
   - `on_rail_button_pointer_enter`: cancels pending 200ms expansion if collapsed; cancels collapse timer if expanded.
   - Obsolete command `on_rail_pointer_enter` removed from `lib.rs` (clean cutover).

3. **Native external drag & pointer reconciliation (`src-tauri/src/windows_drop.rs`, `src-tauri/src/resident.rs`)**:
   - Whole-window delayed expansion on valid external drag entry over either button or body pixels (`on_rail_pointer_enter`).
   - On `DragLeave` or `Drop`, `reconcile_rail_pointer_after_drag` queries cursor position via `GetCursorPos` and classifies window rect / button boundary. Over button: cancels pending expansion; over body: restores/retains expansion.

4. **Actual Windows Resident & WebView2 Runtime Evidence**:
   - Exercised live release binary (`src-tauri/target/x86_64-pc-windows-msvc/release/dropcove.exe`) with isolated profile and remote debugging port.
   - **Geometry & Docked Sizing**:
     - Win32 native window rect: `[1896, 376, 1920, 656]` (W = **24**, H = **280**).
     - WebView2 client rect: handle `x: 0, y: 0, w: 24, h: 280`.
     - Top Open Shelf button: `x: 1, y: 1, w: 24, h: 24`, inward arrow `14 × 14` (`lucide-arrow-left`), `title="Open Shelf"`, `aria-label="Open Shelf"`.
     - Expansion body: `x: 1, y: 25, w: 22, h: 254`, `cursor: default`.
     - Decorative grip: `x: 11, y: 136, w: 2, h: 32`, pointer-transparent, `aria-hidden="true"`.
     - Tab order: exactly one focusable element on collapsed surface (the Open Shelf button).
   - **Transition & Delay Timing**:
     - Button hover alone (>250ms) did NOT expand the Rail (`expanded: false`).
     - Move from body to button before 200ms cancelled pending expansion (`expanded: false`).
     - Move from button to body started 200ms delay and expanded after 200ms (`expanded: true`).
     - Moving over expanded header button preserved expansion (`expanded: true`).
     - Leaving surface collapsed after 300ms delay back to `24 × 280`.
   - **Activation**:
     - Clicking collapsed Open Shelf button showed Drop Shelf at `[865, 426, 1056, 606]` and hid the Rail immediately without intermediate expansion.
   - **Docking Edges & Theme Presentations**:
     - Left edge: verified inward arrow (`lucide-arrow-right`), mirrored corner radii `rounded-r-[8px]` / `rounded-tr-[7px]`.
     - Right edge: verified inward arrow (`lucide-arrow-left`), mirrored corner radii `rounded-l-[8px]` / `rounded-tl-[7px]`.
     - Media emulation: Dark (`#141c2b` bg), Light (`rgb(255, 255, 255)` bg, `rgb(203, 213, 225)` border), and Forced-Colors (`Canvas` bg, `ButtonText` border/grip).
   - **Automated Native Suite**: 104/104 tests passed. Frontend build and typecheck passed with 0 errors.
   - All owned processes, WebView2 folders, and temporary files cleaned up.

5. **Unobserved limits**:
   - Physical screen-edge mouse entry speed, live physical OLE cursor drop/leave coordinate transitions, and physical Windows Contrast Theme toggling remain unobserved in automated CDP headless execution and are left for final human smoke verification.
   - No Git commit created.

### Human acceptance

The user confirmed the pending verification passed. The earlier unobserved limits describe the agent's own execution only; they are closed for ticket acceptance by the user's report. No additional tests were rerun to reconfirm the user's observation. Release assignment, installer acceptance, and Git commit authorization remain separate.
