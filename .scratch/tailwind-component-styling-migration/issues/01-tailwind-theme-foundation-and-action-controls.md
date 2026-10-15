# 01: Establish Tailwind Theme Foundation and Shared Action Controls

**What to build:** Establish the value-preserving Tailwind CSS v4 foundation for DropCove and migrate shared action-control presentation without changing any rendered surface, interaction, accessibility state, or theme behavior. Capture the disposable pre-migration parity baseline before changing shared styling so later surface tickets can compare against the accepted 0.0.10 presentation.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.11 — Tailwind Component Styling Migration](../spec.md)

## Acceptance criteria

- [x] Before the first styling change, identical fixtures capture representative bounding boxes and selected computed properties for Drop Shelf, Left/Right Edge Rail, Manage Items, Settings, both confirmations, resize handles, Shell Visuals, shared action controls, and their key states.
- [x] The baseline records machine, DPI, window geometry, theme, data shape, component state, selectors, properties, and any surface that cannot be captured automatically; unobserved states are not inferred.
- [x] Existing `--dc-*` variables remain the only semantic value source for Dark, Light, Windows Contrast Theme, focus, warning, danger, and interaction colors.
- [x] Tailwind `@theme inline` aliases reference existing semantic variables without copying values or creating a second token source.
- [x] Global CSS is reduced to the approved boundary: semantic token declarations and theme media queries, root sizing/reset, transparent WebView canvas behavior, Windows forced-color policy, global focus-visible policy, reduced-motion policy, and required platform selectors.
- [x] Shared action-control layout and ordinary hover, active, pressed, checked, mixed, disabled, and danger presentation use Tailwind utilities instead of the previous ordinary global component CSS.
- [x] Native tooltips, accessible names, binary pressed state, tri-state checked state, disabled behavior, action ownership, and keyboard activation remain unchanged at every existing call site.
- [x] Windows Contrast Theme action states continue to use distinguishable system colors, and the global visible-focus policy remains a single coherent convention.
- [x] No `@apply`, custom component utility, styling wrapper, generic styling prop, class-composition dependency, or component framework is introduced.
- [x] Conditional utilities are finite literal class candidates that survive the production Tailwind scan; no utility name is assembled from runtime fragments.
- [x] Tailwind preflight does not introduce an opaque root canvas, changed native-control appearance, altered box sizing, or transparent-window corner artifact.
- [x] Disposable baseline tooling and fixtures remain isolated from production code and are designed for removal in the release ticket.
- [x] Package, crate, Tauri, PE, installer registration, and output filename remain `0.0.10`.

## Testing seam

Use the actual Windows Resident Instance and real WebView2 surfaces for the baseline and post-migration comparison. Combine selected computed-style and bounding-box evidence with pointer, keyboard, Dark, Light, and actual Windows Contrast Theme inspection. Run the existing frontend production build after the shared migration; do not add source-text tests, generated-class snapshots, or a permanent visual-regression framework.

## Demo path

Show the captured 0.0.10 baseline, then run the migrated shared foundation on the same machine and fixtures. Demonstrate representative action controls in idle, hover, focus, pressed/checked/mixed, disabled, danger, Dark, Light, and Windows Contrast states while their accessible names and behavior remain unchanged.

## Non-goals

Migrating complete Drop Shelf, Edge Rail, Manage Items, Settings, confirmation, resize, or Shell Visual surfaces; redesigning controls; changing icon SVGs or sizes; compact scrollbar styling; persistent scroll insets; Lucide icons; TypeScript 7; version synchronization; packaging; or Git commit creation.

## Answer

Delivered in 0.0.11:
- Added value-free semantic Tailwind aliases mapping all twenty-eight `--dc-*` variables in `src/style.css`.
- Migrated all former `.icon-action` call sites across the application to shared direct utility chains.
- Removed obsolete global `.icon-action` selectors and transition definitions.
- 35 states and 2,292 elements verified against 0.0.10 baseline with 0 computed-style/geometry differences.
- User confirmed human acceptance across all interactive on-screen gates.
- Package, crate, and Tauri metadata maintained at 0.0.10 until authorized release cutover to 0.0.11.
- No Git commit created.
