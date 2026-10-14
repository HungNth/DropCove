# DropCove 0.0.7 — Presentation and Accessibility Conformance

Status: ready-for-agent
Version: 0.0.7
Design confirmation: approved by user on 2026-10-14
Implementation approval: all eight published tickets approved for implementation by user on 2026-10-14
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.7
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 9, 10, 11, and 19
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md), section 14
Research: [Windows presentation and accessibility APIs](research/windows-presentation-apis.md)

## Problem Statement

DropCove 0.0.6 implements the complete resident Shelf workflow, but its presentation remains a functional placeholder. Shelf Items use text glyphs instead of Windows file/folder icons and image thumbnails. Large Shelf Batch collections and Manage Items projections mount every row and can request unbounded presentation work. Most surfaces repeat hard-coded Dark colors, motion cannot consistently follow the Windows accessibility preference, and existing accessible names and focus behavior do not yet form a verified end-to-end conformance contract.

Users therefore cannot rely on DropCove to match the active Windows Light, Dark, or Contrast Theme; large collections may consume resources in proportion to their total size; visual work may outlive the surface that requested it; and keyboard or screen-reader behavior has not been verified across every existing workflow. These shortcomings must be corrected without changing Shelf state, storage semantics, window geometry, drag behavior, settings schema, or the compact presentation established in earlier versions.

## Solution

Complete the existing presentation layer without redesigning it. Show asynchronous Windows Shell visuals for Shelf Items: image thumbnails for image files and icons for files or folders, with immediate generic fallbacks. Bound visual work to the current viewport and surface lifetime. Drop stale results, release native and browser resources when a surface hides, collapses, closes, or replaces its projection, and rely on the Windows Shell cache rather than creating an application-owned thumbnail cache.

Virtualize large Drop Shelf, expanded Edge Rail, and Manage Items collections while preserving the documented keyboard order, focus restoration, item position semantics, and the first-three-item preview bound. Replace component-local colors with shared semantic resources that follow Windows Light, Dark, and Contrast Themes. Add concise live announcements, complete accessible names and states, preserve full paths in tooltip and accessibility text, and keep all in-application actions keyboard operable.

Add only the documented short entrance, insertion, removal, Edge Rail, and recorder effects. Windows reduced-motion preference disables non-essential motion without changing state transitions or drag-and-drop behavior. Preserve the current layout, dimensions, wording, and external OLE drag model. Verify the complete slice in a real Windows 11 x64 build and installed 0.0.7 artifact.

## User Stories

