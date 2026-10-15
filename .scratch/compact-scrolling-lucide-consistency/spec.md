# DropCove 0.0.12 — Compact Scrolling and Lucide Icon Consistency

Status: resolved
Version: 0.0.12
Design confirmation: approved by user on 2026-10-15
Testing seam: real Windows Resident Instance and WebView2 surfaces plus existing frontend, native, cargo-xwin, and NSIS release checks; approved by user on 2026-10-15
Ticket decomposition: two implementation slices and one release slice approved by user on 2026-10-15
Implementation approval: approved by user on 2026-10-15; Git commit authorization remains separate
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.12
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md)
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md)

## Problem Statement

DropCove's user-facing vertical scroll surfaces still use the default WebView2 scrollbar presentation. The result is visually heavier than the compact Drop Shelf, Edge Rail, Manage Items, and Settings surfaces, and the scrollbar states do not consistently follow DropCove's Dark, Light, and Windows Contrast Theme semantics.

Collection content also loses its visual separation from the containing surface's top edge after scrolling. Ordinary padding moves with the scroll content, so a fully or partially visible Shelf Batch or row can touch the top border. The shared virtual collection calculations are zero-based and must not gain hardcoded offsets for individual components.

Action and empty-state icons remain handwritten SVG paths. Their geometry and current 16-pixel action size are inconsistent with the approved Lucide cutover and the roadmap's fixed 14-pixel action and 28-pixel empty-state glyph sizes. These presentation changes must not alter control targets, accessibility semantics, collection density, virtualization, drag behavior, native placement, or Shelf state.

## Solution

Apply one native WebView2 scrollbar treatment to the four existing user-facing vertical scroll containers: the Drop Shelf batch grid, expanded Edge Rail batch list, Manage Items item list, and Settings content pane. Use an 8-pixel interaction channel, an approximately 4-pixel rounded visible thumb, a transparent track, semantic idle/hover/active colors, Windows system colors in Contrast Theme, and automatic visibility only when content overflows.

Establish a shared 4-pixel top viewport boundary outside the actual scrollport for the Drop Shelf, Edge Rail, and Manage Items collections. The boundary clips scrolling content before it can touch the containing surface's top edge. Preserve each surface's total vertical clearance and scroll bounds by redistributing existing spacing rather than adding height. Keep virtual collection measurement and positioning zero-based against the real inner scrollport, with no component-specific offset parameters.

Add `lucide-vue-next` and replace the existing handwritten action and empty-state SVG paths through the current thin icon-role adapter. Preserve every button's tooltip, accessible name, pressed/checked/mixed/disabled state, focus behavior, and 24-by-24-pixel target. Render action glyphs at 14 by 14 pixels and the empty Drop Shelf glyph at 28 by 28 pixels.

## User Stories

