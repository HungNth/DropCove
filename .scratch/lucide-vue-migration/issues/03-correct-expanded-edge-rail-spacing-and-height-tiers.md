# 03: Correct Expanded Edge Rail Shelf Batch Spacing and Height Tiers

**What to build:** Give the expanded Edge Rail equal 6-pixel left-border-to-card and card-to-scrollbar-channel clearances, balanced 8-pixel top and bottom collection clearance, and the native height budget that fits one through eleven Shelf Batches without premature scrolling while keeping twelve-or-more collections bounded and scrollable. Keep UI, native sizing, regression coverage, runtime behavior, and living Product Specification consistent.

**Blocked by:** None (can start immediately).

**Status:** ready-for-human

**Parent specification:** [Lucide Vue Migration and Shelf Batch Spacing Correction](../spec.md)

## Acceptance criteria

- [x] Both expanded Edge Rail horizontal insets are 6 logical pixels; measured left-border and right-scrollbar-channel clearances are equal.
- [x] Insets remain symmetric with and without overflow; when visible, the rounded 4-pixel painted thumb starts at the content-side edge of its 8-pixel hit channel, keeping the visible right gap equal to the 6-pixel left gap.
- [x] The first Shelf Batch row is separated from the top collection viewport edge by a fixed 8-pixel boundary outside the actual scrollport.
- [x] Fully and partially visible Shelf Batch rows cannot paint into the fixed top boundary at the start, middle, or maximum-scroll position.
- [x] The final Shelf Batch row has 8 pixels of bottom clearance inside the scroll range and remains fully reachable at maximum scroll without extra blank range.
- [x] Short collections remain top-aligned; rows are not centered or distributed through unused height.
- [x] Expanded Rail width remains 280 logical pixels, Shelf Batch rows remain 48 pixels high, row gaps remain 4 pixels, and the existing header and footer geometry remains unchanged.
- [x] Shelf Batch titles and action controls remain usable without overlap or unacceptable truncation with symmetric 6-pixel insets and the scrollbar channel.
- [x] Expanded Rail target height follows `max(280, min(642, 52 × batch count + 70))`.
- [x] Geometry calculations return 280 pixels for zero through four batches, 330 pixels for five batches, and 642 pixels for eleven and twelve-or-more after clamping.
- [x] At runtime an empty Shelf continues to hide the Edge Rail, one through four batches remain at the minimum height, and five through eleven batches do not acquire a premature scrollbar.
- [x] Eleven Shelf Batches fit at 642 pixels without vertical overflow; twelve or more clamp at 642 pixels and use native vertical scrolling.
- [x] The final row remains fully reachable for twelve and more-than-fifty-batch collections.
- [x] Ordinary and virtualized collections retain zero-based coordinates, bounded mounting, total-height calculation, keyboard reveal, and computed-padding width measurement without an Edge Rail-specific offset.
- [x] Existing live growth when content is added and no-live-shrink behavior when content is removed remain unchanged.
- [ ] Left and Right docking, explicit and remembered monitor ownership, temporary primary-display fallback, fullscreen policy, no-activate behavior, expansion/collapse delays, Open Shelf, Clear Temporary Items, Manage Items, and confirmation holds remain unchanged.
- [ ] Focus, drag-in, Shelf Item drag-out, Shelf Batch drag-out, Pin, Remove, Manage Items, Clear, and Copy-only source-file safety remain unchanged.
- [x] Native scrollbar hit-channel width, painted-thumb width, semantic colors, hover/active states, transparent track, and Windows Contrast mappings remain unchanged; only the approved paint alignment changes.
- [x] Existing native Rail geometry coverage is updated for the approved exact heights, and the focused checks plus the complete native regression suite pass.
- [x] Complete Vue single-file-component typechecking, production bundling, and cargo-xwin native compilation pass for `x86_64-pc-windows-msvc`.
- [x] Real Windows WebView2 inspection confirms the approved insets, clearances, exact height tiers, overflow transition, control usability, and final-row reachability on both docking sides.
- [x] The living Product Specification is synchronized in the same change with the 642-pixel maximum, `+70` static budget, five-through-eleven tiers, and twelve-or-more scrolling behavior.
- [x] The completed 0.0.12 roadmap entry, specification, tickets, measurements, artifact hashes, and acceptance record remain unchanged as historical evidence.
- [x] No release version assignment, NSIS packaging, installer evidence, unrelated surface redesign, glossary entry, ADR, or compatibility path is included.
- [x] No Git commit is created without separate explicit user authorization.