1. As a user viewing a single-item Shelf Batch, I want to see the item’s Windows icon or image thumbnail, so that I can identify it without relying only on its name.
2. As a user viewing a multi-item Shelf Batch, I want a stacked preview of at most the first three Shelf Items, so that I can recognize the batch without unbounded visual work.
3. As a user opening Manage Items, I want each visible Shelf Item to show an appropriate visual, so that dense collections remain understandable.
4. As a user expanding the Edge Rail, I want its visible batches to use the same visual language as the Drop Shelf, so that the two surfaces remain consistent.
5. As a user viewing an image file, I want a thumbnail that preserves its aspect ratio, so that the preview is recognizable and not distorted.
6. As a user viewing a non-image file, I want its Windows Shell icon rather than an unrelated content preview, so that the visual matches the file type.
7. As a user viewing a folder, I want a folder icon rather than a folder-content thumbnail, so that folder references remain visually predictable.
8. As a user viewing an unavailable or permission-denied path, I want a safe fallback icon, so that presentation failure does not hide or remove the Shelf Item.
9. As a user opening a surface while visuals are loading, I want an immediate generic file or folder icon, so that the interface never waits on thumbnail extraction.
10. As a user, I want thumbnail extraction failures to remain silent and fall back to icons, so that nonessential presentation errors do not interrupt my workflow.
11. As a user, I want source files to remain untouched while visuals are extracted, so that presentation cannot modify my data.
12. As a user, I want the Drop Shelf to remain responsive while Windows Shell visuals load, so that drag, resize, pin, remove, and management actions are not blocked.
13. As a user who hides and reopens the Drop Shelf, I want obsolete visual work discarded, so that old results cannot appear in a new presentation.
14. As a user who collapses the Edge Rail, I want expanded-preview resources released, so that a collapsed handle does not retain unused images.
15. As a user who closes Manage Items, I want its full-item projection and visuals cleared, so that a closed popup does not retain collection-sized resources.
16. As a user rapidly opening different batches in Manage Items, I want only the current batch’s results displayed, so that late work cannot leak into the replacement popup.
17. As a user with many Shelf Batches, I want only visible and nearby cards rendered, so that collection cost remains bounded.
18. As a user with a very large batch, I want only visible and nearby Manage Items rows rendered, so that thousands of references remain manageable.
19. As a keyboard user in a virtualized list, I want Tab and Shift+Tab to continue through the documented action order, so that virtualization does not trap or lose focus.
20. As a keyboard user, I want the focused virtual row retained until focus moves, so that scrolling cannot unexpectedly return focus to the document body.
21. As a screen-reader user, I want each virtualized item to expose its total set size and position, so that I retain orientation within a large collection.
22. As a keyboard user opening Manage Items, I want focus to start at Bulk Pinning, so that the established workflow remains unchanged.
23. As a keyboard user closing Manage Items, I want focus restored to the originating Manage Items control when it still exists, so that I can continue from the same Shelf Batch.
24. As a keyboard user, I want every in-application action available without pointer input, so that I can manage references, settings, and presentation states from the keyboard.
25. As a user dragging references into another application, I want external OLE drag behavior to remain pointer-driven, so that 0.0.7 does not replace drag-out with a different copy workflow.
26. As a user using Windows Light theme, I want every DropCove surface to remain legible and visually coherent, so that I do not receive a forced Dark interface.
27. As a user using Windows Dark theme, I want every DropCove surface and native frame to follow that choice, so that the application fits the desktop environment.
28. As a user switching the Windows application theme while DropCove is open, I want visible surfaces to update without restarting, so that theme changes apply immediately.
29. As a user using a Windows Contrast Theme, I want system colors, borders, controls, focus indicators, and text to remain distinguishable, so that arbitrary application colors do not defeat my contrast settings.
30. As a user, I do not want a separate DropCove theme preference, so that Windows remains the single theme authority.
31. As a keyboard user, I want a clearly visible focus indicator on every actionable control, so that I can see where input will go.
32. As a screen-reader user, I want each action control to expose a concise accessible name and current state, so that iconography and visual styling are not required to operate DropCove.
33. As a screen-reader user, I want Bulk Pinning to expose Off, On, and Indeterminate states, so that mixed batches are represented accurately.
34. As a user encountering a shortened path, I want its full path available through a tooltip and accessibility text, so that truncation does not remove information.
35. As a screen-reader user, I want accepted batches, pin changes, removals, Clear Temporary results, and hotkey recorder guidance announced politely, so that I receive important state feedback without interruption.
36. As a screen-reader user encountering an actionable error, I want it announced assertively, so that failures are not missed.
37. As a user submitting an exact duplicate Shelf Batch, I want the existing silent no-op preserved, so that accessibility work does not add feedback to a deliberately silent path.
38. As a screen-reader user, I do not want thumbnail loading or fallback events announced, so that decorative resource changes do not create noise.
39. As a privacy-conscious user, I do not want full filesystem paths automatically spoken in general status announcements, so that path details are read only when I inspect the relevant item.
40. As a user, I want short entrance and collection-change effects, so that state changes remain easy to follow without delaying interaction.
41. As a user expanding or collapsing the Edge Rail, I want a short transition, so that the relationship between the handle and expanded presentation remains clear.
42. As a user recording a hotkey, I want the existing pulse to indicate listening state, so that the recorder remains understandable.
43. As a user who disables Windows Animation Effects, I want all nonessential motion removed, so that DropCove respects my accessibility preference.
44. As a user with motion disabled, I want drag-and-drop, insertion, removal, Shelf opening, and Edge Rail state changes to retain identical outcomes, so that reduced motion changes presentation only.
45. As a user, I want the compact layout, dimensions, action wording, and window placement from 0.0.6 preserved, so that presentation conformance does not require relearning the application.
46. As a user, I want Shell Visual loading to remain bounded when a collection contains hundreds or thousands of references, so that the resident process remains stable.
47. As a user, I want all workflows to remain usable when thumbnail extraction is disabled or unavailable, so that Shell Visuals are an enhancement rather than a dependency.
48. As a Windows 11 x64 user, I want the installed 0.0.7 artifact to exhibit the same visual, accessibility, keyboard, resource-lifetime, and motion behavior as the verified build, so that packaging does not change the product.

