# 14: Complete accepted shake drops and bound hook overhead

**What to build:** Complete the shake workflow by accepting supported drops into a new Shelf Batch, retaining the near-cursor shelf after success, suppressing repeated triggers, and proving the hook remains lightweight.

**Blocked by:** 13: Summon the shelf by shake and restore on cancel.

**Status:** ready-for-agent

- [ ] A supported file/folder drop after shake creates and persists a Shelf Batch through the existing drag-in/application seam.
- [ ] After an accepted shake drop, the shelf remains near the cursor for inspection and follows normal Hidden/EdgeDocked rules when dismissed.
- [ ] Unsupported payloads, DragLeave, and cancellation restore the previous display and placement state.
- [ ] Cooldown prevents one movement sequence from repeatedly summoning the shelf.
- [ ] Shake does not alter Temporary Item, Pinned Item, Missing, Unavailable, or persistence semantics.
- [ ] Hook processing stays within Windows timing constraints and queues non-trivial work outside the callback.
- [ ] Packaged release-build measurements meet the idle CPU target and summon-latency target with shake enabled; disabling shake removes hook overhead.
- [ ] Smoke scenarios cover accepted drop, unsupported payload, false positive, cooldown, current-monitor placement, and post-success dismissal.