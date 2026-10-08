# Drop Shelf Visual Alignment

Status: ready-for-agent

## Problem Statement

The Drop Shelf currently uses the standard base-window background in its content area and separate card-colored header and footer bands. In Dark mode, this makes it resemble an ordinary Windows application window instead of a distinct temporary drag-and-drop surface, and it does not share the same visual base as the Edge Rail.

The borderless native window also has no explicit rounded-corner preference. Its shell controls use square hover and pressed states, while the full-height header and footer bands make the chrome feel heavier than the compact utility workflow requires. Shrinking those rows or their interaction targets would weaken pointer and accessibility behavior and would disturb the established `180/236/292/348` logical-pixel sizing contract.

The Drop Shelf needs a quieter, more cohesive Windows 11 treatment: the same theme-aware base surface as the Edge Rail, one continuous shell instead of contrasting chrome bands, native rounded corners, a subtle boundary against windows behind it, and visually smaller shell icons without reducing existing interaction targets or changing Drop Shelf behavior.

## Solution

Restyle the existing Drop Shelf shell without changing its layout or lifecycle semantics. The Drop Shelf uses the same theme-aware layer background as the Edge Rail in Light, Dark, and High Contrast. Header, content, and footer sit on one continuous surface with no separate bands or separators, while Shelf Batch cards and Shelf Item surfaces retain their existing visual hierarchy.

Opt the native Drop Shelf window into the Windows 11 system rounded-corner treatment. Draw a non-layout-affecting, theme-aware neutral outline around the shell and rely on the native DWM shadow when Windows provides it. Do not add Mica, Acrylic, a custom shadow, per-pixel transparency, or a custom clipped window region.

Keep the header and footer at `32` logical pixels and preserve the existing Settings, dismissal, and Clear Temporary Items interaction rectangles. Reduce only their glyph sizes, and use the standard control corner radius for hover and pressed backgrounds. Preserve the current accent resize-hover outline, all resizing behavior, Automatic Shelf Growth tiers, responsive layout, accessibility semantics, and Edge Rail behavior.

## User Stories