## Implementation Decisions

- Preserve existing surface composition, window geometry, compact density, action wording, Core Shelf State ownership, persistence, drag behavior, and settings semantics. This version deepens the presentation modules rather than creating another state owner or workflow.
- Introduce one deep Shell Visual module. Its interface accepts a surface generation, Shelf Item identity and path, visual role, and logical target size; it returns a successful binary visual or a classified fallback outcome. Windows Shell extraction, COM setup, queueing, cancellation, native handle ownership, bitmap conversion, and failure classification remain hidden in its implementation.
- Use Windows Shell as the visual authority. Image files recognized as images receive image-thumbnail requests. Non-image files and folders receive Shell icons. Video, PDF, document, and folder-content thumbnails are not requested.
- Use `32 × 32` logical-pixel visual boxes for single-item cards and Manage Items rows. Use `24 × 24` logical-pixel boxes for each entry in a stacked multi-item preview. Scale native requests for the target monitor DPI, preserve aspect ratio, and never crop.
- Display a generic file or folder icon immediately. Replace it only when a matching, current-generation Shell result succeeds. Unavailable paths, denied access, unsupported formats, queue saturation, extraction failure, or worker failure retain the fallback without a user-facing status message.
- Run Shell extraction away from presentation and persistence work on two dedicated COM STA workers. Use a bounded queue of 128 requests and prioritize work for the visible viewport before overscan work. Queue saturation degrades to fallback visuals rather than blocking the caller.
- Native bitmap and icon handles use deterministic ownership guards. Every success, failure, stale result, and cancellation path releases the owned native resource exactly once.
- Transfer encoded visual bytes through binary IPC rather than base64 text. The frontend creates a temporary Blob URL and revokes it when the result is replaced, the item leaves the active projection, or the owning surface ends.
- Do not create an application-level thumbnail LRU or persist visual bytes. Deduplicate identical active requests within one surface generation and otherwise rely on the Windows Shell cache.
- Each Drop Shelf visibility cycle, expanded Edge Rail cycle, and Manage Items ownership cycle has a distinct surface generation. Hiding the Shelf, collapsing the Rail, closing or replacing Manage Items, removing the owning batch/item, or disposing the surface cancels queued work and invalidates late results.
- Compact Shelf Batch projections contain at most the first three Shelf Item visuals. Offscreen virtualized cards do not request visuals until they enter the viewport or overscan range.
- Continue to create the full metadata projection only while Manage Items is open, but mount DOM rows and request Shell Visuals only for the viewport and overscan range. Closing the surface clears the full projection and all associated visual ownership.
- Use virtualized presentation modules for the Drop Shelf batch grid, expanded Edge Rail batch list, and Manage Items list. Collections of 50 or fewer entries render fully; larger collections render the visible range plus a small overscan range.
- Virtualization preserves the focused item in the DOM until focus moves. Tab and Shift+Tab retain the canonical control order and scroll/mount the next logical target before focusing it. Do not introduce a second arrow-key navigation model.
- Virtualized collection containers and rows expose list semantics, total collection size, and one-based item position. The visual order, keyboard order, and accessible order remain identical.
- Replace component-local hard-coded presentation colors with shared semantic design tokens. Tokens cover canvas, surface, elevated surface, primary and muted text, borders, accent, focus, success, warning, error, disabled state, and drag/resize emphasis.
- Windows is the sole theme authority. Web content follows system Light/Dark media queries and Contrast Themes through forced-colors. Native frames follow the current Windows application theme. No theme field is added to Settings.
- Contrast Theme styles use Windows system colors and preserve explicit visible borders and focus indicators. Do not maintain a custom High Contrast palette or disable forced-color adjustment for decorative fidelity.
- Complete accessible names and states for every action. Bulk Pinning exposes true, false, or mixed checked state. Manage Items triggers and expandable controls expose disclosure state where applicable. Truncated paths expose the full path through tooltip and accessibility text.
- Add one polite, atomic live-announcement channel per active presentation context. Announce accepted Shelf Batch intake, pin changes, removals, Clear Temporary results, and hotkey recorder guidance. Existing actionable errors remain assertive.
- Exact duplicate intake remains silent. Shell Visual loading, fallback, cancellation, and release remain silent. General announcements report item/batch identity or counts without automatically speaking full filesystem paths.
- Keyboard conformance covers all actions inside DropCove. External OLE drag-out remains a pointer or assistive-pointer workflow; no file destination picker, clipboard path workflow, automatic copy command, or extra global keyboard hook is added.
- Add only short opacity/scale effects for Shelf entrance, item or batch insertion/removal, Edge Rail expansion/collapse, and the existing hotkey recorder listening pulse. Durations remain between 120 and 160 milliseconds.
- Reduced-motion media policy suppresses all nonessential transitions and animations while preserving final state, event ordering, focus movement, drag behavior, and resource lifecycle.
- No persistent schema or Settings fields change. No backward-compatibility shim, visual migration, or stored cache format is introduced.
- No ADR is required. The design follows the existing Windows-only architecture, Core Shelf State ownership, surface-lifetime resource rule, and system-authoritative accessibility preferences; it does not introduce a surprising or hard-to-reverse product ownership decision.

