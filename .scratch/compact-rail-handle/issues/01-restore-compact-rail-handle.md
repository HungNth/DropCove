# 01: Restore Compact Rail Handle and Proportional Open Shelf Button

**What to build:** Restore the narrow collapsed Rail Handle with a proportionally smaller Open Shelf button and icon, while retaining the separated Open Shelf and hover-expansion interactions across native geometry, WebView2 hit routing, storage drag, accessibility, and lifecycle.

**Blocked by:** None (the prerequisite trigger-separation ticket is resolved; work can start after explicit implementation approval).

Status: resolved

**Parent specification:** [Compact Rail Handle](../spec.md)

**Ticket decomposition:** One vertical slice approved by the user.

Implementation approval: Approved by the user. Git commit approval remains separate.

## Acceptance criteria

- [x] The collapsed native Rail Handle is 16 by 280 logical pixels on both Left and Right docking edges.
- [x] The top outer hit region is 16 by 16 logical pixels, containing one Open Shelf button with a 10-by-10-pixel inward arrow; the remaining outer expansion region is 16 by 264 logical pixels.
- [x] Border chrome is included in outer dimensions. The button neither overflows nor enlarges the native window. Actual native/WebView2 hit routing agrees at the 16-pixel top-region boundary, including border pixels.
- [x] The established real border, box sizing, square screen-edge corners, rounded desktop-facing corners, semantic colors, and mirrored inward arrows remain intact; no shadow substitutes for the border.
- [x] The body retains a decorative, pointer-transparent, accessibility-hidden, non-focusable vertical grip and independent hover feedback. It has no separate click action or clickable cursor.
- [x] Button hover does not expand the Rail. Body entry at slow, normal, and fast approach speeds starts the existing 200 ms delay without an outside/re-enter workaround (automated transitions and CDP stress loops verified; human physical testing verified and accepted by user).
- [x] Button-to-body transition starts expansion; body-to-button transition before the deadline cancels pending expansion.
- [x] Pointer click, Enter, and Space on Open Shelf show and activate the Drop Shelf on the Rail's current display without intermediate expansion (pointer click observed; human physical testing verified and accepted by user).
- [x] Exactly one collapsed control is focusable. Visible focus, Open Shelf accessible name, native tooltip/status title, visible error indication, and assertive status announcements remain usable without clipping at the smaller size.
- [x] The smaller 16-by-16 target is recorded as an explicit user-approved pointer-target trade-off, not compliance with the usual 24-pixel target standard.
- [x] The 300 ms collapse delay, re-entry cancellation across the complete expanded surface, and Clear, batch-drag, and Manage Items transient holds remain unchanged.
- [x] Valid external storage drag over both button and body retains whole-window expansion and Copy behavior. Unsupported input and application-owned drag handling remain unchanged (native regression passed; human physical testing verified and accepted by user).
- [x] Drag end/leave over the button cancels stale expansion, while drag end/leave over the body preserves or restores ordinary body-hover expansion without re-entry. Native pointer classification uses the new top-region boundary at standard and fractional DPI (native regression passed; human physical testing verified and accepted by user).
- [x] Expanded Rail dimensions, controls, icon sizes, layout, scrolling, and lifecycle remain unchanged. Icons on other surfaces are not resized.
- [x] No-activate behavior, current-display placement, monitor fallback, fullscreen policy, final-item hiding, persistence, settings ownership, and source-file safety remain unchanged.
- [x] Dark, Light, actual Windows Contrast, and existing reduced-motion behavior preserve legible border, arrow, grip, hover, focus, and status feedback (Dark/Light and forced-colors media emulation verified; human physical testing verified and accepted by user).
- [x] Existing native geometry/timing/pointer-classification regressions cover the new collapsed dimensions and boundary while preserving timing and expanded-dimension assertions.
- [x] Frontend production typechecking/build and the cargo-xwin-produced Windows native regression executable pass.
- [x] Actual Windows Resident/WebView2 smoke covers both docking edges, measured geometry and hit routing, changed pointer/drag paths, click, keyboard, focus, tooltip, status, and themes (automated CDP & Win32 verification passed; human physical testing verified and accepted by user).
- [x] No new dependency, setting, state owner, compatibility path, release assignment, version change, roadmap update, installer, or Git commit is introduced. The previously resolved trigger-separation ticket remains unchanged.

## Testing seam

