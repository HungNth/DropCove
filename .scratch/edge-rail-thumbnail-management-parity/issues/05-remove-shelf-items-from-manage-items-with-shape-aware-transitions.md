# 05: Remove Shelf Items from Manage Items with shape-aware transitions

**What to build:** Complete Edge Rail Manage Items parity by adding durable per-item removal and making the flyout, owner row, Shelf Batch, and Edge Rail transition coherently as removal changes the number of remaining Shelf Items.

**Blocked by:** 04: Upgrade the Edge Rail Manage Items flyout for thumbnail and pin parity.

**Status:** ready-for-agent

**Parent specification:** [Edge Rail Thumbnail and Management Parity](../spec.md)

- [ ] Every Manage Items row exposes the Drop Shelf-equivalent 24 by 24 Remove Item action after Pin/Unpin, using the `×` affordance and a target-specific accessible name.
- [ ] Remove Item invokes the existing exact-item manager mutation, commits durably before presentation refresh, and never modifies the referenced filesystem object.
- [ ] The affected Remove Item action is disabled while pending, the flyout and rail remain open, and duplicate activations cannot queue ambiguous removals.
- [ ] When at least two Shelf Items remain, the flyout stays open, its fixed header/count/pin state and item rows refresh, and focus moves to the nearest remaining logical action.
- [ ] When exactly one Shelf Item remains, Manage Items closes, all flyout visual requests and projections are released, and the owner row becomes the direct single-item preview with Pin/Unpin and Remove Item controls.
- [ ] When zero Shelf Items remain, the empty Shelf Batch and its flyout disappear. If other batches remain, expanded height does not live-shrink; the next collapse/expansion recalculates the lower target.
- [ ] Removing the final Shelf Item across all batches transitions the shelf to Hidden and hides the Edge Rail immediately through the existing lifecycle and sizing reset.
- [ ] Removal refresh cannot leave stale realized rows, stale flyout ownership, stale thumbnail ImageSource references, stale aggregate pin count, or focus on a removed element.
- [ ] SQLite/IO failure reports `Couldn’t remove item. Nothing changed.` through the resident tray-warning path, retains current in-memory and durable state, keeps the item visible, restores actionable focus, and performs no automatic retry.
- [ ] Remove Item remains immediate without a new confirmation dialog; Clear Temporary Items remains the only confirmed shelf-wide cleanup in this surface.
- [ ] Individual item drag before and after removals retains existing accepted Temporary consumption, Pinned retention, cancellation, Missing cleanup, and Unavailable retention.
- [ ] Real-SQLite manager tests remain the authoritative durability/failure seam, while installed smoke covers flyout retention at two or more items, closure at one, pruning at zero, final hide, focus recovery, and visual release.
- [ ] Source filesystem fixtures remain unchanged after successful and failed removal scenarios.

## Testing seam

Use DropShelfManager with real temporary SQLite for exact-item removal, pruning, final Hidden transition, durability, and abort behavior. Use installed Manage Items smoke for shape-dependent flyout lifetime, focus, realized-row refresh, resource release, and no-activate behavior.

## Demo path

Start with a four-item Shelf Batch and remove items one at a time. Show the flyout remain open at three and two, close and reproject the owner at one, then remove the final item to hide the rail. Repeat one removal with deterministic persistence failure and show that nothing changes.

## Comments

- Per-item removal in Manage Items flyout implemented with `×` affordance and action-specific accessible names.
- Shape-aware transitions verified in running Release smoke:
  - 3 items remaining: flyout stays open, count and header refresh.
  - 2 items remaining: flyout stays open.
  - 1 item remaining: flyout closes immediately, owner row reprojects to single-item presentation with Pin and Remove Item controls.
  - 0 items remaining: empty batch pruned; remaining batches not live-shrunk.
  - Final item across shelf removed: Edge Rail hides immediately and resets sizing.
- Focus transition verified: removing an item via keyboard moves focus to the nearest surviving Pin/Remove control; removing down to 1 item closes flyout and focuses the rail.
- Rollback on SQLite failure verified: item retention and flyout state preserved.
- Evidence: `artifacts/parity-feature-qualification-record.json`.
