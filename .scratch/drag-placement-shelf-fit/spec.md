# DropCove 0.0.16 — Drag Placement Feedback and Drop Shelf Fit

Status: resolved
Version: 0.0.16
Design confirmation: approved by user on 2026-10-20 through `grill-with-docs`
Testing seam: actual Windows Resident Instance with the real WebView2 Drop Shelf and Edge Rail, supported by the existing Shelf State sizing, native storage-drop eligibility, and Edge Rail timing regressions; approved by user on 2026-10-20
Delivery prerequisite: completed — the roadmap records 0.0.15 as accepted and completed
Ticket decomposition: all four approved slices are resolved; final user acceptance and commit of 0.0.16 were reported on 2026-10-20
Implementation approval: all four tickets authorized by the user on 2026-10-20; final acceptance and commit are complete by user report
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.16
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 4, 8, 11, 12, and 19
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md), especially Sizing State and the existing presentation/storage-drop ownership
Placement ownership: [ADR-0002](../../docs/adr/0002-edge-rail-placement-ownership.md)

## Problem Statement

When a user drags supported files or folders over the Drop Shelf, the complete surface currently changes background and border treatment. The native Copy cursor communicates that a drop is possible, but the full-surface glow does not show where the resulting Shelf Batch will appear. The Edge Rail provides even less placement feedback: it retains the native cursor and delayed expansion, but the expanded collection does not mark the prospective insertion position.

The user needs both surfaces to show one precise Drop Placeholder at the newest-first Shelf Batch position. The feedback must represent one incoming Shelf Batch rather than one placeholder per file, because one accepted drop operation creates at most one Shelf Batch. It must remain truthful before acceptance: unsupported entries may be filtered and duplicate-only input may create no batch, so the placeholder must not claim a filename, thumbnail, or item count that may not survive intake.

The Drop Shelf also grows too early and by more than the current compact card geometry requires. At minimum width, two 48-pixel responsive rows already fit inside the 180-pixel window, yet the current sizing contract expands to 236 pixels. Later tiers retain the same historical excess, leaving a large unused area below two or three Shelf Batches. Automatic sizing is also monotonic during a visible session: removing content or widening the Shelf can leave stale empty space until reopen or restart.

The user needs Automatic Shelf Sizing to be exact and bidirectional whenever no Manual Height Override exists. Content addition and narrower responsive layouts may grow the Drop Shelf; content removal and wider responsive layouts may shrink it. A user-chosen height must remain authoritative while Shelf content exists, and removing the final Shelf Item must retain the established reset to `191 × 180`.

## Solution

Add one transient `48`-logical-pixel Drop Placeholder to the Drop Shelf and expanded Edge Rail whenever the existing native storage-drop adapter reports an eligible external drag over that surface. The placeholder occupies the prospective newest-first Shelf Batch position, participates in the collection layout so retained cards move to the positions they would have after acceptance, uses a dashed drag-hover border and a background distinct from the containing surface, and exposes no pre-acceptance filename, thumbnail, or item count.

Remove only the Drop Shelf's full-surface drag glow. Preserve native Copy cursor feedback, the existing drag-over behavioral state, the guard that prevents Drop Shelf movement during storage drag, and the independent resize-hover accent outline. Keep the Drop Shelf window size stable during hover. Keep the Edge Rail's native `200 ms` drag-entry expansion and `300 ms` leave-collapse behavior; show the placeholder after the Rail expands without adding another timer or changing Rail height before acceptance.

Replace the historical monotonic automatic-height behavior with Automatic Shelf Sizing. When no Manual Height Override exists, derive the exact current height from preferred width and responsive Shelf Batch rows: `180` pixels for zero through two rows, `230` for three rows, `282` for four rows, and `334` for five or more rows, with vertical scrolling from the sixth row. Recalculate through the existing Shelf State interface after accepted content, width-only resize, and every mutation that can reduce responsive rows. Keep a Manual Height Override fixed while content remains, preserve the top-left origin during automatic height adjustment, and reset final-empty sizing to `191 × 180`.

