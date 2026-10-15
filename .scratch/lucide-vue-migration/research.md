# Lucide Package Migration Research: `lucide-vue-next` to `@lucide/vue`

**Date:** 2026-10-15  
**Author:** LucidePackageResearch  
**Scope:** Primary-source empirical verification of npm package metadata, official Lucide documentation, repository structure, and import/component compatibility for migrating DropCove's Vue 3 frontend from `lucide-vue-next` to `@lucide/vue`.

---

## 1. Executive Summary

1. **Official Deprecation Confirmed:**  
   The legacy package `lucide-vue-next` is officially deprecated on npm with the deprecation notice:  
   `"Package deprecated. Please use @lucide/vue instead."`
2. **Official Replacement Package:**  
   The official scoped package is `@lucide/vue`, developed and maintained within the official Lucide monorepo (`packages/vue`).
3. **Current Versions on npm Registry:**  
   - `lucide-vue-next`: `1.0.0` (final release published 2026-03-23; marked deprecated).
   - `@lucide/vue`: `1.57.0` (`latest`, published 2026-10-14).
4. **Drop-in Import & Component API Compatibility:**  
   - Both packages declare peer dependency `"vue": ">=3.0.1"` and distribute ES modules (`dist/esm/`) with `"sideEffects": false`.
   - Component PascalCase naming conventions, props (`size`, `color`, `stroke-width`, `nonScalingStroke`, etc.), and tree-shaking behavior are identical.
   - For DropCove's single icon adapter (`src/components/shelf/ActionIcon.vue`), migration is a 1-line import source replacement: `from "lucide-vue-next"` $\to$ `from "@lucide/vue"`.
5. **Migration Caveats & Behavioral Differences:**  
   - **Brand Icons Removed:** Version 1 removed 14 third-party brand icons (e.g. `Github`, `Slack`, `Facebook`, `RailSymbol`). None of these brand icons are used in DropCove.
   - **Accessibility Defaults:** Lucide v1 sets `aria-hidden="true"` by default on rendered SVG icon elements. In DropCove, icon roles are strictly decorative glyphs nested inside accessible button elements; this change aligns with existing accessibility requirements.

---

## 2. Package Status & Registry Facts

