# 03: Manage Shelf Batches directly from Edge Rail rows

**What to build:** Add the Drop Shelf's direct lifecycle and removal actions to completed Edge Rail preview rows so users can manage single items and whole Shelf Batches without opening the full Drop Shelf, while preserving durable manager semantics and no-activate interaction.

**Blocked by:** 02: Render Drop Shelf-equivalent previews in Edge Rail rows.

**Status:** ready-for-agent

**Parent specification:** [Edge Rail Thumbnail and Management Parity](../spec.md)

- [ ] A single-item row always shows 24 by 24 logical-pixel Pin/Unpin and Remove Item controls in that order.
- [ ] A multi-item row always shows 24 by 24 logical-pixel tri-state Bulk Pinning, Manage Items, and Remove Batch controls in that order.
- [ ] Pin and mixed-state visuals match the Drop Shelf, including shape-distinct Indeterminate state and accurate Off/On/Indeterminate automation state.
- [ ] Remove Item and Remove Batch use the same `×` affordance as the Drop Shelf; the trash glyph remains reserved for Clear Temporary Items.
- [ ] Single Pin/Unpin uses the existing item lifecycle mutation. Bulk Pinning uses the existing atomic all-items mutation, with Off and Mixed targeting all Pinned and On targeting all Temporary.
- [ ] Remove Item removes only the selected Shelf Item reference and prunes its Shelf Batch when empty. Remove Batch removes all references in the selected Shelf Batch regardless of pinned lifecycle.
- [ ] Pin, Bulk Pinning, Remove Item, and Remove Batch call the existing manager directly; Manage Items remains anchored to the rail row. No shared action controller or rail-specific lifecycle path is added.
- [ ] Each affected action is disabled while its durable mutation is pending, and manager-derived control state remains visible until persistence succeeds.
- [ ] Successful mutations refresh row previews, text, aggregate pin state, ordering, and visibility through the existing post-mutation callback.
- [ ] Pinning failure reports `Couldn’t update pinning. Nothing changed.` through the resident tray-warning path, leaves durable/in-memory state unchanged, and performs no automatic retry.
- [ ] Item-removal failure reports `Couldn’t remove item. Nothing changed.`; batch-removal failure reports `Couldn’t remove batch. Nothing changed.`; both preserve visible and durable state without retry.
- [ ] Remove Item and Remove Batch remain immediate reference-removal actions without new confirmation dialogs and never modify source filesystem objects.
- [ ] Removing the final Shelf Item across the shelf hides the Edge Rail immediately and resets sizing through the existing manager lifecycle.
- [ ] Reducing non-empty batch count refreshes rows without live shrinking the expanded rail; the next collapse/expansion recalculates the lower target.
- [ ] Clicking an action never begins whole-batch drag, activates DropCove, or steals foreground focus. Non-action row content remains the drag source.
- [ ] Open Shelf remains the only rail command that activates the full Drop Shelf, and Clear Temporary Items retains its existing confirmed shelf-wide semantics.
- [ ] Manager and real-SQLite tests cover lifecycle and atomic failure at the existing highest seam; installed smoke proves the actual row controls, busy state, tray warning, no-activate behavior, and unchanged source fixtures.

## Testing seam

Use DropShelfManager with real temporary SQLite for pinning, removal, atomic failure, final-item hiding, and restart durability. Use installed Edge Rail smoke for row wiring, accessibility state, pointer/drag precedence, tray feedback, and foreground-focus preservation.

## Demo path

From expanded Edge Rail rows, pin and unpin a single item, drive a mixed Shelf Batch to all Pinned and then all Temporary, remove one single item, and remove one complete Shelf Batch. Inject one persistence failure and show that the row and database remain unchanged while the tray reports the failure.

## Comments

- Direct row actions implemented: always-visible 24x24 Pin/Unpin + Remove Item for single items; tri-state Bulk Pinning + Manage Items + Remove Batch for multi-item batches.
- Non-optimistic UI verified: control state reflects manager truth while pending; controls are disabled during SQLite transactions; rollback verified on SQLite abort.
- Physical pointer interaction verified: no foreground theft from an external anchor window. TogglePattern activation under UIA was confirmed to foreground the window independently of control code and is noted as a harness limitation.
- Native Shell notification verified via breakpoint and one-shot return probe: `Shell_NotifyIconW` received `NIM_MODIFY=1`, `NIF_INFO=0x10`, warning flag=2, title `Edge Rail`, and accepted messages with `delivered:true`.
- All temporary diagnostics and hooks removed. 226 regression tests pass.
- Evidence: `artifacts/parity-ticket03-notification-delivery.json` and `artifacts/parity-feature-qualification-record.json`.
