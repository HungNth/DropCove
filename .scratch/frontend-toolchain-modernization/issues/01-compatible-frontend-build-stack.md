# 01: Compatible Frontend Build Stack

**What to build:** Replace DropCove's aging and partially unused frontend dependency graph with the approved executable stack. Remove inactive routing and store dependencies, retain complete Vue single-file-component typechecking through TypeScript 6, migrate Tailwind CSS to its Vite-native v4 integration, and produce a clean frontend bundle without changing product behavior or release metadata.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.9 — Frontend Toolchain Modernization](../spec.md)

## Acceptance criteria

- [x] `pinia` and `vue-router` are absent from the direct dependency graph, Pinia's empty application registration is removed, and no router, route tree, store, compatibility wrapper, or replacement abstraction is introduced.
- [x] The active graph resolves VueUse `15.0.0`, Vue `3.5.44`, Vite `8.3.4`, the Vite Vue plugin `6.0.10`, Tailwind CSS `4.3.3`, vue-tsc `3.3.13`, TypeScript `6.0.2`, and the matching Tailwind Vite plugin.
- [x] TypeScript `7.x` is absent from the executable graph, and the build still runs vue-tsc before Vite so Vue templates and scripts remain fully typechecked.
- [x] Package metadata declares Node `>=22`; the exercised Node and pnpm versions are recorded with the build evidence.
- [x] The manifest retains its compatible-range convention while the lockfile records one exact, clean graph with no incompatible required peer dependencies.
- [x] Tailwind uses its official Vite-native v4 integration and stylesheet import; direct PostCSS and Autoprefixer dependencies, their configuration, and the empty JavaScript Tailwind configuration are removed.
- [x] Existing body utilities remain generated, and the semantic `--dc-*` token definitions, Light overrides, Windows Contrast overrides, focus rules, and reduced-motion rules remain standard CSS without renaming or value changes.
- [x] Vite preserves the Tauri development host, frontend and WebSocket/HMR ports, strict-port behavior, native-source watch exclusion, and production output consumed by Tauri using supported Vite 8 configuration.
- [x] Existing Tauri JavaScript packages remain unchanged unless clean dependency resolution proves a compatibility update is required.
- [x] A clean pnpm install completes without dependency or required-peer errors, and the existing `vue-tsc && vite build` contract completes with zero errors.
- [x] Package, crate, Tauri, PE, and installer version metadata remain `0.0.8` throughout this ticket.

## Testing seam

Use the approved frontend seam: resolve the real manifest and lockfile with pnpm, then run the unchanged production build contract. Verify resolved versions and removed dependencies through the package manager, not manifest-text tests. The build must prove vue-tsc succeeds before Vite and that Tailwind v4 produces the production CSS bundle.

## Demo path

From a clean dependency state, install the approved graph, list the resolved direct versions, and run the production frontend build. Show that Pinia, Vue Router, PostCSS, and Autoprefixer are absent; TypeScript 6 and vue-tsc 3 are active; and the generated bundle completes without errors while metadata remains 0.0.8.

## Non-goals

Windows runtime acceptance, native test execution, cargo-xwin release proof, release packaging, version 0.0.9 synchronization, UI redesign, TypeScript 7, new tests or lint tooling, routes, stores, or rewriting scoped component CSS into Tailwind utilities.

## Answer

- Removed unused Pinia/Vue Router and the Pinia registration; migrated Tailwind to its Vite-native v4 plugin and removed the obsolete PostCSS/Autoprefixer/configuration path.
- Added Node `>=22`, moved Vite WebSocket configuration to `server.ws`, and added Vite client types for TypeScript 6 side-effect CSS imports.
- `pnpm install --frozen-lockfile --strict-peer-dependencies` passed. The regenerated lockfile resolves VueUse 15.0.0, Vue 3.5.44, plugin-vue 6.0.10, Tailwind/plugin 4.3.3, TypeScript 6.0.2, Vite 8.3.4, and vue-tsc 3.3.13. Tauri packages remain unchanged.
- `pnpm build` passed: full vue-tsc checking followed by Vite production bundling, 66 modules transformed. Metadata remains 0.0.8.
- Runtime proof used Node 24.18.0 and pnpm 12.11.2. Shelf/Manage Items/body utilities rendered in native WebView2; Ticket 03 subsequently verified the embedded standalone production origin. No frontend testing framework was added.