1. As a Drop Shelf user, I want a compact scrollbar that matches the small floating surface, so that scrolling controls do not dominate the workspace.
2. As an Edge Rail user, I want the same compact scrollbar treatment as the Drop Shelf, so that both Shelf surfaces feel consistent.
3. As a Manage Items user, I want item-list scrolling to use the same visible interaction states, so that the scroll affordance remains predictable.
4. As a Settings user, I want the Settings scrollbar to use the same theme-aware presentation, so that the utility has one scrolling convention.
5. As a pointer user, I want an 8-pixel scrollbar interaction channel, so that the compact visible thumb remains practical to target.
6. As a pointer user, I want the scrollbar thumb to remain visible while idle and become stronger on hover and drag, so that I can identify its state.
7. As a keyboard or wheel user, I want native WebView2 scrolling behavior preserved, so that custom controls do not interfere with established input.
8. As a user whose content does not overflow, I want no unnecessary scrollbar channel reserved, so that narrow surfaces retain their accepted width.
9. As a Windows Dark Theme user, I want scrollbar colors drawn from DropCove's semantic theme, so that they retain appropriate contrast.
10. As a Windows Light Theme user, I want the same semantic state progression, so that the thumb remains visible without becoming visually heavy.
11. As a Windows Contrast Theme user, I want scrollbar states expressed with Windows system colors, so that idle, hover, and drag remain distinguishable.
12. As a Drop Shelf user, I want Shelf Batches to remain separated from the top surface border at every scroll position, so that partially visible cards are not visually clipped against the frame.
13. As an Edge Rail user, I want compact Shelf Batch rows to retain a top boundary while scrolling, so that the rail's continuous border remains legible.
14. As a Manage Items user, I want item rows to remain separated from the top viewport edge, so that scrolling does not collapse the popup's visual hierarchy.
15. As a user with an ordinary collection, I want the top boundary to work without virtualization, so that small collections behave like large ones.
16. As a user with more than 50 entries, I want the same boundary while virtualization is enabled, so that collection size does not change presentation.
17. As a keyboard user traversing a virtualized collection, I want focus scrolling to reveal the target within the bounded viewport, so that the focused control is not hidden under the top boundary.
18. As an Edge Rail user with five through eleven batches, I want the existing height tiers to fit without a premature scrollbar, so that compact expansion behavior remains unchanged.
19. As a Drop Shelf user, I want existing responsive columns and card widths preserved, so that the new boundary does not alter narrow-window layout.
20. As a user at maximum scroll, I want the final card or row to remain reachable without extra blank range, so that scrolling bounds remain correct.
21. As a user viewing actions, I want recognizable Lucide glyphs for pinning, management, removal, closing, clearing, and navigation, so that icon language is consistent.
22. As a user viewing mixed Bulk Pinning state, I want a distinct Pin and Minus treatment, so that the visual state agrees with the mixed checkbox state.
23. As a user viewing a pinned action, I want the Pin glyph to retain its filled state, so that pinned and unpinned controls remain distinguishable.
24. As a screen-reader user, I want button labels and pressed or checked states to remain authoritative, so that decorative glyph changes do not alter the accessibility contract.
25. As a keyboard user, I want focus order, focus restoration, Enter, Space, Tab, Shift+Tab, and Escape behavior preserved, so that icon replacement does not change interaction.
26. As a pointer user, I want native tooltips and hover states preserved, so that compact icon controls remain understandable.
27. As a Drop Shelf user viewing an empty Shelf, I want the downward Lucide glyph and `Drop here` message to remain centered and clear, so that the drop target is immediately recognizable.
28. As a user dragging Shelf Items or Shelf Batches, I want drop-in and Copy-only drag-out workflows unchanged, so that presentation work does not affect file safety.
29. As a user relying on the Edge Rail, I want Left and Right placement, expansion, fullscreen handling, and no-activate behavior unchanged.
30. As a maintainer, I want native scrollbar pseudo-elements owned by the existing global platform-style boundary, so that no custom Vue scrollbar or second scrolling model is introduced.
31. As a maintainer, I want one shared viewport-boundary contract without component offsets in virtualization, so that measurements remain understandable.
32. As a maintainer, I want Lucide roles centralized through the existing thin adapter, so that call sites retain their current accessibility ownership without repeated mapping logic.
33. As a release owner, I want actual WebView2 evidence for all four scroll surfaces and every replaced icon, so that build success is not mistaken for visual acceptance.
34. As a release owner, I want exact 0.0.12 installer and installed-executable evidence, so that acceptance applies to known bytes.
35. As a project owner, I want planning, implementation, final acceptance, and Git commit authorization to remain separate decisions.

## Implementation Decisions

