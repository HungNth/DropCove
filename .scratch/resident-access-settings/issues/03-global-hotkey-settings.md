# 03: Global Hotkey and Settings

Status: resolved
Type: task
Version: 0.0.4
Parent: [Specification](../spec.md)
Blocked by: 02
Implementation approval: approved by user on 2026-10-13

## What to build

The tray opens one Settings window where the user records, saves or cancels a global hotkey. The active hotkey toggles the Shelf, while invalid/conflicting drafts cannot replace the prior active shortcut or settings file.

## Acceptance criteria

- [x] The tray right-click menu adds Settings only when this working Settings surface is available; reopening focuses the existing Settings window and draft.
- [x] Settings loads the default `Ctrl + Shift + Space` HotKey when its owned settings file is missing, without warning or eagerly writing a file.
- [x] Malformed or unreadable settings remain untouched, load defaults in memory and queue a resident warning notification.
- [x] The Settings Adapter replaces its owned JSON atomically through a same-directory temporary/flush/replace sequence; a failed replacement preserves the active file.
- [x] Settings changes remain drafts until Save succeeds; Cancel, window close and Settings deactivation during recording discard the appropriate unsaved/listening state.
- [x] The shortcut recorder starts by click, Enter or Space; displays modifiers in `Ctrl + Alt + Shift + Win + key` order; and requires at least one modifier plus an allowed primary key.
- [x] Allowed primary keys are A–Z, 0–9, F1–F24 except platform-reserved keys, navigation/edit keys, OEM punctuation and modified Tab/Enter/Space. Escape, bare modifiers, mouse/media/system keys are rejected or reserved for cancellation.
- [x] Bare Tab leaves and cancels recording; Escape, focus loss or Settings deactivation cancels; modified Tab/Enter/Space are recordable. Save is disabled while listening.
- [x] The prior hotkey stays registered while editing/listening. Activation handling from that old combination is suppressed only while the recorder is listening so the combination can become the draft.
- [x] Save validates and tests the candidate before replacing the active registration. Conflict/rejection keeps Settings open, reports the conflict, and preserves the prior active shortcut and settings file.
- [x] Successful Save persists the HotKey and atomically makes it the active registration; unchanged Save does not duplicate registration.
- [x] Registered hotkeys use no-repeat behavior so key hold cannot cause toggle storms.
- [x] The active hotkey toggles only `Hidden ↔ Visible`, placing show activation on the cursor monitor. Normal/secondary/tray access remains always-show.
- [x] Hotkey registration failure during startup leaves DropCove resident, keeps tray/launch access, and produces a warning.
- [x] This intermediate slice persists only the currently active HotKey field; ticket 04 performs the clean cutover to the final 0.0.4 `HotKey + StartWithWindows` schema before release.

## Testing seam

Resident coordinator tests combine a real temporary Settings Adapter with deterministic hotkey/tray/window adapters. Permanent tests cover parsing/defaults, atomic replacement failure, recorder allowlist/cancel/formatting, conflict rollback and toggle-versus-always-show routing. Native smoke exercises real `RegisterHotKey`, Settings keyboard/pointer flows, conflict warning and cursor-monitor placement.

## Demo path

Open Settings from tray, record a valid new shortcut, cancel and observe the old shortcut still active; reopen, Save the shortcut, observe conflict handling for an occupied combination, then use the accepted shortcut to hide/show the Shelf without auto-repeat toggling.

## Non-goals

Start with Windows and hidden sign-in launch are ticket 04. Rail and Shake settings are not persisted or shown in 0.0.4.

## Comments

- User confirmed the installed fix's focused Step 5 retest passed: alternate chord capture, Save and new shortcut toggle. The earlier physical failure is closed; no rerun of those reported results is required.
- Remaining keyboard-entry smoke on the unchanged installed executable: native Enter and Space each started recording; Save was disabled while listening and re-enabled after native Escape canceled. The owned process (PID 27336) exited 0 through its own menu. Removed only its temp profile/WebView directory after cache flush and restored cursor; no user resident was terminated.

