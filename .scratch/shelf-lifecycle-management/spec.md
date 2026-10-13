# DropCove 0.0.3 — Shelf lifecycle and management

Status: ready-for-agent
Version: 0.0.3
Implementation approval: approved by the user on 2026-10-13
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.3
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md)
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md)

## Problem Statement

The working 0.0.2 Drop Shelf can retain and restore path references and drag them out safely, but users cannot choose which references survive successful Copy, inspect/manage individual items in a multi-item Shelf Batch, clear only Temporary Items, or retain a deliberately chosen shelf height. The current presentation also lacks the management popup and keyboard/focus behavior needed for these workflows.

## Solution

Add durable item-level Pinning, Manage Items, tri-state Bulk Pinning, confirmed Clear Temporary Items, and native user sizing intent to the existing Drop Shelf. Preserve the existing storage-reference, Copy-only, persistence, duplicate, availability, and batch-removal contracts rather than rebuilding them.

The entire version is delivered through small working vertical slices, then verified through the installed Windows x64 artifact. Resident access, Edge Rail, Shake-to-open, advanced shell visuals, and recovery/distribution hardening remain later roadmap versions.

## User Stories

1. As a Windows user, I want to Pin a single Shelf Item, so that successful Copy does not consume its reference.
2. As a Windows user, I want to Unpin an item, so that it becomes Temporary and is consumed by a later successful Copy.
3. As a Windows user, I want a visible and accessible pin indicator, so that I can distinguish Pinned and Temporary Items.
4. As a Windows user, I want pin state to survive application restart, so that retained references keep their lifecycle behavior.
5. As a Windows user, I want mixed batches to consume only participating Temporary Items after Copy, so that Pinned and Unavailable references remain.
6. As a Windows user, I want rejected, canceled, or failed drag-out to retain references, so that unsuccessful transfers do not lose shelf content.
7. As a Windows user, I want Manage Items on multi-item batches, so that I can inspect each contained reference.
8. As a Windows user, I want single-item cards to expose their item directly without Manage Items, so that the same action is not duplicated.
9. As a Windows user, I want each management row to show its name, file/folder fallback visual, shortened path, and availability, so that I can identify the reference and its current usability.
10. As a Windows user, I want the full path in tooltip and accessibility text, so that shortening visual text does not hide the reference identity.
11. As a Windows user, I want to Pin, Unpin, or Remove one managed item, so that I can change a batch without removing unrelated items.
12. As a Windows user, I want to remove a Pinned Item explicitly, so that Pinning does not prevent deliberate cleanup.
13. As a Windows user, I want to drag one managed item directly, so that I do not have to transfer the entire batch.
14. As a Windows user, I want an individual item drag to keep Manage Items open while its batch remains, so that I can continue working through the batch.
15. As a Windows user, I want a 280-logical-pixel management popup even when the Shelf is 180 pixels wide, so that controls are not clipped by the parent surface.
16. As a Windows user, I want popup placement to prefer right, left, below, then above its originating card and remain inside the monitor work area, so that the popup stays usable near screen edges.
17. As a Windows user, I want only one Manage Items popup at a time, so that opening another batch replaces the previous management view.
18. As a Windows user, I want pointer opening not to force keyboard focus into the popup, so that inspecting content does not unexpectedly disrupt focus.
19. As a keyboard user, I want Enter or Space opening to focus Bulk Pinning, so that management begins at a predictable control.
20. As a keyboard user, I want Tab and Shift+Tab to cycle through Bulk Pinning, Close, and each row's Pin/Unpin and Remove controls, so that every management action is reachable.
21. As a keyboard user, I want explicit Close to restore focus to the originating Manage Items control when it still exists, so that I can continue navigating the Shelf.
22. As a Windows user, I want Escape to cancel an active item drag before closing Manage Items, so that cancellation does not also dismiss the management view.
23. As a Windows user, I want Manage Items to close when its batch disappears, the Shelf hides, resizing begins, or whole-batch drag starts, so that detached or obsolete management views do not remain.
24. As a Windows user, I want tri-state Bulk Pinning, so that Off, On, and Indeterminate describe all retained items rather than a separate batch pin flag.
25. As a Windows user, I want Bulk Pinning activation to Pin all Temporary/mixed batches and Unpin an all-Pinned batch atomically, so that no partial bulk transition is visible.
26. As a Windows user, I want Bulk Pinning to include Missing and Unavailable references still retained, so that lifecycle state does not depend on availability.
27. As a Windows user, I want Clear Temporary Items only when temporary content exists, so that the Shelf does not offer an irrelevant action.
28. As a Windows user, I want Clear Temporary Items to require confirmation and preserve Pinned Items across all batches, so that bulk cleanup cannot accidentally clear retained content.
29. As a Windows user, I want canceling Clear or encountering a persistence failure to preserve prior content, so that failed or unconfirmed actions cause no partial mutation.
30. As a Windows user, I want every removal action to leave source filesystem objects untouched, so that the Shelf remains a reference workspace rather than a deletion tool.
31. As a Windows user, I want native resizing from every edge and corner with an approximately 8-logical-pixel hit region, correct cursor, and accent feedback, so that the borderless Shelf remains discoverably resizable.
32. As a Windows user, I want width-only resizing to preserve automatic-height eligibility, so that changing columns does not become a manual-height decision.
33. As a Windows user, I want actual vertical/corner height changes to create a Manual Height Override, so that later drops and restarts respect my chosen height.
34. As a Windows user, I want programmatic growth and monitor/DPI clamping not to be mistaken for manual intent, so that automatic behavior does not overwrite my preference.
35. As a Windows user, I want pending empty-shelf sizing intent to persist with the first accepted batch, so that unsupported or duplicate input does not create durable sizing state.
36. As a Windows user, I want visible automatic growth not to shrink when other content remains, but reopening to recalculate it, so that live cleanup does not move the surface unexpectedly.
37. As a Windows user, I want removal of the final item to clear sizing intent and reset the next opening to 180 × 180, so that an empty Shelf starts from its default size.
38. As a Windows user, I want an installed 0.0.3 build with the same verified behavior, so that development-mode-only functionality is not presented as a completed release.