1. As a Windows user, I want the Drop Shelf to look distinct from an ordinary Dark-mode application window, so that I can identify my temporary drag-and-drop workspace immediately.
2. As a DropCove user, I want the Drop Shelf and Edge Rail to share the same base surface language, so that they feel like two forms of one product.
3. As a Dark-mode user, I want the Drop Shelf to use the same theme-aware layer background as the Edge Rail, so that its appearance is cohesive without a hard-coded dark color.
4. As a Light-mode user, I want the same visual relationship preserved in Light mode, so that the redesign does not optimize only for Dark mode.
5. As a High Contrast user, I want the shell to continue resolving through Windows theme resources, so that the redesign does not replace system contrast behavior with fixed colors.
6. As a Windows user, I want the shell color to follow the active Windows theme automatically, so that it remains consistent when my theme changes.
7. As a Windows user, I want header, content, and footer to read as one continuous surface, so that the Drop Shelf feels lighter and less like a conventional management window.
8. As a Windows user, I do not want a contrasting header band, so that the Settings and dismissal controls remain available without visually dominating the shelf.
9. As a Windows user, I do not want a contrasting footer band, so that Clear Temporary Items remains available without adding a heavy bottom bar.
10. As a Windows user, I want Shelf Batch cards to remain visibly separate from the shell, so that content hierarchy stays clear after the base background changes.
11. As a Windows user, I want Shelf Item rows, status surfaces, and transient drag feedback to retain their established hierarchy, so that the shell redesign does not flatten all content into one layer.
12. As a Windows user, I want a subtle neutral outline around the Drop Shelf, so that its boundary remains visible over similarly colored windows and wallpapers.
13. As a Windows user, I want that outline to adapt to Light, Dark, and High Contrast, so that it never depends on a fixed RGB value.
14. As a Windows 11 user, I want the actual Drop Shelf window to have system-rounded corners, so that it follows the geometry of modern Windows applications.
15. As a Windows user, I want rounded corners to be supplied through native DWM behavior, so that the shelf keeps normal window composition, resizing, and shadow behavior.
16. As a Windows user, I want normal Windows policy exceptions for rounded corners to remain normal, so that DropCove does not fight the operating system when Windows intentionally suppresses rounding.
17. As a Windows user, I do not want a fake clipping fallback when native rounding is unavailable, so that resize regions, shadows, and window composition are not replaced by fragile custom geometry.
18. As a Windows user, I want DropCove to use the native window shadow when Windows provides it, so that the shelf separates naturally from content behind it.
19. As a performance-conscious user, I do not want a custom XAML shadow, Mica, or Acrylic added for this redesign, so that a visual polish change does not add unnecessary resident or rendering cost.
20. As a Windows user, I want the Settings glyph to be visually smaller, so that the header feels less heavy while the control remains easy to target.
21. As a Windows user, I want the dismissal glyph to be visually smaller, so that it aligns with the more compact Settings treatment.
22. As a Windows user, I want the Clear Temporary Items glyph to be visually smaller, so that the footer feels consistent with the header.
23. As a pointer user, I want the existing Settings interaction rectangle preserved, so that visual compactness does not make the action harder to select.
24. As a pointer user, I want the existing dismissal interaction rectangle preserved, so that hiding the Drop Shelf remains predictable.
25. As a pointer user, I want the existing Clear Temporary Items interaction rectangle preserved, so that the cleanup action does not become harder to select.
26. As a keyboard user, I want shell controls to retain their existing focusability and visible focus treatment, so that visual restyling does not change keyboard operation.
27. As an assistive-technology user, I want existing accessible names and action semantics retained, so that Settings, dismissal, and Clear Temporary Items remain understandable.
28. As a Windows user, I want shell-control hover and pressed backgrounds to use the standard control corner radius, so that they match Windows 11 control geometry.
29. As a Windows user, I want Settings, dismissal, and Clear Temporary Items to share the same rounded interaction language, so that dismissal does not look like an unrelated system caption button.
30. As a Windows user, I want tooltips and command meanings to remain unchanged, so that the visual update does not introduce a new interaction model.
31. As a Windows user, I want the current drag region preserved, so that I can continue positioning the Drop Shelf through the established header interaction.
32. As a Windows user, I want the shelf boundary to use a neutral outline while idle, so that the resize affordance does not appear active all the time.
33. As a pointer user, I want the existing accent outline to appear when I hover a resize edge or corner, so that borderless resizing remains discoverable.
34. As a pointer user, I want the accent resize outline to overlay the neutral boundary rather than change layout, so that hovering does not move content.
35. As a pointer user, I want resizing from every existing edge and corner to remain unchanged, so that rounded corners do not reduce window control.
36. As a Windows user, I want the `180 × 180` minimum size preserved, so that the visual redesign does not change the compact starting footprint.
37. As a Windows user, I want Automatic Shelf Growth to retain the `180`, `236`, `292`, and `348` logical-pixel tiers, so that visual polish does not alter established shelf capacity.
38. As a Windows user, I want the current header and footer heights preserved, so that every approved growth tier continues to fit its intended complete Shelf Batch rows.
39. As a Windows user, I want the responsive column boundaries and content padding preserved, so that adding a non-layout outline does not cause cards to reflow at different widths.
40. As a Windows user, I want multi-item popup placement and lifetime unchanged, so that shell styling does not disturb Shelf Batch management.
41. As a Windows user, I want drag-in, drag-out, Pin/Unpin, Remove, and Clear Temporary Items behavior unchanged, so that the redesign is purely presentational.
42. As a Windows user, I want opening Settings and dismissing the Drop Shelf to retain their current behavior, so that smaller glyphs do not imply different commands.
43. As an Edge Rail user, I want Edge Rail dimensions, hover timing, placement, controls, scrolling, and drag behavior unchanged, so that it remains only the visual reference for the Drop Shelf base surface.
44. As a returning user, I want preferred width, Manual Height Override, and Automatic Shelf Growth persistence unchanged, so that a visual update cannot alter saved geometry.
45. As a performance-conscious user, I want the redesign to add no timers, polling, animations, or background work, so that idle CPU remains unchanged.
46. As a performance-conscious user, I want the redesign to avoid new heavyweight composition resources, so that it does not worsen the unresolved native residency gate.
47. As a maintainer, I want one installed self-contained Release seam to be authoritative for rendered appearance and native-window behavior, so that tests prove the actual WinUI and DWM result.
48. As a maintainer, I want Light- and Dark-mode evidence from the same qualified build, so that theme claims are based on observed output rather than resource names alone.
49. As a maintainer, I want High Contrast evidence gathered only in an isolated or safely reversible environment, so that qualification cannot leave the workstation in a damaged theme state.
50. As a maintainer, I want actual normal-window rounding observed at the minimum size and after resizing, so that a successful native API call is not mistaken for visible success.
51. As a maintainer, I want UI Automation to confirm existing names, focusability, and interaction rectangles, so that the redesign does not silently reduce accessibility or target size.
52. As a maintainer, I want unavailable DPI, High Contrast, or DWM scenarios recorded as unobserved, so that the specification never claims evidence that was not exercised.
53. As a maintainer, I want the existing automated suite to remain the regression authority for sizing, persistence, lifecycle, and geometry behavior, so that visual work cannot weaken established contracts.
54. As a maintainer, I do not want tests that assert XAML source text, private element names, resource forwarding, or duplicated style constants, so that tests remain stable across maintainable implementation changes.
55. As a maintainer, I want the existing release performance gate to remain visible and unwaived, so that this styling feature cannot be used to hide the current residency blocker.