| Attribute | Legacy Package: `lucide-vue-next` | Target Package: `@lucide/vue` | Notes / Sources |
| :--- | :--- | :--- | :--- |
| **npm Registry Name** | `lucide-vue-next` | `@lucide/vue` | [npm: `lucide-vue-next`](https://www.npmjs.com/package/lucide-vue-next), [npm: `@lucide/vue`](https://www.npmjs.com/package/@lucide/vue) |
| **Latest Version** | `1.0.0` (published 2026-03-23) | `1.57.0` (published 2026-10-14) | Verified via `https://registry.npmjs.org/` |
| **Deprecation Status** | **Deprecated** (`"Package deprecated. Please use @lucide/vue instead."`) | Active (`latest`) | Verified in npm package manifest |
| **Monorepo Directory** | `packages/lucide-vue-next` | `packages/vue` | [lucide-icons/lucide](https://github.com/lucide-icons/lucide) |
| **License** | ISC | ISC | Root repository license |
| **Peer Dependencies** | `"vue": ">=3.0.1"` | `"vue": ">=3.0.1"` | Fully compatible with DropCove's `vue@^3.5.44` |
| **Side Effects Flag** | `"sideEffects": false` | `"sideEffects": false` | Unused icons are tree-shaken by Vite/Rollup/Rolldown |
| **Module Entry Points** | `dist/esm/lucide-vue-next.js` (ESM)<br>`dist/cjs/lucide-vue-next.js` (CJS) | `dist/esm/lucide-vue.mjs` (ESM)<br>`dist/cjs/lucide-vue.js` (CJS) | Verified in package metadata |
| **TypeScript Types** | `dist/lucide-vue-next.d.ts` | `dist/lucide-vue.d.ts` | Complete functional component typings |

---

## 3. Supported Vue Version & Import Syntax

### Vue Version Support
- Both packages require Vue 3 via `peerDependencies: { "vue": ">=3.0.1" }`.
- DropCove uses `vue@^3.5.44` with `@vitejs/plugin-vue@^6.0.10` and `vite@^8.3.4`. This satisfies all package constraints.

### Import Syntax
Lucide v1 is packaged with ES Modules and marked side-effect free. Named imports from `@lucide/vue` are tree-shakable:

```ts
import {
  Pin,
  Minus,
  List,
  CircleMinus,
  X,
  Trash2,
  ArrowLeft,
  ArrowRight,
  ArrowDown
} from "@lucide/vue";
```

### Component Props API
Props accepted by functional icon components under `@lucide/vue` match those under `lucide-vue-next`:
- `size` (`number`, default `24`)
- `color` (`string`, default `'currentColor'`)
- `strokeWidth` / `stroke-width` (`number | string`, default `2`)
- `nonScalingStroke` (`boolean`, default `false`)
- Any valid SVG presentation attribute (`class`, `style`, `fill`, etc.)

---

## 4. Tree-Shaking & Component Naming Analysis

### Tree-Shaking
- Both packages publish individual ESM files per icon under `dist/esm/icons/*.mjs` and re-export them from `dist/esm/lucide-vue.mjs`.
- With `"sideEffects": false` declared in `package.json`, Vite's bundler extracts only the imported icon functional components and drops unused symbols.

### Component Naming in DropCove
All 9 icon components currently imported by DropCove's icon adapter (`src/components/shelf/ActionIcon.vue`) exist in `@lucide/vue` with exact PascalCase name parity:
1. `Pin`
2. `Minus`
3. `List`
4. `CircleMinus`
5. `X`
6. `Trash2`
7. `ArrowLeft`
8. `ArrowRight`
9. `ArrowDown`

*Verification:* Inspected `https://unpkg.com/@lucide/vue@1.57.0/dist/lucide-vue.d.ts` and confirmed all 9 component identifiers are exported.

---

## 5. Migration Caveats & Impact on DropCove

1. **Brand Icons Removal:**  
   - *Upstream change:* Lucide v1 removed brand logos (such as `Github`, `Twitter`, `Figma`, `Slack`, `RailSymbol`) due to trademark concerns.  
   - *Impact on DropCove:* None. DropCove uses only core geometric action icons (`Pin`, `Minus`, `List`, `CircleMinus`, `X`, `Trash2`, `ArrowLeft`, `ArrowRight`, `ArrowDown`).
2. **Default `aria-hidden="true"`:**  
   - *Upstream change:* Lucide v1 components now set `aria-hidden="true"` by default on the rendered `<svg>` element.  
   - *Impact on DropCove:* DropCove's accessibility model requires glyphs to be decorative while parent buttons own accessible names, tooltips, pressed states, and checked states. This upstream change directly enforces DropCove's accessibility contract.
3. **Single Adapter Seam in DropCove:**  
   - DropCove centralizes all Lucide icon imports inside `src/components/shelf/ActionIcon.vue`.  
   - No application components directly import from `lucide-vue-next`.  
   - Migration scope is isolated to:
     1. Updating `package.json` (`dependencies`: replace `"lucide-vue-next"` with `"@lucide/vue"`).
     2. Running `pnpm install` / updating `pnpm-lock.yaml`.
     3. Updating the import source in `src/components/shelf/ActionIcon.vue` line 12.

---

## 6. Official Primary Sources & References

- **Lucide v1 Announcement & Changelog:**  
  [https://lucide.dev/guide/version-1](https://lucide.dev/guide/version-1)  
  *Covers package rename from `lucide-vue-next` to `@lucide/vue`, removal of brand icons, default `aria-hidden`, and ESM/CJS build changes.*
- **Official Lucide Vue Migration Guide:**  
  [https://lucide.dev/guide/vue/migration](https://lucide.dev/guide/vue/migration)  
  *Details step-by-step upgrade path from `lucide-vue-next` to `@lucide/vue` and confirming identical component APIs.*
- **Official Lucide Vue Getting Started Guide:**  
  [https://lucide.dev/guide/vue/getting-started](https://lucide.dev/guide/vue/getting-started)  
  *Documents installation, tree-shakable ES module imports, and component props.*
- **npm Registry Metadata — `@lucide/vue`:**  
  [https://registry.npmjs.org/@lucide/vue](https://registry.npmjs.org/@lucide/vue)  
  [https://www.npmjs.com/package/@lucide/vue](https://www.npmjs.com/package/@lucide/vue)  
  *Confirms latest version `1.57.0`, `peerDependencies` (`"vue": ">=3.0.1"`), and package structure.*
- **npm Registry Metadata — `lucide-vue-next`:**  
  [https://registry.npmjs.org/lucide-vue-next](https://registry.npmjs.org/lucide-vue-next)  
  [https://www.npmjs.com/package/lucide-vue-next](https://www.npmjs.com/package/lucide-vue-next)  
  *Confirms deprecation notice `"Package deprecated. Please use @lucide/vue instead."` on version `1.0.0`.*
- **Lucide Monorepo Repository:**  
  [https://github.com/lucide-icons/lucide](https://github.com/lucide-icons/lucide)  
  *Packages located at `packages/vue` and legacy `packages/lucide-vue-next`.*
