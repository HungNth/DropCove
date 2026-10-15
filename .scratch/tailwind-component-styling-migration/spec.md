# DropCove 0.0.11 — Tailwind Component Styling Migration

Status: resolved
Version: 0.0.11
Design confirmation: approved by user on 2026-10-15
Testing seam: disposable before/after computed-style and geometry evidence plus actual Windows WebView2 surfaces, approved by user on 2026-10-15
Ticket decomposition: three implementation slices and one release slice approved by user on 2026-10-15
Implementation approval: approved by user on 2026-10-15; Git commit authorization remains separate
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.11
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md)
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md)

## Problem Statement

DropCove already builds with Tailwind CSS v4, but its frontend presentation is still primarily expressed through component-scoped CSS and ordinary global CSS. The application therefore carries an active Tailwind toolchain without using Tailwind as its component styling convention. Layout, spacing, dimensions, typography, interaction states, and surface presentation are distributed across thirteen Vue style blocks plus a global stylesheet.

The current presentation is accepted product behavior. Drop Shelf, Edge Rail, Manage Items, Settings, confirmation dialogs, resize handles, Shell Visuals, focus indicators, drag states, theme integration, and virtualized geometry must not change while styling ownership moves to Tailwind. A broad migration can easily alter selector specificity, scoped-style behavior, forced-color system mappings, transparent window corners, fixed card measurements, native window geometry, or Tailwind source detection even when the templates appear equivalent.

Maintainers need one clear component styling convention without introducing a second semantic token source, a component framework, compatibility styling, or a visual redesign disguised as migration work.

## Solution

Migrate ordinary Vue component presentation to Tailwind CSS v4 utility classes while preserving the exact accepted 0.0.10 rendering and behavior. Keep the existing `--dc-*` semantic variables as the only source of theme values and expose them to Tailwind through value-free `@theme inline` aliases. Dark, Light, and Windows Contrast Theme behavior continues to be owned by the existing semantic variables and system-color mappings.

Retain plain CSS only where it expresses application-wide policy, platform behavior, pseudo-elements, required keyframes, or runtime-computed geometry more correctly than template utilities. Remove superseded component rules as each surface migrates; do not keep parallel Tailwind and handwritten presentation paths.

Use direct utilities, existing class fallthrough, and explicit arbitrary descendant variants before considering a narrowly scoped cross-component CSS exception. Do not add styling props, wrapper components, `@apply` abstractions, custom component utilities, or a general component library.

Prove parity with disposable before/after computed-style and bounding-box evidence on identical fixtures, then exercise the real Windows WebView2 surfaces in Dark, Light, and actual Windows Contrast Theme presentation. Keep the existing frontend, native, cargo-xwin, Tauri, and NSIS release seams. Do not add a permanent screenshot or component-test framework for this migration.

## User Stories