## Implementation Decisions

- The Drop Shelf shell cuts over from the base-window fill to the same theme-aware layer fill used by the Edge Rail. The implementation uses the existing WinUI `ThemeResource`; it does not introduce an app-local RGB/ARGB palette.
- Light, Dark, and High Contrast resolve through platform resources. There is no theme-specific hard-coded exception for Dark mode.
- Header and footer backgrounds become transparent over the root shell surface. No separator lines are added between header, content, and footer.
- Shelf Batch cards, Shelf Item rows, popup rows, status surfaces, drag overlay, and their existing card/control brushes remain unchanged. This specification changes the Drop Shelf shell, not the content design system.
- The shell receives a persistent one-logical-pixel neutral outline using the existing theme-aware card-stroke resource.
- The persistent outline is a non-layout-affecting overlay. It must not consume content width or height, change the `8`-pixel content padding, move controls, or alter responsive column boundaries.
- The shell outline and resize-hover outline follow the system overlay corner geometry. XAML corner treatment exists to render the shell fill and stroke coherently; it is not accepted as a substitute for native window rounding.
- The native Drop Shelf window opts into `DWMWCP_ROUND` through `DwmSetWindowAttribute` after its resizable borderless styles are applied. The narrow native interop belongs with existing window-style and DPI interop rather than in presentation or domain code.
- The DWM call applies only to the Drop Shelf native window. Edge Rail, Settings, confirmation dialogs, popup hosts, and other windows are not restyled by this specification.
- A failed or ignored DWM hint does not trigger per-pixel alpha, a custom window region, custom clipping, or a second fallback implementation. The app remains operational, but installed qualification cannot pass unless normal Drop Shelf windows visibly round on the supported Windows 11 environment.
- Windows-controlled exceptions to rounded corners remain authoritative. DropCove does not attempt to override operating-system policy for snapped, maximized, virtualized, remote, or otherwise policy-excluded window states.
- The redesign relies on the native DWM shadow when available. It adds no XAML `ThemeShadow`, Mica, Acrylic, custom shadow window, or transparent helper window.
- Header and footer remain `32` logical pixels high. Settings remains `32 × 32`, dismissal remains `36 × 32`, and Clear Temporary Items remains `28 × 28` logical pixels.
- The Settings glyph becomes `11` logical pixels, the dismissal glyph becomes `10`, and the Clear Temporary Items glyph becomes `11`. Glyph identity, command meaning, accessible name, tooltip, and tab behavior remain unchanged.
- Settings, dismissal, and Clear Temporary Items use the standard control corner radius for hover and pressed backgrounds while preserving their current layout and hit rectangles.
- The rounded shell-control treatment is scoped to those three shell controls. Shared styles used by Shelf Batch cards, popup actions, Pin/Unpin, Manage Items, and Remove actions must not change accidentally.
- The neutral outline remains visible in the ordinary state. The existing accent one-pixel resize affordance overlays it when a native resize edge or corner is hovered; the current whole-outline behavior is retained rather than adding per-edge drawing logic.
- Borderless native resizing, resize hit regions, cursor selection, preferred width, Manual Height Override, work-area clamping, and resize completion semantics remain unchanged.
- Automatic Shelf Growth retains the `180/236/292/348` logical-pixel tiers. The redesign must continue to fit the same complete row counts at those heights and must not change the no-live-shrink contract.
- No application state, database schema, settings record, domain model, view model contract, drag contract, or lifecycle transition changes.
- No new dependency, runtime package, UI mode, animation, timer, polling loop, or telemetry is introduced.
- User-facing product documentation is updated after implementation to describe the unified theme-aware shell, native rounded corners, neutral outline, preserved chrome geometry, and unchanged Edge Rail scope.

