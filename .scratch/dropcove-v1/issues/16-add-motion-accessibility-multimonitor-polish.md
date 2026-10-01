# 16: Add motion, accessibility, and full multi-monitor polish

**What to build:** Finish the V1 interaction surfaces with responsive motion, reduced-motion behavior, accessibility basics, and full Edge Rail multi-monitor/DPI verification without weakening native workflow performance.

**Blocked by:** 15: Add Compact and Expanded shelf modes.

**Status:** ready-for-agent

- [x] Shelf appearance, rail expansion, Compact/Expanded transition, item removal, and batch insertion use short Composition-based motion only after state changes are correct.
- [x] Windows reduced-motion and animation settings disable or reduce non-essential motion.
- [x] Interactive controls expose accessible names, keyboard focus, and visible focus states appropriate to their actions.
- [x] Compact, Expanded, Settings, flyouts, and Edge Rail remain usable at supported scaling levels and per-monitor DPI changes.
- [x] Rail placement, remembered monitor/side, fullscreen hiding, shake placement, and shelf placement remain correct when monitors are added, removed, rearranged, or use different DPI.
- [x] Motion, thumbnail work, database work, and virtualization do not block pointer or drag interaction.
- [ ] Packaged accessibility, reduced-motion, keyboard, DPI, multi-monitor, focus, and animation smoke scenarios pass.
- [ ] Release-build CPU, latency, and working-set measurements remain within the product targets after polish.

## Comments

- 2026-10-01: Implemented Composition-based entrance, insertion, removal, rail-transition, Compact/Expanded, and state-change motion. Composition animations are released on completion; Windows client-area animation settings skip non-essential motion. Added visible system focus visuals, control automation names, dynamic DPI sizing for Settings and confirmation windows, remembered-monitor fallback, and event-driven display/device/DPI reflow for the shelf and Edge Rail.
- 2026-10-01: Final automated suite passes 72/72. Packaged Release UIA smoke verified Compact `180 × 236`, Expanded `720 × 640`, named/focusable management controls, Settings accessibility labels, and committed item removal. A synthetic display-change message kept the shelf within the monitor work area. Current environment exposed one monitor at 96 DPI, so physical monitor add/remove/rearrange scenarios remain unverified.
- 2026-10-01: Reviewed final `100` Shelf Batch / `1,000` Shelf Item Release run measured `0%` CPU per logical processor, `160.93 MB` visible working set, and `164.00 MB` hidden idle working set. The working-set target is therefore still missed; this remains the Ticket 19/native-residency blocker. The performance script also reports summon latency as blocked because synthetic global keyboard injection cannot trigger the registered hotkey in this session. Direct resident-process `WM_HOTKEY` dispatch measured `3.15 ms`, but it is not a substitute for the required interactive hotkey smoke.
- 2026-10-01: Changes remain uncommitted pending explicit approval and resolution of the acceptance blockers.
- 2026-10-01: Corrected installed-EXE hotkey smoke delivered `Ctrl+Shift+Space` as 6/6 native input events for 10 toggles; every main-shelf visibility transition was observed. Measured latency was `2.929–13.555 ms`, p95 `10.845 ms`; the earlier `SendInput` error-87 probe was discarded as invalid harness evidence.
- 2026-10-01: Two-monitor same-DPI smoke used a temporary `RailMonitorId=DISPLAY2` profile and one seeded batch. Dismissal placed the Edge Rail on `DISPLAY2` at the requested right edge, fully inside that monitor's work area. Both connected monitors were 96 DPI; mixed-DPI and physical hot-plug/rearrange scenarios remain unverified. The temporary settings and shelf database were restored successfully per the probe; the final settings file and empty shelf contents were verified, then the user's installed profile was relaunched.
- 2026-10-01: Bound installed UI Automation to the known DropCove main HWND. Settings opened through its supported `InvokePattern`; a distinct child Settings HWND was identified by its `RailMonitorComboBox`. The probe found 18 named Settings controls, all tested controls enabled, and successfully focused `ControlCheckBox`; the Settings window then closed. Main-shelf named/focusable controls were also present.
- 2026-10-01: Normal Compact/Expanded transition smoke completed with the installed process responsive. The reversible reduced-motion probe restored the original Windows animation setting exactly, but `SPI_SETCLIENTAREAANIMATION(false)` was followed by `SPI_GETCLIENTAREAANIMATION=1`; the disabled-state branch is therefore environment-unverified. Acceptance checkbox 15 remains unchecked; checkbox 16 remains unchecked and its working-set portion is intentionally deferred to Tickets 18/19.