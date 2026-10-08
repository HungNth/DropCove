# 01: Match Edge Rail Manage Items presentation to Drop Shelf

**What to build:** Restyle the existing Edge Rail Manage Items Flyout so its complete rendered presentation matches the Drop Shelf Manage Items popup while preserving the Edge Rail's established placement, ownership, focus, motion, drag, lifecycle, scrolling, no-activate, and collapse-suppression behavior.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Parent specification:** [Edge Rail Manage Items Visual Parity](../spec.md)

- [ ] The Drop Shelf Manage Items popup remains visually and behaviorally unchanged and is the visual source of truth for this ticket.
- [ ] The Edge Rail Manage Items surface uses the same solid theme-aware background as the Drop Shelf popup rather than the native Flyout acrylic-style background.
- [ ] The Edge Rail surface renders the same one-logical-pixel card-stroke boundary, eight-logical-pixel corner radius, and six-logical-pixel inner padding as the Drop Shelf popup.
- [ ] The native Flyout default shadow is disabled, no visible elevation remains that is absent from the Drop Shelf popup, and no custom replacement shadow, backdrop, helper window, Mica, or Acrylic is introduced.
- [ ] The fixed header matches the Drop Shelf popup's rendered summary typography, eight-logical-pixel column gap, 24-by-24 Bulk Pinning target, alignment, and separation from the item scroller.
- [ ] Shelf Item rows match the Drop Shelf popup's rendered background, four-logical-pixel padding and corners, 24-logical-pixel visual, column spacing, text inset, 10-logical-pixel semibold name, 9-logical-pixel metadata, trimming, and tooltip behavior.
- [ ] The Edge Rail list has the same effective row-to-row separation as the Drop Shelf popup; matching one internal margin or spacing value is insufficient if the rendered total gap differs.
- [ ] Pin/Unpin and Remove Item remain 24-by-24 logical-pixel controls whose glyph size, alignment, and normal, pointer-over, pressed, disabled, checked, indeterminate, and focused presentation match the Drop Shelf popup where applicable.
- [ ] Existing theme resources remain authoritative for Light, Dark, and High Contrast; no fixed RGB/ARGB values or Edge Rail-only palette is added.
- [ ] The Edge Rail continues using its existing native Flyout ownership. It is not replaced with the Drop Shelf Popup, and no shared interactive Shelf Item template, action controller, coordinator, command bus, or new public interface is introduced.
- [ ] Flyout width remains content-sized from 320 through 480 logical pixels, total height remains bounded at 480 logical pixels, the header remains fixed, item rows scroll vertically, and horizontal scrolling remains disabled.
- [ ] Left and Right Edge Rail configurations continue opening Manage Items toward the desktop using existing work-area behavior.
- [ ] Pointer opening remains non-disruptive, keyboard opening focuses Bulk Pinning first, Tab and Shift+Tab traversal and wrapping remain intact, and Escape closes the Flyout and restores focus to the originating Manage Items control.
- [ ] Light dismissal, toggle-close behavior, no-activate behavior, pending mutation and drag guards, Flyout-held expansion, and normal post-close rail collapse timing remain unchanged.
- [ ] Pin/Unpin, Bulk Pinning, Remove Item, direct drag-out, Successful Drag-Out, cancellation, rejection, Missing cleanup, Unavailable retention, and source-filesystem ownership remain unchanged.
- [ ] Existing Flyout open/close motion and reduced-motion handling remain unchanged; no custom transition or animation timing is added.
- [ ] Deferred Shelf Item projections, visual request cancellation, thumbnail/native-icon policy, and release-on-close behavior remain unchanged.
- [ ] No persistence schema, settings, domain model, lifecycle state, availability state, runtime dependency, timer, polling loop, cache, telemetry, or background worker is added.
- [ ] The complete existing automated suite passes without adding tests that assert XAML source text, private element names, resource forwarding, exact template structure, or duplicated visual constants.
- [ ] An installed self-contained Release smoke run with an isolated representative multi-item Shelf Batch visibly proves the flat matched chrome, header alignment, item-row alignment, long-text trimming, vertical scrolling, primary pointer interaction, keyboard opening, and Escape focus restoration before this ticket is complete.

## Testing seam

Use the complete existing automated suite as regression coverage for lifecycle, persistence, sizing, placement, drag, visual-resource lifetime, settings, and native interop. Use the installed self-contained Release with an isolated profile as the authoritative seam for the actual WinUI Flyout background, boundary, corners, padding, shadow suppression, header, item rows, focus, and interaction. Do not add a screenshot-baseline or lower-level style-constant test.

## Demo path

Create one mixed-lifecycle multi-item Shelf Batch containing an image, a non-image file, a folder, long text, and enough Shelf Items to scroll. Open Manage Items from the Drop Shelf, then dismiss to the Edge Rail and open Manage Items for the same Shelf Batch. Show the matched flat surface, header, row spacing, typography, controls, and absence of shadow; scroll the item list; open by keyboard; traverse actions; and close with Escape to demonstrate unchanged focus and rail behavior.