## Testing Decisions

- The primary seam is the running Resident Instance with its real presentation surfaces. Tests exercise behavior through the same commands, events, window lifecycle, Core Shelf State, and settings interactions used by the installed application. This highest seam proves that visuals, virtualization, focus, announcements, theme, motion, and existing workflows compose correctly.
- The Shell Visual module receives focused contract tests behind its single interface. A deterministic extractor adapter controls success, fallback, delay, stale completion, cancellation, worker failure, and queue saturation. Tests assert returned behavior and resource accounting, not worker implementation or private message shapes.
- Native Windows adapter checks use temporary image, non-image, folder, unavailable, and permission-denied paths. They verify actual Shell thumbnail/icon fallback and deterministic native-handle release without asserting incidental pixel bytes from a particular Windows build.
- Surface-lifecycle tests delay extraction, then hide, collapse, close, replace, remove, or reopen the owner. Late results must never enter the current projection, and every owned native/browser resource must be released.
- Virtualization tests use the actual rendered surfaces. They verify bounded DOM and visual-request counts, correct visible content, first-three preview limits, scroll replacement, `aria-setsize`/`aria-posinset`, retained focus, canonical Tab/Shift+Tab order, Escape behavior, and focus restoration.
- Accessibility tests observe UI Automation behavior rather than source attributes alone. Windows Narrator and UI Automation inspection are the release gate. NVDA/JAWS compatibility is supported through standard semantics but is not a separate certification target.
- Theme tests open each surface in Light, Dark, and a Windows Contrast Theme, then change the active theme while surfaces remain open. Verify legibility, focus visibility, system-color use, native-frame alignment, and unchanged behavior.
- Motion tests exercise every animated transition with Windows Animation Effects enabled and disabled. Reduced motion must remove nonessential effects while producing the same final state and command outcome.
- Announcement tests observe polite and assertive UI Automation live-region events for accepted intake, pinning, removal, Clear Temporary, recorder guidance, and errors. Exact duplicates and Shell Visual lifecycle events must produce no announcement.
- Large-collection verification uses 200 Shelf Batches for Drop Shelf and expanded Edge Rail behavior and one 2,000-item Shelf Batch for Manage Items. Only viewport and overscan entries may own mounted rows or active visual requests.
- Repeated open/close/reopen and scroll cycles observe process private memory, GDI resources, active request counts, and Blob URL ownership for bounded behavior rather than asserting a fragile exact allocation count.
- Regression verification exercises every 0.0.1–0.0.6 workflow with thumbnail extraction unavailable: intake, duplicate handling, persistence, pinning, remove/clear, drag-out, Manage Items, resize, hotkey, tray, startup, Edge Rail, fullscreen policy, Settings, and Shake-to-open.
- Existing native unit and coordinator tests remain prior art for deterministic state, transaction, and lifecycle behavior. Existing installed-artifact smoke is prior art for real HWND, OLE, UI, registry, profile, and cleanup verification.
- Release verification builds once, pins installer and executable hashes, runs the complete native test binary, installs the exact packaged bytes into an owned location/profile, exercises the Windows behavior above, exits and uninstalls cleanly, and restores ambient registration, shortcuts, processes, theme/motion settings, and test fixtures exactly.
- Good permanent tests catch consumer-visible regressions: wrong visual class, stale projection, unbounded mounted/request counts, lost focus, incorrect accessible state, missing/extra announcement, unreadable theme, motion-policy violation, or changed workflow outcome. They do not pin implementation text, private queue layout, incidental CSS values outside the approved duration/size contracts, or mock forwarding.

