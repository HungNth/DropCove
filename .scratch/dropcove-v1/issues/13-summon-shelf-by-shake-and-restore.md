# 13: Summon the shelf by shake and restore on cancel

**What to build:** Let users summon the Drop Shelf near the cursor during an external drag through a bounded event-driven shake heuristic, then restore the previous state when no supported drop is accepted.

**Blocked by:** 12: Drop into and scroll the Edge Rail.

**Status:** ready-for-agent

- [ ] Shake-to-open is enabled by default, can be disabled, and exposes configurable sensitivity.
- [ ] Enabling shake installs the low-level mouse hook; disabling shake removes it without restarting DropCove.
- [ ] The hook callback performs minimal input-state work and queues gesture analysis elsewhere.
- [ ] DropCove does not poll cursor coordinates or attempt to inspect another application's data object before DragEnter.
- [ ] A qualifying direction-reversal gesture summons the shelf near the cursor on its current monitor.
- [ ] Hidden and EdgeDocked states are captured before summon and restored after mouse release/cancellation when no supported drop is accepted.
- [ ] False-positive gestures while dragging non-file content restore the prior state and do not create a Shelf Batch.
- [ ] Application-level detector tests use deterministic movement sequences; packaged smoke validates real external-drag summon, cancel, hook toggle, and state restoration.