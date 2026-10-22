# DropCove 0.0.22 — Compact Manage Items Popup

Status: resolved — user tested, accepted, and authorized commit on 2026-10-22
Design confirmation: user approved the 240-logical-pixel popup, availability-only second line, retained non-visual full path, shared Drop Shelf/Edge Rail behavior, and 0.0.22 assignment on 2026-10-22.
Testing seam: user approved actual Windows Resident/WebView2 and installed-candidate verification as the primary seam, supported by existing native placement regressions, frontend production build, and the cargo-xwin-built native Windows suite on 2026-10-22.
Ticket decomposition: user approved two sequential vertical slices on 2026-10-22; published as [01 — Compact Manage Items presentation](issues/01-compact-manage-items-presentation.md) and [02 — Release DropCove 0.0.22](issues/02-release-dropcove-0.0.22.md).
Implementation approval: user requested implementation of both tickets on 2026-10-22; release human acceptance and commit authorization remain separate gates.
Release assignment: 0.0.22
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.22
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), section 10
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md)

## Problem Statement

Manage Items is wider than the user needs. Its current 280-logical-pixel surface dedicates each row's second text line to a shortened filesystem path while placing availability beside the trailing actions. The visible path competes with the item name and controls, and the resulting layout needs more horizontal space than the management workflow warrants.

The user wants a smaller popup without losing item identity, availability, Pin/Unpin, Remove Item, direct drag-out, keyboard operation, virtualization, placement safety, or source-file safety.

## Solution

Reduce Manage Items to a fixed width of 240 logical pixels when opened from either the Drop Shelf or the Edge Rail. Keep each row's item name on the first text line and move the availability value to the second line where the shortened path currently appears. Show exactly the current availability classification—`Available`, `Unavailable`, or `Missing`—without the duplicate `Pinned ·` prefix.

Remove the path only from visible row content. Preserve the full path in the drag region's tooltip and accessibility text so users can still distinguish identically named items and inspect the complete reference without consuming row width.

Preserve the established 54-logical-pixel row height, 5-pixel row gap, 480-logical-pixel maximum popup height, bounded scrolling, virtualization, actions, direct drag-out, placement order and gap, focus behavior, lifecycle, appearance semantics, and accessibility contracts.

## User Stories

1. As a Manage Items user, I want a narrower popup, so that item management occupies less screen space.
2. As a Drop Shelf user, I want Manage Items to use the 240-pixel width, so that the popup feels proportional to its compact content.
3. As an Edge Rail user, I want the same Manage Items width and content layout, so that the popup does not change behavior by origin.
4. As a user scanning a row, I want the item name on the first line, so that the primary identity remains prominent.
5. As a user checking current usability, I want availability on the second line, so that `Available`, `Unavailable`, or `Missing` is easy to find in the narrower layout.
6. As a user managing a Pinned Item, I want the availability line to omit the duplicate `Pinned ·` prefix, so that one line communicates one state.
7. As a user inspecting Pin state, I want the existing Pin icon state and `X/N pinned` summary preserved, so that removing the text prefix does not hide lifecycle state.
8. As a pointer user, I want the full path available through the existing tooltip, so that removing visible path text does not prevent exact identification.
9. As a screen-reader user, I want the full path retained in accessibility text, so that compact presentation does not remove reference identity.
10. As a user with identically named files in different folders, I want the complete path available non-visually, so that I can distinguish the references when needed.
11. As a user, I want Pin/Unpin and Remove Item controls unchanged, so that compacting the popup does not alter management commands.
12. As a user, I want direct item drag-out unchanged, so that I can still copy one managed reference without moving the entire Shelf Batch.
13. As a keyboard user, I want the established Tab, Shift+Tab, Enter, Space, and Escape behavior preserved, so that the narrower layout does not change navigation or dismissal.
14. As a keyboard user, I want explicit close to restore focus to the originating Manage Items control, so that I can continue from the same Shelf Batch.
15. As a user with a large Shelf Batch, I want bounded virtualization and scrolling preserved, so that narrowing the popup does not make large batches slower or unreachable.
16. As a user near a monitor edge, I want the popup's established placement and work-area containment preserved using its new width, so that it remains fully usable.
17. As a Dark, Light, or Windows Contrast user, I want the existing semantic appearance retained, so that compacting content does not introduce a separate visual language.
18. As a release owner, I want the change delivered as version 0.0.22 with synchronized release metadata and exact-candidate evidence, so that the roadmap remains sequential and auditable.
19. As a DropCove user, I want Shelf state, availability classification, lifecycle, drag results, persistence, and source files unchanged, so that this presentation refinement cannot change stored references or filesystem content.

