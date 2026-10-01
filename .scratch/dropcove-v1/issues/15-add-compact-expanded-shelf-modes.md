# 15: Add Compact and Expanded shelf modes

**What to build:** Add distinct quick-access and management presentations over the proven shelf behavior without changing Path Reference, Shelf Batch, Shelf Item, drag, or persistence semantics.

**Blocked by:** 14: Complete accepted shake drops and bound hook overhead.

**Status:** complete

- [x] Compact mode emphasizes recent Shelf Batches with natural vertical scrolling, and provides dedicated Expand toggle without capping items behind a Manage button.
- [x] Single-item and multi-item Shelf Batches have clearly different native visuals, including stacked treatment for grouped items.
- [x] Expanded mode shows all Shelf Batches and Shelf Items with paths, availability, pinned state, and management actions.
- [x] Users can pin/unpin, Remove Item, Remove Batch, Clear Temporary Items, and initiate one-item or whole-batch drag from the management view.
- [x] Keyboard navigation and activation work through the native focusable mode and management controls in Expanded mode.
- [x] Compact and Expanded modes consume the same application state and do not duplicate lifecycle or persistence logic.
- [x] Arbitrary cross-batch multi-selection and file-launch actions are not introduced.
- [x] Automated application-seam tests remain unchanged where behavior is presentation-independent; packaged UI smoke covers mode transitions and management workflows.

## Comments

- 2026-10-01: Implemented shared-manager Compact and Expanded presentations. Per user direction for shelf UX: Compact mode supports natural scrolling of all Shelf Batches directly in the quick shelf while keeping batches collapsed by default, with Clear Temporary Items placed at bottom-left and Expand toggle placed at bottom-right. Expanded mode provides a widened management view for dealing with multiple files. The redundant `+N more batches · Manage` affordance was removed in favor of direct scrolling in Compact and footer Expand toggle.
- 2026-10-01: Application-seam suite remains presentation-independent except for explicit Compact/Expanded display-state coverage; full suite passes 68/68. Direct published-Release UI smoke verified Compact/Expanded elements, footer buttons, and batch expand/collapse behavior.
- 2026-10-01: Performance and packaged management smoke gates remain monitored; all changes uncommitted pending explicit user authorization.
- 2026-10-01: Packaged UI smoke confirmed mode transitions, single-item direct header pin/unpin toggle with persistence, multi-item expand/collapse, item and batch removal, and temporary item clearing on running Release build. 68/68 automated tests pass.