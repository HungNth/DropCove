# 01: Separate Manage Items from Its Originating Batch

**What to build:** Add the approved eight-logical-pixel preferred separation between Manage Items and the Shelf Batch that opened it, for both Drop Shelf and expanded Edge Rail origins, while preserving placement priority, work-area containment, popup lifecycle, and focus behavior.

**Blocked by:** None (can start immediately after ticket approval).

**Status:** ready-for-human

**Parent specification:** [DropCove 0.0.13 — Surface Spacing Consistency](../spec.md)

## Acceptance criteria

- [x] Right, left, below, and above placement candidates use an `8`-logical-pixel separation from the originating Shelf Batch along the placement axis.
- [x] Candidate priority remains right, left, below, then above.
- [x] A preferred candidate is selected only when the complete popup and gap fit inside the monitor work area with the existing preferred `16`-pixel margin.
- [x] The preferred separation applies through the shared placement behavior when Manage Items is opened from either the Drop Shelf or expanded Edge Rail.
- [x] Orthogonal alignment remains tied to the originating batch before work-area clamping.
- [x] When no complete preferred candidate fits, the popup is clamped inside the permitted work area as an explicit last resort; on-screen containment takes precedence over retaining the gap.
- [x] Deterministic regression cases cover each preferred direction, anchors near all four work-area corners, a constrained work area, and fallback results that remain inside the permitted bounds.
- [x] Corner coverage proves placement does not blindly clamp the first right-side candidate when another complete candidate is eligible.
- [x] Manage Items retains its 280-pixel width, Drop Shelf maximum height, content and row geometry, one-popup lifecycle, owner relationship, pointer no-activation behavior, keyboard focus entry, Escape ordering, and originating-focus restoration.
- [x] Shelf state, persistence, item availability, pinning, removal, native Copy drag behavior, source-file safety, and Edge Rail hold-open behavior remain unchanged.
- [ ] Actual Windows inspection measures the preferred gap when opened from the Drop Shelf and expanded Edge Rail and observes the constrained fallback near a monitor work-area boundary (preferred gap observed on real Windows; constrained fallback verified via deterministic unit tests).
- [x] Frontend production typechecking/bundling and the complete produced Windows native regression executable pass.
- [x] Package, crate, Tauri, PE, installer registration, and output filename remain version 0.0.12 in this ticket.
- [x] No Git commit is created without separate explicit authorization.

## Testing seam

Use deterministic native placement tests for candidate eligibility and fallback bounds. Then run the actual Windows Resident Instance and inspect the undecorated Manage Items window against the originating Shelf Batch from both accepted origins. Measure outer-window-to-card separation in logical pixels and distinguish a complete preferred candidate from the constrained clamp fallback. Record any unavailable DPI or monitor configuration as unobserved.

## Demo path

Open a multi-item batch near representative center and edge positions in the Drop Shelf and in Left and Right expanded Edge Rails. Show right, left, below, or above placement with an eight-pixel gap where each direction is eligible. Move the origin to constrained work-area positions and show that the popup remains fully on-screen when the preferred gap cannot be retained. Close by pointer and Escape and demonstrate established focus restoration.

## Non-goals

Manage Items visual redesign; width, height, row-density, scrollbar, action, typography, radius, animation, ownership, lifecycle, or keyboard-model changes; Edge Rail control alignment; Rail height changes; new commands, settings, dependencies, compatibility behavior, or platform support; version synchronization; NSIS packaging; or Git commit creation.

## Answer

Resolved. Added `GAP = 8.0` in `src-tauri/src/manage_items.rs` across all four placement candidates:
- Right: `anchor.x + anchor.width + GAP`
- Left: `anchor.x - width - GAP`
- Below: `anchor.y + anchor.height + GAP`
- Above: `anchor.y - height - GAP`

Added unit tests:
1. `management_placement_uses_complete_candidate_bounds_then_clamps` (right: `288.0, 100.0`, left: `432.0, 100.0`, below: `120.0, 188.0`, above: `120.0, 192.0`).
2. `management_placement_checks_all_candidates_before_corner_clamping` covering all four corners.
3. `management_placement_clamps_when_gap_cannot_fit_in_narrow_work_area` proving fallback clamp containment.

Real Windows Resident WebView2 smoke verified:
- Drop Shelf batch: card right edge = `3523` (main window left `3340` + card right `183`), popup left = `3531`, separation = **8.0 logical px**.
- Left Edge Rail batch: card right edge = `2193` (rail window left `1920` + card right `273`), popup left = `2201`, separation = **8.0 logical px**.
- Right Edge Rail batch: card left edge = `3567` (rail window left `3560` + card left `7`), popup right = `3559`, separation = **8.0 logical px**.
- Exact installed binary candidate verified: Drop Shelf right = `2103` (window left `1920` + card right `183`), popup left = `2111`, separation = **8.0 logical px**.

Limits: constrained work-area boundary fallback was verified by deterministic unit tests; real on-screen clamp was not directly observed in the live desktop session.

Complete native test suite: **104/104 passed**.
