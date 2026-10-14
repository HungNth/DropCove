# 01: Gesture Hook, Detector, Session and Settings

Status: resolved
Type: task
Version: 0.0.6
Parent: [Specification](../spec.md)
Blocked by: None
Implementation approval: approved by user on 2026-10-14

## What to build

Deliver a working Shake-to-open path from Windows pointer events to a no-activate Drop Shelf near the pointer, with release restoration and complete settings ownership. This slice must work end to end before storage-outcome edge cases are deepened.

## Acceptance criteria

- [x] One owner-thread `WH_MOUSE_LL` adapter installs, reconfigures and uninstalls without polling or a new dependency.
- [x] Hook callback performs bounded capture only, posts ordered work, always continues the hook chain, and safely handles teardown/re-entry.
- [x] Low, Normal and High profiles use 24/4, 10/2 and 8/2 movement/reversal thresholds.
- [x] Equal-axis ties select X; sub-threshold samples do not advance the baseline; the first accepted direction starts the 600 ms window; expiry and queue overflow reset the reversal run without losing current button state.
- [x] Native transient show/hide uses symmetric no-activate Win32 operations and verifies the Shelf is actually hidden after restoration.
- [x] Detection requires the left button, uses dominant-axis reversals, expires after 600 ms and respects a 500 ms cooldown.
- [x] Queue overflow cannot grow memory or create a stale gesture sequence.
- [x] A qualifying gesture from Hidden or EdgeDocked saves prior presentation and shows the Shelf near the current cursor without activating the in-progress drag.
- [x] Button release or false positive restores Hidden or the exact prior EdgeDocked placement/presentation.
- [x] Normal launch, tray, hotkey, Open Shelf, dismissal and final-item behavior remain unchanged.
- [x] Settings expose Enable shake gesture and Low/Normal/High sensitivity as drafts with enabled/Normal defaults.
- [x] JSON schema has exactly seven required fields and preserves valid 0.0.5 bytes through the clean-cutover warning path.
- [x] Save/Cancel and transaction rollback include actual Shake runtime state; failed enable/reconfigure does not commit JSON.
- [x] Startup hook failure warns while resident access and existing Shelf/Rail workflows continue.
## Testing seam

Pure detector tests use synthetic point/button events and timestamps. Coordinator tests use the existing deterministic window/rail adapters and real temporary Shelf/Settings stores. Settings transaction tests use a deterministic Shake integration that can fail after effect and fail restoration. Actual hook delivery, no-activate behavior and Settings controls require Windows runtime smoke.

## Demo path

Start from Hidden, hold the left button and perform the Normal reversal sequence within 600 ms. Observe the Drop Shelf near the pointer without losing the drag; release without dropping and observe Hidden restoration. Repeat from EdgeDocked and observe the same rail placement/presentation restored. Change sensitivity and disable/enable through Settings, proving Cancel and Save.

## Non-goals

Accepted/duplicate/unsupported OLE result completion beyond the basic release path, packaged 0.0.6 release, later visuals/accessibility/recovery, or adaptive gesture tuning.

## Comments

## Answer

Resolved. Implemented `windows_shake` (`WH_MOUSE_LL` on dedicated message-pumping hook thread `dropcove-shake-hook`, bounded 256-sample `EventQueue`, `ShakeDetector` with Low 24px/4, Normal 10px/2, High 8px/2, 600ms window, 500ms cooldown, and UI-thread `WM_APP` subclass marshalling), transient `ShakeSession` in `ResidentState`, 7-field clean-cutover `Settings` schema (`ShakeEnabled`, `ShakeSensitivity`), transactional rollback in `settings_transaction`, and `ShakeSettingsFields.vue` in Settings.

### Exercised evidence

- `pnpm build` (`vue-tsc && vite build`) and `cargo xwin` release + test builds succeeded with zero source warnings; **82/82** native tests passed.
- Live Windows smoke against `dropcove.exe` (`8f1d76164de3feddb40166285328c35ccf5ec255ef446e547eff004f5855c917`) verified: left-button requirement, Normal (10px/2), Low (24px/4; rejected 15px/2, accepted 28px/4), and High (8px/2) thresholds, 600ms window expiry, 500ms cooldown, no-activate near-cursor placement (`[497, 395, 693, 584]` preserving foreground HWND), Hidden and EdgeDocked (`[1904, 376, 1920, 656]`) release restoration, Settings UI layout (`footer.bottom=519 <= 520`), Cancel draft discard, Save persistence, live `ReplaceFileW` error-1175 rollback and retry, disable/re-enable hook lifecycle, and clean Exit (`0`).
- Final installed-artifact regression and native installation/runtime failure evidence are recorded in [ticket 03](03-release-0.0.6.md#answer), including actual Windows rejection under one-call debugger argument fault injection, native warnings, rollback, and successful explicit retry. This is not a spontaneous host-policy/startup-denial run.
