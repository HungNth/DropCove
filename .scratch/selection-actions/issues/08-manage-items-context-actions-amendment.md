# 08: Manage Items Context Actions Amendment

**Amends:** [Ticket 07 — Batch Card Context Actions Amendment](07-batch-card-context-actions-amendment.md), [DropCove 0.0.28 — Selection Actions](../spec.md), and [DropCove 0.0.27 — Type-aware Item Actions](../../type-aware-context-actions/spec.md)

**Authorization:** The user directly requested Manage Items parity with the Drop Shelf and Edge Rail behavior. This ticket authorizes the scoped contract amendment and implementation; it does not authorize a Git commit or release.

**Status:** resolved

**What changes:** Remove the per-row More trigger from Manage Items and resolve secondary-click/keyboard menus from the resulting Item Selection. One selected row uses the existing ownership-safe type-aware Item Actions. Several selected rows retain lifecycle actions with the same All Selected wording as Shelf Batch Selection.

## Acceptance criteria

- [x] One External Reference row exposes **Show in File Explorer** followed by **Copy Path** through secondary-click, Context Menu key, or `Shift+F10`.
- [x] One Managed Text row exposes **Copy Text**. A Managed Image row exposes no menu and never reveals its internal path.
- [x] Manage Items rows render no More/ellipsis action. Direct Pin/Unpin and Remove controls remain unchanged.
- [x] Two or more selected rows expose **Pin All Selected** or **Unpin All Selected**, followed by **Remove All Selected**.
- [x] Secondary-clicking a selected row preserves the complete Item Selection; secondary-clicking an unselected row replaces selection before resolving actions.
- [x] Drop-Shelf-origin Manage Items focuses the first menu action and restores focus to the invoking row after Escape or Copy. Rail-origin Manage Items remains pointer-only/no-activate and never acquires focus.
- [x] Item Action requests still send only the exact Shelf Item identity plus finite action. Managed File paths remain private.
- [x] Multi-selection Pin/Unpin and removal retain existing atomic mutation, confirmation, cleanup, source-safety, virtualization, and popup-lifetime contracts.

## Testing seam

Use the mounted Manage Items component and Shelf Item row seams for type-aware singleton mapping, multi-selection scope, keyboard parity, focus/no-activate behavior, More removal, action payload, and unsupported Managed Image suppression. Preserve existing Core Shelf State Item Action and selection mutation regressions; no backend contract change is required.

## Verification

- Red-capable loop: `node --test tests/manageItemsSelection.test.mjs tests/itemActionsMenu.test.mjs` failed on rendered More controls and the old `Pin, More, Remove` row order before implementation.
- Focused Manage Items regressions passed **19/19**, including External/Managed Text/Managed Image mapping, pointer and keyboard invocation, focus restoration, 65-item virtualization, complete Item Selection scope, exact Item Action payload, and Rail-origin singleton plus multi-row no-activate behavior.
- Complete frontend suite passed **87/87**; `pnpm build` passed Vue typechecking and the Vite production build after the source cutover.
- A Chromium-rendered Manage Items surface showed no row More button, Show in File Explorer/Copy Path through pointer and `Shift+F10`, Copy Text for Managed Text, no menu for Managed Image, and Pin All Selected/Remove All Selected for two selected rows. The mock native boundary recorded the exact `{ itemId: "ext-a", action: "CopyPath" }` request.
- No Rust/native contract changed. Native clipboard and Explorer side effects were not re-exercised for this presentation-only amendment.
