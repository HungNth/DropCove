# Bulk Pinning for Shelf Batches Specification

Status: ready-for-agent

## Problem Statement

A multi-item Shelf Batch can contain enough Shelf Items that pinning them individually becomes repetitive and error-prone. The current Drop Shelf requires the user to open the batch popup and activate Pin on every item. The effort grows linearly with batch size and makes repeated reuse of a whole dropped group unnecessarily difficult.

The product already defines pinning as an item-level lifecycle choice: a Pinned Item remains after a Successful Drag-Out, while a Temporary Item is consumed. The improvement must make that existing lifecycle efficient for an entire Shelf Batch without introducing a separate batch-owned pin state, partial persistence, ambiguous mixed-state behavior, or regressions to drag-out, popup, keyboard, accessibility, sizing, and performance contracts.

## Solution

Add **Bulk Pinning** to the Drop Shelf. A multi-item Shelf Batch exposes a tri-state bulk toggle directly on its card and in a fixed header inside its anchored popup. One action sets every Shelf Item in that Shelf Batch to the Pinned or Temporary lifecycle state.

The toggle derives its state from the Shelf Items: unchecked when all are Temporary, checked when all are Pinned, and indeterminate when the batch is mixed. Activating an unchecked or mixed toggle pins all items; activating a checked toggle unpins all items. The Shelf Batch never owns pin state.

Bulk Pinning is one all-or-nothing persistence mutation. A failure leaves both in-memory and persisted Shelf Item states unchanged and reports the failure through the existing Drop Shelf status surface. The Edge Rail, individual Pin/Unpin actions, availability classification, and drag-out lifecycle remain unchanged.

## User Stories

