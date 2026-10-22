# DropCove 0.0.19 — Appearance modes and palette refresh

Status: ready-for-agent
Version: 0.0.19
Design confirmation: approved by the user on 2026-10-22 through `grill-with-docs`
Testing seam: exact installed Windows Resident/WebView2 smoke across real DropCove WebView surfaces and native frames, supported by the existing deterministic Settings Store and Settings transaction/coordinator regression seams; approved by the user on 2026-10-22
Delivery prerequisite: completed — DropCove 0.0.18 is accepted and the canonical roadmap defines 0.0.19 as the next release
Ticket decomposition: five tracer-bullet slices and the linear blocker chain approved by the user on 2026-10-22
Implementation approval: the user instructed implementation of the approved tickets on 2026-10-22; exact-candidate human acceptance and Git commit authorization remain separate gates
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), especially Settings and Accessibility, appearance, and motion
Architecture: [Architecture](../../docs/ARCHITECTURE.md), especially Settings persistence and Appearance synchronization
Appearance ownership: [ADR-0005](../../docs/adr/0005-application-appearance-mode.md)
Domain language: [Glossary](../../GLOSSARY.md), especially **Settings** and **Appearance Mode**
Roadmap: [DropCove 0.0.19](../../docs/ROADMAP.md#0019--appearance-modes-and-palette-refresh)

## Problem Statement

DropCove currently follows the Windows application Light or Dark theme without offering an application preference. A user who wants DropCove to remain Dark while Windows is Light, or remain Light while Windows is Dark, cannot express that choice. The native frames and WebView surfaces are both system-driven, so changing this behavior must remain coherent across every DropCove window rather than styling only the Settings content or the Drop Shelf.

The current Settings surface also presents every preference in one scrolling view. Adding another durable preference without navigation would make the fixed-size window denser and harder to scan. Existing settings must remain easy to find, and Appearance must use the same draft, Save, Cancel, close, failure, and persistence lifecycle rather than becoming a second settings system.

The existing navy and indigo presentation does not match the requested neutral black/light-gray visual direction. DropCove needs a coordinated semantic palette based on exact application-background and Shelf Batch card colors, with accessible text, borders, focus, buttons, icons, feedback states, shadows, Dark/Light rendering, and Windows Contrast Theme behavior. The visual refresh must not change Shelf state, source files, drag behavior, window geometry, accessibility semantics, or Resident Instance ownership.

Adding `AppearanceMode` also extends DropCove's strict persisted settings contract. A pre-0.0.19 settings file is intentionally not migrated: it must remain byte-for-byte untouched, load all current defaults with a warning, and be replaced only after explicit successful Save. This resets every existing preference in memory and is a deliberate, user-approved release consequence that must be visible in implementation, tests, release notes, and acceptance evidence.

## Solution

Add horizontal **General** and **Appearance** tabs to the existing fixed-size Settings surface. General contains every existing control. Appearance contains a three-option preview selector for **System**, **Dark**, and **Light**. Settings opens on General, does not persist the selected tab, retains one shared draft while switching tabs, and keeps one Save/Cancel footer.

Persist one application-wide **Appearance Mode** in the existing settings contract. System is the default and follows live Windows application-theme changes. Dark and Light remain fixed across ordinary Windows theme changes. Windows Contrast Theme always has higher rendering authority while preserving the selected Appearance Mode for restoration when Contrast Theme ends.

Selecting an Appearance Mode previews it immediately across every current DropCove WebView surface and native frame. Surfaces shown or created while Settings remains open use the same preview. Save atomically commits the complete Settings candidate and retains the preview as active. Cancel, native Settings close, or failed Save restores every surface and frame to the previously persisted mode. The Resident Instance remains the single appearance authority; individual windows only render its projection.

Refresh the semantic palette around Dark `#000000` application backgrounds and `#101010` Shelf Batch cards, Light `#F3F3F3` application backgrounds and `#F8F8F8` cards, and accessible cobalt `#2B63EB` primary actions. Preserve one semantic-token source, Windows system colors in Contrast Themes, visible focus, distinct warning and danger roles, and restrained surface-specific shadows.

Extend the strict settings schema with required `AppearanceMode: System | Dark | Light`. A pre-0.0.19 file without the field follows the existing malformed-settings path: preserve original bytes, load the complete active defaults, warn, and replace only after explicit successful Save. Do not add a migration, optional compatibility field, fallback parser, browser-local store, or second appearance file.

## User Stories

1. As a DropCove user, I want Settings divided into General and Appearance tabs, so that existing controls remain easy to find after Appearance is added.
2. As a DropCove user, I want every existing setting to remain under General, so that the new feature does not reorganize familiar controls unpredictably.
3. As a DropCove user, I want Settings to open on General, so that the established settings workflow remains the default entry point.
4. As a DropCove user, I want tab selection to remain transient, so that navigation state does not become another persistent preference.
5. As a DropCove user, I want one Save and Cancel lifecycle across both tabs, so that Settings behaves as one coherent transaction.
6. As a DropCove user, I want switching tabs to retain every unsaved draft change, so that navigation does not discard my work.
7. As a user opening Appearance, I want to choose System, Dark, or Light, so that DropCove can match either Windows or my explicit preference.
8. As a new user, I want System to be the default Appearance Mode, so that DropCove initially follows Windows as it does today.
9. As a System-mode user, I want DropCove to react when Windows changes between Light and Dark, so that the application remains synchronized without a restart.
10. As a Dark-mode user, I want DropCove to remain Dark when Windows changes to Light, so that my explicit preference is authoritative.
11. As a Light-mode user, I want DropCove to remain Light when Windows changes to Dark, so that my explicit preference is authoritative.
12. As a Windows Contrast Theme user, I want Windows system colors to override every DropCove Appearance Mode, so that application colors do not reduce accessibility.
13. As a Windows Contrast Theme user, I want my selected Appearance Mode retained while Contrast Theme is active, so that it returns when Contrast Theme ends.
14. As a Windows Contrast Theme user, I want Settings to explain that Windows currently overrides the selected appearance, so that the rendered result is not confusing.
15. As a user in Contrast Theme, I want the Appearance selector to remain operable, so that I can choose what DropCove will use after Contrast Theme ends.
16. As a DropCove user, I want Appearance Mode applied to the Drop Shelf, so that the primary workspace matches my choice.
17. As a DropCove user, I want Appearance Mode applied to the Rail Handle and expanded Edge Rail, so that docked presentation does not diverge from the Drop Shelf.
18. As a DropCove user, I want Appearance Mode applied to Manage Items, so that management surfaces remain coherent with their owner.
19. As a DropCove user, I want Appearance Mode applied to Settings and About, so that top-level utility windows match the rest of the application.
20. As a DropCove user, I want Appearance Mode applied to confirmations, menus, status messages, and drag feedback, so that transient surfaces do not expose the old palette.
21. As a Windows user, I want native title bars and frames to match the active DropCove appearance, so that WebView content and native chrome do not conflict.
22. As a user with hidden DropCove surfaces, I want those surfaces to use the current preview when they next appear, so that stale styling is never shown.
23. As a user opening a new auxiliary surface during preview, I want it to inherit the preview immediately, so that window creation cannot bypass the active draft.
24. As a user selecting an Appearance Mode, I want to see the result immediately, so that I can judge the appearance before saving.
25. As a user previewing Appearance, I want the focused control to stay focused, so that the preview does not remount the interface or interrupt keyboard use.
26. As a user previewing System mode, I want live Windows theme changes reflected during the preview, so that System behavior is accurately represented before Save.
27. As a user previewing Dark or Light, I want ordinary Windows theme changes ignored, so that the explicit preview remains stable.
28. As a user who chooses Save, I want the mode persisted with every other Settings change, so that one successful action commits the complete candidate.
29. As a user who chooses Cancel, I want every surface and native frame restored to the previously saved appearance, so that preview remains reversible.
30. As a user who closes Settings with the native close button, I want unsaved Appearance changes discarded, so that close remains equivalent to Cancel.
31. As a user whose Save fails, I want the previously persisted appearance restored everywhere, so that a failed transaction does not leave a partial visual state.
32. As a user whose Save rollback is incomplete, I want a warning describing the retained safe state, so that the application does not hide divergence.
33. As a user reopening Settings after Cancel, I want the persisted mode selected, so that discarded drafts do not return.
34. As a user restarting DropCove after Save, I want the persisted mode restored before a visible surface appears, so that startup does not flash the wrong appearance.
35. As a user restarting DropCove without Save, I want the previous persisted mode restored, so that preview cannot leak across process lifetime.
36. As a keyboard user, I want General and Appearance exposed as standard tabs, so that I can navigate them without pointer input.
37. As a keyboard user, I want standard arrow-key navigation within the Appearance radio group, so that choosing a mode follows familiar Windows interaction.
38. As a keyboard user, I want Tab and Shift+Tab to enter and leave the selector predictably, so that the group does not trap focus.
39. As a screen-reader user, I want each Appearance option exposed as a named radio with its selected state, so that visual preview tiles are not required to understand the control.
40. As a screen-reader user, I want the active Contrast Theme explanation associated with the selector, so that override behavior is announced in context.
41. As a low-vision user, I want a visible focus indicator distinct from resting and hover borders, so that I can locate keyboard focus in every mode.
42. As a DropCove user, I want the Settings window to remain `420 × 520` logical pixels, so that the feature does not change accepted placement or window geometry.
43. As a Dark-mode user, I want the application background to be `#000000`, so that DropCove uses the approved neutral black base.
44. As a Dark-mode user, I want Shelf Batch cards to be `#101010`, so that batches remain distinguishable without returning to a navy surface.
45. As a Light-mode user, I want the application background to be `#F3F3F3`, so that DropCove uses the approved neutral light base.
46. As a Light-mode user, I want Shelf Batch cards to be `#F8F8F8`, so that batches remain distinct from the application background.
47. As a DropCove user, I want primary actions to use accessible cobalt `#2B63EB`, so that the new identity remains clear and legible.
48. As a Dark-mode user, I want accent text and icons to use the lighter approved cobalt, so that emphasis remains readable on black surfaces.
49. As a Light-mode user, I want accent text and icons to use the darker approved cobalt, so that emphasis remains readable on light surfaces.
50. As a DropCove user, I want ordinary icons to inherit secondary text color, so that every icon does not compete with primary actions.
51. As a DropCove user, I want destructive actions to retain a distinct danger color, so that cobalt cannot obscure destructive meaning.
52. As a DropCove user, I want warnings to retain a distinct amber role, so that warning and selection states remain distinguishable.
53. As a Dark-mode user, I want Shelf Batch cards to avoid decorative shadows, so that hierarchy comes from surface and border contrast.
54. As a Light-mode user, I want Shelf Batch cards to use only slight separation shadow, so that the surface remains restrained.
55. As a DropCove user, I want larger shadows reserved for floating menus and dialogs, so that shadows communicate elevation rather than decorate every card.
56. As a user who prefers reduced motion, I want appearance changes to preserve the existing motion policy, so that theme preview does not add nonessential animation.
57. As a user dragging files and folders, I want Appearance changes to leave native drag-in and copy-only drag-out behavior unchanged, so that visual work cannot alter storage workflows.
58. As a user with retained Shelf Items, I want Appearance changes to leave Shelf state and item lifecycle unchanged, so that changing appearance cannot consume or remove references.
59. As a user managing source files, I want Appearance changes never to modify filesystem objects, so that the feature remains presentation-only.
60. As a user with a positioned Drop Shelf or Edge Rail, I want Appearance changes to preserve all window bounds and placement, so that preview cannot move my workspace.
61. As a user with configured hotkey, startup, Rail, fullscreen, and Shake preferences, I want normal Save/Cancel behavior preserved, so that Appearance composes with all existing Settings integrations.
62. As a user upgrading from 0.0.18, I want DropCove to preserve my old settings file bytes when the strict 0.0.19 schema rejects it, so that the application does not silently rewrite evidence.
63. As a user upgrading from 0.0.18, I want a warning that defaults were loaded, so that the preference reset is visible.
64. As a user upgrading from 0.0.18, I want all documented defaults loaded together, so that no partial old configuration is mistaken for a valid 0.0.19 candidate.
65. As a user upgrading from 0.0.18, I want `AppearanceMode` to default to System, so that the clean cutover begins from Windows-following behavior.
66. As a user with a rejected old settings file, I want explicit successful Save to write the new complete schema, so that persistence changes only after my action.
67. As a user with a malformed or unreadable settings file, I want the existing warning and preservation behavior retained, so that Appearance does not weaken the trust boundary.
68. As a maintainer, I want `AppearanceMode` represented by a closed System/Dark/Light contract, so that unknown values remain invalid rather than becoming silent fallbacks.
69. As a maintainer, I want one Resident-owned resolved appearance, so that windows cannot drift through local or browser-only state.
70. As a maintainer, I want one semantic token source for Dark, Light, and Contrast mappings, so that Vue surfaces cannot develop competing palettes.
71. As a maintainer, I want Appearance integrated into the existing Settings transaction and rollback reporting, so that a second transaction framework is unnecessary.
72. As a maintainer, I want the existing Windows theme adapter deepened rather than duplicated, so that HWND lifecycle and theme messages remain centralized.
73. As a maintainer, I want the installed Lucide icon dependency reused for the Appearance selector, so that no icon dependency or handwritten icon system is added.
74. As a maintainer, I want the approved exact palette roles represented semantically rather than repeated as component-local colors, so that later maintenance preserves hierarchy.
75. As a release reviewer, I want the exact installed 0.0.19 candidate to demonstrate every appearance mode from embedded assets, so that development-server behavior is not mistaken for release behavior.
76. As a release reviewer, I want installation, restart, Cooperative Shutdown, reinstall, and uninstall exercised with owned Profile Data, so that Appearance cannot regress the resident lifecycle.
77. As a release reviewer, I want ambient Windows theme, Contrast Theme, registry, shortcuts, processes, and user Profile Data restored after verification, so that acceptance testing does not alter the workstation.

## Implementation Decisions

- Keep the Resident Instance as the single owner of active application appearance. Do not let individual WebView windows, native windows, components, or browser storage own durable or independent theme state.
- Deepen the existing Settings Adapter and Windows appearance adapter. Do not create a parallel settings service, theme store, generalized window manager, or second native-frame observer.
- Add required persisted `AppearanceMode` values `System`, `Dark`, and `Light`. System is the default. Reject missing, unknown, or malformed values through the existing strict settings contract.
- Preserve the existing clean-cutover policy. A pre-0.0.19 file without `AppearanceMode` is malformed for the active version, remains untouched, loads the complete current defaults, warns, and is replaced only after explicit successful Save.
- Do not migrate or partially deserialize old fields. Do not use an optional field, deserialization default, compatibility alias, fallback parser, browser-local persistence, or separate appearance file.
- Extend the Settings snapshot exposed to the frontend with the current draft `AppearanceMode`. Add one Settings operation that changes the draft and publishes process-wide preview state through the resident coordinator.
- Keep one resident-owned committed Settings value and one existing draft candidate. Appearance preview projects the draft without turning it into the committed value.
- Retain the last committed appearance as the rollback authority before preview begins. Save failure must not snapshot the already-previewed candidate as the previous state.
- Save commits Appearance together with hotkey, startup, Rail, fullscreen, Shake, and the atomic settings file. The existing transaction remains the only commit boundary.
- Cancel, native Settings close, Settings teardown during Resident exit, and failed Save restore the committed appearance across every live surface and native frame.
- A complete rollback restores both the persisted bytes and rendered appearance. If rollback cannot fully restore a live integration, report the actual retained safe state through the existing warning path.
- Treat a surface destroyed during appearance fan-out as no longer live rather than as a failed application. A still-live owned surface that cannot apply the authoritative appearance must not be silently ignored.
- Resolve System mode from the live Windows application Light/Dark preference. Observe Windows theme changes and update every current WebView surface and native frame without restart.
- Explicit Dark and Light modes ignore ordinary Windows application-theme changes. They still yield to Windows Contrast Theme.
- Give Windows Contrast Theme the highest rendering precedence. Forced system colors govern WebViews and Windows governs native-frame appearance; entering or leaving Contrast Theme does not rewrite `AppearanceMode`.
- Establish the resolved appearance before any owned surface becomes visibly presented. Newly shown or created surfaces must never display a stale committed or system-derived frame before receiving preview state.
- Broadcast one resolved appearance projection from the Resident Instance to all WebViews. Frontends reflect that projection through one root-level appearance marker and the existing semantic-token system rather than recomputing preference ownership.
- Keep `color-scheme` aligned with the resolved Light or Dark appearance so browser-native controls, scrollbars, form controls, and UA rendering match the semantic palette.
- Preserve forced-color rules as a higher-priority mapping. Application appearance markers must not suppress or override Windows system colors.
- Add horizontal General and Appearance tabs below the Settings title. Keep the fixed `420 × 520` logical-pixel outer size, existing taskbar behavior, invocation placement, singleton reuse, and footer.
- General contains every setting introduced before 0.0.19. Appearance contains only the application-wide mode selector and active-Contrast explanation.
- Settings opens on General. Active tab is transient, not persisted, and does not affect the shared draft.
- Implement the tabs with standard tablist/tab/tabpanel semantics. Keyboard navigation follows standard horizontal tab behavior, and focus moves into only the active panel.
- Implement Appearance options as one radio group of compact horizontal preview tiles using the installed Lucide Monitor, Moon, and Sun icons. Arrow keys move selection; Tab and Shift+Tab enter and leave the group; selection remains exposed independently of visual previews.
- Keep the focused Appearance option focused when preview updates the application. Do not remount Settings or replace the control tree to apply a mode.
- While Contrast Theme is active, keep the selector enabled and associate the exact explanatory message: “Windows Contrast Theme currently overrides this appearance.”
- Apply resolved appearance to Drop Shelf, collapsed and expanded Edge Rail, Manage Items, Settings, About, native and WebView confirmations, status messages, utility menus, drag feedback, scrollbars, and every native frame.
- Preserve the existing Tailwind exposure layer and one `--dc-*` semantic-token source. Do not add a second token configuration or component-local palette copies.
- Add a dedicated Shelf Batch card semantic role rather than forcing batch-card colors onto every elevated control or menu. Continue mapping controls, menus, hover states, borders, text, feedback, and focus by semantic purpose.
- Use the following approved palette contract:

| Semantic role | Dark | Light |
|---|---:|---:|
| Application background | `#000000` | `#F3F3F3` |
| Shelf Batch card | `#101010` | `#F8F8F8` |
| Raised control or menu | `#181818` | `#FFFFFF` |
| Hover surface | `#232323` | `#EAEAEA` |
| Primary text | `#F5F5F5` | `#111111` |
| Secondary text | `#C7C7C7` | `#3F3F3F` |
| Muted text | `#929292` | `#686868` |
| Default border | `#383838` | `#D0D0D0` |
| Interactive border | `#5A5A5A` | `#858585` |
| Primary action | `#2B63EB` | `#2B63EB` |
| Focus and accent text | `#7AA2FF` | `#2B63EB` |

- Primary actions use `#2B63EB` with white text. Dark accent text and icons use `#7AA2FF`; Light accent text and icons use `#2B63EB`.
- Treat default and interactive borders as resting and hover resources, not as focus. Keep a dedicated focus indicator meeting the applicable non-text contrast requirement.
- Use no Shelf Batch card shadow in Dark. Use only a slight one-pixel separation shadow in Light. Reserve larger shadows for menus, dialogs, and floating surfaces.
- Keep ordinary icons on secondary text, selected and action icons on the applicable cobalt, destructive icons on danger, and warnings on amber. Warning and danger values remain distinct and contrast-checked rather than inheriting cobalt.
- Preserve reduced-motion policy. Appearance preview may change colors immediately but must not add page transitions, staggered motion, or decorative animation.
- Preserve all Shelf state, Profile Data, source-file safety, OLE drag behavior, virtualization, accessibility names and states, window geometry, monitor placement, fullscreen policy, topmost behavior, and Resident lifecycle.
- Add no runtime dependency, component framework, state library, visual-regression framework, screenshot framework, migration framework, telemetry, or generalized theming abstraction.
- Keep version metadata at 0.0.18 during behavior work. Synchronize package, crate, lock, Tauri, PE, installer, and About metadata to 0.0.19 only in the release slice after behavior acceptance.

## Testing Decisions

- The user-approved primary seam is the exact installed Windows Resident running embedded production assets through real WebView2 surfaces and native HWND frames with owned Profile Data. This is the highest seam that proves Settings, process-wide preview, CSS tokens, Windows messages, native-frame treatment, auxiliary-window creation, persistence, and rollback compose correctly.
- Use the existing deterministic Settings Store and Settings transaction/coordinator regression seams as support. They prove strict schema behavior, raw-byte preservation, defaults, draft ownership, atomic Save, failure rollback, and retained-state reporting without replacing the installed runtime seam.
- Add deterministic appearance-resolution coverage through the resident/coordinator boundary. Inputs are committed mode, optional draft preview, Windows Light/Dark state, and Contrast Theme state; outputs are the externally observable resolved appearance and events sent to owned surfaces. Do not test private helper call counts or duplicated enum forwarding.
- Extend the existing Settings integration abstraction with appearance snapshot/apply/restore behavior only as needed by the current transaction. Tests assert final committed and rendered state after success or failure, not trait method choreography.
- Good permanent tests catch consumer-visible failures: wrong mode, stale surface, incorrect frame, lost focus, broken keyboard semantics, overwritten old settings bytes, partial Save, failed rollback, missing warning, wrong exact contractual color, Contrast precedence failure, or changed Shelf workflow.
- Do not add permanent tests that assert source text, class strings, event names, internal maps, mock echoing, bare non-empty values, incidental DOM depth, or implementation-specific ordering that cannot affect the user.
- Prove defaults and serialization for `AppearanceMode`, including exact PascalCase field ownership, valid System/Dark/Light values, rejection of unknown values, and complete current-schema round trip.
- Add the 0.0.19 cutover regression using a real pre-feature settings file. Assert complete defaults, warning presence, byte-for-byte preservation before Save, and complete new-schema replacement only after successful explicit Save.
- Preserve existing missing, oversized, malformed, unreadable, invalid-hotkey, atomic-write, share-lock, and rollback regressions while extending expected settings state with Appearance Mode.
- Prove preview does not mutate persisted bytes or the committed mode.
- Prove Save commits the selected Appearance Mode together with other draft changes and restart restores it.
- Prove Cancel, native close, and Settings teardown restore every live appearance projection to the committed mode.
- Prove a Save failure after preview restores the committed appearance and original settings bytes. Cover both complete rollback and retained-safe-state warning behavior.
- Prove switching General and Appearance retains the shared draft and does not apply tab-local persistence.
- Prove Settings opens on General after close and reopen regardless of the previously active tab.
- Prove the Appearance selector exposes one radio group, correct names and selected states, arrow-key movement, Tab/Shift+Tab behavior, and retained focus through preview.
- Prove the active Contrast Theme explanation is visible and associated with the selector while controls remain operable.
- Exercise System mode while each relevant surface is open, then change Windows between Light and Dark. Observe updated computed semantic colors and matching native-frame treatment without restart or focus loss.
- Exercise explicit Dark and Light modes while changing ordinary Windows application theme. The selected explicit appearance must remain unchanged.
- Enter and leave an actual Windows Contrast Theme with System, Dark, and Light selected. Verify system colors and native-frame precedence while active and exact selected-mode restoration afterward. Media emulation may support development, but it is not the final Contrast acceptance seam.
- Exercise surfaces created or shown after preview begins: hidden Drop Shelf, collapsed and expanded Edge Rail, Manage Items, About, Settings reuse, utility menu, and both native and WebView confirmation surfaces.
- Exercise multiple simultaneously live surfaces and verify one preview and one rollback converge everywhere without per-window drift.
- Inspect contractual computed colors in representative roles for Dark and Light: application background, Shelf Batch card, raised control/menu, hover, text hierarchy, borders, primary action, focus, and accent icons. Exact approved values are behavior; unrelated antialiasing and derived feedback values are not.
- Verify primary action contrast with white text, Light accent contrast, Dark accent contrast, focus visibility, resting/hover border distinction, warning/danger distinction, and Windows system colors in Contrast Theme.
- Exercise Shelf Batch cards, item rows, empty states, status/error states, drag placeholders, selected/pinned states, disabled controls, scrollbars, menus, dialogs, and focus across both appearances.
- Verify Dark Shelf Batch cards have no decorative shadow, Light cards have only slight separation, and menus/dialogs retain stronger elevation without using shadow as the only boundary.
- Exercise reduced-motion preference during preview and mode changes. Final appearance, focus, state, and event ordering must remain identical without nonessential animation.
- Run regression workflows for intake, duplicate handling, Pin, Remove, Clear Temporary, Manage Items, Copy drag-out, Drop Shelf dismissal, Edge Rail expansion/collapse, fullscreen policy, hotkey, startup, Shake-to-open, Settings Save/Cancel, About, Cooperative Shutdown, and source-file safety in the refreshed palette.
- Run the complete native Windows regression executable directly, the existing frontend Node regression suite, frontend typecheck/production build, cargo-xwin Windows release/test compilation, and NSIS packaging.
- Install the exact packaged 0.0.19 candidate into an owned location/profile. Verify embedded assets with the development server unavailable, restart persistence, secondary activation, same-candidate reinstall, Cooperative Shutdown, uninstall, and Profile Data preservation.
- Capture and restore ambient Windows application theme, Contrast Theme, animation settings, startup registration, installer registration, shortcuts, processes, browser attachments, test files, and user Profile Data. Never use the ambient user profile for destructive schema-cutover tests.
- Final human acceptance must inspect the exact installed candidate in System, Dark, Light, and Windows Contrast Theme. Automated regressions and media emulation do not replace this gate.

## Out of Scope

- Custom accent-color selection.
- User-authored palettes or arbitrary color values.
- Independent Appearance Modes per surface, window, monitor, Shelf Batch, or profile sub-area.
- Scheduled Light/Dark switching outside Windows System mode.
- Wallpaper-, time-, application-, or monitor-dependent appearance automation.
- Settings migration or preservation of pre-0.0.19 preference values.
- Optional or compatibility `AppearanceMode` fields.
- A second settings file, browser-local storage, registry-backed appearance preference, or database column for Appearance Mode.
- Custom Windows Contrast palettes or suppression of forced colors.
- Replacing Windows Contrast Theme with an application-owned high-contrast mode.
- A component framework, state-management package, theme library, visual-regression package, screenshot framework, or new icon dependency.
- Changes to Settings outer dimensions, taskbar behavior, invocation placement, singleton behavior, or footer placement.
- Persisting the last selected Settings tab.
- Separate Save or Cancel actions per tab.
- Reorganizing existing General controls beyond placing them under the General tab.
- New settings unrelated to Appearance.
- Changing Shelf state, item lifecycle, persistence, source files, drag semantics, virtualization thresholds, or availability classification.
- Changing Drop Shelf, Edge Rail, Manage Items, About, or confirmation geometry.
- Changing Rail placement, fullscreen behavior, Shake-to-open behavior, startup behavior, hotkey semantics, or Resident lifetime.
- Adding nonessential theme-transition animation.
- Retrofitting or rewriting completed historical roadmap evidence for 0.0.7 or earlier releases.
- Synchronizing release metadata before the release slice.
- Creating a Git commit without separate explicit user authorization after implementation, verification, review, and final candidate acceptance.

## Further Notes

- ADR-0005 explicitly supersedes the prior decision that “Windows is the sole theme authority” for ordinary Light/Dark rendering. Windows Contrast Theme remains authoritative.
- **Appearance Mode** is the canonical domain term. Avoid treating “theme,” “color scheme,” or “skin” as separate persisted concepts.
- The clean settings cutover is intentionally destructive to existing preference values but not to source files, Shelf references, or the old settings-file bytes before Save. This consequence must remain prominent in tickets, release notes, smoke plans, and user acceptance.
- The existing 0.0.7 system-theme implementation is prior art for semantic tokens, forced colors, live Windows messages, and native-frame synchronization. This version deepens those seams rather than adding another visual subsystem.
- The existing Settings Store, Settings coordinator, and Settings transaction are prior art for strict schema replacement, shared draft state, warning delivery, native integration application, rollback, and retained-safe-state reporting.
- The approved testing seam is intentionally one highest integration seam with deterministic support, not a collection of new test frameworks.
- Specification readiness means this document is complete enough for ticket decomposition. It does not authorize implementation, release, or a Git commit.
- The next workflow step is `to-tickets`, followed by explicit user approval of the ticket set before implementation.
