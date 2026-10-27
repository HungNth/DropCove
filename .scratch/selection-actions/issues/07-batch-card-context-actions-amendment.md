# 07: Batch Card Context Actions Amendment

**Amends:** [DropCove 0.0.28 — Selection Actions](../spec.md) and [DropCove 0.0.27 — Type-aware Item Actions](../../type-aware-context-actions/spec.md)

**Authorization:** The user directly requested this behavior and selected the ownership-safe **single item, type-aware** rule before implementation. This ticket authorizes the scoped contract amendment and implementation; it does not authorize a Git commit or release.

**Status:** resolved

**What changes:** Remove the per-card More trigger from Shelf Batch cards and move the existing type-aware Item Actions to secondary-click for an eligible single-item batch. Keep lifecycle actions in the batch menu only when several Shelf Batches are selected.

## Acceptance criteria

- [x] A secondary-click on one selected or unselected single-item External Reference batch opens **Show in File Explorer** followed by **Copy Path**.
- [x] A secondary-click on one single-item Managed Text batch opens **Copy Text**. A Managed Image batch opens no menu and never exposes its internal path.
- [x] A secondary-click on one multi-item batch opens no menu because the card has no unambiguous Shelf Item target. The WebView default menu remains suppressed and normal Batch Selection targeting remains intact.
- [x] Shelf Batch cards on the Drop Shelf and expanded Edge Rail render no More/ellipsis action. Manage Items rows keep their existing More button and type-aware Item Actions Menu.
- [x] Two or more selected Shelf Batches expose **Pin All Selected** (or **Unpin All Selected** when every target item is Pinned) followed by **Remove All Selected**.
- [x] Multi-selection Pin/Unpin and removal retain the existing atomic mutation, confirmation, Managed File cleanup, source safety, selection, virtualization, and Edge Rail no-activate contracts.
- [x] Context Menu key and `Shift+F10` on the Drop Shelf use the same single-target type-aware or multi-selection action mapping. The Edge Rail remains pointer-only/no-activate.
- [x] Item Action requests still send only Shelf Item identity plus finite action. No caller-provided path/text becomes authoritative, and no Managed File Profile Data path is shown or copied.
- [x] Existing direct Pin/Unpin, Bulk Pinning, Manage Items, and Remove controls remain unchanged.

## Testing seam

Use the mounted Drop Shelf and Edge Rail component seams to prove the rendered labels, selection targeting, absence of the batch-card More button, native command identity/action payload, focus/no-activate behavior, and unsupported single-batch suppression. Preserve the existing Core Shelf State Item Action and selection mutation regressions; no backend contract change is required.

## Verification

- Red-capable loop: `node --test tests/dropShelfSelectionActions.test.mjs tests/itemActionsMenu.test.mjs` failed on the old singleton labels, old multi-selection labels, and rendered batch-card More button before implementation.
- Focused mounted regressions passed **26/26**, including Drop Shelf pointer and keyboard invocation, Edge Rail no-focus behavior, type-aware singleton mapping, ambiguous singleton suppression, selection confirmation, and More removal.
- Complete frontend suite passed **89/89**; `pnpm build` passed Vue typechecking and the Vite production build.
- A real Chromium-rendered Drop Shelf surface showed no batch-card More button, Show in File Explorer/Copy Path through pointer and `Shift+F10`, Pin All Selected/Remove All Selected for two selected cards, no menu for one multi-item batch, and the exact `{ itemId, action }` Copy Path request.
- No Rust/native contract changed. Native clipboard and Explorer side effects retain their existing Core Shelf State and Windows adapter coverage and were not re-exercised for this presentation-only amendment.