- Scope the scrollbar change to the four existing user-facing vertical scroll containers: Drop Shelf batch grid, expanded Edge Rail batch list, Manage Items item list, and Settings content pane.
- Keep overflow automatic. Do not force a scrollbar when content fits and do not reserve a permanent channel on narrow surfaces.
- Use native WebView2 scrollbar pseudo-elements rather than a custom Vue scrollbar component. Preserve native wheel, pointer, keyboard, accessibility, and scroll-position behavior.
- Use an 8-pixel scrollbar channel with an approximately 4-pixel visible rounded thumb and transparent track.
- Map the idle thumb to the semantic default-border color, hover to the semantic hover-border color, and active drag to the semantic accent color.
- Under Windows Contrast Theme, use `ButtonText` for the idle thumb and `Highlight` for hover and active drag while keeping the track transparent.
- Apply the scrollbar policy through the existing global CSS boundary for platform pseudo-elements. Do not establish a component styling abstraction or parallel token source.
- Apply a canonical 4-pixel fixed top viewport boundary to Drop Shelf, Edge Rail, and Manage Items collection surfaces. Settings receives scrollbar styling but no collection inset.
- Place the boundary outside the actual scrollport. Do not rely on scrollable top padding or a sticky overlay, because those do not define the clip boundary for ordinary and virtualized content.
- Preserve total collection clearance by redistributing existing spacing. The Edge Rail retains 4 pixels above and below its rows; its expanded height uses `max(280, min(640, 52 × batch count + 62))`, approved by the user after runtime measurement exposed the previous 10-pixel chrome-budget deficit. Header, footer, row density, targets, minimum height, and maximum height remain unchanged.
- Keep virtual collection coordinates zero-based against the real scrollport. Do not add component-specific inset arguments or hardcoded Drop Shelf, Edge Rail, or Manage Items offsets to shared virtualization logic.
- Preserve virtualized start/end calculation, bounded mounting, focused-item retention, keyboard reveal, column width, row height, gap, total height, and maximum-scroll behavior.
- Preserve the Drop Shelf's responsive columns, minimum card width, card density, automatic growth, resize behavior, and transparent border geometry.
- Preserve Edge Rail compact row geometry, bounded expansion tiers, Left and Right placement, open/clear controls, no-activate behavior, and fullscreen policy.
- Preserve Manage Items popup geometry, row density, Tab cycle, Escape handling, focus restoration, item drag behavior, and status presentation.
- Add only `lucide-vue-next`; retain the existing Vue, TypeScript, Vite, Tailwind, and Tauri versions. TypeScript 7 remains deferred.
- Keep the existing action-icon adapter as a thin role-to-Lucide mapping boundary. Remove every handwritten action and empty-state SVG path owned by that adapter.
- Map Pin/Unpin to `Pin`, mixed Bulk Pinning to a composed `Pin` and `Minus`, Manage Items to `List`, removal to `CircleMinus`, close to `X`, Clear Temporary Items to `Trash2`, Edge Rail direction to `ArrowLeft` or `ArrowRight`, and the empty Drop Shelf to `ArrowDown`.
- Keep glyphs decorative. Existing buttons continue to own accessible names, titles, pressed states, checked and mixed states, disabled state, keyboard activation, and focus presentation.
- Render every action glyph at 14 by 14 pixels inside the existing 24-by-24-pixel target. Render the empty-state glyph at 28 by 28 pixels instead of the current 32-pixel override.
- Exclude textual status-dismiss `×` controls and non-interface SVG assets because they are not handwritten action or empty-state SVG paths.
- Make no Shelf state, persistence, database, native command, drag result, window placement, settings-schema, or installer-behavior change.
- Keep package, crate, Tauri, PE, and installer metadata at 0.0.11 until implementation and pre-release gates pass. Synchronize all version sources to 0.0.12 in the release slice.
- No glossary update is required because viewport boundaries, scrollbars, and Lucide glyphs are presentation implementation language rather than DropCove domain concepts.
- No ADR is required because these choices are reversible presentation details that do not change durable ownership, platform boundaries, persistence, or architecture.
- Do not create a Git commit without separate explicit user authorization.

## Testing Decisions

- The primary acceptance seam is the actual running Windows Resident Instance and its real WebView2 surfaces. This seam proves scrollbar rendering, clipping, transparent windows, native themes, pointer states, keyboard behavior, virtualization, and native geometry together.
- Exercise the Drop Shelf, expanded Edge Rail, Manage Items, and Settings scroll containers with content that does and does not overflow.
- Exercise ordinary collections with at most 50 entries and virtualized collections with more than 50 entries at the start, middle, and maximum-scroll positions.
- Measure the 4-pixel top viewport boundary against fully and partially visible cards or rows. Verify that content cannot paint into the boundary while scrolling.
- Verify that the actual inner scrollport remains the virtualization measurement seam and that no fixed component offset is required for start/end calculations, absolute positioning, total height, focused-item reveal, or maximum scroll.
- Verify the Drop Shelf at minimum width and representative multi-column widths. Confirm unchanged card width, column count, horizontal overflow policy, resize behavior, and drag workflows.
- Verify expanded Edge Rail with one through four, five through eleven, twelve, and more than fifty batches. Confirm existing height tiers, no premature scrolling, bounded mounting, and correct final-row reachability.
- Verify Manage Items with short and virtualized lists, pointer and keyboard opening, complete Tab and Shift+Tab traversal, Escape, focus restoration, item drag, removal, and maximum scroll.
- Verify Settings with short and overflow content while preserving fields, recorder behavior, Save, Cancel, status presentation, and keyboard focus.
- Exercise scrollbar idle, hover, and active drag states in Dark, Light, and actual Windows Contrast Theme. Confirm an 8-pixel channel, approximately 4-pixel visible thumb, rounded shape, transparent track, semantic colors, and system-color overrides.
- Do not substitute forced-colors emulation for final Windows Contrast Theme acceptance. Emulation may support development measurements but must be identified as such.
- Inspect every Lucide role at normal and narrow surface widths. Confirm 14-pixel action glyphs, 28-pixel empty-state glyph, 24-pixel control targets, native tooltips, accessible names, pressed, checked, mixed, disabled, hover, and focus-visible states where applicable.
- Verify filled Pin and composed mixed Pin/Minus presentation without moving the button's accessibility state onto the decorative glyph.
- Run the existing frontend production build for Vue single-file-component typechecking, Vite bundling, Tailwind generation, and dependency integration.
- Run the complete newly produced Windows native regression executable. Existing tests remain the regression seam for Shelf state, persistence, drag, placement, settings, shutdown, and installer integration.
- Build through cargo-xwin for `x86_64-pc-windows-msvc`; do not introduce a Visual Studio or commercial MSVC Build Tools dependency.
- Do not add a permanent frontend test framework for CSS pseudo-elements, visual clipping, or glyph geometry. Use disposable runtime measurements and direct WebView2 inspection, then remove the tooling before release completion.
- After implementation gates pass, synchronize every version source to 0.0.12, produce one exact NSIS installer, record installer and installed-executable byte sizes and SHA-256 hashes, and exercise those exact installed bytes.
- Exact-candidate smoke covers embedded frontend loading, all changed surfaces, ordinary and virtualized scrolling, intake, drag-out, pinning, management, hotkey, notification-area access, Settings, Shake-to-open, Edge Rail behavior, Cooperative Shutdown, update, uninstall, and Profile Data preservation.
- Record every unobserved hardware, DPI, theme, or interaction limit explicitly. Do not infer acceptance from an adjacent state.