1. As a Drop Shelf user, I want the empty and populated surfaces to look identical after the migration, so that a styling implementation change does not disrupt my workspace.
2. As a Drop Shelf user, I want the accepted rounded transparent-window corners and continuous border preserved, so that the surface does not regain corner gaps or opaque artifacts.
3. As a user dragging references into the Drop Shelf, I want drag-over and resize-hover presentation preserved exactly, so that established feedback remains reliable.
4. As a user resizing the Drop Shelf, I want all resize handles, cursors, hit regions, and responsive grid behavior unchanged.
5. As a user viewing Shelf Batches, I want card height, spacing, Shell Visual overlap, truncation, action placement, and drag regions unchanged.
6. As a user opening Manage Items, I want popup geometry, title, summary, rows, actions, status presentation, scrolling, and focus restoration unchanged.
7. As a user viewing Shelf Items, I want row height, availability presentation, Shell Visual sizing, truncation, and controls unchanged.
8. As a user of the Edge Rail, I want the Rail Handle and expanded surface to retain their accepted Left and Right corner ownership.
9. As a user of the expanded Edge Rail, I want compact card geometry, bounded height, scrolling, Open Shelf placement, Clear placement, and no-activate behavior unchanged.
10. As a user switching between ordinary and virtualized collections, I want mounted content, scroll bounds, card positions, column calculations, and keyboard navigation unchanged.
11. As a user opening Settings, I want every section, field, checkbox, select, action, status banner, and listening state to render and behave exactly as before.
12. As a user configuring Shake-to-open, I want sensitivity fields, recorder feedback, pulse animation, and reduced-motion behavior unchanged.
13. As a user configuring Edge Rail placement, I want monitor and edge controls to preserve their dimensions, focus states, and disabled states.
14. As a user responding to a Batch Drag confirmation, I want dialog bounds, backdrop, copy choices, buttons, focus, and cancellation behavior unchanged.
15. As a user responding to Clear Temporary Items confirmation, I want danger presentation, backdrop, focus, confirmation, and cancellation behavior unchanged.
16. As a pointer user, I want all hover, active, grab, grabbing, disabled, and drag states preserved.
17. As a keyboard user, I want the existing Tab order and every visible focus indicator preserved.
18. As a screen-reader user, I want accessible names, pressed states, checked states, mixed states, roles, and live announcements unchanged.
19. As a Windows Dark Theme user, I want all surfaces, text, borders, controls, warnings, and danger states to retain their accepted colors and contrast.
20. As a Windows Light Theme user, I want the same accepted surface hierarchy and transparent-window treatment preserved.
21. As a Windows Contrast Theme user, I want system colors, focus indicators, borders, controls, and disabled states to remain distinguishable.
22. As a user who prefers reduced motion, I want animations and transitions suppressed exactly as before.
23. As a user working with unavailable paths or persistence errors, I want status and error presentation preserved without changing Shelf state behavior.
24. As a user dragging Shelf Items or Shelf Batches out of DropCove, I want native Windows Copy behavior and source-file safety unchanged.
25. As a user with Pinned and Temporary Items, I want pinning, removal, Clear confirmation, and consumption rules unchanged.
26. As a user relying on Shell Visuals, I want runtime icon and thumbnail dimensions, clipping, image fit, and fallback presentation unchanged.
27. As a maintainer, I want ordinary component presentation expressed through Tailwind utilities, so that the installed styling toolchain is the active convention.
28. As a maintainer, I want `--dc-*` variables to remain the only semantic theme value source, so that Dark, Light, and Contrast mappings cannot drift.
29. As a maintainer, I want readable Tailwind color utilities backed by `@theme inline` aliases, so that templates do not repeat arbitrary CSS variable expressions.
30. As a maintainer, I want obsolete scoped rules removed as their replacements land, so that there is no hidden fallback or second styling path.
31. As a maintainer, I want dynamic virtualization and Shell Visual geometry to remain runtime-owned, so that static utility conversion does not corrupt measured layout.
32. As a maintainer, I want Tailwind class candidates to remain statically discoverable, so that production builds do not silently omit conditional styles.
33. As a maintainer, I want cross-component styling solved without generic styling props or wrapper abstractions, so that a mechanical migration does not widen component APIs.
34. As a release owner, I want disposable before/after evidence for representative computed styles and bounding boxes, so that subtle geometry and specificity regressions are visible.
35. As a release owner, I want the actual Windows WebView2 surfaces exercised in all supported presentation modes, so that build success is not mistaken for visual parity.
36. As a release owner, I want all version sources synchronized to 0.0.11 only after implementation gates pass, so that release metadata identifies a verified candidate.
37. As a release owner, I want one exact 0.0.11 NSIS installer and installed executable identified by size and SHA-256, so that acceptance applies to known bytes.
38. As a project owner, I want implementation and Git commit authorization to remain separate decisions, so that planning approval does not create repository history automatically.

## Implementation Decisions

