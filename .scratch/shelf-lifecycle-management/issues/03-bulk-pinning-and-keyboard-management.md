# 03: Bulk Pinning and keyboard management lifecycle

Status: resolved
Type: task
Version: 0.0.3
Parent: [Specification](../spec.md)
Blocked by: 02
Implementation approval: approved by the user on 2026-10-13

## What to build

A user can manage the entire batch through tri-state Bulk Pinning on the card and management header, and perform the complete Manage Items workflow using predictable keyboard/focus behavior. Extend the already-working pointer/item management slice rather than recreating its popup adapter.

## Acceptance criteria

- [x] Bulk state is derived from item states only: all Temporary is Off, all Pinned is On, and mixed is Indeterminate; no batch pin field is introduced.
- [x] Off or Indeterminate activation Pins every retained item; On activation makes every retained item Temporary in one durable operation.
- [x] Bulk Pinning includes retained Available, Missing, and Unavailable items without reclassifying, reordering, resizing, or changing source objects.
- [x] The multi-item card and management header expose accessible tri-state controls; the header also shows an accurate X/N pinned summary and explicit Close.
- [x] Enter/Space opening focuses Bulk Pinning; pointer opening still does not force keyboard focus.
- [x] Tab/Shift+Tab cycle through Bulk Pinning, Close, and every row's Pin/Unpin and Remove controls in the canonical order.
- [x] Explicit Close restores focus to the originating Manage Items control when it still exists.
- [x] Opening another batch replaces the view with the correct new origin and data, without restoring obsolete focus as an intermediate action.
- [x] During an active item drag, Escape cancels the native drag first; a later Escape can close Manage Items. The same Escape event does not dismiss multiple surfaces.
- [x] Whole-batch drag closes Manage Items before native drag begins; item drag preserves it while the batch remains.
- [x] Owner hiding/removal and batch removal close the management surface and release its projection.
- [x] Failure during Bulk Pinning preserves the previous visible/durable tri-state and reports the failure.

## Testing seam

Core checks exercise all three bulk states, atomic transitions, availability-independent membership, pin state changes around prepared drag completion, and rollback. Native smoke exercises pointer versus keyboard opening, traversal, explicit-close focus restoration, Escape ordering, replacement, and item/batch drag lifecycle.

## Demo path

Open a multi-item batch by keyboard, Pin one row to create mixed state, activate Bulk Pinning to Pin all, then activate On to make all Temporary. Traverse in both directions, cancel an item drag with Escape, close with a later Escape, and observe restored origin focus.

## Non-goals

Resident shelf dismissal and EdgeDocked transitions remain the later roadmap versions. Ticket 05 adds closure at actual native resize start; do not substitute a programmatic-size notification or process exit for that behavior.

## Comments

- Baseline committed before implementation, as explicitly requested: `0cee503` — `feat: deliver DropCove core shelf and native item management` (43 files). Ticket 03 changes remain uncommitted; this commit is not a claim that 0.0.3 is released.
- Added atomic `ShelfState::set_batch_pinned(batch_id, pinned)` and matching async IPC. Desired flag is explicit, all retained item IDs participate, snapshot is constructed inside the transaction, and durable commit precedes publication. No schema, batch pin flag, dependency, or second state owner was added.
- One shared BulkPinning button derives `aria-pressed=false/true/mixed` from item flags on both card and popup. Header reports X/N pinned. Keyboard versus pointer opening is explicit through the existing popup adapter; keyboard focus waits for the management projection/render. Explicit Close emits the current origin ID and restores its Manage Items button if present; internal lifecycle closure does not restore obsolete origins.
- Document-level keyboard capture cycles Bulk, Close, each row's Pin/Remove in both directions, including when disabling controls during native drag moves DOM focus to BODY. Native canceled-item notification guards the cancellation Escape through keyup; a later Escape dismisses. The initial root-only handler failed this smoke and was replaced; that failed attempt is not passing evidence.
- `pnpm build` passed on the final frontend. Windows x64 release with `tauri/custom-protocol` built successfully. Direct native test execution reported **23 passed, 0 failed**, including three new real-SQLite checks for bulk transitions/availability invariance, partial-update rollback across reopen, and current pin state winning over an earlier prepared drag. External CRT PDB linker warnings remained non-fatal.
- Native app smoke exercised Enter and Space activation, initial Bulk focus, the complete eight-control Tab cycle and reverse wrap, Off→Mixed→On→Off and both-surface summary/state synchronization, real SQLite bulk-update failure with unchanged rows/state, and explicit-close restoration to the exact current origin.
- Controlled physical mouse/Escape input inside the owned popup proved the first Escape retained the item references and popup after OLE cancellation; a separate later Escape closed it and restored the opening control. Whole-batch OLE Copy closed management and copied both participating files with matching source bytes while retaining the all-Pinned batch. No DOM-only event was counted as native OLE cancellation proof.
- Replacement reused the same native window and switched origin/projection without pointer activation; subsequent explicit Close focused the new origin rather than the old one. Batch removal and owner hide closed/released management. Source fixtures were retained through the exercised lifecycle.
- Closed managed CDP tabs, terminated/waited only the known owned app/Copy-target processes, released mouse/Escape and SQLite handles, removed the owned test profile/fixtures/WebView/copies, and deleted the temporary Copy-target script. Final owned-directory existence check was false. No ambient Shelf profile/recovery data used.
- Resident dismissal/EdgeDocked behavior and resize-start integration remain their approved later tickets. Version metadata stays at 0.0.2 until release ticket 06; full installer acceptance has not been claimed.