## Out of Scope

- TypeScript 7 or any dependency upgrade other than `lucide-vue-next`.
- A custom Vue scrollbar, scroll-position model, gesture system, or replacement for native WebView2 scrolling.
- Scrollbar styling for confirmation dialogs, status messages, non-scrollable containers, native Windows controls, or future surfaces not named by the roadmap.
- A collection top inset for Settings.
- New component-specific virtualization offsets, fixed geometry guesses, compatibility branches, or a second virtual collection implementation.
- Card, row, window, popup, header, footer, spacing, typography, radius, animation, or density redesign beyond the approved boundary and glyph-size changes.
- Changes to textual status-dismiss controls or non-interface SVG assets such as application and packaging icons.
- Changes to Shelf state, persistence, database schema, Temporary or Pinned lifecycle, duplicate handling, drag result semantics, source-file safety, settings fields, notification-area behavior, startup integration, Shake-to-open, monitor ownership, Edge Rail timers, fullscreen policy, recovery, or Cooperative Shutdown.
- New permanent screenshot, visual-regression, frontend component-test, or accessibility-test frameworks.
- macOS, Linux, x86, or ARM64 support.
- Signing, automatic updates, or product behavior after version 0.0.12.
- Creating a Git commit without separate explicit user authorization.

## Further Notes

- Version 0.0.11 is the accepted Tailwind styling baseline and remains uncommitted pending separate Git authorization.
- The 4-pixel boundary and 8-pixel scrollbar channel are separate contracts: the boundary controls where collection content may paint, while the scrollbar channel controls pointer interaction.
- Edge Rail height includes the accepted 28-pixel header, 28-pixel footer, 2-pixel borders, row gaps, and 8 pixels of total collection clearance. The user approved correcting the additive height budget from 52 to 62 on 2026-10-15.
- The planned ticket graph should keep compact scrolling, Lucide replacement, and release verification independently reviewable while avoiding overlapping scroll-container rewrites.
- Publishing this specification authorizes planning only. Ticket approval, source implementation, final acceptance, and Git commit creation remain separate gates.

## Development evidence and resolved planning blocker — 2026-10-15

- Tickets 01 and 02 are resolved after automated checks and user-reported interaction acceptance. Version metadata is synchronized to 0.0.12 in release ticket 03; exact installed-candidate verification and final release acceptance remain separate gates.
- Frontend typechecking/bundling and cargo-xwin release compilation passed. The compiled Windows native test executable passed 101/101 tests.
- Real WebView2 confirmed 4-pixel collection boundaries, an 8-pixel scrollbar channel, 14-pixel action glyphs, 24-pixel targets, a 28-pixel empty state, filled/mixed Pin states, ordinary/virtualized collections, reactive columns, maximum-scroll reachability, and management focus reveal/restoration.
- Historical pre-correction finding: `52*n+52` provided 624 pixels at 11 batches, leaving a 10-pixel deficit after the preserved header/footer/border budget. The correction and measured resolution are recorded below.
- User approved changing the additive height constant from 52 to 62 on 2026-10-15. Corrected native regression passed 1/1; the complete rebuilt native suite passed 101/101; real WebView2 verified no premature overflow across every count 5–11 and reachable final card at count 12.
- User reported all requested interactions successful after being asked to verify actual Windows Contrast Theme, physical scrollbar/drag interactions, Settings, and Left Rail. Agent-side Light/forced-colors measurements used media emulation and remain distinct from that human acceptance. Exact installed-candidate acceptance remains pending.
- Static Standards and Spec reviews returned no source findings; their rail-fit inference was disproved by runtime measurement and is not acceptance evidence.
- Owned automated smoke processes and data were cleaned up. The user's separate review Resident was later closed through profile-scoped Cooperative Shutdown to release the executable lock before packaging. No Git commit was created.
- A development regression moved horizontal padding outside the scrollport, eliminating card-to-scrollbar clearance. It was corrected before delivery: padding remains inside each scrollport and shared virtual positioning measures its content origin from computed padding. Rebuilt real-WebView2 assertions confirmed Shelf left/right clearance 7/7 pixels at widths 180/356/524, Rail 6/6, and Manage Items 7/7, all with 4-pixel top boundaries. No per-component virtual offset constant was added.

