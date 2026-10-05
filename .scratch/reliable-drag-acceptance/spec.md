# Reliable Native Drag Acceptance Specification

Status: blocked for planning — the approved test-profile seam enables a real DropCove side-by-side Release matrix, which passes using the existing WinUI source engine. No native advantage is established; historical destination identity remains unknown. See [real-app verification](installed-profile-verdict.md). Native cutover requires a new planning decision.

## Problem Statement

DropCove removes a Temporary Item after a Successful Drag-Out and retains a Pinned Item. Cancellation, rejection, failure, and a destination that stops before it confirms acceptance must retain the Path Reference.

The current WinUI source-drag completion signal does not always preserve this contract. Installed testing showed that a destination can stop during `DragEnter`, before it sets an effect or receives `Drop`, while WinUI still reports `Copy`. DropCove then removes the Temporary Item even though the destination did not confirm acceptance.

Changing the requested operation to `None` is not a solution. It made rejection and the disappearing destination safe, but it also prevented a normal Copy destination from receiving `Drop`.

DropCove needs an evidence-backed drag acceptance signal. The design must not infer acceptance from file existence, destination-side later processing, timeout, or the last advertised Copy effect.

## Solution

First build a small throwaway native OLE probe. The probe must correlate the drag source result with direct events from controlled destinations for accepted Copy, rejection, user cancellation, and a destination that stops during `DragEnter`. A native result predicate is valid only if it distinguishes all four outcomes consistently. `DRAGDROP_S_DROP`, a Copy effect, or any other OLE result is a hypothesis until this probe proves the complete predicate.

If the probe remains ambiguous, stop. Keep release blocked and return to design. Do not change the Path Reference lifecycle contract and do not ship a heuristic.

If the probe establishes a reliable predicate, replace the WinUI source-drag path with one managed native OLE drag implementation. Use it for individual Shelf Items and whole Shelf Batches from both the Drop Shelf and Edge Rail. Preserve Copy-only behavior, file and folder support, current same-integrity destination support, current Missing and Unavailable handling, and equivalent Windows drag feedback.

A native startup or execution error retains every valid participating Path Reference and uses the existing warning surface. There is no runtime fallback to the WinUI source-drag path. After the native path passes acceptance, remove the old source-drag path completely.

## User Stories

1. As a user, I want a Temporary Item removed only after the destination confirms acceptance, so that a failed destination does not lose my Path Reference.
2. As a user, I want a destination that stops during drag entry to leave the Temporary Item unchanged, so that an external application failure is safe.
3. As a user, I want cancellation to retain every valid participating Path Reference, so that pressing `Esc` is safe.
4. As a user, I want rejection to retain every valid participating Path Reference, so that dropping over an unsupported target is safe.
5. As a user, I want a Pinned Item retained after a Successful Drag-Out, so that repeated reuse remains available.
6. As a user, I want a successfully accepted Temporary Item removed, so that completed workflows do not leave stale shelf entries.
7. As a user, I want whole-Shelf-Batch drag-out to use the same acceptance rule as one-item drag-out, so that lifecycle behavior is consistent.
8. As a Drop Shelf user, I want item and batch drag-out to use the reliable acceptance path, so that the visible shelf is safe.
9. As an Edge Rail user, I want item and batch drag-out to use the same reliable acceptance path, so that rail interaction is not less safe than the Drop Shelf.
10. As a user, I want Copy-only drag-out, so that DropCove never asks a destination to move or delete my source filesystem object.
11. As a user, I want file drag-out to continue working with the destinations I use today, so that the safety change does not reduce interoperability.
12. As a user, I want folder drag-out to continue working with Explorer, so that folder workflows remain available.
13. As a browser user, I want supported file inputs to continue accepting DropCove drags, so that web upload workflows remain available.
14. As a VS Code user, I want file drag-out to continue working, so that development workflows remain available.
15. As a chat user, I want a common chat application to continue accepting file drag-out, so that sharing workflows remain available.
16. As a user, I want standard Copy and Not-allowed cursor feedback, so that I can predict whether the current target accepts the drag.
17. As a user, I want an equivalent drag image, so that I can see what item or batch I am dragging.
18. As a user, I want Windows drag-threshold behavior, so that a normal click does not start a drag.
19. As a user, I want `Esc` to cancel a native drag, so that cancellation remains immediate.
20. As a user, I want native drag failures to show an actionable warning and retain my Path References, so that failure is visible and safe.
21. As a user, I want Missing cleanup to happen before drag-out as it does now, so that confirmed invalid references do not enter the payload.
22. As a user, I want Unavailable Items retained and excluded or confirmed according to the existing partial-batch rules, so that temporary access failures do not destroy references.
23. As a user, I want a mixed Pinned and Temporary Shelf Batch to retain its Pinned Items after accepted Copy, so that batch semantics do not change.
24. As a user, I want a destination that accepts the drop and later fails its own processing to remain a Successful Drag-Out, so that DropCove does not claim to monitor another application's later work.
25. As a user, I want source files and folders unchanged by DropCove, so that the shelf remains a Path Reference workspace rather than a file store or file manager.
26. As a maintainer, I want one drag acceptance implementation, so that the Drop Shelf and Edge Rail cannot drift into different lifecycle rules.
27. As a maintainer, I want the unreliable WinUI source-drag path removed after cutover, so that it cannot silently return as a fallback.
28. As a maintainer, I want the native predicate proved before production integration, so that a lower-level implementation does not repeat the same false-positive contract gap.

