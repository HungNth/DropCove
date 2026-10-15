# 02: Replace Action and Empty-State Icons with Lucide

**What to build:** Replace DropCove's handwritten user-facing action and empty-state SVG paths with the approved Lucide glyph roles while preserving every control's tooltip, accessible name, pressed or checked state, mixed-state meaning, disabled behavior, focus behavior, target size, and action ownership.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.12 — Compact Scrolling and Lucide Icon Consistency](../spec.md)

## Acceptance criteria

- [x] `lucide-vue-next` is the only new dependency, and the package lock records the resolved dependency without unrelated upgrades.
- [x] The existing thin icon-role adapter renders Lucide components and contains no handwritten action or empty-state SVG paths.
- [x] Pin and Unpin use `Pin`; pinned presentation remains visibly filled.
- [x] Mixed Bulk Pinning uses a composed `Pin` and `Minus` treatment while the button remains the owner of `aria-checked="mixed"`.
- [x] Manage Items uses `List`, removal uses `CircleMinus`, close uses `X`, Clear Temporary Items uses `Trash2`, Edge Rail direction uses `ArrowLeft` or `ArrowRight`, and the empty Drop Shelf uses `ArrowDown`.
- [x] Every action glyph renders at 14 by 14 pixels inside the existing 24-by-24-pixel control target.
- [x] The empty Drop Shelf glyph renders at 28 by 28 pixels and remains centered with the existing `Drop here` message.
- [x] Glyphs remain decorative and hidden from assistive technology; existing buttons retain all accessible names, native titles, pressed states, checked and mixed states, disabled states, keyboard activation, and focus-visible presentation.
- [x] Pin, mixed Pin, Manage Items, Remove, Close, Clear, Edge Rail direction, and empty-state roles remain recognizable at normal and compact widths.
- [x] Existing Dark, Light, and Windows Contrast Theme color inheritance remains based on `currentColor` and the control's established semantic state.
- [x] Existing hover, danger, pressed, checked, mixed, disabled, and focus-visible control states remain distinguishable.
- [x] Existing Tab order, Shift+Tab behavior, Enter, Space, Escape, focus restoration, and native tooltips remain unchanged at every affected call site.
- [x] Drop Shelf and Edge Rail drag-in, item and batch drag-out, pinning, management, removal, and Clear workflows remain unchanged.
- [x] Textual status-dismiss `×` controls, Shell Visuals, application icons, installer artwork, and other non-action SVG assets remain outside this ticket.
- [x] No second icon wrapper, generic icon styling API, component library, compatibility renderer, or handwritten fallback path is introduced.
- [x] Frontend production typechecking and bundling pass with the Lucide dependency and every icon role included in production output.
- [x] Package, crate, Tauri, PE, installer registration, and output filename remain version 0.0.11 in this ticket.
- [x] No Git commit is created without separate explicit user authorization.

## Testing seam

Use the actual Windows Resident Instance and real WebView2 surfaces. Inspect every mapped role in normal and compact widths across Dark, Light, and actual Windows Contrast Theme. Measure the 14-pixel action glyphs, 28-pixel empty-state glyph, and 24-pixel button targets; exercise tooltip, accessible name, pressed, checked, mixed, disabled, hover, and focus-visible states. Use existing frontend production checks for dependency and Vue integration; do not add tests that assert component source text, icon component names, or generated SVG markup.

## Demo path

Show the empty Drop Shelf, single-item and multi-item Shelf Batches, Manage Items header and rows, expanded Edge Rail on both docking sides, and Clear Temporary Items controls. Demonstrate pinned, unpinned, mixed, remove, close, directional, disabled, hover, keyboard-focus, Dark, Light, and Windows Contrast states while the established button labels and workflows remain unchanged.

## Non-goals

Compact scrollbar styling; viewport boundaries; textual dismiss controls; Shell Visual rendering; application or installer icon redesign; TypeScript 7; component-library adoption; accessibility-model changes; native command or state changes; version synchronization; NSIS packaging; work after 0.0.12; or Git commit creation.

## Answer

Resolved with `lucide-vue-next` 1.0.0 and the approved role adapter. Real WebView2 measured 14-pixel actions, 24-pixel button targets, and a 28-pixel empty-state glyph; observed filled Pin, mixed Pin/Minus with `aria-checked="mixed"`, decorative SVG semantics, management keyboard reveal, and originating-focus restoration. Frontend and cargo-xwin builds passed; 101/101 native tests passed. Static Standards and Spec reviews reported no icon source defects.

The user's subsequent interaction acceptance below closes the human gates; these are user-reported results, not agent-observed actual OS theme measurements. Metadata remained 0.0.11 through this implementation slice and was synchronized to 0.0.12 only in the release slice. No Git commit created.


### User acceptance — 2026-10-15
- User confirmed human acceptance across all interactive on-screen gates: all Lucide icon roles, 14px action glyphs, 28px empty-state glyph, filled/mixed pin states, tooltips, keyboard navigation, and theme appearances.