1. As a user with a large multi-item Shelf Batch, I want to pin every item with one action, so that I do not have to pin items individually.
2. As a user with a large multi-item Shelf Batch, I want to unpin every item with one action, so that I can return the whole group to temporary lifecycle behavior efficiently.
3. As a user, I want Bulk Pinning available directly on the Shelf Batch card, so that pinning the whole group does not require opening the popup.
4. As a user already managing a Shelf Batch popup, I want Bulk Pinning available in that popup, so that I do not have to leave the current context.
5. As a user with a single-item Shelf Batch, I want the existing direct Pin/Unpin interaction unchanged, so that the simplest workflow remains simple.
6. As a user, I want an all-Temporary Shelf Batch to show an unchecked bulk toggle, so that its lifecycle status is clear.
7. As a user, I want an all-Pinned Shelf Batch to show a checked bulk toggle, so that I can see that every item will be retained after accepted drag-out.
8. As a user, I want a Shelf Batch containing both Temporary and Pinned Items to show an indeterminate state, so that the UI does not misrepresent the group as uniformly pinned or unpinned.
9. As a user, I want activating an indeterminate bulk toggle to pin all items, so that the action completes the partially pinned group rather than unexpectedly unpinning existing Pinned Items.
10. As a user, I want activating an unchecked bulk toggle to pin all items, so that the control follows normal bulk-selection expectations.
11. As a user, I want activating a checked bulk toggle to unpin all items, so that the control remains reversible.
12. As a user, I want Bulk Pinning to happen without a confirmation dialog, so that a reversible lifecycle action remains fast.
13. As a user, I want Available, Unavailable, and confirmed-Missing references still present in the Shelf Batch included in Bulk Pinning, so that “all items” has one predictable meaning.
14. As a user, I want bulk-pinned items to remain pinned after restarting DropCove, so that the operation is durable.
15. As a user, I want bulk-unpinned items to remain temporary after restarting DropCove, so that the reverse operation is equally durable.
16. As a user, I want a bulk-pinned Shelf Batch retained after a Successful Drag-Out, so that I can reuse every participating item.
17. As a user, I want a mixed Shelf Batch drag-out to retain participating Pinned Items and consume participating Temporary Items, so that Bulk Pinning does not change existing lifecycle semantics.
18. As a user, I want a bulk-unpinned Shelf Batch consumed after a Successful Drag-Out, so that unpinning all items restores the existing temporary workflow.
19. As a user, I want cancellation, rejection, drag failure, and an unaccepted destination to leave participating references unchanged, so that Bulk Pinning does not weaken drag-out safety.
20. As a user, I want DropCove to keep referencing rather than modifying source filesystem objects, so that Bulk Pinning cannot copy, move, rename, overwrite, or delete my files and folders.
21. As a user, I want the bulk control described as “Pin all items” or “Unpin all items,” so that the UI does not imply that Shelf Batch itself owns pin state.
22. As a user, I want the multi-item card actions ordered Pin, Manage, and Remove, so that the new action follows the existing card interaction pattern.
23. As a user, I want the mixed card state visually distinct from both checked and unchecked states, so that partial pinning is recognizable without opening the popup.
24. As a high-contrast user, I want the mixed state distinguished by shape as well as color, so that the state remains perceivable under system contrast settings.
25. As a user, I want the popup header to show how many items are pinned, so that I can understand the group state before acting.
26. As a user, I want the popup summary to read `0 of N pinned`, `X of N pinned`, or `N of N pinned`, so that mixed state is explicit rather than icon-only.
27. As a user, I want the popup header fixed above the scrolling item list, so that Bulk Pinning remains reachable even in a very large Shelf Batch.
28. As a user, I want scrolling the popup list not to scroll the bulk header away, so that list length does not recreate the original repetitive UX problem.
29. As a user, I want the bulk control’s tooltip and accessible name to include its actual action and item count, so that its result is predictable.
30. As an assistive-technology user, I want an all-Temporary bulk toggle reported as Off, so that automation state matches the visible state.
31. As an assistive-technology user, I want an all-Pinned bulk toggle reported as On, so that automation state matches the visible state.
32. As an assistive-technology user, I want a mixed bulk toggle reported as Indeterminate, so that partial pinning is conveyed truthfully.
33. As an assistive-technology user, I want a mixed accessible name such as “Pin all 10 items, 3 currently pinned,” so that both the next action and current state are clear.
34. As a keyboard user, I want keyboard-opening a multi-item popup to focus the bulk control first, so that the new primary action is immediately available.
35. As a keyboard user, I want `Tab` to move from the bulk control through Pin and Remove for every item row, so that every popup action remains reachable.
36. As a keyboard user, I want `Shift+Tab` to traverse the same actions in reverse, so that popup navigation is symmetric.
37. As a keyboard user, I want forward traversal after the final item action to wrap to the bulk control, so that the popup remains a bounded keyboard context.
38. As a keyboard user, I want reverse traversal from the bulk control to wrap to the final item action, so that virtualized rows remain reachable in both directions.
39. As a keyboard user, I want both `Enter` and `Space` to activate Bulk Pinning, so that the action follows the established Drop Shelf keyboard model.
40. As a keyboard user, I want closing the popup to return focus to its originating chevron, so that the existing focus restoration contract remains intact.
41. As a user acting inside the popup, I want Bulk Pinning to leave the popup open, so that I can inspect or adjust individual items afterward.
42. As a user acting inside the popup, I want focus to remain on the bulk control after success or failure, so that the interface does not move unexpectedly.
43. As a user activating Bulk Pinning from a card while a popup is open, I want the existing outside-click rule to close the popup, so that the transient surface keeps one consistent dismissal model.
44. As a user, I want individual Pin/Unpin actions to update the card and popup aggregate state immediately, so that mixed status never becomes stale.
45. As a user, I want removing an item to recompute the aggregate state and count immediately, so that the bulk control continues to represent the remaining Shelf Items.
46. As a user, I want drag-out consumption to recompute or remove the relevant card and popup state immediately, so that no stale aggregate survives a lifecycle change.
47. As a user, I want Bulk Pinning not to resize or reposition the Drop Shelf, so that lifecycle management does not disturb my workspace.
48. As a user, I want Bulk Pinning not to change popup placement or the user-selected shelf bounds, so that the new header remains presentation-neutral.
49. As a user, I want successful Bulk Pinning to communicate through the changed controls and counts without an additional status message, so that routine interaction stays quiet.
50. As a user, I want a persistence failure to leave every item in its previous state, so that a single action cannot silently produce an unintended mixed Shelf Batch.
51. As a user, I want a persistence failure reported as “Couldn’t update pinning. Nothing changed.”, so that I know the operation is safe to retry manually.
52. As a user, I do not want DropCove to retry a failed Bulk Pinning mutation automatically, so that one activation causes at most one intentional persistence attempt.
53. As a user, I want every item in the target Shelf Batch changed atomically, so that restart cannot reveal partial success.
54. As an Edge Rail user, I want the Edge Rail interaction model unchanged, so that this Drop Shelf management improvement does not add rail clutter or inconsistent item controls.
55. As a performance-conscious user, I want Bulk Pinning to use one persistence mutation rather than one write per Shelf Item, so that large batches remain responsive.
56. As a performance-conscious user, I want unopened batch details to remain deferred, so that adding a card-level bulk action does not materialize every popup row.
57. As a maintainer, I want one manager operation to own Bulk Pinning invariants, so that card and popup callers cannot diverge.
58. As a maintainer, I want Bulk Pinning tested through the existing manager and real SQLite persistence seam, so that tests prove production behavior without a test-only persistence abstraction.
59. As a maintainer, I want installed Release UI Automation evidence for tri-state, focus, popup, and status behavior, so that XAML and accessibility claims are based on the real shipped surface.
60. As a maintainer, I want existing individual pinning and drag-out tests to remain valid, so that Bulk Pinning is additive rather than a lifecycle rewrite.

