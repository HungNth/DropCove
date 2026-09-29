# 02: Summon and manage the resident Drop Shelf

**What to build:** Make DropCove behave as a resident Windows utility that users can summon, dismiss, configure, and exit predictably across launches and monitors.

**Blocked by:** 01: Boot the CLI-installed DropCove shell.

**Status:** ready-for-agent

- [x] DropCove runs as a single instance; launching it again activates the existing process instead of creating competing state.
- [x] `Ctrl + Shift + Space` opens and activates the Drop Shelf, and the user can configure a different hotkey that survives restart.
- [x] Hotkey registration conflict leaves DropCove running, reports the conflict through the tray, and directs the user to choose another combination.
- [x] The hotkey toggles the shelf; `Esc` and the close button dismiss it without terminating the process.
- [x] The tray exposes Settings and an explicit Exit DropCove action that terminates the process.
- [x] Start with Windows is enabled by default, configurable, survives restart, and restores the process with the UI hidden and unfocused.
- [x] Hotkey invocation positions and sizes the shelf correctly within the work area of the monitor containing the cursor at that monitor's DPI.
- [x] Before Shelf Batches exist, the bounded unified Drop Shelf opens at `180 × 180` logical pixels with a 32-pixel integrated drag strip, Settings and Close controls, top-edge anchoring, and work-area clamping at the current monitor DPI.
- [x] The visible shelf remains topmost and does not auto-hide solely because another application receives focus.
- [x] Installed-EXE smoke scenarios exercise second-launch redirection, hotkey conflict, startup, tray exit, current-monitor placement, DPI behavior, and the `180 × 180` bounded shelf.

## Comments

- 2026-09-29: Release build completed with zero warnings. Installed-EXE smoke verified `180 × 180` placement, close-to-hide, second-launch activation, hidden `--autostart`, Settings persistence, HKCU startup registration, and a forced default-hotkey conflict that left the resident process running. A direct native tray callback opened the Settings/Exit context menu; clicking `Exit DropCove` terminated the process.
- 2026-09-29: Installer and uninstaller now send a dedicated shutdown message instead of `WM_CLOSE`, preserving resident close-to-hide behavior while still terminating the process for update/removal.
- 2026-09-29: Window placement now validates Win32 geometry queries and clamps physical size to the monitor work area before calculating an anchored origin, preventing invalid `Math.Clamp` bounds on unusually small work areas.
- 2026-09-29: Windows `SendKeys` for the configured default `Ctrl + Shift + Space` toggled the installed shelf hidden and visible, confirming the registered hotkey path after cleanup of the forced-conflict probe.
- 2026-09-29: User rechecked Ticket 02 and confirmed all acceptance criteria complete.