## Implementation Decisions

- Set the shared native Manage Items width to 240 logical pixels. The same width applies when opened from the Drop Shelf or Edge Rail; do not add per-origin width configuration.
- Preserve the existing right, left, below, then above placement preference, preferred 8-pixel origin gap, preferred 16-pixel work-area margin, and containment-first clamp fallback. Recalculate placement using the new popup width.
- Keep the existing dynamic height behavior and 480-logical-pixel maximum. Preserve 54-pixel rows, 5-pixel gaps, collection padding, scrollbar behavior, and virtualization measurements.
- Replace only the visible shortened path with the item's availability value on the second text line. Continue rendering the item name on the first line.
- Render only `Available`, `Unavailable`, or `Missing` on the availability line. Do not prepend Pin state because the Pin icon, pressed state, accessible name, and header summary already own that information.
- Preserve the full path in the existing drag-region tooltip and accessibility text. Do not add another visible path, disclosure control, copy-path action, or custom tooltip.
- Preserve the header title, item count, `X/N pinned` summary, Bulk Pinning, Close, row Shell Visual, Pin/Unpin, Remove Item, direct drag-out, status reporting, and action order.
- Retain the established semantic color tokens, Segoe UI typography, Lucide icons, radii, focus treatment, reduced-motion behavior, and Windows Contrast mappings. Add no new visual token, animation, shadow, dependency, or component abstraction.
- Keep one shared management projection and the existing lifecycle. Add no state owner, persistence field, settings option, command, compatibility path, or telemetry.
- Keep version metadata at 0.0.21 during implementation. Synchronize canonical package, crate, lockfile, Tauri, executable, installer, and uninstall metadata to 0.0.22 only for the approved release gate.
- Historical specifications and tickets that accurately describe earlier 280-pixel releases remain unchanged. The canonical Product Specification and 0.0.22 roadmap entry supersede that historical geometry for future behavior.
- No glossary update is required because `Manage Items`, `Shelf Item`, availability states, and full-path identity retain their existing meanings.
- No ADR is required because this is a reversible presentation refinement inside established ownership, placement, and lifecycle boundaries.
- Do not create a Git commit without separate explicit user authorization.

## Testing Decisions