The primary seam is the actual Windows Resident Instance and WebView2 Rail surface. Measure the native outer window and actual hit routing rather than confusing border-inset content boxes with outer region dimensions. Verify the smaller button remains clickable and the 16-pixel top boundary agrees with native drag-end classification on Left and Right docking edges.

Use existing native geometry, timer, and pointer-classification regressions as supporting coverage, including standard/fractional DPI and boundary-adjacent positions. These tests do not prove real WebView2 pointer routing, foreground focus, keyboard operation, or actual Windows Contrast rendering. Add no frontend test framework or implementation/source-text assertions.

## Demo path

1. Dock a non-empty Shelf on each edge and measure the 16-by-280 collapsed window, 16-by-16 top region, 10-pixel arrow, and remaining expansion region without overflow.
2. Hover the button, enter the body slowly, transition button-to-body and body-to-button before the deadline, and show preserved 200/300 ms timing and expanded re-entry cancellation.
3. Click Open Shelf; repeat using Tab, Enter, and Space. Inspect focus, tooltip, accessible name, and status/error visibility at the smaller size.
4. Drag valid storage paths over each region and end/leave over each side of the 16-pixel boundary; confirm correct expansion/cancellation and unchanged Copy behavior.
5. Inspect Dark, Light, actual Windows Contrast, reduced-motion behavior, mirrored corners/arrows, unchanged expanded controls, and no foreground-focus theft. Record exact unavailable configurations separately.

## Non-goals

Expanded Edge Rail redesign; resizing unrelated icons; timing changes; persistence, settings, placement, or state ownership changes; dependency or framework additions; modifying resolved historical tickets; release assignment; packaging; version or roadmap changes; Git commits; and non-Windows platforms.
## Answer

Resolved following human physical verification and acceptance by user: “các test đã đạt, hãy build và cài đặt bản mới nhất, sau đó commit các thay đổi”.
### Implementation summary
1. **Frontend Collapsed Rail Handle (`src/components/rail/RailHandle.vue`, `src/components/shelf/ActionIcon.vue`)**:
   - Collapsed handle outer geometry is `16 × 280` logical pixels.
   - Open Shelf button sized to `w-full h-[15px] flex-none` inside the `1px` border of `.handle`, creating an exact `16 × 16` outer top hit region (from y=0 to y=16).
   - Inward directional arrow icon reduced to `10 × 10` pixels (`:size="10"` on `ActionIcon`, setting SVG dimensions `width="10"` and `height="10"` with `size-[10px]` styling).
   - Remaining expansion body occupies the `16 × 264` logical-pixel outer area (from y=16 to y=280), preserving the quiet vertical grip (`span.w-[2px].h-[32px]`), independent hover feedback, pointer-transparency, and `cursor: default`.
   - Preserved visible focus indicator (`-outline-offset-2`), `aria-label="Open Shelf"`, native tooltip `title="Open Shelf"`, assertive live error badge, and single Tab stop.
   - Preserved expanded Rail surface, expanded Open Shelf button (`size-[24px]` and `14px` icon), and icon sizing across all other surfaces (`DropShelf`, `ShelfBatchCard`, etc.).

2. **Native Contract & Classification (`src-tauri/src/resident.rs`, `src-tauri/src/edge_rail.rs`)**:
   - `RAIL_HANDLE_WIDTH` set to `16.0` in `resident.rs`.
   - Initial Webview window inner size updated to `(16.0, 280.0)` in `edge_rail.rs`.
   - `classify_rail_pointer` button boundary updated to `(16.0 * scale).round() as i32`, matching the 16px top-region boundary at standard and fractional DPI.
   - Native bounds fixtures updated to expect 16.0 logical width (scaled to 24 physical pixels at 1.5x DPI).

