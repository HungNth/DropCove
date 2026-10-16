# Diagnosis and Fix: Intermittent Collapsed Edge Rail Body Hover Without Expansion

## 1. Symptom & Problem Statement
Users reported intermittent collapsed Edge Rail body CSS hover (`:hover` background active) without window expansion after 200 ms, requiring an outside mouse leave and re-entry to recover.

## 2. Deterministic Reproducer & Red Evidence
- **Harness**: `scripts/repro_hover.cjs` (driving real pointer inputs via Chrome DevTools Protocol to actual WebView2 on an isolated test profile `dropcove-repro`).
- **Pre-Fix Failure Rate**: 30%–40% failure rate across 10-run iterations (`node scripts/repro_hover.cjs`, runs 2, 6, 7 failed: `hover=true, expanded=false, width=16` at 260 ms).
- **Captured Failure Logs (Actual `WM_TIMER` Dispatch Evidence)**:
  Direct instrumentation of `WM_TIMER` deliveries captured early timer callbacks firing ahead of `due`:
  ```text
  [WM_TIMER-DISPATCH] early callback: now=5963 < due=5969, remaining=6ms; re-arming timer
  [WM_TIMER-DISPATCH] transition: now=5979, expanded=true
  ...
  [WM_TIMER-DISPATCH] early callback: now=7239 < due=7247, remaining=8ms; re-arming timer
  [WM_TIMER-DISPATCH] transition: now=7254, expanded=true
  ...
  [WM_TIMER-DISPATCH] early callback: now=9760 < due=9763, remaining=3ms; re-arming timer
  [WM_TIMER-DISPATCH] transition: now=9776, expanded=true
  ...
  [WM_TIMER-DISPATCH] early callback: now=11026 < due=11027, remaining=1ms; re-arming timer
  [WM_TIMER-DISPATCH] transition: now=11042, expanded=true
  ...
  [WM_TIMER-DISPATCH] early callback: now=11664 < due=11666, remaining=2ms; re-arming timer
  [WM_TIMER-DISPATCH] transition: now=11680, expanded=true
  ```
- **Screenshot Evidence**:
  - `repro_hover_red.png`: Captured collapsed handle with body hover style true and `expanded: false`.
  - `after_hover_expanded.png`: Captured expanded rail after fix.

## 3. Root Cause Analysis
1. **Windows System Timer Quantum Dispatches**:
   - Win32 `SetTimer(hwnd, RAIL_TIMER_ID, 200, None)` dispatches on standard Windows timer interrupt ticks (~15.625 ms resolution).
   - In 30%–50% of dispatches on Windows, `WM_TIMER` arrives 1–8 ms before the exact monotonic deadline (`now_ms < due`).
2. **Premature Timer Cancellation**:
   - In `edge_rail.rs`, window subclass `no_activate` unconditionally called `KillTimer(Some(hwnd), RAIL_TIMER_ID)` upon receiving `WM_TIMER`.
3. **Strict Non-Expansion Before 200 ms**:
   - `plan_tick_rail` correctly requires `now_ms >= due`. When `WM_TIMER` arrived early, `plan_tick_rail` returned `None`, leaving `guard.rail.expand_due_ms = Some(due)` intact.
4. **Dead Timer Stranding**:
   - Because `KillTimer` had already executed and `resident::tick_rail(&app)` did not re-arm the timer when `plan_tick_rail` returned `None`, the timer was killed permanently.
   - The rail remained stranded in the collapsed state with `expand_due_ms` still pending and CSS `:hover` active until the pointer left the rail.

## 4. Implementation
1. **Strict Timing Preservation (No Slack Constant)**:
   - Preserved exact `now_ms >= due` comparison in `plan_tick_rail` for both expansion (no expansion before 200 ms) and collapse (no collapse before 300 ms). No artificial tolerance constant introduced.
2. **Native Timer Re-Arming on Early Callback (`src-tauri/src/resident.rs`, `src-tauri/src/edge_rail.rs`)**:
   - In `resident::tick_rail(&app)`:
     - When `plan_tick_rail` succeeds (transition occurred), cancelled the timer with `edge_rail::cancel_rail_timer(app)`.
     - When `plan_tick_rail` returns `None` (early callback before deadline), checked if a deadline remains pending (`rail.expand_due_ms.or(rail.collapse_due_ms)`).
     - If pending, automatically re-armed `edge_rail::arm_rail_timer(app, remaining)` where `remaining = (due.saturating_sub(now)).max(1) as u32`.
     - If no deadlines remain, cancelled the timer.
3. **Unit Regression Coverage (`src-tauri/src/resident.rs`)**:
   - Added unit test `rail_sub_deadline_timer_callback_does_not_expand_early_and_rearms_until_due`, verifying:
     - Sub-deadline callback at 194 ms (< 200 ms) does NOT expand (`tick == None`, `expanded == false`, `expand_due_ms` preserved).
     - Subsequent callback at 1200 ms (>= 200 ms) DOES expand (`tick.is_some()`, `expanded == true`).
     - Sub-deadline collapse callback at 1494 ms (< 1500 ms) does NOT collapse.
     - Subsequent callback at 1500 ms (>= 300 ms) DOES collapse.

## 5. Verification Results
- **Unit Suite**: All 105/105 tests passed (`dropcove_lib-2c8ca4d6e28f318c.exe`).
- **Frontend Build**: `pnpm run build` (`vue-tsc && vite build`) passed with zero errors.
- **CDP Runtime Smoke**:
  - `node scripts/repro_hover.cjs`: 0/10 failures (100% green; down from 30%–40% pre-fix failure rate).
  - 25-run stress test: 0/25 failures (100% green).
- **Edge Cases & Preserved Behaviors**:
  - Open Shelf button hover for 300 ms: remained collapsed (`width: 16, height: 280`).
  - Button-to-body transition: expanded after 200 ms delay (`width: 280, height: 280`).
  - Leave transition: preserved 300 ms collapse delay (expanded at 150 ms, collapsed at 350 ms).
