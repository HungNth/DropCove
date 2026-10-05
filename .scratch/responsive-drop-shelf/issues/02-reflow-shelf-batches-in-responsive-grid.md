# 02: Reflow Shelf Batches in a responsive grid

**Parent specification:** [Responsive Resizable Drop Shelf Specification](../spec.md)

**What to build:** Make the single Drop Shelf use its user-selected width efficiently by wrapping Shelf Batch cards into deterministic columns while retaining one compact card design, newest-first ordering, vertical scrolling, direct actions, virtualization, and stable window bounds.

**Blocked by:** 01: Cut over to one resizable Drop Shelf.

**Status:** complete

**Testing seam:** Test decision-rich column and geometry calculations as pure behavior when they are exposed independently; use the installed Release surface as the authority for WinUI wrapping, stretching, scrolling, virtualization, and pointer responsiveness.

**Demo path:** Resize a populated shelf across each column boundary, add and remove batches without changing its bounds, scroll a long list, and invoke the existing card actions at narrow and wide sizes.

- [x] Shelf Batches use one responsive wrapping grid and one card presentation; resizing never activates a second card design, management mode, or former Expanded surface.
- [x] The content surface uses `8` logical pixels of horizontal padding per side, cards have a minimum width of `164`, and adjacent columns have a `4` logical-pixel gap.
- [x] The grid chooses the largest column count that fits the approved formula, producing one column at widths `180–347`, two at `348–515`, three at `516–683`, and subsequent columns by the same rule.
- [x] Cards stretch evenly across the usable row width, and resizing across a boundary reflows content without changing Shelf Batch or Shelf Item state.
- [x] Newest-first Shelf Batches fill left-to-right and then top-to-bottom.
- [x] Overflow scrolls vertically only; horizontal scrolling is disabled, and natural mouse-wheel scrolling remains available.
- [x] Single-item direct drag, Pin/Unpin, Remove, whole-batch drag, Remove Batch, drop targeting, and Clear Temporary Items remain reachable at every supported width.
- [x] Adding, removing, pinning, clearing, or restoring content never changes the user-selected window bounds while at least one item remains.
- [x] Virtualization and bounded card previews remain active so width changes do not eagerly materialize every Shelf Item or retain off-screen visual requests.
- [x] Focus order, automation names, visible focus indicators, reduced-motion behavior, and keyboard activation remain usable after grid reflow.
- [x] Focused automated tests cover exact column boundaries at `347/348`, `515/516`, and later formula boundaries when the layout calculation is exposed as pure logic; no tests assert XAML source text or event forwarding.
- [x] Installed Release smoke proves one-, two-, three-, and four-column layouts, even stretching, ordering, vertical-only scrolling, stable bounds during content mutations, and responsive interaction with the canonical 100-batch/1,000-item data set.

## Comments

- Installed self-contained Release smoke started with 100 batches/1,000 items. Actual columns: 180→1, 347→1, 348→2, 515→2, 516→3, 683→3, 684→4, 900→5. Minimum outer card width is 164 (the automation header rectangle is 154 plus two 1-pixel borders and two 4-pixel padding insets). At 348/516/684, card origins are separated by 168 pixels.
- `UniformGridLayout` still owns virtualized arrangement and stretching. A thin size-change handler caps columns with the approved `(usableWidth + 4) / 168` formula; no separately exposed calculation seam or source-text tests were added.
- UI Automation reported horizontal scrolling disabled and vertical scrolling enabled. Actual wheel input moved vertical scroll from 0% to 42.7989%; bottom reached 100%. At the bottom, 76 cards were realized rather than all 99 remaining batches. Preview projections remain bounded to three per card.
- Screenshot `ticket02-grid.png` in the disposable smoke directory was inspected: five uniform columns, consistent gutters, vertical scrollbar, no horizontal scrollbar or card overlap. Ellipsis and the partly visible bottom row are expected scrolling behavior. Black preview tiles are the deliberate 1×1 PNG fixtures.
- Actual single-path OLE drop added `Ticket01.txt` without changing 900×500 bounds. Keyboard Space pinned it at 180×180. A separate 900×500 run exercised Unpin and Pin, with identical before/after bounds. Remove Batch retained bounds; Clear Temporary Items retained the one pinned item and the same 900×500 bounds.
- Native geometry coverage now has 56 passing cases, including 96/144/192 DPI and all border directions. Full release/performance qualification belongs to ticket 04; a preliminary wide-grid observation was 239.22 MB and must be resolved or qualified against the gate before release completion.
- Inline batch expansion is removed in ticket 03 as explicitly specified there; the obsolete Expanded shelf surface and presentation mode are already gone. A pointer-drag attempt followed by Esc preserved the pinned reference but dismissed the shelf; this is not claimed as proof of actual drag initiation. Real item/batch drag qualification remains required in tickets 03/04.
