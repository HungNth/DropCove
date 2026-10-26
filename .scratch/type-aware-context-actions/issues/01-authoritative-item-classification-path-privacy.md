# 01: Authoritative Item Classification and Managed File Path Privacy

**Parent specification:** [DropCove 0.0.27 — Type-aware Item Actions](../spec.md)

**What to build:** Give every projected Shelf Item an authoritative External/Managed ownership classification and, for Managed Files, a Text/Image content kind. Cut all current presentation behavior over to that contract so DropCove never exposes Managed File Profile Data paths in tooltips or accessibility text while External References retain their complete paths.

**Blocked by:** None (can start immediately)

**Status:** resolved

## Acceptance criteria

- [ ] Every Shelf snapshot and Manage Items projection carries a valid authoritative classification: External with no Managed content kind, or Managed with exactly Text or Image.
- [ ] Classification comes from Core Shelf State and existing Managed File ownership records; WebView code does not infer ownership or text capability from `/managed/`, filename extensions, Profile Data roots, or display labels.
- [ ] No persistence schema, durable setting, or item lifecycle is added or changed.
- [ ] External Reference card and row tooltip/accessibility behavior retains the exact full path.
- [ ] Managed Text card and row presentation uses its user-visible name and text content/preview where appropriate, never its Profile Data path.
- [ ] Managed Image card and row presentation uses its user-visible name and `DropCove-created image`, never its Profile Data path.
- [ ] Shell Visuals, Pin/Unpin, Remove, Batch Selection, Item Selection, drag-in, drag-out, restart restore, and virtualization retain existing behavior.
- [ ] Frontend behavior tests prove privacy and labels through observable DOM/ARIA output rather than source-text assertions.
- [ ] Native tests use temporary SQLite/Profile Data to prove External, Managed Text, and Managed Image classification across fresh intake and restart.
- [ ] Frontend production build and freshly cargo-xwin-compiled native regression tests pass for this slice.

## Testing seam

Use current Shelf snapshot/Manage Items projection seams with real temporary SQLite and Managed File records, plus the existing Vue component renderer tests for single-item cards and rows. Compare user-visible tooltip/accessibility output and authoritative serialized projection fields; do not test SQL strings or private helpers.

## Demo path

1. Create one External Reference, one Managed Text item, and one Managed Image item.
2. Inspect their projected classification on the Drop Shelf and in Manage Items.
3. Verify the External path remains available while neither Managed item exposes its Profile Data path.
4. Restart the owned profile and verify the same classification and presentation return.

## Dependencies and limits

- Reuses the existing External Reference versus Managed File ownership model and ADR-0006.
- This ticket adds no Item Actions Menu or native clipboard/Explorer operation; those begin in Ticket 02.