The primary acceptance seam is the running Windows Resident Instance with the real WebView2 Drop Shelf and Edge Rail. Existing Shelf State regression coverage proves the sizing invariant through authoritative operations; existing native storage-drop and Rail timing coverage supports the actual-surface proof. No new testing framework or lower duplicate seam is introduced.

## User Stories

1. As a Drop Shelf user dragging supported files, I want to see where the incoming Shelf Batch will appear, so that the drop result is spatially predictable.
2. As an Edge Rail user dragging supported files, I want the expanded Rail to mark the prospective Shelf Batch position, so that the Rail provides the same placement clarity as the Drop Shelf.
3. As a user dragging several files together, I want one Drop Placeholder, so that the preview matches the one-Shelf-Batch-per-drop model.
4. As a user dragging one file, I want the same one-placeholder contract, so that single-item and multi-item drops behave consistently.
5. As a user, I want the Drop Placeholder at the newest-first position, so that it matches the ordering of an accepted Shelf Batch.
6. As a user with retained Shelf Batches, I want the placeholder to precede the newest retained batch and shift existing cards through the normal layout, so that the preview shows the actual insertion position.
7. As a user with an empty Drop Shelf, I want the placeholder to replace the ordinary `Drop here` prompt during a valid drag, so that the empty surface shows a concrete destination.
8. As a user with a populated Drop Shelf, I want the placeholder to use the same 48-pixel card height as retained Shelf Batches, so that the preview fits the established compact geometry.
9. As a user of the expanded Edge Rail, I want the placeholder to use the same 48-pixel row height as retained Rail cards, so that the preview does not change row density.
10. As a user, I want a dashed border around the placeholder, so that it is visibly provisional rather than mistaken for accepted Shelf content.
11. As a user, I want the placeholder background to differ from the containing surface, so that its insertion position is legible without a full-surface effect.
12. As a user, I do not want the placeholder to display a filename before acceptance, so that filtered or unsupported input is not misrepresented.
13. As a user, I do not want the placeholder to display a thumbnail before acceptance, so that a transient preview does not imply accepted Shelf state.
14. As a user, I do not want the placeholder to display an item count before acceptance, so that mixed supported and unsupported payloads are not described incorrectly.
15. As a user dragging unsupported data, I do not want a Drop Placeholder, so that rejection remains clear.
16. As a user dragging from DropCove itself, I do not want an intake placeholder, so that application-owned Copy drag-out cannot be mistaken for external drop-in.
17. As a user dragging a valid duplicate-only payload, I want existing duplicate behavior preserved after release, so that no batch, resize, reorder, or persistence change occurs even though pre-drop eligibility could show a placeholder.
18. As a user leaving the Drop Shelf before release, I want the placeholder removed immediately, so that stale drag feedback is not left behind.
19. As a user leaving the Edge Rail before release, I want the placeholder removed and the existing Rail pointer reconciliation retained, so that collapse behavior remains predictable.
20. As a user completing a drop, I want the transient placeholder removed before the authoritative result is rendered, so that provisional and accepted content are never shown as two batches.
21. As a user whose drop is accepted, I want the resulting Shelf Batch to appear in the placeholder's newest-first position, so that the preview and result agree.
22. As a user whose drop is rejected, unsupported, or duplicate-only, I want the Shelf collection to remain unchanged, so that visual feedback does not mutate data by itself.
23. As a Drop Shelf user, I do not want the complete surface to glow during drag entry, so that the new placement marker is the only drag emphasis.
24. As a Drop Shelf user, I want the native Copy cursor retained, so that Windows continues to communicate drop eligibility.
25. As a user resizing the Drop Shelf, I want the existing resize-hover accent outline retained, so that removing drag glow does not weaken resize discoverability.
26. As a user moving the Drop Shelf, I want movement to remain blocked while a valid external storage drag is over it, so that one drag cannot become a window-move gesture.
27. As a user hovering a valid drag over the Drop Shelf, I do not want the window to resize before acceptance, so that canceling or leaving cannot make the window jump.
28. As an Edge Rail user, I want valid drag entry to retain the established `200 ms` expansion delay, so that incidental travel across the edge does not expand the Rail immediately.
29. As an Edge Rail user, I want the placeholder to appear after the existing native expansion completes, so that there is no second expansion state model.
30. As an Edge Rail user, I do not want the Rail height to preview the possible new batch, so that canceling or duplicate-only drops do not change native window geometry.
31. As an Edge Rail user, I want accepted content to update Rail height through the existing retained-batch formula, so that placeholder behavior does not create a second sizing contract.
32. As a user working in another application, I want Edge Rail drag entry and placeholder display to remain no-activate, so that DropCove does not steal focus.
33. As a user with zero, one, or two responsive Drop Shelf rows, I want the automatic height to remain `180` pixels, so that content that already fits does not create empty space.
34. As a user with three responsive rows, I want the automatic height to be `230` pixels, so that the window fits the cards and established chrome without the historical excess.
35. As a user with four responsive rows, I want the automatic height to be `282` pixels, so that every added row contributes the established 48-pixel card plus 4-pixel gap.
36. As a user with five responsive rows, I want the automatic height to be `334` pixels, so that five rows fit without an oversized window.
37. As a user with six or more responsive rows, I want the Drop Shelf to stay at `334` pixels and scroll vertically, so that automatic growth remains bounded.
38. As a user accepting a Shelf Batch that increases responsive row count, I want the Drop Shelf to grow to the exact applicable tier, so that the new batch remains reachable.
39. As a user widening an automatically sized Drop Shelf, I want it to shrink when the wider grid uses fewer responsive rows, so that a width-only resize does not retain stale vertical space.
40. As a user narrowing an automatically sized Drop Shelf, I want it to grow when the narrower grid uses more responsive rows, so that width-only resizing does not create premature overflow.
41. As a user removing a Shelf Batch, I want an automatically sized Drop Shelf to shrink immediately when responsive row count decreases, so that the surface continues to fit retained content.
42. As a user removing the last Shelf Item from a multi-item batch, I want implicit batch pruning to trigger the same automatic shrink, so that removal method does not change sizing behavior.
43. As a user removing an item while its batch remains non-empty, I want height to change only if the resulting authoritative Shelf layout uses fewer responsive rows, so that sizing follows batches rather than raw item count.
44. As a user confirming Clear Temporary Items, I want automatic height recalculated from surviving batches, so that pruned temporary-only batches do not leave empty space.
45. As a user completing a Successful Drag-Out of Temporary Items, I want automatic height recalculated from retained batches, so that consumed references do not leave stale height.
46. As a user triggering confirmed-Missing cleanup during drag preparation, I want automatic height recalculated if batches are pruned, so that cleanup and explicit removal follow one sizing invariant.
47. As a user canceling or receiving a rejected drag-out, I want sizing unchanged when no Shelf content was consumed, so that failed transfers do not alter presentation.
48. As a user pinning or unpinning retained items without changing batch layout, I want the automatic height to remain at the same exact tier, so that unrelated lifecycle changes do not move the window.
49. As a user with a Manual Height Override, I want content removal to preserve my chosen height while any Shelf content remains, so that automatic sizing does not override explicit vertical intent.
50. As a user with a Manual Height Override, I want a later width-only resize to preserve the chosen height, so that horizontal adjustment does not clear manual vertical intent.
51. As a user performing only a width resize without a Manual Height Override, I want Automatic Shelf Sizing to remain active, so that width and responsive rows continue to determine height.
52. As a user removing the final Shelf Item, I want the next opening reset to `191 × 180`, so that an empty Drop Shelf does not retain an oversized automatic or manual layout.
53. As a user watching an automatic shrink, I want the top-left Drop Shelf origin to remain fixed, so that the window does not move across the desktop while fitting its content.
54. As a user near a monitor work-area edge, I want existing work-area clamping retained after automatic sizing, so that controls remain reachable.
55. As a user reopening or restarting DropCove, I want automatic height recomputed from current width and retained batches, so that the visible surface starts at the exact applicable tier.
56. As a user with a multi-column Drop Shelf, I want the same row-based sizing rules as a one-column Shelf, so that width changes affect height only through responsive row count.
57. As a user with many virtualized Shelf Batches, I want the Drop Placeholder to shift visible batch positions without overlaps or unreachable content, so that large collections remain usable during drag hover.
58. As a screen-reader user, I do not want a provisional placeholder announced as an accepted Shelf Batch, so that collection semantics continue to describe authoritative content.
59. As a Dark Theme user, I want the dashed border and distinct placeholder background to remain legible, so that placement feedback works in the default presentation.
60. As a Light Theme user, I want the same placement distinction, so that the feedback does not depend on a dark background.
61. As a Windows Contrast Theme user, I want the placeholder to use Windows system colors, so that forced-color presentation remains perceivable.
62. As a user with reduced motion, I want drag intake and sizing to remain functional without new decorative animation, so that the feature does not depend on motion.
63. As a user, I want every placeholder and sizing path to preserve source files and folders, so that presentation changes cannot imply Move or deletion behavior.
64. As a maintainer, I want the existing native storage-drop eligibility interface reused, so that visual feedback does not create a second drop parser or acceptance model.
65. As a maintainer, I want Automatic Shelf Sizing derived in the authoritative Shelf State module, so that every caller receives one exact sizing invariant.
66. As a maintainer, I want the existing nullable Manual Height Override to remain the only persisted vertical intent, so that no schema migration or second sizing flag is required.
67. As a maintainer, I want Drop Shelf and Edge Rail placeholders to share one visual contract without a generalized drag-preview framework, so that the change remains small and consistent.
68. As a maintainer, I want completed historical specifications to remain archival, so that 0.0.16 supersedes behavior without rewriting prior delivery evidence.
69. As a release owner, I want actual Windows OLE and WebView2 behavior verified on both surfaces, so that native cursor, timing, hit testing, layout, and sizing are proven together.
70. As a release owner, I want one exact Windows x64 installer candidate produced only after implementation acceptance, so that version 0.0.16 identifies verified delivered behavior.