- Treat version 0.0.11 as a mechanical styling cutover. Preserve rendered appearance, dimensions, DOM behavior, accessibility, motion, drag behavior, virtualization, and native window geometry.
- Do not normalize existing spacing, typography, radii, control dimensions, card dimensions, row dimensions, or surface hierarchy even where values appear duplicated.
- Keep the existing `--dc-*` semantic variables as the sole source of Dark, Light, Windows Contrast Theme, warning, danger, focus, and interaction-state values.
- Add `@theme inline` aliases that reference the existing semantic variables without copying their values. Tailwind utility aliases are an exposure layer, not a second token system.
- Use Tailwind utilities for ordinary layout, spacing, dimensions, borders, radii, overflow, typography, cursor behavior, user selection, touch behavior, and ordinary interaction states.
- Do not use `@apply` or custom component utilities to recreate the existing component CSS under new names.
- Retain plain CSS only for semantic token declarations and theme media queries, root sizing and reset, transparent WebView canvas behavior, Windows forced-color policy, global focus-visible policy, reduced-motion policy, required pseudo-elements or keyframes, and runtime-computed geometry.
- Preserve dynamic style bindings for virtual collection positioning, virtual spacer sizing, and runtime Shell Visual dimensions. These values are behavior and measurement inputs rather than ordinary presentation declarations.
- Keep conditional classes finite and literal so the Tailwind Vite integration can discover every production class. Do not construct utility names dynamically from runtime fragments.
- Replace cross-component scoped selectors with utilities, existing class fallthrough, or explicit arbitrary descendant variants where that preserves the current relationship. Do not add styling props, generic variants, or wrapper components solely to eliminate a selector.
- A narrow cross-component CSS selector may remain only when Tailwind cannot express the exact accepted relation without a larger component API or less maintainable markup. Record the reason at the retained rule.
- Preserve fixed measurements coupled to virtualization and native geometry, including card and row heights, gaps, minimum grid width, icon-control dimensions, Shell Visual overlap, dialog bounds, transitions, and box sizing.
- Preserve the established transparent borderless-window contract. Tailwind preflight must not introduce an opaque root background, changed native-control appearance, corner artifact, or altered box model.
- Preserve current command, event, persistence, drag, placement, focus-restoration, lifecycle, and accessibility ownership. No backend or native behavior change is part of the migration.
- Add no runtime dependency, component framework, class-composition package, visual-regression package, or frontend component-test framework.
- Keep package, crate, Tauri, PE, and installer metadata at 0.0.10 during the three implementation slices. Synchronize to 0.0.11 only in the release slice after parity and regression gates pass.
- Deliver the migration in four slices: shared tokens/base/action controls; Shelf and Edge Rail surfaces; Settings and confirmation surfaces; final release verification and version synchronization.
- No glossary update is required because Tailwind, semantic tokens, utilities, and computed styles are implementation language rather than DropCove domain concepts.
- No ADR is required because the migration changes styling representation without changing durable ownership, platform boundaries, persistence, windowing, or domain architecture.

## Testing Decisions

