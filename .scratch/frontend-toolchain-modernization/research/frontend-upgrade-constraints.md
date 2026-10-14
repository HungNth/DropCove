# Frontend Upgrade Constraints: Version 0.0.9 Modernization

**Date:** 2026-10-14  
**Author:** UpgradeDocsResearch  
**Target Architecture:** Windows x86_64 (`x86_64-pc-windows-msvc`), Tauri v2 (`cargo-xwin`)  
**Scope:** Primary-source empirical research across all 8 exact roadmap targets in `docs/ROADMAP.md` and companion ecosystem packages.

---

## 1. Executive Summary & Critical Findings

1. **Exact Requested Versions Exist on npm:**  
   Every single version requested in `docs/ROADMAP.md` exists and was verified directly on the official npm registry:
   - `@vueuse/core@15.0.0`
   - `pinia@4.0.3`
   - `vue-router@5.4.0`
   - `@vitejs/plugin-vue@6.0.10`
   - `tailwindcss@4.3.3`
   - `typescript@7.0.2`
   - `vite@8.3.4`
   - `vue-tsc@3.3.13`

2. **CRITICAL INCOMPATIBILITY / BLOCKER IDENTIFIED: `vue-tsc@3.3.13` + `typescript@7.0.2`:**  
   - **Empirical Cause:** `typescript@7.0.2` is a complete native rewrite of the TypeScript compiler toolchain (distributing native binaries such as `tsc.exe` via platform-specific packages like `@typescript/typescript-win32-x64@7.0.2`). It completely removes the legacy JavaScript compiler implementation `lib/_tsc.js` and `typescript.js`.
   - **Breakage Mechanism:** `vue-tsc@3.3.13` relies on `@volar/typescript@2.4.28` (`runTsc.js`), which attempts to load TypeScript via `require.resolve('typescript/lib/tsc')` and perform AST regex string monkey-patching (`transformTscContent`) on the JavaScript source of `tsc`. When pointed at `typescript@7.0.2`, `runTsc` throws:  
     `Error: Failed to locate tsc module path from shim`.
   - **Ecosystem Conformance:** `vue-tsc@3.3.13` was built for TypeScript 5.x / 6.x (which retained `lib/_tsc.js`). Running `npm run build` (`vue-tsc && vite build`) will fail immediately under `typescript@7.0.2`.
   - **Resolution Required:** Either retain `typescript@~5.7.3` / `typescript@~6.0.2` for `vue-tsc` compatibility, or use an unbundled typechecking flow until Volar / `vue-tsc` provides native TypeScript 7 binary plugin bindings.

3. **Vite 8 Architecture Evolution:**  
   - `vite@8.3.4` replaces Rollup and esbuild with native Rust tooling (`rolldown ~1.2.12`, `lightningcss ^1.33.0`, and Oxc via `rolldown/utils`).
   - `build.rollupOptions` is officially deprecated in favor of `build.rolldownOptions`, though backward-compatible property aliasing is preserved.
   - Server WebSocket and HMR configurations have shifted: top-level `server.hmr` options (`protocol`, `host`, `port`, `path`, `timeout`) are deprecated in favor of `server.ws`.
   - Node runtime requirement is strictly `^20.19.0 || >=22.12.0`. Current developer environment (`Node v24.18.0`) satisfies this requirement.

4. **Tailwind CSS v3 → v4 Architecture Evolution:**  
   - `tailwindcss@4.3.3` moves away from JavaScript configuration (`tailwind.config.js`) and PostCSS-based `@tailwind` directives (`@tailwind base; @tailwind components; @tailwind utilities;`).
   - Tailwind 4 uses native CSS `@theme` / `@import "tailwindcss";` driven by native Rust Oxide engine and Lightning CSS.
   - DropCove's extensive CSS variable system (`:root`, `@media (prefers-color-scheme: light)`, and `@media (forced-colors: active)`) in `src/style.css` is standard CSS and remains fully functional, but imports must transition to `@import "tailwindcss";` and bundler integration must transition to `@tailwindcss/vite` (or `@tailwindcss/postcss`).

