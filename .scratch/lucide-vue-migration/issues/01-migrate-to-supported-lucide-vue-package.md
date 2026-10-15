# 01: Migrate to the Supported Lucide Vue Package

**What to build:** Replace DropCove's deprecated Lucide Vue dependency with the supported scoped package while preserving every existing icon role, visual state, control target, tooltip, accessible name, and interaction. The completed slice must install and build from the regenerated lockfile and render the current icons through the existing adapter on the real application surfaces.

**Blocked by:** None (can start immediately).

**Status:** ready-for-human

**Parent specification:** [Lucide Vue Migration and Shelf Batch Spacing Correction](../spec.md)

## Acceptance criteria

- [x] The deprecated `lucide-vue-next` direct dependency is removed completely and `@lucide/vue` is declared at the approved compatible range `^1.57.0`.
- [x] The pnpm lockfile resolves the supported package and no unrelated direct dependency version changes are introduced.
- [x] No alias, compatibility shim, fallback import, duplicate Lucide package, or parallel icon implementation remains.
- [x] The existing action-icon adapter remains the sole direct Lucide import boundary; its consumers retain their current interface and behavior.
- [x] Pin, Minus, List, Circle Minus, Close, Trash, Left, Right, and Down continue to render through the same approved roles.
- [x] Action glyphs remain 14 by 14 pixels, the empty Drop Shelf glyph remains 28 by 28 pixels, and action targets remain 24 by 24 pixels.
- [x] Filled Pin and mixed Pin/Minus presentation remain visually distinct without moving checked, mixed, pressed, disabled, or accessible-name ownership from the surrounding controls.
- [ ] Lucide SVGs remain decorative while existing buttons retain native titles, keyboard activation, focus-visible presentation, tooltips, and assistive-technology semantics.
- [ ] The empty Drop Shelf, Drop Shelf actions, Shelf Batch actions, Manage Items, Bulk Pinning, Shelf Item actions, and Left/Right Edge Rail actions render their existing icon roles in the running application.
- [x] Existing Dark, Light, and Windows Contrast color inheritance remains based on each control's established semantic state.
- [x] A frozen lockfile installation with strict peer dependency enforcement succeeds, followed by complete Vue single-file-component typechecking and production bundling.
- [x] The resolved dependency graph contains the supported package and no deprecated Lucide Vue package.
- [x] No Vue, TypeScript, Vite, Tailwind CSS, VueUse, Tauri, Rust crate, pnpm toolchain, release metadata, or installer change is included.
- [x] No Git commit is created without separate explicit user authorization.

## Testing seam

Use the existing frontend production dependency/build path for package resolution, Vue typing, and bundling. Then use the actual Windows Resident Instance and real WebView2 surfaces to inspect every current icon role, including filled and mixed Pin states, the empty Drop Shelf glyph, compact action targets, tooltips, keyboard focus, accessible button ownership, and theme color inheritance. Do not add source-text, generated-SVG, or forwarding tests.

## Demo path

Open the empty Drop Shelf, populated single-item and multi-item Shelf Batches, Manage Items, Bulk Pinning, and the expanded Edge Rail on both docking sides. Demonstrate every existing icon role plus filled, mixed, disabled, hover, focus-visible, Dark, Light, and Windows Contrast states while the established controls and workflows remain unchanged.

## Non-goals

Shelf collection spacing; scrollbar presentation; Edge Rail height tiers; icon redesign; new icon roles; component-library adoption; unrelated dependency upgrades; release versioning; NSIS packaging; or Git commit creation.

## Answer

Source implementation and automated checks complete. Frozen/strict install, frontend build, cargo-xwin compilation, and 102 native tests passed. Real WebView2 observed decorative glyphs, action/empty sizes, filled and mixed Pin, management Close, List, Remove, Clear, and Left Rail direction. Final exhaustive tooltip/disabled/keyboard and every-role interaction acceptance remains human-owned; unchecked criteria are not claimed observed in full.

See [shared verification evidence and cleanup limits](../verification.md). Actual Windows Contrast was exercised, but exact pre-smoke theme cache and global startup registry-byte restoration were not established. No release packaging or commit. Remaining work is human acceptance, not deferred source implementation.