## Out of Scope

- Redesigning surface layout, changing window dimensions, changing action wording, adding a design-system library, or introducing new product workflows.
- Video, PDF, document, archive, executable, or folder-content thumbnails.
- Image editing, generated previews, custom thumbnail codecs, preview quality settings, adaptive quality, telemetry, or background prewarming.
- An application-owned persistent visual cache or resident-session LRU cache.
- A DropCove Light/Dark/Contrast preference or custom Contrast Theme palette.
- A new keyboard-driven external file copy workflow, destination picker, clipboard history, path copy command, or second global keyboard hook.
- Formal WCAG certification or guaranteed behavior for every third-party screen reader. Standard semantics remain required.
- Changes to Shelf persistence schema, Settings schema, intake identity, availability persistence, drag result rules, display states, monitor placement, or Edge Rail docking choices.
- Recovery/database repair, installer lifecycle hardening, signing, updater support, or other Version 0.0.8 work.
- macOS or Linux support.

## Further Notes

- The approved testing seam and release scale were confirmed during the design grill: Windows Narrator/UI Automation, keyboard-only operation, live theme and motion changes, 200 Shelf Batches, 2,000 Manage Items rows, and regression operation with thumbnail extraction unavailable.
- Shell Visual is presentation-only. It never becomes Core Shelf State, persisted Shelf data, source-file content, or a prerequisite for drag-and-drop.
- The Windows Shell cache remains an external platform facility; DropCove owns only resources attached to its current presentation surfaces.
- Version metadata remains 0.0.6 until the 0.0.7 implementation and release gates pass together. The expected final installer is `DropCove-Setup-0.0.7.exe`.
- Ticket decomposition and implementation remain separate workflow steps. This specification being `ready-for-agent` does not authorize a Git commit or start Version 0.0.8.
