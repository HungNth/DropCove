# 04: Qualify the clean native drag cutover

**Parent specification:** [Reliable Native Drag Acceptance Specification](../spec.md)

**What to build:** Prove the complete clean native drag-out implementation in the installed self-contained Release application, remove all prototype and obsolete paths, update the blocked responsive-shelf evidence, and leave the user's original profile restored.

**Blocked by:** 03: Cut over whole-Shelf-Batch drag-out.

**Status:** blocked — Tickets 02–03 await a planning decision; existing WinUI passes controlled real-app acceptance. See [real-app verdict](../installed-profile-verdict.md). No native/NSIS release qualification claimed.

**Testing seam:** Run the complete automated suite once after source stabilization. Use installed Release UI Automation and direct pointer/keyboard interaction for source gestures, popup/flyout lifetime, real targets, and the disappearing-destination case. Use the established resource measurement procedure without changing its gates.

**Demo path:** Exercise one-item and whole-batch drag from the Drop Shelf and Edge Rail to Explorer/Desktop, a browser file input, VS Code, and a common chat application; repeat cancel, reject, disappearing target, files, folders, Temporary, Pinned, and mixed batches.

- [ ] The production tree contains one native drag engine and no WinUI source-drag fallback, dormant compatibility path, prototype diagnostic write, or shipped probe helper.
- [ ] The clean self-contained Release publish completes without new warnings or errors and is the artifact used for installed acceptance.
- [ ] Explorer/Desktop accepts file, folder, and multi-path Copy from both the Drop Shelf and Edge Rail.
- [ ] Edge or Chrome file input, VS Code, and at least one common chat application accept supported file drags at the same integrity level.
- [ ] Accepted destinations receive every offered path; source files and folders remain unchanged.
- [ ] Cancel, Reject, native failure, and destination stop during `DragEnter` retain every valid participating Path Reference.
- [ ] Accepted Temporary, Pinned, mixed, one-item, and whole-Shelf-Batch outcomes match the domain lifecycle contract.
- [ ] Item popup and Edge Rail flyout lifetime, one-click Settings, outside dismissal, and whole-batch closure remain correct after the gesture cutover.
- [ ] The previously failed clean popup keyboard replay is rerun with controlled foreground and focus; a failed replay remains an explicit blocker rather than being inferred from earlier evidence.
- [ ] The full automated suite passes after obsolete WinUI source-drag tests and assumptions are removed or updated.
- [ ] Installed resource qualification uses exactly 100 Shelf Batches / 1,000 Shelf Items, 30 seconds of quiescence, a 60-second CPU window, and six memory samples at 10-second intervals. Every memory sample must remain below 150 MiB and CPU at or below 0.1%; shelf-show p95 must remain at or below 150 ms.
- [ ] A failing keyboard, CPU, memory, latency, DPI, or target-compatibility gate remains red; no result is rounded, averaged away, cherry-picked, or waived.
- [ ] Responsive Drop Shelf Tickets 03 and 04 are updated only with exercised evidence. Final NSIS qualification is claimed only if every required gate passes.
- [ ] Probe processes and disposable runtime diagnostics are stopped/removed, referenced fixture sources remain unchanged, and the latest original profile is restored before delivery.
- [ ] All repository changes remain uncommitted until the user explicitly approves a commit.
