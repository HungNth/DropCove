# DropCove 0.0.9 — Frontend Toolchain Modernization

Status: resolved
Version: 0.0.9
Design confirmation: approved by user on 2026-10-14
Testing seams: approved by user on 2026-10-14
Ticket decomposition: three vertical slices approved by user on 2026-10-14
Implementation approval: all three tickets approved by user on 2026-10-14; final human acceptance and commit authorization remain separate gates
Release-gate clarification: on 2026-10-14 user approved carrying actual Settings UI, expanded Rail UI, and physical Explorer Copy/Move/cancel from Ticket 02 to the exact-candidate human gate; technical closure permits the 0.0.9 candidate bump but does not imply these checks passed
Packaging-gap approval: on 2026-10-14 user approved adding Tauri's standard default custom-protocol feature to embed production assets; the earlier candidate hashes are invalid and must be replaced after standalone verification
Startup restoration clarification: on 2026-10-14 user approved reconciling the smoke-modified shared Run value to the verified original installation and real profile; exact pre-session startup bytes were not captured and are not claimed restored
Release acceptance: user confirmed all outstanding exact-candidate tests passed on 2026-10-14; Git commit authorization remains separate
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.9
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md)
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md)
Research: [Frontend upgrade constraints](research/frontend-upgrade-constraints.md)

## Problem Statement

DropCove 0.0.8 ships on an aging frontend build graph: VueUse 12, Vite 6, the Vite Vue plugin 5, Tailwind CSS 3, TypeScript 5.7, and vue-tsc 2. The Tailwind integration still uses JavaScript configuration, PostCSS, Autoprefixer, and v3 directives. The current manifest also carries Vue Router without any router and installs Pinia without any store, adding dependency and compatibility surface without delivering product behavior.

A direct upgrade to every version originally listed in the roadmap is not executable. TypeScript 7.0.2 replaced the JavaScript compiler implementation with native binaries, while vue-tsc 3.3.13 still loads and transforms the JavaScript compiler through Volar. Installing those exact versions together makes the existing `vue-tsc` build fail before Vite runs. Removing vue-tsc would also remove complete typechecking for Vue single-file components.

Users need the modernization to preserve DropCove rather than redesign it. Drop Shelf, Edge Rail, Manage Items, Settings, virtualization, native drag-and-drop, and the existing Dark, Light, and Windows Contrast presentation must remain observably equivalent while the dependency graph, typechecking, bundling, and Windows packaging move to a supported stack.

## Solution

Modernize the active frontend stack and delete inactive dependencies. Remove Vue Router and Pinia because DropCove uses Tauri window labels and composables rather than a route tree or Pinia stores. Upgrade VueUse to 15.0.0, Vite to 8.3.4, the Vite Vue plugin to 6.0.10, Tailwind CSS to 4.3.3, vue-tsc to 3.3.13, and Vue to the compatible 3.5.44 patch. Declare Node 22 or newer as the frontend development baseline.

Keep full Vue single-file-component typechecking by using TypeScript 6.0.2 for this release. Defer TypeScript 7 until vue-tsc/Volar supports the native compiler. Preserve the existing build command and reject any pipeline that silently omits Vue template or script typechecking.

Migrate Tailwind through its Vite-native v4 plugin. Replace v3 stylesheet directives with the v4 import, remove the obsolete PostCSS, Autoprefixer, and empty JavaScript Tailwind configuration, and keep the existing body utilities and semantic `--dc-*` token system. Do not convert DropCove tokens or scoped component styles into a second design system.

Keep all release metadata at 0.0.8 while the modernization is implemented and exercised. After dependency resolution, typechecking, bundling, native tests, cargo-xwin release build, Tauri runtime smoke, and packaging pass, synchronize metadata to 0.0.9, build one exact installer, record hashes, and submit that artifact for the user's final Dark, Light, Contrast, and native drag-and-drop acceptance.

## User Stories

