# 13: Summon the shelf by shake and restore on cancel

**What to build:** Let users summon the Drop Shelf near the cursor during an external drag through a bounded event-driven shake heuristic, then restore the previous state when no supported drop is accepted.

**Blocked by:** 12: Drop into and scroll the Edge Rail.

**Status:** complete

- [x] Shake-to-open is enabled by default, can be disabled, and exposes configurable sensitivity.
- [x] Enabling shake installs the low-level mouse hook; disabling shake removes it without restarting DropCove.
- [x] The hook callback performs minimal input-state work and queues gesture analysis elsewhere.
- [x] DropCove does not poll cursor coordinates or attempt to inspect another application's data object before DragEnter.
- [x] A qualifying direction-reversal gesture summons the shelf near the cursor on its current monitor.
- [x] Hidden and EdgeDocked states are captured before summon and restored after mouse release/cancellation when no supported drop is accepted.
- [x] False-positive gestures while dragging non-file content restore the prior state and do not create a Shelf Batch.
- [x] Application-level detector tests use deterministic movement sequences; packaged smoke validates real external-drag summon, cancel, hook toggle, and state restoration.

## Implementation seam

- `ShakeDetector` owns deterministic bounded direction-reversal analysis in `DropCove.Core`; `LowLevelMouseHook` forwards only left-button and movement samples from `WH_MOUSE_LL`.
- `ShakeInputQueue` bounds hook-to-UI dispatch; `MainWindow` owns summon, cursor placement, previous-state capture, release/drop outcome gating, and restoration.
- `MainPage` reports DragEnter, DragLeave, accepted, unsupported, and failed drop outcomes to the shake session; `SettingsStore` persists enabled state and sensitivity.

## Comments

- 2026-09-30: Implemented the detector, low-level hook lifecycle, configurable settings, cursor-near summon, Hidden/EdgeDocked restoration, and application drag/drop outcome callbacks.
- 2026-09-30: Refactored ShakeDetector to stroke-anchor displacement model for natural low-amplitude wiggles (2 reversals, matching Yoink/Dropover/Holdem UX). Removed GetCursorPos at summon time in favor of hook coordinates. Implemented auto-hide on drag-out completion when shelf is empty. All 61 automated tests passing and packaged release smoke verified by user.