- The primary acceptance seam is the actual running Windows Resident Instance and its real WebView2 surfaces. This is the only existing seam that proves transparent windows, system themes, focus, drag presentation, virtualization, and native geometry together.
- Before migrating a slice, capture disposable baseline evidence for representative bounding boxes and selected computed properties on fixed fixtures. After migration, repeat the capture on the same machine, DPI, window geometry, theme, data, and state.
- Require exact equality for accepted fixed dimensions, spacing, borders, radii, fonts, box sizing, overflow, and semantic computed colors where WebView2 exposes stable values. Any difference must be explained and explicitly accepted rather than hidden behind a tolerance.
- Use visual inspection for whole-surface composition, transparent corners, native control rendering, anti-aliasing, and Windows Contrast Theme behavior that a property snapshot cannot prove.
- Exercise the Drop Shelf in empty, populated, drag-over, resize-hover, status, confirmation, ordinary collection, and virtualized collection states.
- Exercise the Edge Rail on both Left and Right edges in collapsed and expanded states, including bounded growth, scrolling, compact cards, hover/hold behavior, Open Shelf, Clear, and no-activate behavior.
- Exercise Manage Items from both Drop Shelf and Edge Rail origins with short and virtualized lists, status, pointer and keyboard closure, Escape, and focus restoration.
- Exercise Settings fields, checkboxes, selects, recorder/listening state, status, Save, Cancel, reduced motion, and disabled conditions.
- Exercise both confirmation dialogs with pointer and keyboard, backdrop, primary/secondary actions, danger presentation, Escape, focus, cancellation, and failure retention.
- Exercise resize handles in all eight directions and Shell Visual image, icon, thumbnail, stacked, unavailable, and fallback states.
- Run surfaces in Dark, Light, and actual Windows Contrast Theme presentation. Do not substitute forced-colors emulation for final Contrast Theme acceptance.
- Inspect focus, hover, active, disabled, pressed, checked, mixed, warning, danger, drag, grab, and grabbing states where applicable.
- Use enough Shelf Batches and Shelf Items to cross the existing virtualization threshold. Confirm bounded mounting, scroll positions, measured extents, column calculations, Tab behavior, and focused-item retention.
- Run the existing frontend production contract for Vue single-file-component typechecking, Vite bundling, and Tailwind generation. Confirm production output contains every conditional utility state used by the migrated surfaces.
- Run the complete produced Windows native regression executable. Existing native tests remain the behavioral seam for Shelf state, persistence, drag, placement, shutdown, and installer integration.
- Build through cargo-xwin for `x86_64-pc-windows-msvc`; do not introduce a Visual Studio or commercial MSVC Build Tools dependency.
- After implementation gates pass, synchronize every version source to 0.0.11, produce one exact NSIS installer, record installer and installed-executable byte sizes and SHA-256 hashes, and exercise those exact installed bytes.
- Exact-candidate smoke covers embedded frontend loading, all migrated surfaces, intake, drag-out, hotkey, notification-area access, startup integration, Settings, Shake-to-open, Edge Rail behavior, Cooperative Shutdown, repair/update, uninstall, and Profile Data preservation.
- Remove disposable baseline scripts, captured fixtures, temporary profiles, WebView data, and other smoke scaffolding before release completion. Preserve only written evidence in the ticket, specification, and roadmap.
- Do not add permanent tests that assert CSS source text, utility class strings, removed selector names, generated stylesheet bytes, or mock forwarding. Tests must catch user-visible behavior or geometry regressions.
- Record hardware, DPI, theme, and interaction limits exactly. Do not infer unobserved acceptance.

## Out of Scope

- Any visual redesign, spacing cleanup, typography system, radius normalization, density change, animation redesign, or accessibility-model change.
- Compact scrollbar styling, persistent scroll-viewport insets, Lucide icon replacement, or any other 0.0.12 work.
- TypeScript 7.
- New component libraries, CSS frameworks, class-composition dependencies, visual-regression infrastructure, or frontend component-test frameworks.
- New styling props, generic component variants, wrapper components, compatibility classes, fallback styles, or a second token source.
- Changes to Shelf state, persistence, database schema, drag result semantics, source-file safety, settings fields, notification-area behavior, startup integration, Shake-to-open, monitor ownership, Edge Rail timers, fullscreen policy, or native window ownership.
- Changes to native window size, placement, border ownership, DPI behavior, transparent-frame geometry, or supported Windows architecture.
- macOS, Linux, x86, or ARM64 support.
- Automatic updates, signing, or product behavior after version 0.0.11.
- Creating a Git commit without separate explicit user authorization.

## Further Notes