## Implementation Decisions

- The domain boundary remains: a Successful Drag-Out occurs only when the destination confirms acceptance before the drag operation ends. Acceptance does not guarantee later destination processing.
- The first deliverable is a throwaway native OLE probe, not production integration.
- The probe uses a real OLE source and controlled real destinations. It records source results and destination events for the same attempt.
- The probe covers accepted Copy, explicit rejection, user cancellation, and a destination that stops during `DragEnter` before setting an effect or receiving `Drop`.
- No OLE result constant or effect is declared reliable before the probe establishes the complete predicate.
- The probe must pass every recorded attempt. Results are not averaged and failed attempts are not discarded.
- If no reliable predicate is found, implementation stops and release remains blocked. A timeout, file-existence check, destination output check, last advertised effect, or user-invisible heuristic is not an acceptable substitute.
- If the probe passes, production uses managed C# COM/P/Invoke in the native interop layer. It does not add a C++/WinRT project, enable unsafe code, or add a third-party OLE package unless later evidence proves the chosen approach impossible and a new planning decision approves the change.
- The production native adapter owns OLE invocation, the data object, source callbacks, result capture, drag feedback integration, unmanaged resource lifetime, and conversion to an application drag result.
- The application drag service remains the single presentation adapter for drag preparation and completion. The Core manager remains the lifecycle and persistence authority.
- The native acceptance path applies to one-item and whole-Shelf-Batch drag-out from both the Drop Shelf and Edge Rail.
- The data object exposes the standard filesystem path formats required by the approved target matrix. Copy is the only allowed operation.
- Native drag starts only after the pointer moves beyond the Windows drag threshold. It runs through the normal STA OLE modal drag loop and supports `Esc` cancellation.
- Drag feedback must preserve standard Copy and Not-allowed cursors and provide an equivalent item or batch drag image. Pixel-identical WinUI visuals are not required.
- Existing preparation semantics remain unchanged: validate availability immediately before drag, remove confirmed-Missing references durably, retain Unavailable references, and capture participating Shelf Item identities before the operation.
- A production result is mapped to accepted Copy only by the predicate proved by the probe. Every cancellation, rejection, disappearing destination, COM failure, data-object failure, or unclassified result is non-successful and retains valid participating references.
- A native startup or execution error uses the existing warning surface. It never falls back to WinUI source drag.
- After the native path passes the complete acceptance matrix, remove WinUI source-drag properties, events, result mapping, and obsolete code from both surfaces. Do not keep a dormant compatibility path.
- Existing same-integrity limitations remain. Elevated cross-integrity drag/drop is not added.
- Existing popup lifetime behavior remains: individual item drag keeps its popup open; whole-batch drag closes its popup; cancellation and failure do not consume references.
- No new persistence schema, telemetry, background polling, or periodic target inspection is introduced.
- The accepted architecture decision is recorded as proposed until the OLE probe proves the signal.
- The existing responsive-shelf keyboard-smoke blocker and release CPU/performance blockers are independent. This feature does not mark them resolved.

## Testing Decisions