### Corrected Edge Rail height budget verification — 2026-10-15

- Corrected executable: `src-tauri/target/x86_64-pc-windows-msvc/release/dropcove.exe` (13,385,728 bytes, SHA-256 `db96271dd6654f184f7f8ff644ef1d8ba2be0940c1235bcd71b781bfa18d7276`).
- Real Windows WebView2 expanded Edge Rail verified across batch counts 5 through 11 and 12 on an isolated profile with unique CDP port:
  - Seeded owned fixture files via `accept_drop`.
  - Docked Drop Shelf to Edge Rail via native main window `WM_CLOSE`.
  - Hover-expanded rail via `on_rail_pointer_enter` and `tick_rail`.
  - Batch counts 5 through 11 proved exact heights 322 through 634 progressing by 52px steps with zero overflow (`scrollHeight == clientHeight`, no premature scrollbar):
    - 5 batches: height 322, scroll content 260, client height 260, overflow=false.
    - 6 batches: height 374, scroll content 312, client height 312, overflow=false.
    - 7 batches: height 426, scroll content 364, client height 364, overflow=false.
    - 8 batches: height 478, scroll content 416, client height 416, overflow=false.
    - 9 batches: height 530, scroll content 468, client height 468, overflow=false.
    - 10 batches: height 582, scroll content 520, client height 520, overflow=false.
    - 11 batches: height 634, scroll content 572, client height 572, overflow=false (resolving the previous 10px deficit).
  - Batch count 12 proved clamped maximum height 640, scroll content 624, client height 578, vertical overflow present (`scrollHeight > clientHeight`), and reachable maximum scroll with the 12th card fully visible in the viewport.
  - All tested counts preserved the 4px fixed top viewport boundary outside the scrollport and 6px horizontal clearance between cards and the rail container / scrollbar channel.
- Clean cooperative shutdown (`--shutdown --profile`) returned exit code 0; primary process exited cleanly; owned profiles, WebView2 data, fixtures, and scaffolds deleted.
- Verification limits: Batch counts 1-4 and virtualized >50 items not re-exercised in this smoke slice; left-docked rail edge, high DPI display scaling, actual Windows Contrast Theme, physical mouse drag/hover, Settings runtime, and packaging/installer remain unverified here.

### User acceptance and release candidate — 2026-10-15
- User confirmed all interactions successful on the corrected development executable, closing the implementation slices' human gates. This confirmation predates the exact NSIS candidate and does not imply exact-installed-artifact acceptance.
- Version sources synchronized to 0.0.12; official `make build` completed frontend typechecking/bundling, cargo-xwin release compilation, and NSIS packaging, producing `build/DropCove-Setup-0.0.12.exe`.
- Exact installed-candidate lifecycle smoke passed. The user explicitly accepted the installed 0.0.12 candidate on 2026-10-15; release ticket 03 and the specification are resolved.


### Exact installed candidate — 2026-10-15
- Installer: 3,233,822 bytes, SHA-256 `27f569cf9d37c59380a090d9e531128531689fd4958b056db40f78ec27428411`.
- Installed executable: 13,385,728 bytes, SHA-256 `296b50c7c31bcb335148c6b8042744375746f7c246f353a1b094efe8035eff0a`.
- Silent isolated installation, embedded production loading, intake/pinning/management/mixed-icon smoke, secondary activation, Cooperative Shutdown, same-candidate reinstall/update, safe shortcut-conflict abort, and uninstall with Profile Data preservation passed.
- Owned smoke data/processes were cleaned up and existing user installation, profile, shortcut, and registry baseline restored. Final source reviews have no actionable finding. User explicitly accepted this installed candidate on 2026-10-15; physical/theme acceptance is user-reported. Fractional/high-DPI and multi-monitor combinations remain unmeasured by the agent. No Git commit created.