## Evidence and manual retest

### Confirmed bug and fix

- User report: manual Step 5 could not capture a different chord; the other supplied test steps succeeded. This invalidates the previous claim that recording was blocked only by the automation environment. Earlier Save/switch claims were premature and are superseded below.
- Focused reproduction on an owned profile: actual Windows `SendInput` Ctrl+Alt+K left the old F24 draft unchanged with `listening: true`. Temporary callback counting measured **0 hook callbacks** before/after that chord with the real Settings HWND foreground. No keyboard content was globally logged; instrumentation was removed.
- Clean cutover: removed the entire WH_KEYBOARD_LL recorder/thread/join pipeline. The focused Settings control transports Windows WebView keydown VK/modifiers into a validated coordinator operation. Escape/bare Tab/blur cancel; modifier-only/repeat/composition events do not commit. The active registered chord is captured from its resident WM_HOTKEY identity instead of toggling. Focus is restored after recorder-start acknowledgement. Global registration remains Win32 MOD_NOREPEAT, and Save registration/retirement is dispatched to its owning UI thread.

### Exercised verification

- Fresh final-source integrated Windows test artifact `dropcove_lib-1a9ebbf412a1544f.exe`: **55 passed, 0 failed**. Added a real-settings-backed coordinator regression for a recorded chord ending listening, preserving active registration/file, rejecting invalid input and rejecting late input once recording ends. Existing settings load/atomic replace/raw rollback tests remain passing.
- Actual native input into the real Settings control captured Ctrl+Alt+K, Ctrl+Tab, Ctrl+Enter, Ctrl+Space, Shift+A and Ctrl+Shift+F23. Capturing the currently active chord did not show/toggle the hidden Shelf. Escape/bare Tab canceled. A blur probe initially sampled before its asynchronous cancellation completed; later projection was `listening: false` with focus on Cancel.
- Save closed Settings, wrote exactly the captured HotKey and activated Ctrl+Alt+K; the previous registered F24 chord no longer toggled. A real separately owned RegisterHotKey reservation created a Save conflict: error remained visible in Settings, previous JSON bytes were unchanged, and Cancel closed without applying the draft while the active K chord still toggled.
- Packaged through `pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc`, copied to `build/DropCove-Setup-0.0.3.exe`, and silently updated the existing current-user installation only after proving its executable had no running PID. Installed executable: `C:\Users\Hung\AppData\Local\Programs\DropCove\dropcove.exe`. Installed SHA-256 `7583ab748d4d84736babe696daafffd276bb95d821f5ea1f748188f5e3d826ab`; installer SHA-256 `a2150cd95f2063b0aeec37eed843f873e3c95e514dbef5cee34e818928f30568`. Byte comparison found exactly **3** differing bundle-marker bytes (`UNK` versus `NSS`), and every other byte matched the just-built source executable. This is a user test build, not the 0.0.4 release.
- Installed-app smoke on an owned profile/PID 24176 independently captured Ctrl+Alt+K, closed on Save, persisted VK 75, toggled through the new shortcut and exited through the native menu with code 0. Closed attached tabs, removed only the owned temp profile/WebView/fixtures and restored cursor. The installation remains for the user; their existing `C:\DropCoveTest` and ambient Shelf data were not modified. No broad process-name termination or registry deletion was used.

### Coverage limits

- User physical Step 5 retest passed, closing the installed-recorder acceptance gate. Full OEM/Win-key and live mixed-DPI hardware permutations were not exhaustively exercised; no such coverage is inferred. Startup transaction/rollback-failure retention and final release verification remain owned by tickets 04–05.


## Answer

Ticket 03 is resolved: physical alternate-chord capture/Save/new-hotkey activation passed on the user's machine; 55 final-source tests plus actual native/installed conflict/Cancel/keyboard-entry/suppression smoke support the approved behavior. Ticket 04 is unblocked. Version metadata remains 0.0.3 until the release ticket; no commit was created.