## Implementation Decisions

- **Bulk Pinning** remains an item-level lifecycle action. No `Pinned Batch` domain type, batch-owned pin field, batch database column, compatibility alias, or secondary source of truth is introduced.
- The manager module remains the single state-mutation interface used by both card and popup callers. It gains one asynchronous bulk operation that accepts a Shelf Batch identity and a target pinned boolean, returns whether the target batch exists, and owns lookup, serialization, persistence ordering, and in-memory replacement.
- The public bulk operation is item-centric in name and contract, such as `SetAllItemsPinnedAsync`. It does not expose or imply a mutable batch pin property.
- The manager’s existing mutation gate serializes Bulk Pinning with individual pinning, removal, clear, availability preparation, and drag completion. Callers do not coordinate these mutations themselves.
- A missing Shelf Batch returns `false` and performs no persistence mutation. A batch already entirely in the requested target state returns `true` without writing unchanged rows.
- For a real state change, persistence completes before the manager replaces the in-memory Shelf Batch. Persistence or cancellation failure before commit leaves the in-memory batch untouched and propagates the failure to the application caller.
- Once persistence reports a successful commit, the manager performs the non-cancelable in-memory replacement immediately. It must not observe cancellation between durable commit and local state replacement.
- The in-memory update creates one replacement item collection for the target Shelf Batch and preserves batch identity, creation time, item identity, ordering, paths, folder flags, and availability. Only the per-item pinned lifecycle value changes.
- Persistence adds one bulk mutation keyed by Shelf Batch identity. It updates every persisted Shelf Item in that batch to the same `is_pinned` value in one SQLite transaction or single atomic SQLite statement; it never loops through the existing per-item persistence method.
- Existing per-item `is_pinned` storage remains authoritative. No schema migration is required.
- Available, Unavailable, and confirmed-Missing Shelf Items still present in the target Shelf Batch participate equally. Bulk Pinning neither reclassifies availability nor removes references.
- Existing individual Pin/Unpin behavior remains available and continues to write one Shelf Item. Bulk Pinning does not replace the individual interface.
- Existing Successful Drag-Out behavior remains authoritative. Accepted batch drag consumes only participating Temporary Items, retains participating Pinned Items, and removes the Shelf Batch only when no items remain.
- The multi-item Shelf Batch card exposes a `24 × 24` tri-state pin toggle before Manage and Remove. The existing single-item toggle remains binary and keeps its current behavior.
- The multi-item card toggle derives its state from current child items: `false` for all Temporary, `true` for all Pinned, and `null` for mixed. This is a projection only; the Shelf Batch model does not store the aggregate.
- Activating an unchecked or indeterminate bulk toggle targets `isPinned = true`. Activating a checked bulk toggle targets `isPinned = false`. The application calculates this binary target from the pre-activation aggregate and does not rely on native cyclic three-state transitions.
- The current compact pin icon remains the visual basis. Mixed state adds a clear dash or badge shape to the pin. Opacity alone is insufficient. Theme-aware resources must preserve the distinction in light, dark, and high-contrast modes.
- The installed Windows App SDK research indicates that the existing WinUI `ToggleButton` can expose `IsThreeState`, nullable `IsChecked`, and UI Automation `ToggleState.Indeterminate`; the implementation must still prove these behaviors against the repository’s actual SDK through build and installed UI Automation smoke.
- Accessible names are item-centric and count-aware. All-Temporary and mixed controls use “Pin all N items,” with mixed adding “X currently pinned”; all-Pinned controls use “Unpin all N items.” UI strings do not use “Pin Batch” or “Pinned Batch.”
- The anchored multi-item popup gains a fixed, non-scrolling header above its existing item scroller. The header shows `X of N pinned` at the left and the tri-state bulk toggle at the right.
- The popup’s existing overall width and height bounds remain governing. The fixed header consumes part of the bounded popup height; only the item list scrolls within the remaining space. Horizontal scrolling remains disabled.
- Popup opening still defers full item projections until needed. Adding the card-level aggregate and popup header must not materialize item row visuals for closed popups or retain them after popup closure.
- Keyboard-opened popups initially focus the header bulk toggle. Pointer-opened popup behavior remains non-disruptive unless the user explicitly moves focus.
- The popup focus cycle models one fixed header action followed by two actions per item row: individual Pin/Unpin, then Remove. Forward and reverse traversal wrap across the complete logical sequence.
- Keyboard traversal continues to realize only the target virtualized row. Adding the header action must not eagerly realize all item rows.
- `Enter` and `Space` both activate the focused bulk toggle. `Esc` precedence, popup closure, and focus restoration to the originating chevron remain unchanged.
- Bulk Pinning from inside the popup keeps that popup open, updates card and popup projections, and preserves focus on the header control after success or failure.
- A card bulk action remains outside the popup. Existing outside-click behavior closes any open popup before the card mutation proceeds; no same-batch exception is added.
- Every mutation that can change aggregate state—including individual Pin/Unpin, item removal, accepted drag-out, and restored persistence—recomputes card and open-popup projections from the manager’s current Shelf Items.
- A successful bulk mutation has no toast, confirmation, or transient success status. The checked state, mixed visual, subtitle, popup count, and item rows provide the feedback.
- The asynchronous card and popup handlers catch persistence failures. They retain or restore the manager-derived visual state, preserve popup focus when applicable, and reuse the current Drop Shelf status surface with the message “Couldn’t update pinning. Nothing changed.” No automatic retry occurs.
- Bulk Pinning never changes preferred shelf size, Manual Height Override, Automatic Shelf Growth, actual window bounds, popup anchor ownership, or batch ordering.
- The Edge Rail adds no Bulk Pinning control. It continues to reflect item lifecycle through its existing pinned indicators and current manager state.
- No new NuGet dependency, background worker, telemetry, polling, WinUI test project, or persistence adapter interface is introduced.
- The implementation remains within the established CLI-built, self-contained WinUI 3 toolchain.

