# 15: Add Compact and Expanded shelf modes

**What to build:** Add distinct quick-access and management presentations over the proven shelf behavior without changing Path Reference, Shelf Batch, Shelf Item, drag, or persistence semantics.

**Blocked by:** 14: Complete accepted shake drops and bound hook overhead.

**Status:** ready-for-agent

- [ ] Compact mode emphasizes recent Shelf Batches, remains bounded, and provides access to additional batches without unbounded growth.
- [ ] Single-item and multi-item Shelf Batches have clearly different native visuals, including stacked treatment for grouped items.
- [ ] Expanded mode shows all Shelf Batches and Shelf Items with paths, availability, pinned state, and management actions.
- [ ] Users can pin/unpin, Remove Item, Remove Batch, Clear Temporary Items, and initiate one-item or whole-batch drag from the management view.
- [ ] Keyboard navigation and activation work in Expanded mode.
- [ ] Compact and Expanded modes consume the same application state and do not duplicate lifecycle or persistence logic.
- [ ] Arbitrary cross-batch multi-selection and file-launch actions are not introduced.
- [ ] Automated application-seam tests remain unchanged where behavior is presentation-independent; packaged UI smoke covers mode transitions and management workflows.