## Implementation Decisions

- Deliver the approved work as one `0.0.16` release slice covering Drop Placeholder feedback on both Shelf surfaces, removal of Drop Shelf drag glow, and exact Automatic Shelf Sizing. Do not split sizing into a later version.
- Keep version metadata at `0.0.15` until implementation and behavioral acceptance are complete. A `ready-for-agent` specification does not grant implementation, release, commit, or version-bump approval.
- Reuse the existing native storage-drop eligibility and `shelf-drag-over` event interface. The event remains authoritative for whether a Drop Placeholder is visible. Do not add a second native drop target, parser, eligibility calculation, or Rail expansion timer solely for presentation.
- Preserve native `Copy` effect handling, unsupported-input rejection, application-owned drag exclusion, drop completion, drag leave, and Rail pointer reconciliation. Native storage-drop code changes are unnecessary unless implementation discovers a contract gap that cannot be solved at the existing event interface.
- Preserve the Drop Shelf's `draggingOver` behavioral state. Remove only the outer surface background/border/box-shadow treatment associated with storage drag. Keep the state as the source for Drop Placeholder visibility and the existing guard against starting Drop Shelf movement during an external storage drag.
- Render exactly one transient Drop Placeholder per eligible drag operation. It represents one prospective Shelf Batch regardless of input item count.
- Use the established 48-logical-pixel Shelf Batch card/row height, 4-pixel collection gap, dashed drag-hover border, and existing drag-hover semantic background. Reuse existing Dark, Light, and Windows Contrast tokens; add no second color system.
- Expose no filename, Shell Visual, thumbnail, folder/file classification, or item count in the placeholder. Do not perform new shell lookup or duplicate preflight during drag hover.
- Insert the transient placeholder at visual index zero before the newest retained Shelf Batch. It participates in frontend collection layout and shifts retained cards to their prospective post-acceptance positions.
- Keep the placeholder out of authoritative Shelf state and persistence. It must not alter retained Shelf Batch count, duplicate comparison, Automatic Shelf Sizing input, Edge Rail target-height input, accessibility collection size, or restart state.
- Ensure ordinary and virtualized collections account for the transient visual index so retained cards do not overlap and the final retained card remains reachable. Do not disable virtualization or create a parallel collection implementation.
- On an empty Drop Shelf, replace the ordinary downward icon and `Drop here` prompt with the placeholder while the eligible drag remains over the surface. Restore the empty prompt on leave, rejected completion, or duplicate-only completion.
- On the Edge Rail, listen to the existing window-local drag-over event. Keep the native `200 ms` expansion timer authoritative; do not implement a frontend timer. Show the placeholder only in the expanded collection. Preserve the `300 ms` leave-collapse delay and every existing transient hold.
- Do not temporarily resize either native surface for the placeholder. The Drop Shelf remains at its current height during hover. The Edge Rail remains sized from retained batch count. An accepted Shelf Batch then updates each surface through its established authoritative path.
- Keep the Edge Rail height contract unchanged: `max(280, min(642, 52 × batch count + 70))`, one through four retained batches at minimum height, bounded growth through eleven, and scrolling from twelve. Drop Shelf Automatic Shelf Sizing is a separate contract.
- Define Automatic Shelf Sizing as an exact derived value whenever no Manual Height Override exists. The authoritative state projection must not retain a larger prior automatic height through a monotonic `max(previous, target)` rule.
- Derive Drop Shelf automatic height from preferred width and authoritative retained Shelf Batch count using responsive columns and rows. Apply `180` pixels for zero through two rows, `230` for three, `282` for four, and `334` for five or more; six or more rows scroll.
- Centralize the sizing invariant in the Shelf State module's authoritative snapshot/result seam rather than adding mutation-specific sizing formulas to Remove, Clear, drag completion, and cleanup callers. Every successful operation returns a coherent state with the exact automatic height or the retained Manual Height Override.
- Width-only size saves preserve automatic eligibility when no Manual Height Override exists and recalculate both upward and downward from the new responsive row count. If a Manual Height Override already exists, a width-only save preserves it.
- A vertical or corner resize that actually changes height remains the only way to create or replace a Manual Height Override. Horizontal-only resize does not create one.
- While any Shelf content remains, a Manual Height Override wins over automatic calculation for accepted input, removal, Clear Temporary Items, Successful Drag-Out consumption, confirmed-Missing cleanup, width-only resize, refresh, reopen, and restart.
- Preserve the existing final-empty reset: removing the final Shelf Item clears retained sizing intent and makes the next opening `191 × 180`.
- Apply automatic window-size changes through the existing Drop Shelf window sizing interface. Keep the current top-left origin during automatic height change, then retain existing monitor work-area clamping and Per-Monitor V2 DPI behavior. Do not create a new placement owner or modify ADR-0002.
- Duplicate-only and unsupported drops remain no-ops for state, persistence, order, scrolling intent, and surface sizing. A valid duplicate may display a provisional placeholder because duplicate identity is resolved only on acceptance; it disappears without creating content.
- Preserve Shelf Batch newest-first ordering, item source order, mixed supported-input filtering, exact duplicate suppression, Temporary/Pinned lifecycle, Copy-only drag-out, failure rollback, and source-file safety.
- Preserve no-activate Edge Rail behavior, fullscreen policy, configured/remembered monitor precedence, Open Shelf placement, status ownership, Manage Items, Clear Temporary Items, and final-item hiding.
- Add no persistence field, database migration, setting, telemetry, dependency, drag-preview framework, frontend test framework, compatibility layer, or platform abstraction.
- Use the canonical domain terms Drop Placeholder, Automatic Shelf Sizing, and Manual Height Override. Do not introduce alternate sizing terminology in live documentation or new implementation comments.
- No ADR is required. The feature is reversible presentation behavior plus a clarified derived sizing invariant within existing state, presentation, and placement ownership.
- Do not create a Git commit without separate explicit user authorization.