## Testing Decisions

- Good permanent tests assert consumer-visible lifecycle state, transaction boundaries, restart behavior, and invariants through the highest existing interface. They do not assert SQL text, XAML source, event forwarding, template structure, private helper calls, duplicate counts, or mock echoes.
- The primary automated seam is `DropShelfManager` backed by a real temporary SQLite database. This is the existing highest seam for state and persistence and avoids adding a persistence interface solely to inject failures.
- Existing manager tests are prior art for per-item pinning and drag lifecycle. Existing persistence tests are prior art for immediate commit, restart restoration, temporary database isolation, and direct SQLite setup used to create controlled database conditions.
- Manager-plus-SQLite tests cover all-Temporary to all-Pinned, mixed to all-Pinned, and all-Pinned to all-Temporary transitions. Assertions inspect every Shelf Item rather than only aggregate counts.
- Persistence tests reopen the manager after each bulk transition and verify every Shelf Item retains the intended lifecycle state, identity, order, availability, path, and folder flag.
- Tests cover Available, Unavailable, and Missing classifications in one target Shelf Batch and verify Bulk Pinning changes only lifecycle state.
- Tests cover a missing batch identity and an already-satisfied target state. Both preserve all existing batches and items; the no-op case remains successful.
- Tests cover Successful Drag-Out after bulk pin, bulk unpin, and a subsequently created mixed state. They preserve the existing rule that only participating Temporary Items are consumed.
- Atomic failure is tested at the manager seam with real SQLite by installing a deterministic aborting trigger for the target update. The bulk operation must fail, the current manager must retain every pre-operation state, and reopening the database must show the same pre-operation states.
- The failure test must include a mixed pre-operation batch so that it can detect accidental partial normalization as well as all-or-none corruption.
- No permanent test is added merely to prove that the application forwards a batch identity or boolean to the manager. Installed behavior is the proof for card and popup wiring.
- Installed self-contained Release UI Automation and smoke are the acceptance authority for the card control, fixed popup header, tri-state visual, UI Automation toggle state, accessible names, keyboard focus, popup lifetime, and stable geometry.
- Installed smoke covers all-Temporary, mixed, and all-Pinned card and popup states; Pin All from Off and Mixed; Unpin All from On; and immediate recomputation after individual Pin/Unpin and Remove.
- UI Automation verifies Off, On, and Indeterminate toggle states plus the exact count-aware accessible action names on both card and popup controls.
- Keyboard smoke verifies keyboard-open focus starts on the bulk control; `Tab` and `Shift+Tab` traverse the header and every virtualized row action in both directions; `Enter` and `Space` activate; wrap is correct; and `Esc` restores chevron focus.
- Popup smoke verifies an in-popup mutation keeps the popup open and focused, while a card mutation follows outside-click dismissal. Long-list scrolling keeps the header fixed and does not change popup or shelf bounds.
- An isolated installed test profile may use the same deterministic SQLite abort trigger to verify the visible failure message, unchanged toggle/count/item states, retained popup focus, and absence of automatic retry. Live user data must never be altered for this test.
- Visual smoke covers light, dark, and high-contrast presentation. Mixed state must remain distinguishable without relying only on color or opacity.
- Performance smoke retains deferred popup projections and responsive interaction for a long Shelf Batch. The existing `100` Shelf Batch / `1,000` Shelf Item release gate remains unchanged: visible and post-dismissal working set below `150 MB`, idle CPU at or below `0.1%`, responsive pointer/scroll/drag interaction, and shelf-show latency p95 at or below `150 ms`.
- The complete existing automated suite must remain green. Existing individual Pin/Unpin, persistence, removal, availability, popup, drag-out, sizing, shake, Edge Rail, and visual-coordinator contracts are not rewritten to match implementation details.