## Testing Decisions

- Good tests assert user-observable output and behavior: the rendered shell hierarchy, actual native-window corners, visible outlines and control states, unchanged interaction rectangles, successful resize interaction, and preserved accessibility. They do not assert private fields, XAML source text, resource-key forwarding, element names, or implementation-specific style structure.
- The single authoritative behavioral seam is the installed self-contained Release Drop Shelf running with an isolated test profile. This is the highest existing seam that can prove both WinUI rendering and native DWM behavior.
- The existing automated suite remains the regression authority for Drop Shelf state, responsive layout policy, Automatic Shelf Growth, preferred width, Manual Height Override, persistence, work-area geometry, popup lifecycle, drag lifecycle, and final-item reset. No new lower-level seam is introduced for unchanged behavior.
- A successful build or successful `DwmSetWindowAttribute` return is insufficient evidence for rounded corners. Installed smoke must visually observe rounded normal-window corners at `180 × 180` and at a larger resized geometry.
- Installed smoke verifies that all eight existing resize directions still work, the resize cursor remains correct, the accent resize outline appears over the neutral outline, and resize-hover does not move content.
- Installed smoke verifies that the persistent neutral outline is visible without becoming an accent border and that the native shadow is used when supplied by the environment.
- Light- and Dark-mode evidence uses the same qualified build and records the Drop Shelf with content at the minimum size and at a wider responsive size. Evidence shows one continuous shell with no contrasting header/footer bands and Shelf Batch cards remaining visually distinct.
- Light- and Dark-mode evidence also records Settings, dismissal, and Clear Temporary Items in normal and hover states so the reduced glyph sizes and rounded control-state backgrounds are observable.
- High Contrast evidence is gathered only in Windows Sandbox or another environment with proven restoration. The host theme must not be mutated when restoration is uncertain. If safe High Contrast execution is unavailable, the scenario is recorded as unobserved and no pass is claimed.
- UI Automation verifies that Settings, dismissal, and Clear Temporary Items retain their existing accessible names, remain enabled and focusable, expose visible focus, and retain their established logical interaction rectangles at the exercised DPI.
- Keyboard smoke verifies that the shell controls remain reachable and activatable and that their focus indicators remain legible over the new layer background.
- Pointer smoke verifies that the drag region, Settings, dismissal, Clear Temporary Items, card actions, and content drop target do not conflict with the rounded shell or outline overlays.
- Edge Rail smoke verifies that dismissal still produces the established Edge Rail geometry and behavior, reopening the Drop Shelf still works, and no Edge Rail control or hover timing changed.
- Automatic-growth smoke verifies representative `180`, `236`, `292`, and `348` heights after the visual change and confirms complete row fit, unchanged responsive columns, and no content shift from the persistent outline.
- No permanent unit test is added merely to prove that a presentation sets a brush, glyph size, corner radius, or native corner preference. Such tests would pin implementation or wiring rather than user-visible behavior.
- No screenshot-baseline framework is introduced. Screenshots are qualification evidence for human review, not pixel-perfect permanent tests tied to OS rendering revisions.
- Existing installed Release and UI Automation evidence for responsive layout, automatic growth, Bulk Pinning, focus, and named controls is prior art for the qualification procedure.
- Environmental limitations are explicit. Unavailable non-100% DPI, High Contrast, DWM policy state, or multi-monitor scenarios are recorded as unobserved rather than inferred from automated coverage.
- The existing canonical performance gate remains unchanged: `100` Shelf Batches and `1,000` Shelf Items, production show/dismiss lifecycle, idle CPU at or below `0.1%`, shelf-show p95 at or below `150 ms`, and visible/post-dismissal working set below `160 MB`. This feature does not waive the current residency blocker or claim to solve it.
- Performance measurement and UI Automation remain separate runs against the same build fingerprint so UI Automation peer realization cannot contaminate residency evidence.