- Tailwind CSS v4 and its Vite integration are already installed and active; this specification adds no styling dependency.
- The migration currently covers thirteen Vue style blocks plus the global stylesheet. Runtime style bindings and shared virtual positioning remain intentionally outside ordinary utility conversion.
- The accepted 0.0.10 UI is the baseline. Existing visual quirks that are not regressions remain unchanged and can be considered only in a later approved version.
- The planned ticket graph has one shared foundation, two surface migrations that may proceed after the foundation, and one release ticket blocked by both surface migrations.
- Publishing this specification and its tickets authorizes planning only. Source implementation, final acceptance, Git commit creation, and any work from 0.0.12 remain separate gates.

## Implementation evidence — 2026-10-15

- User directed source migration to proceed with unobserved baseline states recorded explicitly. This does not waive ticket parity or release gates.
- Disposable `baseline.cjs` launched a separate Resident Instance using an owned `--profile`, isolated `WEBVIEW2_USER_DATA_FOLDER`, and WebView2 CDP endpoint. The measured 0.0.10 executable SHA-256 is `8e4a5cdf3dc79ee64d028a604396b0718964d84e9b379d2017a48d1024941a29`.
- Final identity-verified before/after captures recorded 35 corresponding states and 2,292 stable DOM-path-aligned elements on Windows `10.0.26300`, x64, Intel i5-12400F, devicePixelRatio `1`. `parity-differences.json` reports zero selected style/geometry differences, zero missing states, and zero image/fallback-state differences. Surface selection requires live Tauri window identity and mounted DOM markers; earlier title-race captures are superseded. Animated snapshots settle for 500 ms and pause infinite animations at time zero; physical motion is not inferred.
- Light and forced-colors measurements used WebView2 media emulation. Actual Windows Contrast Theme, Left/expanded Rail, native drag/resize states, transparent desktop corners, complete Settings recorder flows, and final human acceptance remain unobserved. Expanded Rail probes whose required visible marker was absent were excluded from observations.
- A freshly compiled Windows native regression executable built through cargo-xwin passed all 101 tests. Vendor CRT missing-PDB `LNK4099` warnings were non-fatal.
- Final `pnpm build` passed vue-tsc/Vite/Tailwind production generation, and `cargo xwin build --release --target x86_64-pc-windows-msvc` produced the migrated embedded-frontend smoke executable at version `0.0.10`, SHA-256 `ba8fefb89944a76931e68d5bb65d3aa8f72cc17dca3d718bf25c2a9cb864d901`.
- Actual browser-driven WebView2 smoke opened Shelf from Rail, toggled Pin and restored it, observed a visible 2px keyboard-focus outline, opened 55-item Manage Items with 10 mounted 54px rows and a normal-weight title, and used Escape to restore focus to Manage Items. The owned Resident exited 0 through profile-scoped Cooperative Shutdown. Both review axes report no remaining source finding after measured parity corrections.
- Tickets 01–03 are `ready-for-human`, not resolved. Ticket 04 is `needs-info`, blocked on their unchecked native/human acceptance criteria. No 0.0.11 candidate or installer acceptance is claimed; disposable parity artifacts remain isolated until release continuation.
- Version synchronization, 0.0.11 packaging, exact-installed-artifact acceptance, and release completion remain gated on the required parity evidence. No Git commit has been created.

### User acceptance and exact 0.0.11 candidate — 2026-10-15
- User confirmed human acceptance across all interactive on-screen gates: Windows Light/Dark/Contrast Themes, Animation Effects, Settings flows, physical drag-and-drop, and Edge Rail docking.
- Exact packaged installer: `build/DropCove-Setup-0.0.11.exe` (3,232,765 bytes, SHA-256 `81e551a81e716f9d4bf40bcdec082537c8222559650e0a93bb33b9f8ea85d337`).
- Exact installed executable: `13,385,728 bytes`, SHA-256 `d528988819c05e3bfb4c2a0c96ccdb5cdb52d6affd2fb872d8265b2696240abe`.
- Installed lifecycle smoke verified silent installation, secondary routing, Cooperative Shutdown, in-place upgrade, and uninstallation.
- Disposable baseline tooling and temporary data cleaned up.
- Changes remain uncommitted pending separate user authorization.