1. As a DropCove user, I want the frontend toolchain modernized without changing how the application works, so that maintenance does not disrupt my workflow.
2. As a user opening the Drop Shelf, I want its layout, colors, focus behavior, and interactions to remain observably equivalent after the upgrade.
3. As a user relying on the Edge Rail, I want collapsed and expanded Rail behavior to remain unchanged after the frontend rebuild.
4. As a user managing a Shelf Batch, I want Manage Items to preserve its item list, virtualization, pinning, removal, keyboard, and close behavior.
5. As a user changing DropCove settings, I want Settings controls and persistence-facing interactions to remain unchanged.
6. As a user dragging files or folders into DropCove, I want native intake to continue creating the same Shelf references and batches.
7. As a user dragging Shelf Items or Shelf Batches out of DropCove, I want native Copy, Move, and cancel behavior to remain unchanged.
8. As a user with a large Shelf, I want virtualized Shelf, Edge Rail, and Manage Items collections to continue observing resize and scroll events correctly.
9. As a user in the default Dark presentation, I want every semantic surface, text, border, accent, warning, danger, drag, and focus token to retain its current meaning and appearance.
10. As a user in Light presentation, I want the same surfaces and interactions to use the existing light token overrides without regression.
11. As a Windows Contrast user, I want system colors, focus indicators, controls, and content to remain legible and operable.
12. As a user who prefers reduced motion, I want DropCove's existing reduced-motion behavior to remain intact.
13. As a user selecting text or interacting with the desktop utility, I want the existing antialiasing, body colors, and no-selection behavior to remain intact.
14. As a maintainer, I want unused Vue Router and Pinia dependencies removed, so that the supported stack reflects actual application architecture.
15. As a maintainer, I want the two active VueUse composables to keep their current cleanup and resize-observation behavior under VueUse 15.
16. As a maintainer, I want Vue single-file components checked by vue-tsc before bundling, so that modernization does not trade away template and script type safety.
17. As a maintainer, I want TypeScript 7 explicitly deferred while it is incompatible with vue-tsc, so that the manifest does not claim an impossible toolchain combination.
18. As a maintainer, I want Node 22 or newer declared as the development baseline, so that incompatible environments fail or warn before building.
19. As a maintainer, I want Tailwind v4 integrated directly with Vite, so that obsolete PostCSS and Autoprefixer plumbing is removed.
20. As a maintainer, I want Vite's Tauri development host, ports, strict-port behavior, and native-source watch exclusion preserved, so that the desktop development loop remains reliable.
21. As a maintainer, I want the production bundle emitted to the same frontend distribution directory, so that Tauri packaging continues to embed the application correctly.
22. As a maintainer, I want a clean pnpm resolution with one exact lockfile graph and no missing or incompatible peers, so that the release is reproducible.
23. As a maintainer, I want existing Tauri JavaScript packages left unchanged when no compatibility update is required, so that the release avoids unrelated churn.
24. As a maintainer, I want the full native Windows x64 test suite and cargo-xwin release build to remain green, so that frontend modernization cannot conceal a desktop regression.
25. As a maintainer, I want runtime smoke to exercise every Tauri surface, so that a successful web bundle is not mistaken for a working desktop application.
26. As a maintainer, I want final release evidence tied to one exact 0.0.9 installer and installed executable, so that acceptance applies to distributed bytes.
27. As a release owner, I want version metadata to remain 0.0.8 until all technical gates pass, so that failed intermediate builds are not presented as 0.0.9.
28. As a release owner, I want the final visual and native drag-and-drop gate performed against the exact candidate installer, so that human-visible behavior is accepted before completion.

## Implementation Decisions

- Preserve the existing Vue 3 Composition API and Tauri multi-window architecture. Do not introduce a route tree, a global store, an alternative frontend framework, or a second application shell.
- Remove Vue Router completely because there are no routes, router instances, route components, navigation guards, or imports. Tauri window-label dispatch remains authoritative.
- Remove Pinia and its empty application registration because there are no stores, Pinia plugins, subscriptions, or store consumers. Existing composables remain the state boundary.
- Target VueUse 15.0.0, Vite 8.3.4, the Vite Vue plugin 6.0.10, Tailwind CSS 4.3.3, vue-tsc 3.3.13, TypeScript 6.0.2, and Vue 3.5.44.
- TypeScript 7.0.2 is not part of the executable 0.0.9 graph. Its native compiler cannot be loaded by the Volar version used by vue-tsc 3.3.13. Do not replace vue-tsc with plain tsc or otherwise reduce Vue single-file-component checking.
- Preserve the manifest's existing compatible-range convention while recording one exact dependency graph in the pnpm lockfile. Verification reports the actually resolved versions.
- Declare Node 22 or newer in package metadata because VueUse 15 requires it. This is a development/build requirement and does not add a Node dependency to the installed desktop application.
- Use the official Tailwind v4 Vite plugin. Replace the v3 Tailwind directives with the v4 import and remove the direct PostCSS and Autoprefixer dependencies and their obsolete configuration.
- Remove the JavaScript Tailwind configuration because it contains no theme extension or plugin behavior. Keep Tailwind's generated body utilities and the existing semantic `--dc-*` variables.
- Keep the semantic token definitions as standard CSS custom properties. Do not migrate them into Tailwind `@theme`, rename them, change their values, or rewrite scoped component CSS into utility classes.
- Preserve Vite's dev server port, HMR/WebSocket port, strict-port behavior, optional Tauri dev host, and exclusion of native source from frontend file watching. Use supported Vite 8 configuration rather than introducing a compatibility shim.
- Preserve the production frontend output and Tauri commands. The native project continues to consume the same built frontend directory and package through cargo-xwin for Windows x64.
- Enable Tauri's standard default `custom-protocol` Cargo feature, forwarding to `tauri/custom-protocol`. Production packaging embeds the generated frontend; Tauri development continues to disable default features. This required packaging correction was separately approved after the original release binary was observed loading the development URL.
- Update Vue and other companion packages only where the approved graph requires compatibility. Keep the current Tauri API, CLI, and opener packages when the clean resolution proves no update is required.
- Do not add a frontend testing framework, router, store, CSS abstraction, compatibility layer, or migration helper solely for this dependency upgrade.
- Keep package, crate, Tauri, PE, and installer metadata at 0.0.8 during implementation. Synchronize all version metadata to 0.0.9 only after the implementation and pre-release runtime gates pass.
- Record the TypeScript 7 deferral in the roadmap. No ADR is required because this is an upstream compatibility constraint, is easy to reverse, and does not change product architecture. No glossary change is required because the work introduces no domain concept.