## Out of Scope

- A Pinned Batch domain object, batch-owned pin state, batch persistence column, or compatibility alias.
- Bulk Pinning controls or individual Pin/Unpin controls on the Edge Rail.
- Changing Temporary Item, Pinned Item, Missing Item, Unavailable Item, Path Reference, Shelf Batch, Successful Drag-Out, or accepted-drag cleanup semantics.
- Changing the drag engine, Copy-only behavior, destination acceptance rules, source filesystem ownership, or same-integrity limitations.
- Arbitrary multi-selection, cross-batch Bulk Pinning, partial item selection, saved selections, named groups, or multiple shelves.
- Adding items to an existing Shelf Batch after its originating drop operation.
- Confirmation dialogs, undo history, scheduled retry, background retry, or partial-success reporting for Bulk Pinning.
- Removing Missing Items, probing availability, or reconnecting Unavailable Items as part of Bulk Pinning.
- Redesigning batch cards, item rows, popup placement, popup size limits, shelf sizing, Automatic Shelf Growth, or Manual Height Override beyond the fixed popup header and tri-state control.
- New telemetry, polling, network traffic, development dependencies, NuGet packages, persistence abstractions, or UI test projects.
- New localization infrastructure or a broader rewrite of existing Drop Shelf status messages.

## Further Notes

- This specification extends the responsive Drop Shelf specification. It changes multi-item card action availability, adds a fixed popup header, and makes the bulk header the first keyboard-focused popup action. Other responsive layout, popup placement, virtualization, sizing, drag, accessibility, and performance requirements remain governing.
- The canonical domain term is **Bulk Pinning**, recorded in `CONTEXT.md`. All-pinned, all-temporary, and mixed status are derived from Shelf Items; they are never owned by Shelf Batch.
- WinUI tri-state research findings are recorded in `.scratch/bulk-pinning/winui-tristate-research.md`. They guide control selection but do not replace build and installed UI Automation verification against the actual packaged application.
- The rejected native drag-out ADR remains governing context: this feature preserves the existing verified WinUI drag path and does not reopen drag-engine selection.
- No ADR is required for Bulk Pinning. The feature uses the existing item lifecycle model, manager seam, SQLite schema, and Drop Shelf presentation rather than making a hard-to-reverse architectural choice.
- All repository changes remain uncommitted until the user explicitly authorizes a commit.