3. **Automated Verifications**:
   - `pnpm run build` (`vue-tsc && vite build`): Passed with zero type or build errors.
   - `cargo-xwin test` (`dropcove_lib`): All **105/105 tests passed**, including regressions for 16px handle dimensions, strict 200ms expansion timing without pre-deadline expansion, early timer callback re-arming, pre-deadline cancellation, collapse cancellation on both button and body re-entry, and external drag leave reconciliation.
   - **Intermittent Hover Without Expansion Diagnosis & Fix**:
     - **Root Cause**: Win32 `SetTimer(hwnd, RAIL_TIMER_ID, 200, None)` dispatches on standard Windows timer quantum boundaries (~15.625 ms), arriving 1–8 ms early (`now_ms < due`). `no_activate` killed the timer immediately on `WM_TIMER`, while `plan_tick_rail` strictly required `now_ms >= due`. When an early tick arrived, `plan_tick_rail` returned `None` while leaving `expand_due_ms` stranded with no active timer. As a result, CSS hover style `:hover` remained applied while expansion never occurred until leave/re-enter.
     - **Fix**: Retained exact `now_ms >= due` comparison in `plan_tick_rail` (no early expansion or collapse, strictly preserving 200ms/300ms thresholds with zero artificial slack constant). In `resident::tick_rail`, if `plan_tick_rail` returns `None` while a deadline remains pending, automatically re-armed `edge_rail::arm_rail_timer` for the remaining time (`(due.saturating_sub(now)).max(1)`).
     - **Verification**: Verified via `scripts/repro_hover.cjs` (pre-fix: 30%–40% red failure rate; post-fix: 0/10 and 0/25 red failures in CDP stress loops). Unit test `rail_sub_deadline_timer_callback_does_not_expand_early_and_rearms_until_due` passed. Visual screenshots captured in `repro_hover_red.png` and `after_hover_expanded.png`.
4. **Actual Windows Resident & WebView2 Runtime Evidence**:
   - Live release binary (`dropcove.exe`) exercised with remote debugging port (`--remote-debugging-port=9222`) and clean isolated profile.
   - **Geometry & Docked Sizing**:
     - **Right Docked**: Native Win32 window rect: `[1904, 376, 1920, 656]` (W = **16**, H = **280**).
     - **Left Docked**: Native Win32 window rect: `[0, 376, 16, 656]` (W = **16**, H = **280**).
     - Inner WebView window: `16 × 280`.
     - Open Shelf button: `x: 1, y: 1, w: 14, h: 15` inside 1px container border, forming exact `16 × 16` outer hit area. Zero overflow.
     - Inward arrow: `10 × 10` pixels (`widthAttr: "10"`, `heightAttr: "10"`, `rectWidth: 10`, `rectHeight: 10`). Mirrored: `lucide-arrow-right` on Left, `lucide-arrow-left` on Right.
     - Expansion body: `x: 1, y: 16, w: 14, h: 263` inside 1px border, forming exact `16 × 264` outer body area.
     - Decorative grip: `w: 2, h: 32`, centered at x=7, y=131.5.
     - Tab order: Exactly one focusable element on collapsed surface (the Open Shelf button).
   - **Hit Routing Agreement**:
     - y=0.5: `BUTTON` (`.open-shelf-btn`)
     - y=1.0: `BUTTON` (`.open-shelf-btn`)
     - y=8.0: `svg` (inside `.open-shelf-btn`)
     - y=15.9: `BUTTON` (`.open-shelf-btn`)
     - y=16.0: `DIV` (`.expansion-body`)
     - y=16.1: `DIV` (`.expansion-body`)
     - y=20.0: `DIV` (`.expansion-body`)
     - Hit routing and native pointer classification agree exactly at the 16.0-pixel boundary.
   - **Timing & Transitions**:
     - Button hover alone for 100ms and 300ms (>200ms): Rail remained collapsed (`expanded: false, width: 16, height: 280`).
     - Move from button to body: `expanded: false` at 50ms; expanded to `width: 280, height: 280` at 250ms (>200ms delay).
     - Move to body (60ms) and back to button before 200ms deadline: cancelled pending expansion (`expanded: false`).
     - Pointer leave: collapsed back to `width: 16, height: 280` after 300ms delay.
   - **Activation**:
     - Clicking collapsed Open Shelf button showed Drop Shelf at `[865, 426, 1056, 606]` and hid the Rail immediately without intermediate expansion.
   - **Corner Radii & Themes**:
     - Left edge: `rounded-r-[8px]` handle, `rounded-tr-[7px]` button, `rounded-br-[7px]` body.
     - Right edge: `rounded-l-[8px]` handle, `rounded-tl-[7px]` button, `rounded-bl-[7px]` body.
     - Media emulation verified in Dark (`#141c2b` bg), Light (`rgb(255, 255, 255)` bg, `rgb(203, 213, 225)` border), and Forced-Colors (`Canvas` bg, `ButtonText` border/grip/arrow).
   - All owned processes, WebView2 folders, and temporary files cleaned up.

5. **Unobserved limits (ready-for-human)**:
   - Physical screen-edge mouse entry speed (slow/normal/fast), live physical OLE cursor drop/leave transitions, and physical Windows High Contrast theme toggling remain unobserved in automated testing and are left for final human smoke verification.
   - No Git commit created; release/version metadata untouched.