5. **Pinia 4 & Vue Router 5 Compatibility:**  
   - Both `pinia@4.0.3` and `vue-router@5.4.0` remain backward-compatible with DropCove's existing Composition API usage.
   - `pinia@4.0.3` adds a required peer dependency on `@vue/devtools-api@^8.1.5` and peer dependency on `vue@^3.5.11`.
   - DropCove does not currently import or use `vue-router` in application source code, so upgrading `vue-router` introduces zero breaking surface in the codebase.

---

## 2. Target Version Verification & Compatibility Matrix

All version information and dependency trees were fetched directly from the official npm registry (`https://registry.npmjs.org/`).

| Requested Target Package | Requested Version | Published Timestamp | Registry Status | Engine Requirements | Peer Dependencies | Upstream Direct Source URL |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `@vueuse/core` | `15.0.0` | 2026-09-16T11:21:30.050Z | Exact match (`latest`) | `node: >=22` | `vue: ^3.5.0` | [https://github.com/vueuse/vueuse](https://github.com/vueuse/vueuse) |
| `pinia` | `4.0.3` | 2026-08-12T12:21:06.901Z | Exact match | None | `vue: ^3.5.11`<br>`@vue/devtools-api: ^8.1.5`<br>`typescript: >=5.6.0` (opt) | [https://github.com/vuejs/pinia](https://github.com/vuejs/pinia) |
| `vue-router` | `5.4.0` | 2026-10-07T15:34:51.836Z | Exact match (`latest`) | None | `vue: ^3.5.34 \|\| ^4.0.0`<br>`vite: ^7.3.0 \|\| ^8.0.0` (opt)<br>`pinia: ^3.0.4 \|\| ^4.0.2` (opt)<br>`@pinia/colada: >=0.21.2` (opt) | [https://github.com/vuejs/router](https://github.com/vuejs/router) |
| `@vitejs/plugin-vue`| `6.0.10`| 2026-10-10T07:40:21.880Z | Exact match (`latest`) | `node: ^20.19.0 \|\| >=22.12.0` | `vue: ^3.2.25 \|\| ^3.6.0-rc.10`<br>`vite: ^5.0.0 \|\| ^6.0.0 \|\| ^7.0.0 \|\| ^8.0.0` | [https://github.com/vitejs/vite-plugin-vue](https://github.com/vitejs/vite-plugin-vue) |
| `tailwindcss` | `4.3.3` | 2026-07-16T12:03:35.267Z | Exact match (`latest`) | None | None | [https://github.com/tailwindlabs/tailwindcss](https://github.com/tailwindlabs/tailwindcss) |
| `typescript` | `7.0.2` | 2026-07-08T15:55:18.431Z | Exact match (`latest`) | `node: >=16.20.0` | None | [https://github.com/microsoft/TypeScript](https://github.com/microsoft/TypeScript) |
| `vite` | `8.3.4` | 2026-10-08T12:07:04.441Z | Exact match (`latest`) | `node: ^20.19.0 \|\| >=22.12.0` | `esbuild: ^0.27.0 \|\| ^0.28.0` (opt)<br>`@types/node: ^20.19.0 \|\| >=22.12.0` (opt) | [https://github.com/vitejs/vite](https://github.com/vitejs/vite) |
| `vue-tsc` | `3.3.13` | 2026-10-10T14:32:43.495Z | Exact match (`latest`) | None | `typescript: >=5.0.0` | [https://github.com/vuejs/language-tools](https://github.com/vuejs/language-tools) |

---

## 3. Required Companion Packages Matrix

The following companion packages must be installed or updated to guarantee peer dependency satisfaction and build compatibility across the stack:

| Package | Current Version | Required Companion Version | Publish Date | Reason for Requirement | Source URL |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `vue` | `^3.5.43` | `^3.5.44` | 2026-10-13T07:45:32.806Z | Satisfies `vue-router@5.4.0` (`^3.5.34`), `pinia@4.0.3` (`^3.5.11`), and `@vueuse/core@15.0.0` (`^3.5.0`). Latest patch release. | [https://github.com/vuejs/core](https://github.com/vuejs/core) |
| `@vue/devtools-api`| _Not installed_ | `^8.2.1` | 2026-07-25T07:46:45.228Z | Required non-optional peer dependency of `pinia@4.0.3` (`^8.1.5`). Missing dependency causes pnpm strict peer warnings/errors. | [https://github.com/vuejs/devtools](https://github.com/vuejs/devtools) |
| `@tailwindcss/vite`| _Not installed_ | `^4.3.3` | 2026-07-16T12:04:06.316Z | Recommended official Vite integration plugin for Tailwind CSS v4. Replaces `postcss` + `autoprefixer` setup. | [https://github.com/tailwindlabs/tailwindcss](https://github.com/tailwindlabs/tailwindcss) |
| `@tailwindcss/postcss`| _Not installed_ | `^4.3.3` | 2026-07-16T12:03:56.054Z | Alternative PostCSS integration plugin if PostCSS pipeline is retained. | [https://github.com/tailwindlabs/tailwindcss](https://github.com/tailwindlabs/tailwindcss) |
| `@tauri-apps/api` | `^2.12.3` | `^2.12.3` | 2026-10-10T17:44:26.833Z | Verified latest stable Tauri v2 JS API. Fully compatible with Vite 8. | [https://github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| `@tauri-apps/cli` | `^2.12.2` | `^2.12.2` | 2026-10-10T20:02:07.833Z | Verified latest stable Tauri v2 CLI. Fully compatible with Node 24 and Vite 8 dev server port 1420. | [https://github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| `@tauri-apps/plugin-opener`| `^2.7.1` | `^2.7.1` | 2026-10-10T16:56:52.724Z | Verified latest stable plugin. Requires `@tauri-apps/api: ^2.12.0`. | [https://github.com/tauri-apps/plugins-workspace](https://github.com/tauri-apps/plugins-workspace) |
| `@types/node` | _Not installed_ | `^22.12.0` | Latest | Resolves Node types for `vite.config.ts` (`process.env.TAURI_DEV_HOST`) and satisfies Vite 8 optional peer. | [https://github.com/DefinitelyTyped/DefinitelyTyped](https://github.com/DefinitelyTyped/DefinitelyTyped) |

---

## 4. Deep Technical Analysis by Toolchain Component

### 4.1. TypeScript 7.0.2 and vue-tsc 3.3.13 Incompatibility

#### Primary Source Facts:
- `typescript@7.0.2` package tarball contains `bin/tsc` and `lib/tsc.js`, which invoke a native precompiled executable via `lib/getExePath.js`:
  ```javascript
  const expectedPackage = baseName + "-" + process.platform + "-" + process.arch;
  const platformPackageName = "@typescript/" + expectedPackage; // e.g. @typescript/typescript-win32-x64
  ...
  let exe = path.join(exeDir, binName) + ".exe";
  execFileSync(exe, process.argv.slice(2), { stdio: "inherit" });
  ```
- The native executable for Windows x64 is `@typescript/typescript-win32-x64@7.0.2` (`lib/tsc.exe`, size 24.5 MB). Execution of `tsc.exe --version` yields `Version 7.0.2`.
- In `typescript@7.0.2`, the legacy JavaScript compiler files (`lib/typescript.js` and `lib/_tsc.js`) have been entirely removed.
- `vue-tsc@3.3.13` executes `require('@volar/typescript/lib/quickstart/runTsc').runTsc(...)`.
- `@volar/typescript@2.4.28` hooks into Node's `fs.readFileSync` and attempts to intercept reading of `tscPath`:
  ```javascript
  // Support the tsc shim used in Typescript v5.7 and up
  const requireRegex = /module\.exports\s*=\s*require\((?:"|')(?<path>\.\/\w+\.js)(?:"|')\)/;
  const requirePath = requireRegex.exec(tsc)?.groups?.path;
  if (requirePath) {
      tsc = readFileSync(path.join(path.dirname(tscPath), requirePath), 'utf8');
      return transformTscContent(tsc, proxyApiPath, extraSupportedExtensions, extraExtensionsToRemove, __filename, typescriptObject);
  } else {
      throw new Error('Failed to locate tsc module path from shim');
  }
  ```
- Because TypeScript 7's `lib/tsc.js` is an ES module executing a child process rather than requiring a CommonJS module matching `requirePath`, Volar immediately throws:
  `Error: Failed to locate tsc module path from shim`.

#### Severity:
- **BLOCKING BUG** if both `typescript@7.0.2` and `vue-tsc@3.3.13` are enforced simultaneously in `npm run build` (`vue-tsc && vite build`).
- **Remediation Option A (Recommended for 0.0.9):** Retain `typescript@~6.0.2` (or keep `typescript@~5.7.3`) specifically for `vue-tsc`, or mark TypeScript 7 compatibility as deferred/impossible until Volar 3.x ships native TypeScript 7 binary support.
- **Remediation Option B:** Run `tsc --noEmit` directly using native TypeScript 7.0.2 compiler for `.ts` files, and use Vite with `@vitejs/plugin-vue` for `.vue` compilation without Volar CLI monkey-patching.

---

### 4.2. Vite 8.3.4 Rolldown, Oxc, and Configuration Migration

#### Primary Source Facts:
- `vite@8.3.4` bundle inspection shows that Vite 8 now bundles `rolldown ~1.2.12` directly as its core bundling engine (`dist/node/chunks/node.js`), along with `lightningcss ^1.33.0`.
- **Rolldown Configuration (`BuildEnvironmentOptions`):**
  - In `vite@8.3.4/dist/node/index.d.ts`:
    ```typescript
    interface BuildEnvironmentOptions {
      /**
       * @deprecated Use `rolldownOptions` instead.
       */
      rollupOptions?: RolldownOptions;
      rolldownOptions?: RolldownOptions;
    }
    ```
  - For projects not specifying custom `rollupOptions`, Vite 8 builds out of the box with zero config changes. DropCove's current `vite.config.ts` does not define `rollupOptions`, so no build config breakage occurs.
- **Oxc & Transform APIs:**
  - Vite 8 exposes `transformWithOxc(code, filename, options, inMap, config, watcher)`.
  - Development transforms for standard TypeScript files use fast native Rust transforms.
  - `transformWithEsbuild` remains supported in parallel.
- **Server WebSocket & HMR deprecations:**
  - In `vite@8.3.4`:
    ```typescript
    interface HmrOptions {
      /** @deprecated Use `server.ws.protocol` instead. */
      protocol?: string;
      /** @deprecated Use `server.ws.host` instead. */
      host?: string;
      /** @deprecated Use `server.ws.port` instead. */
      port?: number;
      ...
    }
    ```
  - DropCove's current `vite.config.ts`:
    ```typescript
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    ```
  - While `hmr` options continue to function via backward-compatible shim, Vite 8 logs deprecation warnings. Recommended modernization is to configure `server.ws` directly if desired, or retain `hmr` for backward compatibility.
- **Server Allowed Hosts:**
  - Vite 8 introduces strict `allowedHosts` checking to protect against DNS rebinding attacks on non-localhost IP bindings. When `TAURI_DEV_HOST` is bound, `server.allowedHosts` may need to include the dev host or `true` if remote device debugging is used. For default localhost desktop builds (`1420`), no change is needed.

---

### 4.3. Tailwind CSS v3 → v4 Migration & Token Rendering

#### Primary Source Facts:
- `tailwindcss@4.3.3` is built on the Oxide engine and Lightning CSS.
- **Directive Deprecations:**
  - `@tailwind base;`, `@tailwind components;`, `@tailwind utilities;` are replaced with a single `@import "tailwindcss";`.
- **Configuration Migration:**
  - In Tailwind v4, configuration is CSS-first via `@theme` directives rather than `tailwind.config.js`.
  - Standard CSS variables declared on `:root` continue to work without modification.
- **Preservation of DropCove Theme Tokens:**
  - In `src/style.css`, DropCove defines:
    - `:root` dark tokens (`--dc-canvas`, `--dc-surface`, etc.)
    - `@media (prefers-color-scheme: light)` overrides
    - `@media (forced-colors: active)` High Contrast system color overrides (`Canvas`, `ButtonFace`, `Highlight`, `LinkText`, etc.)
  - Inspection of `tailwindcss@4.3.3/index.css` shows it uses `@layer theme, base, components, utilities;`. Because DropCove's semantic variables are standard CSS custom properties declared on `:root` outside Tailwind utility classes, **Tailwind 4 does not alter, override, or regress any of DropCove's Light, Dark, or Contrast tokens**.
- **Build Plugin Integration (`@tailwindcss/vite` vs `@tailwindcss/postcss`):**
  - **Option 1 (`@tailwindcss/vite`):** Add `@tailwindcss/vite` plugin to `vite.config.ts` plugins array. Remove `postcss.config.js`, `tailwind.config.js`, and `autoprefixer`. This is the official Tailwind v4 recommended approach for Vite projects.
  - **Option 2 (`@tailwindcss/postcss`):** Replace `tailwindcss` with `@tailwindcss/postcss` in `postcss.config.js`. Remove `autoprefixer` (Tailwind 4 includes vendor prefixing via Lightning CSS).
  - Both options produce identical CSS output.

---

### 4.4. VueUse 15.0.0 Migration

#### Primary Source Facts:
- Manifest inspection of `@vueuse/core@15.0.0`:
  - `engines: { node: ">=22" }`
  - `peerDependencies: { vue: "^3.5.0" }`
- Codebase grep for VueUse imports:
  - `src/composables/useVirtualCollection.ts`:
    ```typescript
    import { useEventListener, useResizeObserver } from "@vueuse/core";
    ```
- Inspection of `@vueuse/core@15.0.0/dist/index.d.ts`:
  - `useResizeObserver` and `useEventListener` remain fully exported and maintain identical function signatures and reactivity contracts with Vue 3.5.
  - No breaking API changes affect DropCove.

---

### 4.5. Pinia 4.0.3 Migration

#### Primary Source Facts:
- Manifest inspection of `pinia@4.0.3`:
  - `peerDependencies: { vue: "^3.5.11", "@vue/devtools-api": "^8.1.5", typescript: ">=5.6.0" }`
- Codebase grep for Pinia imports:
  - `src/main.ts`:
    ```typescript
    import { createPinia } from "pinia";
    app.use(createPinia());
    ```
- Inspection of `pinia@4.0.3/dist/pinia.d.ts`:
  - Core functions `createPinia`, `defineStore`, `storeToRefs`, `getActivePinia`, `setActivePinia` remain exported with identical runtime signatures.
  - Required peer dependency `@vue/devtools-api@^8.2.1` must be added to `package.json` to prevent package manager peer resolution errors.

---

### 4.6. Vue Router 5.4.0 Migration

#### Primary Source Facts:
- Manifest inspection of `vue-router@5.4.0`:
  - `peerDependencies: { vue: "^3.5.34 || ^4.0.0", vite: "^7.3.0 || ^8.0.0", pinia: "^3.0.4 || ^4.0.2" }`
- Codebase grep for `vue-router`:
  - **Zero application files import or call `vue-router` in `src/`!**
  - DropCove uses native multi-window Tauri surfaces (`shelf`, `rail`, `settings`, `manage_items`) rather than a single-page route tree.
  - Updating `vue-router` to `5.4.0` in `package.json` has zero impact on application runtime behavior.

---

### 4.7. Tauri v2 Desktop & Windows x64 Compatibility

#### Primary Source Facts:
- Tauri v2 frontend contract:
  - `src-tauri/tauri.conf.json` specifies:
    - `"frontendDist": "../dist"`
    - `"devUrl": "http://localhost:1420"`
  - Tauri CLI runs `npm run build` (`vue-tsc && vite build`) to generate `dist/`, which is then embedded into the native Windows binary via `cargo-xwin`.
- Tauri packages compatibility:
  - `@tauri-apps/api@2.12.3`, `@tauri-apps/cli@2.12.2`, and `@tauri-apps/plugin-opener@2.7.1` have no restrictive Vite or PostCSS peer constraints.
  - They operate identically under Vite 8 so long as dev server port 1420 and strictPort remain configured.
- Windows Native MSVC Build:
  - Rust backend (`src-tauri/`) compiles independently with `cargo xwin build --target x86_64-pc-windows-msvc`.
  - Frontend upgrades do not touch Rust code, Cargo dependencies, or NSIS installer configurations.

---

## 5. Mandatory Source & Config Changes vs. Optional Modernization

### Mandatory Changes (approved for 0.0.9):
1. **`package.json` Dependency Updates:**
   - Update active packages:
     - `@vueuse/core`: `15.0.0`
     - `vue`: `3.5.44`
     - `@vitejs/plugin-vue`: `6.0.10`
     - `tailwindcss`: `4.3.3`
     - `vite`: `8.3.4`
     - `typescript`: `6.0.2`
     - `vue-tsc`: `3.3.13`
   - Add `@tailwindcss/vite` `4.3.3` and declare Node `>=22`.
   - Remove unused `pinia` and `vue-router`; do not add Pinia's devtools peer.
   - Remove obsolete `postcss` and `autoprefixer` dependencies.
2. **`src/style.css` Directives:**
   - Replace lines 1–3:
     ```css
     /* BEFORE (Tailwind v3): */
     @tailwind base;
     @tailwind components;
     @tailwind utilities;

     /* AFTER (Tailwind v4): */
     @import "tailwindcss";
     ```
3. **`vite.config.ts` Plugin Configuration:**
   - Add `@tailwindcss/vite` plugin:
     ```typescript
     import { defineConfig } from "vite";
     import vue from "@vitejs/plugin-vue";
     import tailwindcss from "@tailwindcss/vite";

     export default defineConfig(async () => ({
       plugins: [vue(), tailwindcss()],
       ...
     }));
     ```
4. **Remove Obsolete Configuration Files:**
   - Delete `postcss.config.js` (handled natively by Vite + `@tailwindcss/vite`).
   - Delete `tailwind.config.js` (Tailwind 4 uses CSS-first configuration; no custom JS extensions exist in DropCove).

### Optional Modernization:
1. **`vite.config.ts` WebSocket / HMR Config:**
   - Migrate deprecated `hmr.protocol`, `hmr.host`, `hmr.port` to `server.ws` (optional; existing syntax still works with deprecation warnings).
2. **TypeScript Version Configuration:**
   - Address the `typescript@7.0.2` + `vue-tsc@3.3.13` blocker (see Section 6).

---

## 6. Research Recommendations and Approved Resolutions

1. **Resolution of TypeScript 7 Blocker:**
   - Research recommendation: retain TypeScript 5 or 6 while `vue-tsc` depends on the JavaScript compiler, rather than replace full Vue single-file-component checking with plain `tsc`.
   - **Approved 2026-10-14:** Use TypeScript `6.0.2` with `vue-tsc` `3.3.13`; defer TypeScript 7 until Volar supports its native compiler.
2. **Tailwind v4 Integration Method:**
   - Research recommendation: use `@tailwindcss/vite`, remove the PostCSS/Autoprefixer path, and keep DropCove's standard CSS token system.
   - **Approved 2026-10-14:** Migrate to Tailwind 4 through `@tailwindcss/vite`.
3. **Inactive Dependencies:**
   - Research found no Vue Router callsite and no Pinia store or consumer.
   - **Approved 2026-10-14:** Remove `vue-router` and `pinia` instead of upgrading unused packages.