## Testing Decisions

- Use two approved high-level seams: the pnpm install/build flow for dependency resolution, Vue single-file-component typechecking, Tailwind/Vite transformation, and production bundling; and the exact Windows x64 installer/installed application for native packaging and observable behavior.
- Resolve the dependency graph from the manifest and verify the lockfile records the approved versions, removed dependencies are absent, Node compatibility is explicit, and pnpm reports no incompatible required peers.
- Run the unchanged build contract so vue-tsc completes before Vite. A passing Vite bundle without passing vue-tsc is not acceptance.
- Exercise the Vite/Tauri development path far enough to prove the configured host, ports, HMR/WebSocket behavior, and native-source watch exclusion still support the desktop development loop.
- Verify Tailwind v4 emits the body utilities used by the application and preserves all standard CSS token declarations. Assert behavior through rendered surfaces rather than source-text or generated-bundle snapshots.
- Exercise virtualization on Shelf cards, Edge Rail cards, and Manage Items rows so both VueUse event listeners and resize observation are proven in the application.
- Run the complete produced Windows native test executable after compiling it with cargo-xwin. Do not treat a cross-compilation command that attempts to invoke Wine as runtime evidence.
- Run a cargo-xwin release build and the existing Tauri/NSIS packaging flow for Windows `x86_64-pc-windows-msvc`.
- Standalone installed-artifact smoke must load the embedded production origin (`tauri.localhost` on Windows), not a Vite/dev HTTP server. Verify the origin, bundled scripts, mounted surfaces, and native IPC before accepting artifact hashes.
- Before the version bump, launch the real Tauri application and smoke Drop Shelf, Edge Rail, Manage Items, Settings, dialogs, Shell Visuals, virtualization, intake, and drag-out. Fix only modernization regressions.
- At the release gate, synchronize metadata to 0.0.9, build once, record the exact setup and installed executable byte sizes and SHA-256 hashes, and use those exact bytes for release acceptance.
- Agent verification covers clean install, typecheck/build, native tests, cargo-xwin build, packaging, launch, and runtime smoke. The user performs the final Dark, Light, Windows Contrast, and native drag-and-drop acceptance on the exact candidate.
- Observable equivalence is the visual contract. Generated CSS is allowed to differ internally, but layout, semantic token rendering, focus, forced colors, reduced motion, and interaction behavior must not regress.
- Do not add permanent tests that assert package manifest text, lockfile formatting, generated CSS bytes, configuration forwarding, or the mere presence of output. Add a focused permanent test only if implementation exposes a plausible consumer-visible regression that an existing high-level seam cannot retain.

## Out of Scope

- Drop Shelf or Edge Rail visual refinement assigned to version 0.0.10.
- New routes, navigation, Pinia stores, global state architecture, or changes to Tauri window-label dispatch.
- TypeScript 7 until vue-tsc/Volar supports its native compiler without dropping Vue single-file-component checking.
- Rewriting scoped CSS into Tailwind utilities, converting `--dc-*` variables into Tailwind theme tokens, or changing token values.
- Product behavior, persistence, installer lifecycle, Recovery Backup, Cooperative Shutdown, startup, hotkey, Shake-to-open, or placement changes.
- A new frontend unit, component, browser, screenshot, lint, or formatting framework.
- Updating Rust dependencies, the Tauri Rust architecture, or NSIS behavior except synchronized release metadata and required packaging verification.
- Supporting Node versions below 22, Windows versions before Windows 11 build 22000, x86, ARM64, macOS, or Linux.
- Advancing to 0.0.10, 0.1.0, or 1.0.0.
- Creating a Git commit without separate explicit user authorization.

## Further Notes

- The current developer environment uses Node 24.18.0 and pnpm 12.11.2, satisfying the approved Node baseline.
- All originally requested package versions exist. The only rejected pairing is TypeScript 7.0.2 with vue-tsc 3.3.13 because it is demonstrably non-executable, not because either package is unavailable.
- The application currently uses VueUse only for event-listener cleanup and resize observation in virtualization. This narrow callsite keeps the VueUse migration surface small.
- Tailwind currently contributes the body utilities and CSS layers; the DropCove token system and component styling are ordinary CSS. The v4 migration preserves that boundary.
- Implementation, code-level review, and exact-candidate human acceptance are complete. Git commit approval remains a separate gate.