## Implementation Decisions

- Keep one Core Shelf State owner. Pinning, bulk lifecycle changes, individual removal, clearing, drag completion, and sizing intent all pass through its serialized mutation interface.
- Reuse the existing item-level `is_pinned` field and preferred-width/nullable-manual-height state. Do not add a batch pin field, compatibility layer, or speculative schema/configuration.
- Persist before exposing a changed projection. Construct replacement state within the durable transaction and preserve prior observable lifecycle/sizing state on failure.
- Derive Bulk Pinning from every retained item: all Temporary is Off; all Pinned is On; mixed is Indeterminate. Off/Indeterminate activation Pins all; On activation makes all Temporary. This operation does not reorder, reclassify availability, or resize content.
- Reuse existing Copy-only drag execution and prepared participant semantics. Current pin state protects participants from Temporary consumption. Explicit Remove Item/Batch can still remove pinned references.
- Preserve confirmed-Missing batch preparation and Unavailable retention from 0.0.2. Pinning changes lifecycle only; it does not create a new availability policy.
- Remove Item prunes an empty batch. Confirmed Clear Temporary Items removes only Temporary references in one durable operation across all batches. Existing Remove Batch is a regression path, not a new implementation deliverable.
- Use one transient native management popup owned by the Drop Shelf rather than stretching the Shelf or clipping a 280-pixel view inside a 180-pixel webview. Keep its width at 280 logical pixels, maximum height at 480, and rows in a bounded vertical scroller.
- Place the popup relative to the originating card in right/left/below/above order, then clamp to the monitor work area with a preferred 16-pixel margin. Convert logical and physical bounds using monitor DPI.
- Share projections from the same core across Shelf and management surfaces. Load full management projections on open and release them on close or owner disappearance; compact cards keep bounded previews. Do not introduce a second durable frontend state owner.
- Separate pointer and keyboard activation. Pointer opening does not force focus; keyboard opening focuses Bulk Pinning. Preserve the specified traversal, close/focus restoration, replacement, and Escape/active-drag guards.
- Management closure follows owner/batch lifetime, whole-batch drag, and native resize-start events. Individual item drag preserves the popup while the batch exists, including a batch that shrinks to one item.
- Classify user sizing intent at native input/event boundaries. A generic size-change event alone is not proof of manual height intent. Programmatic growth, DPI conversion, and bounds clamping must not produce Manual Height Override.
- Retain width-only versus height-changing intent distinctly. Persist logical dimensions while content exists; stage empty-shelf intent until the first accepted batch; clear it when the final item is removed. Preserve the established automatic row/height formulas and live-growth rules.
- Reuse installed Vue, Tauri, Windows bindings, and SQLite dependencies where sufficient. Introduce no general popup framework, alternative state store, or new package without a demonstrated unmet requirement.
- All controls introduced here have accessible names and usable keyboard focus. Full shell imagery, final theme resources, virtualization hardening, and motion polish remain the later presentation-conformance version.