## Out of Scope

- Redesigning, resizing, recoloring, rounding, or otherwise changing the Edge Rail.
- Changing Edge Rail collapsed or expanded geometry, hover delays, monitor placement, no-activate behavior, fullscreen policy, scrolling, flyouts, or drag behavior.
- Defining custom RGB/ARGB colors or creating a parallel DropCove color system.
- Adding Mica, Acrylic, custom XAML shadows, transparent shadow windows, per-pixel alpha, custom window regions, or custom clipping as a DWM fallback.
- Reducing header/footer height below `32` logical pixels or reducing the established interaction rectangles.
- Enlarging shell controls to the general `40 × 40` touch recommendation; this specification preserves current geometry rather than reopening the complete input-density design.
- Replacing Settings, dismissal, or Clear Temporary Items glyphs, commands, accessible names, tooltips, positions, or keyboard semantics.
- Restyling Shelf Batch cards, Shelf Item rows, popup content, Bulk Pinning controls, status surfaces, empty-state content, or drag-overlay content.
- Changing responsive card width, content padding, row/column gaps, scrolling, column boundaries, minimum shelf size, or Automatic Shelf Growth tiers.
- Changing preferred-width, Manual Height Override, persistence, work-area clamping, monitor selection, popup placement, drag/drop, pinning, removal, or final-item reset behavior.
- Adding motion, transition animation, new resize affordances, or per-edge accent drawing.
- Adding a screenshot-regression framework or source-text tests for XAML values.
- Solving or waiving the native residency release blocker as part of this visual change.
- Supporting operating systems below the existing Windows 11 minimum target.

## Further Notes

- `CONTEXT.md` already defines **Drop Shelf** and **Edge Rail**. No new domain term is introduced, and implementation language should continue to avoid “main window,” “drop zone,” “sidebar,” and “launcher.”
- The exact effective colors are supplied by WinUI theme resources at runtime. The repository does not own concrete RGB values for the selected layer, card, or stroke resources.
- Microsoft documents `DWMWCP_ROUND` as a DWM hint rather than a guarantee. Installed observation remains mandatory because customized borderless windows can fall outside automatic rounding heuristics.
- The current native window retains a sizing frame while removing its caption, which is compatible with the intended DWM opt-in path but does not replace runtime verification.
- The existing unresolved Release residency result remains approximately `165.61 MB` after the documented 30-cycle stress scenario against a `<150 MB` target. Avoiding new backdrop and shadow resources limits risk but does not constitute a performance fix.
- No ADR is required. The change is a reversible presentation contract, introduces no new architectural boundary, and uses existing platform resources and native interop conventions.
- All repository changes remain uncommitted until the user explicitly authorizes a commit.