## Testing Decisions

- Use one primary behavioral seam: the actual Windows Resident Instance with the real WebView2 Drop Shelf and Edge Rail. This is the highest existing seam that proves native OLE eligibility and cursor effect, Rail timing, no-activate behavior, Vue rendering, ordinary and virtualized layout, native window sizing, monitor geometry, themes, and accepted Shelf state together.
- A good test observes user-visible state, returned authoritative Shelf state, native window bounds, or durable state. It does not assert CSS class names, Vue handler names, private helper calls, source text, forwarding, or that one event merely invokes another interface.
- Use the existing Shelf State interface as supporting permanent regression coverage for Automatic Shelf Sizing. Exercise accepted input, width-only size save, Remove Item, Remove Batch, Clear Temporary Items, Successful Drag-Out completion, confirmed-Missing cleanup, Manual Height Override, reopen, restart, rollback, and final-empty reset through public state operations.
- Prefer behavior tests through authoritative Shelf operations over a private formula-only test. A small formula boundary test may support the suite, but it cannot replace operation-level proof that state, persistence, and returned height remain coherent.
- Replace historical automatic-height expectations `236`, `292`, and `348` with exact `180`, `230`, `282`, and `334` behavior. Replace the historical no-live-shrink expectation for the Drop Shelf; do not change the separate Edge Rail no-live-shrink contract.
- Exercise zero through six responsive rows at one-column width and representative two- and three-column widths. Verify two rows remain at `180`, three rows use `230`, four use `282`, five use `334`, and six retain `334` with reachable vertical scrolling.
- Exercise responsive width boundaries around one/two/three-column transitions. Verify a width-only save can grow height when rows increase and shrink height when rows decrease, while an existing Manual Height Override continues to win.
- Exercise every content-reduction path that can prune a Shelf Batch: explicit Remove Batch, Remove Item of a final batch member, Clear Temporary Items, accepted Temporary item drag-out, accepted Temporary whole-batch drag-out, and confirmed-Missing cleanup during preparation.
- For each content-reduction path, verify automatic mode returns the exact lower tier when responsive rows decrease, retains the tier when row count is unchanged, preserves a Manual Height Override while content remains, and resets to `191 × 180` when the final Shelf Item is removed.
- Exercise canceled, rejected, and failed drag-out outcomes. Verify no content consumption means no automatic row-count shrink and persistence rollback leaves the prior coherent height and Shelf content intact.
- Exercise restart and reopen with both automatic and manual sizing. Verify automatic mode derives the current exact tier rather than restoring a stale larger automatic height; verify manual mode restores the retained Manual Height Override.
- Verify automatic height changes keep the current top-left origin and remain within the monitor work area. Cover available 100% DPI and monitor configurations directly; record unavailable high-DPI or mixed-DPI hardware as unobserved rather than inferred.
- Reuse the existing native storage eligibility regression for supported paths, unsupported payloads, missing `Copy`, and application-owned drag exclusion. Treat it as supporting coverage, not proof of actual placeholder rendering.
- Reuse the existing Edge Rail timer regression for `200 ms` expansion, leave reconciliation, `300 ms` collapse, re-entry cancellation, and transient holds. Do not add a second timer seam.
- In the actual Drop Shelf, drag supported external files over empty, populated ordinary, scrollable, and virtualized collections. Verify exactly one 48-pixel placeholder, newest-first placement, retained-card displacement, dashed border, distinct background, stable native window bounds, Copy cursor, and no surface-wide drag glow.
- In the actual expanded Edge Rail on both docking edges, drag supported external files over the collapsed Rail, wait through the native delay, and verify one placeholder before the newest batch without focus theft, second timer behavior, surface-wide highlight, or hover-time Rail resizing.
- Exercise drag leave before expansion, drag leave after expansion, drop completion, unsupported payload, application-owned drag, and duplicate-only input on both surfaces. Verify placeholder cleanup and unchanged existing acceptance/rejection behavior.
- Exercise valid multi-file input and verify one placeholder, one accepted Shelf Batch, supported-entry filtering, source order, and skipped-entry feedback.
- Verify the Drop Shelf resize-hover outline independently after drag glow removal. Verify Drop Shelf movement remains blocked while `draggingOver` is true and becomes available again after drag leave or completion.
- Inspect ordinary and virtualized collection geometry during placeholder display. Verify no overlap, correct 48-pixel item measure and 4-pixel gap, stable newest-first order, bounded mounting, reachable final retained batch, and unchanged accessibility set size for authoritative Shelf Batches.
- Inspect Dark, Light, actual Windows Contrast Theme where available, and reduced-motion presentation. Verify placeholder border/background remain distinguishable and that disabling non-essential motion does not affect drag eligibility, placeholder lifecycle, or sizing.
- Run the complete frontend production build for Vue single-file-component typechecking and bundling. Do not add a frontend component-test framework or screenshot snapshot suite.
- Compile with cargo-xwin for `x86_64-pc-windows-msvc`, then execute the produced Windows native regression binary directly. An unavailable Wine runner is not runtime proof.
- After implementation acceptance, synchronize canonical version metadata to `0.0.16`, build one exact current-user Windows x64 NSIS candidate, and record installer and installed-executable sizes and SHA-256 hashes.
- Exercise the exact installed candidate with embedded frontend assets and the development server unavailable. Cover Drop Shelf and Edge Rail drag feedback, automatic/manual sizing, resident activation, Open Shelf, Cooperative Shutdown, same-candidate reinstall/update, uninstall, and Profile Data preservation.
- Add no permanent tests that assert source text, CSS utility strings, non-empty markup, mock echoes, forwarding-only behavior, incidental animation frames, or duplicate lower-seam copies of actual-surface behavior.