- Primary seam: the actual Windows Resident Instance and real WebView2 Manage Items surface opened from both the Drop Shelf and Edge Rail. Measure the 240-logical-pixel client width and inspect the row layout rather than inferring appearance from source.
- Exercise short and long item names, identical names from different full paths, and `Available`, `Unavailable`, and `Missing` rows. Verify the path is absent visually, the correct availability value occupies the second line, and no `Pinned ·` prefix remains.
- Hover the draggable row region and inspect accessibility output to prove the exact full path remains available despite its removal from visible content.
- Exercise all-Temporary, all-Pinned, and mixed batches. Verify Pin icon state, `X/N pinned`, Bulk Pinning, Pin/Unpin, Remove Item, status reporting, and availability remain coherent.
- Exercise pointer opening, keyboard opening, complete Tab and Shift+Tab traversal, Enter/Space activation, Escape drag cancellation and later dismissal, explicit Close, and focus restoration.
- Exercise ordinary and virtualized batches, scrolling to the beginning, middle, and maximum position. Preserve bounded mounting, 54-pixel row geometry, 5-pixel gaps, focused-row retention, reachable final rows, and correct accessibility set position.
- Update the existing native placement regression expectations to 240 logical pixels. Cover right, left, below, above, corner, constrained-work-area, and clamp fallback cases so the narrower width cannot regress placement safety.
- Inspect Dark, Light, and actual Windows Contrast Theme where available. Preserve focus, muted availability text, borders, actions, disabled state, and reduced-motion behavior. Record unavailable physical coverage as unobserved rather than inferred.
- Run the existing frontend production typecheck/build and compile through cargo-xwin for `x86_64-pc-windows-msvc`; execute the produced Windows native regression binary directly.
- After implementation acceptance, synchronize version 0.0.22, build one exact current-user Windows x64 NSIS candidate, and exercise the installed bytes with embedded assets and the development server unavailable. Cover Manage Items from both origins plus resident startup/access, Cooperative Shutdown, same-candidate reinstall, uninstall, and Profile Data preservation.
- Add no frontend component-test framework, screenshot snapshots, source-text assertions, CSS-class assertions, or tests that merely prove copied labels or forwarding. Actual-surface verification is the highest existing seam for this geometry and content change.

## Out of Scope

- Changing Shelf state, persistence, duplicate handling, availability classification, Temporary/Pinned lifecycle, Copy-only drag semantics, source-file behavior, or management projection ownership.
- Changing row height, row gap, popup maximum height, dynamic height policy, scrollbar styling, collection padding, virtualization threshold, overscan, or keyboard-navigation model.
- Changing the header layout, item count, pinned summary, Bulk Pinning, Close, Shell Visual size, action target size, icons, action order, status surface, or drag threshold.
- Adding a path toggle, copy-path action, details view, resizable popup, adaptive width setting, per-origin geometry, or user-configurable density.
- Redesigning the Drop Shelf, Edge Rail, Settings, About, confirmation surfaces, palette, typography, animation, or broader design system.
- Adding dependencies, telemetry, schema migration, platform support beyond Windows x64, signing, or automatic updates.
- Rewriting historical 0.0.3, 0.0.10, 0.0.12, or 0.0.13 evidence.
- Creating a Git commit without separate explicit user authorization.

## Further Notes

The user selected 240 logical pixels over 220 and 260. The 220-pixel option would truncate names more aggressively; 260 pixels would make only a modest reduction from the current 280-pixel width. The approved layout spends its single visual change on density: name first, availability second, actions trailing, and full path available only when requested through tooltip or accessibility tooling.

The design deliberately reuses the current palette, typography, iconography, spacing rhythm, and interaction model. No decorative treatment is added; removing the visible path and duplicate Pin text is the compaction mechanism.

## Delivery Evidence — 2026-10-22

- Ticket 01 implementation and Ticket 02 exact installed verification are complete; candidate 0.0.22 was accepted by the user on 2026-10-22 and the commit was authorized.
- Final actual-surface smoke found and corrected Flexbox shrink in ordinary overflow rows by making each row non-shrinking. Both popup origins now preserve the approved 240-pixel width, 54-pixel rows, five-pixel gap, and bounded height, with no new geometry policy or abstraction.
- Frontend checks passed 13/13 and native Windows checks passed 124/124. Final installed smoke, exact artifact hashes, supporting provisional evidence, restoration, and unobserved hardware/accessibility configurations are recorded in [ROADMAP 0.0.22](../../docs/ROADMAP.md#0022--compact-manage-items-popup) and the release ticket.
- User acceptance and separate commit authorization were completed on 2026-10-22.

### User acceptance — 2026-10-22

- User confirmed `tất cả đã đạt, hãy commit các thay đổi`, authorizing the release commit.