## Testing seam

Use the actual Windows Resident Instance and real WebView2 expanded Edge Rail. Assert equal 6-pixel left-border and right-to-painted-thumb clearances using computed geometry and screenshot pixels for fitting, overflowing, and virtualized collections; channel-only equality is insufficient. Preserve the 8-pixel hit channel, 4-pixel rounded painted thumb, vertical boundaries, exact height tiers, reachability, controls, and native behavior on both docking sides. No new visual-test framework.

## Demo path

Expand the Edge Rail on both docking sides with one through four, five, eleven, twelve, and a virtualized collection of more than fifty Shelf Batches. Show 280-, 330-, and 642-pixel tiers; demonstrate that eleven batches fit without a scrollbar, twelve batches scroll with the final row fully reachable, partially visible rows remain behind the fixed top boundary, action controls remain usable, and Open Shelf, Manage Items, Pin, Remove, drag, and Clear retain their established behavior.

## Non-goals

Drop Shelf or Manage Items spacing; collapsed Rail Handle redesign; custom scrollbar work; row or action redesign; new placement behavior; altered monitor ownership; changed timers or focus policy; shared virtualization refactoring; release versioning; NSIS packaging; rewriting historical 0.0.12 evidence; glossary or ADR work; or Git commit creation.

## Earlier verification — superseded horizontal gutter

Source implementation and measured geometry complete. Real Right Rail counts 1–4 stayed at 280 pixels; counts 5–11 grew from 330 through 642 pixels with no overflow. Eleven batches measured scroll/client height 576/576; twelve measured 628/576 and scrolled at height 642. Both Left and Right 65-batch Rails reached final index 64 at scrollTop 2808 with 8-pixel bottom clearance, 10-pixel right inset, 48-pixel rows, bounded mounting, and no horizontal overflow. Left Rail remained no-activate and its controls were usable. Actual Windows Contrast and supporting Light/Dark emulation were inspected. Frontend/cargo-xwin builds and all 102 native tests passed; living Product Specification updated.

Unchecked criteria retain final human acceptance for physical OLE drag, exhaustive monitor/fullscreen/focus interactions, and the related preserved workflows. See [shared verification evidence and cleanup limits](../verification.md), including theme-cache and startup-registry restoration limits. No commit or release packaging. Remaining work is human acceptance, not deferred source implementation.

## Answer

The user-approved horizontal-balance correction is implemented and verified: both Left and Right Rails measure 6-pixel left-border and right-scrollbar-channel clearances. Eleven-batch fitting and twelve-batch overflow states passed; both docking sides also passed 65-batch virtualized start/middle/end checks with bounded mounting, no horizontal overflow, and final index 64 reachable with 8-pixel bottom clearance. The native height formula and 642-pixel maximum are unchanged. Frontend/cargo-xwin builds and 102 native regressions passed. See the dated [symmetric-gutter verification](../verification.md). Remaining physical-interaction acceptance is still human-owned; no commit.

### Painted-thumb follow-up — 2026-10-15

The user-approved paint-only fix retains the 8-pixel channel and moves its rounded 4-pixel paint to the content edge. Rebuilt Right Rail pixels place the paint at columns 271–274 against card edge 265, proving a 6-pixel visible gap. Ordinary 11/12-batch fitting/overflow states and 65-batch virtualized start/middle/end checks passed. Native height and row geometry are unchanged. Left docking, fractional DPI, and physical hover/drag were not re-exercised in this follow-up; see the latest [painted-thumb verification](../verification.md).