## Out of Scope

- Showing incoming filenames, paths, thumbnails, Shell Visuals, file/folder types, or item counts before drop acceptance.
- Creating one placeholder per dragged item or supporting more than one prospective Shelf Batch for one drag operation.
- Arbitrary insertion positions, pointer-selected insertion, batch reordering, or dropping between retained Shelf Batches.
- Preview-resizing the Drop Shelf or Edge Rail during drag hover.
- Changing the Edge Rail `200 ms` expansion delay, `300 ms` collapse delay, transient holds, or native pointer reconciliation.
- Changing the Edge Rail height formula, height tiers, live-shrink behavior, expanded width, row density, footer/header geometry, scrolling, or final-item hiding.
- Adding a second native storage-drop event, frontend expansion timer, drop parser, duplicate preflight, or eligibility model.
- Changing supported storage path rules, mixed-input filtering, duplicate identity, one-batch-per-drop behavior, Copy-only drag-out, Temporary/Pinned lifecycle, or availability classification.
- Changing Manual Height Override persistence, adding a sizing mode setting, adding a reset-size action, or persisting automatic height as user intent.
- Keeping a Manual Height Override after the final Shelf Item is removed.
- Changing Drop Shelf placement ownership, persisting Drop Shelf coordinates, keeping the bottom edge fixed during shrink, or modifying configured/remembered Edge Rail monitor ownership.
- Redesigning Shelf Batch content, actions, Manage Items, Clear Temporary Items, Utility Menu, About Surface, status messages, scrollbars, control strip, resize handles, or empty-state copy outside the transient drag state.
- Adding animation beyond existing state-change policy, a new design system, icon dependency, testing framework, telemetry, compatibility path, migration, or non-Windows platform support.
- Rewriting completed historical specifications or delivery evidence. This specification and the canonical product documents govern 0.0.16 behavior prospectively.
- Implementing tickets, bumping version metadata, packaging 0.0.16, or creating a Git commit without the subsequent workflow gates and explicit approval.

