# DropCove 0.0.13 — Surface Spacing Consistency

Status: implementation exercised; awaiting user acceptance
Version: 0.0.13
Design confirmation: approved by user on 2026-10-15
Testing seam: native placement regression tests plus actual Windows Resident Instance and WebView2 surface inspection; approved by user on 2026-10-15
Ticket decomposition: three approved slices (`01` ready-for-human, `02` ready-for-human, `03` ready-for-human)
Implementation approval: approved by user on 2026-10-15; Git commit authorization remains separate
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.13
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 10 and 11
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md)

## Problem Statement

Manage Items currently opens flush against the Shelf Batch that requested it. The zero-pixel join makes two independent bordered surfaces read as one accidental shape and is inconsistent with the compact spacing rhythm used inside DropCove.

The expanded Edge Rail also uses different outer vertical offsets for its two corner actions. Open Shelf sits two logical pixels from the top edge, while Clear Temporary Items sits four logical pixels from the bottom edge. The mismatch is visible because both controls use the same 24-pixel target and occupy opposing ends of one narrow surface.

These are presentation-geometry defects. Shelf state, drag behavior, popup lifecycle, work-area containment, Edge Rail tiers, collection clearance, themes, accessibility semantics, and source-file safety must remain unchanged.

## Solution

Introduce an eight-logical-pixel preferred separation between Manage Items and the originating Shelf Batch. Apply the same placement contract when Manage Items is opened from the Drop Shelf or expanded Edge Rail. Preserve the existing right, left, below, then above preference and the preferred 16-pixel monitor work-area margin. When no complete candidate can satisfy both the gap and work-area bounds, clamp the popup inside the work area as an explicit last resort, allowing the visual gap to shrink or disappear rather than placing the popup off-screen.

Align the expanded Edge Rail actions to four-logical-pixel outer vertical offsets. Keep each action target at 24 by 24 logical pixels and each horizontal inset at six logical pixels. Move Open Shelf within the existing 28-pixel header budget by using four pixels above the control and no added space below it. Preserve the existing Clear footer budget and four-pixel bottom offset. Mirror the result across Left and Right docking without changing the Rail height formula or collection geometry.

## User Stories

1. As a Drop Shelf user, I want Manage Items visually separated from its originating batch, so that the popup reads as a related but independent surface.
2. As an Edge Rail user, I want the same Manage Items separation, so that opening management from either Shelf presentation follows one spatial rule.
3. As a user near a monitor edge, I want Manage Items kept within the work area even when the preferred gap cannot fit, so that the popup remains usable.
4. As a user with the Rail docked Left or Right, I want Open Shelf and Clear to use matching outer offsets, so that the narrow surface feels intentionally balanced.
5. As a pointer user, I want the existing 24-pixel action targets, tooltips, and hover states preserved.
6. As a keyboard or assistive-technology user, I want existing focus, activation, names, roles, and state semantics preserved.
7. As a user with many batches, I want Rail height tiers and scrolling behavior unchanged.
8. As a release owner, I want actual native-window and WebView2 evidence for the spacing rather than treating successful compilation as visual proof.

## Design Decisions

### Visual tokens

Retain the existing semantic design system and its Windows Light and Contrast mappings:

- Canvas: `#0b111e`;
- Surface: `#141c2b`;
- Elevated surface: `#202a3b`;
- Default border: `#46536a`;
- Accent: `#6366f1`;
- Danger: `#f87171`.

Retain Segoe UI, Lucide glyphs, current control radii, focus policy, and motion policy. Add no color, typeface, shadow, animation, token, or component abstraction.

### Spatial contract

- Measure the Manage Items gap from the originating Shelf Batch border box to the outer edge of the undecorated Manage Items window along the placement axis.
- Use `8` logical pixels as the preferred gap for right, left, below, and above candidates.
- Preserve orthogonal alignment with the originating batch before work-area clamping.
- Treat the `16`-pixel work-area margin as part of complete-candidate eligibility.
- If no complete candidate fits, clamp inside the permitted work area. Work-area containment takes precedence over the decorative gap; reduced gap, contact, or unavoidable overlap is accepted only in this fallback.
- Keep Open Shelf and Clear targets at `24 × 24` logical pixels and their positioned horizontal insets at `6` logical pixels.
- Keep Open Shelf `4` logical pixels from the top edge within the existing `28`-pixel header budget.
- Keep Clear `4` logical pixels from the bottom edge with the existing `min-height: 24px` footer contract.
- Apply the Rail rule symmetrically to Left and Right docking.

### Design review

A two-pixel top and bottom offset would be mathematically symmetric but would move the already accepted Clear control and weaken the established four/eight-pixel spacing rhythm. The approved design instead moves only Open Shelf to the existing four-pixel outer offset. No decorative treatment is added; the single visual change is clearer spatial ownership between compact Windows surfaces.

## Implementation Decisions

- Treat 0.0.13 as a clean presentation follow-up to the completed 0.0.12 release. Do not reopen or rewrite resolved 0.0.10 or 0.0.12 scope.
- Add the preferred gap in the shared native Manage Items placement path so Drop Shelf and Edge Rail origins cannot diverge.
- Preserve the placement order: right, left, below, above.
- Candidate eligibility must include the full popup bounds, the eight-pixel origin gap, and the preferred monitor margin.
- Keep clamping as an explicit fallback after complete candidates are exhausted. Add deterministic corner and narrow-work-area coverage so the fallback cannot silently regress to a first-candidate assumption.
- Adjust only the Open Shelf vertical alignment inside the existing Edge Rail header budget. Preserve the Clear footer classes and behavior.
- Preserve `24 × 24` action targets, 14-pixel Lucide glyphs, six-pixel horizontal insets, tooltips, accessible names, focus indicators, and action ownership.
- Preserve the current canonical Rail height contract: `max(280, min(642, 52 × batch count + 70))`.
- Do not change Rail minimum or maximum height, row height, row gap, collection top boundary, bottom clearance, scrollbar geometry, native placement, expansion/collapse timers, or fullscreen behavior.
- The historical 0.0.12 `+62/640` roadmap and ticket statements remain historical records. They are not the source baseline for this follow-up; current source, tests, and the canonical product specification use `+70/642`.
- Introduce no persistence schema, settings field, backend command, compatibility layer, telemetry, dependency, or generalized layout abstraction.
- No glossary update is required because the approved work changes no domain concept or term.
- No ADR is required because the spacing choices are reversible presentation details within established ownership boundaries.
- Do not create a Git commit without separate explicit user authorization.

