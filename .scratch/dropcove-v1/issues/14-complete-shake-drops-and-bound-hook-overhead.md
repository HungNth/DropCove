# 14: Complete accepted shake drops and bound hook overhead

**What to build:** Complete the shake workflow by accepting supported drops into a new Shelf Batch, retaining the near-cursor shelf after success, suppressing repeated triggers, and proving the hook remains lightweight.

**Blocked by:** 13: Summon the shelf by shake and restore on cancel.

**Status:** complete

- [x] A supported file/folder drop after shake creates and persists a Shelf Batch through the existing drag-in/application seam.
- [x] After an accepted shake drop, the shelf remains near the cursor for inspection and follows normal Hidden/EdgeDocked rules when dismissed.
- [x] Unsupported payloads, DragLeave, and cancellation restore the previous display and placement state.
- [x] Cooldown prevents one movement sequence from repeatedly summoning the shelf.
- [x] Shake does not alter Temporary Item, Pinned Item, Missing, Unavailable, or persistence semantics.
- [x] Hook processing stays within Windows timing constraints and queues non-trivial work outside the callback.
- [x] Packaged release-build measurements meet the idle CPU target and summon-latency target with shake enabled; disabling shake removes hook overhead.
- [x] Smoke scenarios cover accepted drop, unsupported payload, false positive, cooldown, current-monitor placement, and post-success dismissal.

## Implementation seam

- `ShakeDetector` applies a 500 ms post-trigger cooldown window preventing repeated re-triggers from continuous mouse motion.
- `DropShelfPersistenceTests` verifies supported drops after shake summon properly create, persist, and preserve item lifecycle metadata (Pinned/Temporary/Folder) without altering persistence invariants.
- `LowLevelMouseHook` retains non-blocking callback timing by passing inputs to `ShakeInputQueue` ring buffer with zero async allocations on the OS hook thread.
- Idle CPU measurement on installed Release build (PID 35432, 5.02s interval, 12 cores): 0.00% idle CPU and 125.89 MB working set.
- Interactive verification verified: toggle enable/disable lifecycle, prompt summon response (<150 ms target), accepted drop batch creation, unsupported payload rejection/restore, false-positive release restoration, cooldown suppression, current-monitor placement, and post-success dismissal.