## Further Notes

- The approved domain term is **Drop Placeholder**, not file placeholder, hover card, or ghost batch.
- The approved sizing term is **Automatic Shelf Sizing**, paired with **Manual Height Override**.
- The Drop Placeholder is provisional presentation state. It predicts insertion position, not acceptance. Duplicate-only and mixed unsupported input explain why it intentionally omits content details.
- Automatic Shelf Sizing and Edge Rail sizing remain separate contracts. The new `180/230/282/334` tiers apply only to the Drop Shelf; the Rail retains its current retained-batch formula and no-live-shrink behavior.
- The preferred implementation shape is one deep Shelf State sizing invariant behind the existing state interface, plus presentation-only placeholder state behind the existing native drag-over event. Avoid mutation-specific formulas and duplicate native drag state.
- ADR-0002 remains authoritative for Edge Rail placement ownership. Automatic Drop Shelf height changes create no new placement owner and do not alter configured or remembered Rail monitor state.
- No new ADR is warranted because the decisions are reversible and fit established state, presentation, native adapter, and placement ownership.
- Implementation and installed-candidate verification are complete. The user subsequently reported that Ticket 04 passed and `0.0.16` was committed, closing the human gate; historical observation limits remain preserved in the roadmap and release ticket.
- Git commit authorization remains separate from specification, ticket, implementation, test, acceptance, and release approval.