## Testing Decisions

- The agreed permanent seam is the Core Shelf State interface backed by isolated real SQLite storage. Reuse existing temporary-database, reopen/restore, drag outcome, participant, and persistence-failure patterns.
- Test consumer-visible state and durability: pinned/temporary transitions, accepted/rejected/canceled Copy behavior, atomic bulk states, explicit removal and empty pruning, confirmed clear, rollback, width-only eligibility, manual-height retention, and final-item reset.
- Include meaningful boundaries: mixed lifecycle/availability, identical paths in different batches, state changes between preparation and completion, empty-shelf pending intent, and programmatic resize versus user height intent.
- Do not assert forwarding, event wiring, copied values, source text, component structure, or incidental wording. Add no redundant frontend test framework for these native surfaces.
- Prove native popup size/placement, no-focus pointer opening, keyboard traversal/restoration, drag dismissal ordering, resize hit regions, and cursor/outline behavior through the actual Windows app. Permanent core tests alone do not prove these surfaces.
- Finish with the installed Windows x64 artifact: verify lifecycle and sizing restore across process restart, item/batch Copy and cancellation, clear/removal source safety, and real management/resize interactions.
- Use explicitly owned test profiles and fixtures. Close only known owned processes, restore input state/temporary drive mappings, release handles, and remove only owned test data. Preserve the ambient per-user profile and recovery files.

## Out of Scope

- Reimplementing the existing Remove Batch, duplicate intake, SQLite/WAL setup, basic automatic growth, or Copy-only drag engine.
- Resident process entry points, tray, single-instance activation, global hotkey, sign-in startup, and Settings.
- Edge Rail and its management flyout, fullscreen rail policy, and Shake-to-open.
- Final shell icons/thumbnails, full virtualization/theme/motion conformance, database recovery, and installer update/uninstall coordination hardening.
- Arbitrary selection across batches, source-file deletion/move, clipboard history, accounts, sync, or application-owned content storage.
- Automatically advancing beyond 0.0.3 or creating a Git commit.

## Further Notes

- The user approved reuse of the existing core test seam and a version-focused vertical-slice plan. The Manage Items work is separated into pointer/item management and bulk/keyboard integration rather than splitting Rust and frontend into horizontal tickets.
- Publish this feature separately from the closed 0.0.2 effort so its parent spec and ticket history remain unchanged. Only the approved current version may be implemented.
- In this staged roadmap, 0.0.3 adds cancellation/closure ordering for its new surfaces. Resident shelf dismissal, final-empty surface hiding, and EdgeDocked transitions remain their later roadmap versions; do not substitute process exit or an unfinished rail for those contracts.
- Keep 0.0.3 Planned until implementation is explicitly approved. Completion requires synchronized version metadata, the NSIS artifact, installed-app smoke evidence, and updated roadmap implementation/verification records.
