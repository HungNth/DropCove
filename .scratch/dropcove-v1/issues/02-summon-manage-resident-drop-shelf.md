# 02: Summon and manage the resident Drop Shelf

**What to build:** Make DropCove behave as a resident Windows utility that users can summon, dismiss, configure, and exit predictably across launches and monitors.

**Blocked by:** 01: Boot the packaged DropCove shell.

**Status:** ready-for-agent

- [ ] DropCove runs as a single instance; launching it again activates the existing process instead of creating competing state.
- [ ] `Ctrl + Shift + Space` opens and activates the Drop Shelf, and the user can configure a different hotkey that survives restart.
- [ ] Hotkey registration conflict leaves DropCove running, reports the conflict through the tray, and directs the user to choose another combination.
- [ ] The hotkey toggles the shelf; `Esc` and the close button dismiss it without terminating the process.
- [ ] The tray exposes Settings and an explicit Exit DropCove action that terminates the process.
- [ ] Start with Windows is enabled by default, configurable, survives restart, and restores the process with the UI hidden and unfocused.
- [ ] Hotkey invocation positions and sizes the shelf correctly within the work area of the monitor containing the cursor at that monitor's DPI.
- [ ] The visible shelf remains topmost and does not auto-hide solely because another application receives focus.
- [ ] Packaged smoke scenarios exercise second-launch redirection, hotkey conflict, startup, tray exit, current-monitor placement, and DPI behavior.