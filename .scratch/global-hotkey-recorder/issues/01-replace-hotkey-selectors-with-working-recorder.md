# 01: Replace hotkey selectors with a working recorder

**What to build:** Replace the separate global-hotkey modifier selectors and primary-key list with one button-like recorder that lets the user press a shortcut, review it as a draft, and save it through the existing registration and settings transaction. This first vertical slice supports the current Space, A–Z, and F1–F12 primary-key set end to end while establishing the complete recorder lifecycle and preserving the existing hotkey backend.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Parent specification:** [Global Hotkey Recorder](../spec.md)

- [x] Settings presents one button-like Global hotkey recorder instead of separate Ctrl, Alt, Shift, and Win controls plus a primary-key list.
- [x] Opening Settings displays the saved shortcut in normalized Ctrl, Alt, Shift, Win, primary-key order, with `Ctrl + Shift + Space` still used for default settings.
- [x] This slice records Space, A–Z, and F1–F12 with at least one aggregate Ctrl, Alt, Shift, or Win modifier and persists the existing modifier booleans plus virtual-key value without a schema change.
- [x] Pointer click starts listening. Keyboard focus alone does not; the focused idle recorder starts listening when activated with Enter or Space.
- [x] The Enter or Space input used to activate the recorder is consumed as the activation command and excluded until that activation input completes, so it cannot become the recorded primary key through repeat or event fallthrough.
- [x] Listening presents a visible and screen-reader-observable state, shows the currently held modifiers, and completes on the first supported primary-key press while at least one modifier is held.
- [x] Modifier-only input keeps listening active; releasing modifiers updates the visible in-progress chord without canceling the session.
- [x] A bare supported primary key or unsupported key leaves listening active, preserves the pre-listening draft, and shows an inline, announced message that at least one modifier and one supported key are required.
- [x] Escape cancels only the current recorder session, restores the pre-listening draft, clears inline validation, and leaves Settings open.
- [x] Moving focus to another Settings control or deactivating the Settings window cancels listening and restores the pre-listening draft. Listening has no timeout.
- [x] Save is disabled while listening. Cancel and the window close action always discard every unsaved Settings change and close the window, including while listening.
- [x] Completing capture updates only the draft. The newly recorded shortcut is not registered or persisted until Save.
- [x] The currently saved global hotkey cannot toggle the Drop Shelf while the recorder is listening, including the input event that completes capture.
- [x] The saved global hotkey resumes normal operation immediately after capture completes, is canceled, or loses focus, and remains the active shortcut while a different draft waits for Save.
- [x] Recorder suppression occurs through the existing global-hotkey message-dispatch path; entering listening does not unregister the active hotkey and adds no low-level keyboard hook.
- [x] Save attempts native registration through the existing settings transaction. Success persists the draft, updates the active hotkey, and closes Settings.
- [x] Native registration rejection keeps Settings open, retains the draft, leaves the previous hotkey registered, leaves durable settings unchanged, and reports that DropCove could not register the hotkey because it may be unavailable or reserved by Windows.
- [x] Registration-failure text does not claim another application is the confirmed cause.
- [x] A later settings-persistence or settings-application failure retains the existing rollback contract: the previous hotkey and previous settings are restored where currently possible and the save failure remains visible in Settings.
- [x] Startup continues loading and registering the persisted shortcut. Startup registration failure still warns the user while leaving the DropCove resident process and tray access available.
- [x] The recorder exposes an accessible name, current value, help text, enabled state, listening state, validation state, and visible focus using built-in WinUI behavior and theme-aware resources.
- [x] Idle, focused, listening, invalid, draft, and disabled-Save states remain legible in Light and Dark themes without relying only on color.
- [x] No clear, reset, disable-hotkey, live registration probe, timer, polling loop, background worker, permanent input hook, new runtime dependency, view-model migration, public interface hierarchy, or settings migration is introduced.
- [x] The complete existing automated suite passes without adding tests for XAML source text, private control names, handler forwarding, or implementation-only state.
- [x] A targeted installed self-contained Release smoke run with an isolated profile and actual keyboard input proves pointer and keyboard activation, activation-key exclusion, live capture, Escape and focus-loss cancellation, active-hotkey suppression, draft behavior, successful Save, persistence, restart restoration, and a real registration rejection before this ticket is complete.

## Testing seam

Use the installed self-contained Release with an isolated profile, UI Automation, and actual keyboard input as the authoritative seam. Reuse the existing automated suite only as regression coverage. Do not introduce a second feature-specific test seam, mock native registration layer, product test hook, or UI testing framework.

## Demo path

Open Settings with the default shortcut visible. Focus the recorder with Tab without entering listening, activate it with Enter, record `Ctrl + K`, and show that the Drop Shelf does not toggle during capture. Before saving, demonstrate that the old shortcut still works and `Ctrl + K` does not. Save, reopen Settings, show the persisted draft, restart DropCove, and invoke the Drop Shelf with `Ctrl + K`. Repeat with a genuinely unavailable test combination to show the accurate warning, retained draft, unchanged durable settings, and continued operation of the previous shortcut.

## Comments

- 2026-10-08: Implementation complete. The installed self-contained Release smoke proves the recorder lifecycle, suppression through the completing input, explicit Save, clean restart, native rejection with an accurate warning, durable-state retention, and deterministic persistence-failure rollback. Startup conflict recovery exercises the real native tray Settings menu after the Shell callback; tray balloon rendering is not claimed.
- The reusable regression command is `pwsh -NoProfile -File scripts/test-hotkey-recorder.ps1 -ExecutablePath <installed-release-exe> -ArtifactPath <report>`. [Functional evidence](../../../artifacts/hotkey-recorder-qualification.json) and [visual/accessibility evidence](../../../artifacts/hotkey-recorder-visual.json) record exact payload identities and cleanup.
- Final automated suite: 226 passed, zero failed/skipped. Separate Standards and Spec reviews found zero actionable source findings. Release performance qualification remains the responsibility of Ticket 03 and is not implied by completing this implementation slice.