## Testing Decisions

- Add deterministic native placement regression cases for right, left, below, and above candidates with the full eight-pixel gap.
- Cover anchors near all four work-area corners and work areas where no complete candidate fits. Prove the popup remains within the clamped bounds and that fallback is used only after complete candidates fail.
- Keep the existing placement dimensions, preferred 16-pixel margin, one-popup lifecycle, pointer no-activation behavior, keyboard focus entry, Escape ordering, and focus restoration under regression coverage.
- Run `pnpm build` for Vue single-file-component typechecking and production bundling.
- Compile through cargo-xwin for `x86_64-pc-windows-msvc` and run the complete produced Windows native test executable directly.
- Use an actual Windows Resident Instance and real WebView2 surfaces to inspect Manage Items from the Drop Shelf and from both Left and Right Edge Rail origins.
- Measure the eight-pixel preferred gap where a complete candidate fits and observe the clamped fallback near constrained monitor edges.
- Inspect Left and Right expanded Rails and verify four-pixel top and bottom action offsets, six-pixel horizontal insets, unchanged 24-pixel targets, and unchanged height tiers and scroll behavior.
- Exercise Dark, Light, and actual Windows Contrast Theme where available. Record fractional-DPI, mixed-DPI, monitor, theme, or interaction configurations that were not directly observed; do not infer them from 100% DPI or forced-colors emulation.
- Keep version metadata at 0.0.12 until implementation and real-surface acceptance pass. Then synchronize all version sources to 0.0.13, build one exact NSIS candidate, record installer and installed-executable sizes and SHA-256 hashes, and exercise those exact installed bytes.
- Exact-candidate smoke covers the changed surfaces plus embedded frontend loading, intake, management, Edge Rail transitions, Cooperative Shutdown, update/reinstall, uninstall, and Profile Data preservation.
- Add no permanent screenshot, source-text, CSS-class, or frontend component-test framework for these two geometry changes.

## Out of Scope

- Any Shelf state, persistence, Temporary/Pinned lifecycle, availability, duplicate, drag-result, or source-file behavior change.
- Manage Items width, maximum height, content density, row geometry, action layout, lifecycle, keyboard model, native ownership, or focus-restoration redesign.
- Edge Rail width, row height, row gap, collection spacing, scrollbar, corner radius, height formula, min/max height, timers, monitor ownership, fullscreen policy, or collapsed Rail Handle redesign.
- New palette, typography, icon, animation, tooltip, component library, design-system migration, setting, dependency, or platform support.
- TypeScript 7, signing, automatic updates, macOS, Linux, x86, or ARM64.
- Rewriting resolved 0.0.10 or 0.0.12 evidence.
- Creating a Git commit without separate explicit authorization.

## Further Notes

- Design discovery and the acceptance seam were confirmed by the user on 2026-10-15.
- Specification and ticket approval authorize implementation. Final candidate acceptance and Git commit creation remain separate gates.

## Delivery Evidence — 2026-10-15

- **Ticket 01 (Manage Items separation):** Ready for human acceptance. Added `GAP = 8.0` in `src-tauri/src/manage_items.rs`. Directional, corner, and narrow-work-area tests passed. Real Windows Resident measured 8px gap on Drop Shelf, Left Rail, Right Rail, and the installed candidate. Live boundary-clamp inspection remains pending.
- **Ticket 02 (Edge Rail alignment):** Ready for human acceptance. Set `rail-header` to `items-start pt-[4px] pb-0` with `28px` height; preserved the footer. Pre-bump Windows smoke measured both docking edges; the installed candidate measured Right Rail. Offsets were 4px inside the border (5px from the outer edge). Light/Contrast inspection remains pending.
- **Ticket 03 (Release DropCove 0.0.13):** Exact candidate produced and verified.
  - NSIS Installer: `build/DropCove-Setup-0.0.13.exe` (3,233,859 bytes, SHA-256 `2f0e9588ef72b8ca098da23a81e1e967587dcc4ebcecb024457dcbc54aa65ac3`).
  - Installed binary: `dropcove.exe` (13,387,264 bytes, SHA-256 `a84dc4b6692947211d0c5f808fb2525951d673a471225f8cece03c19dc1ff3f9`).
  - Target unbundled binary: `src-tauri/target/x86_64-pc-windows-msvc/release/dropcove.exe` (13,387,264 bytes, SHA-256 `268ebb2f0fbe2274ecc4ab8454d4abf006ba4058c3f5afb0693ab234f5104f74`).
  - 104/104 native tests passed.
  - Embedded production assets verified with Vite dev server stopped.
  - Silent install, secondary activation, Cooperative Shutdown (exit 0), reinstall/update, shortcut creation, uninstall removing binary, and profile `shelf.db` retention verified.
  - Temporary smoke files cleaned; registry baseline DisplayVersion 0.0.12 restored.
  - Unobserved limits: mixed-DPI / fractional scaling, Windows Light Theme, actual Windows Contrast Theme, and live on-screen work-area boundary clamp.