- The probe is the authority for whether native OLE offers a stronger acceptance signal than WinUI in the disappearing-destination scenario.
- Each probe attempt records one attempt identifier, source input paths, source return values, target mode, target `DragEnter`/`DragOver`/`Drop` events, and whether the target intentionally stopped.
- Run at least five correlated attempts for each probe outcome: accepted Copy, rejection, cancellation, and stop-during-`DragEnter`. Every attempt must match the proposed predicate.
- Accepted Copy requires the controlled destination to receive `Drop` and all offered paths. Rejection, cancellation, and stop-during-`DragEnter` must not satisfy the accepted predicate.
- File existence and later destination processing are never acceptance assertions.
- The throwaway probe is removed from the production tree after its verdict is recorded in the issue. No probe diagnostics or helper processes ship.
- Existing manager tests remain the highest stable seam for Temporary/Pinned consumption, Missing cleanup, partial-batch preparation, persistence, and at-least-once crash behavior.
- After the probe proves a predicate, focused native tests cover only decision-rich result classification and unmanaged resource ownership. They do not mock COM event forwarding or assert P/Invoke declarations.
- Application tests verify that item and batch identities captured before drag are the identities completed afterward, and that non-successful results retain references.
- Installed self-contained Release smoke is the acceptance authority for pointer threshold, drag image, cursor feedback, `Esc`, popup/flyout lifetime, and real destination interoperability.
- Installed Release smoke covers individual Temporary and Pinned Items, whole Temporary and mixed Shelf Batches, files and folders, Drop Shelf and Edge Rail, accepted Copy, rejection, cancellation, disappearing destination, and native execution failure.
- The destination matrix includes Explorer/Desktop, Edge or Chrome file input, VS Code, and at least one common chat application at the same integrity level.
- Each accepted target must receive every offered path. The source filesystem objects must remain byte-for-byte unchanged.
- The complete automated suite must pass after the old WinUI source-drag path is removed.
- Existing installed Release resource gates remain unchanged: every visible and post-dismissal `WorkingSet64` sample below `150 MB`, idle CPU at or below `0.1%`, and shelf-show p95 at or below `150 ms`. Native drag qualification does not waive an existing failure.
- The previously failed clean keyboard popup smoke must be rerun with controlled focus. Native drag acceptance does not count as keyboard-smoke evidence.

## Out of Scope

- Proving that a destination completes later asynchronous processing after it accepts the drop.
- Detecting copied files in a destination folder as an acceptance signal.
- Monitoring or injecting into destination processes.
- Cross-integrity or elevated-target drag/drop support.
- Move or Link operations.
- Clipboard workflows, file storage, file copying by DropCove, or source filesystem mutation.
- Changing Temporary Item, Pinned Item, Missing Item, Unavailable Item, Shelf Batch, or Path Reference semantics.
- A runtime switch between native and WinUI drag engines.
- Keeping the current WinUI source-drag path as a hidden fallback.
- New telemetry or network communication.
- Resolving the separate popup keyboard-smoke, idle CPU, memory, shelf-show latency, mixed-DPI, or final installer qualification blockers through this feature.

## Further Notes

- The previous WinUI drag-out research was correct for normal accepted and canceled targets but over-relied on `DropCompleted=Copy` as an acceptance signal. The disappearing-destination installed repro supersedes that assumption.
- The responsive Drop Shelf work remains blocked until failed-target retention is solved or the product contract changes. This specification keeps the contract.
- ADR 0003 records the evidence gate and clean-cutover direction without claiming that a particular OLE result is already reliable.
- All repository changes remain uncommitted until the user explicitly authorizes a commit.

## Approved test-profile verification amendment — 2026-10-05

The user approved an explicit test-only launch profile to safely investigate the installed-app premise: database/settings roots and mutex/activation names are profile-specific, while normal launch remains unchanged and test mode never writes Windows startup registration. This seam does not approve native production integration or change drag lifecycle semantics. A side-by-side clean self-contained Release run of the actual DropCove popup passed five attempts for each required outcome over Reject backing; five additional attempts proved genuine acceptance by Copy backing after primary exit. See [verdict](installed-profile-verdict.md) and [correlated evidence](installed-profile-evidence.json). No final NSIS, resource, or full destination-matrix qualification is claimed.
