# 01: Extract shared shelf visual presentation primitives

**What to build:** Refactor the existing Drop Shelf visual presentation so its bounded Shelf Batch preview, Shelf Item visual projection, and thumbnail realization lifecycle can be reused by another presentation without changing any user-visible Drop Shelf behavior. This is the prefactor that makes Edge Rail parity possible without duplicating thumbnail, fallback, cancellation, or ImageSource ownership logic.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Parent specification:** [Edge Rail Thumbnail and Management Parity](../spec.md)

- [ ] A reusable presentation primitive represents one Shelf Item visual with native-icon fallback, optional Windows Shell thumbnail, thumbnail/native-icon accessibility state, and no persisted derived content.
- [ ] A reusable bounded Shelf Batch preview represents one visual for a single-item Shelf Batch and at most the first three visuals for a multi-item Shelf Batch in item order.
- [ ] The existing visual coordinator/provider remains the only thumbnail/native-icon policy seam; no second thumbnail service, path-keyed thumbnail cache, background prefetcher, or filesystem watcher is introduced.
- [ ] Each presentation remains responsible for starting, canceling, and releasing its own realized visual requests; shared primitives do not retain Drop Shelf or Edge Rail visual trees.
- [ ] Drop Shelf cards and the anchored item popup retain their current preview composition, title/subtitle content, Pin/Unpin, Bulk Pinning, Manage Items, Remove, drag, focus, and dismissal behavior.
- [ ] Existing Drop Shelf lazy realization and release behavior remains bounded: closed popup rows and unrealized batch cards do not retain item thumbnail requests or ImageSource state.
- [ ] Drop Shelf and future Edge Rail callers continue to use the existing manager and drag service directly; no action controller, forwarding coordinator, public mutation interface, command bus, or compatibility layer is added.
- [ ] The existing visual-coordinator tests remain authoritative for image selection, native fallback, expected failure behavior, and cancellation; add permanent tests only for new consumer-visible shared visual policy.
- [ ] The complete automated suite passes without tests that assert private projection placement, XAML source, event forwarding, or resource-key plumbing.
- [ ] A targeted running Drop Shelf smoke shows unchanged single-image, non-image, folder, three-preview Shelf Batch, popup item, and drag behavior after the refactor.

## Testing seam

Use the existing visual coordinator as the automated seam and the running Drop Shelf as the presentation-regression seam. Manager and persistence behavior is unchanged and remains covered by its existing tests.

## Demo path

Open the Drop Shelf with one image, one non-image file, one folder, and a Shelf Batch containing at least three items. Show that card previews and the Manage Items popup look and behave the same before and after the prefactor, including Pin/Unpin and drag-out.

## Comments

- Shared projections, factory/eligibility rules, realization lifetime, bounded preview template, and Bulk Pinning template extracted. MainPage now consumes the shared implementation; provider callbacks capture only the storage dictionary, not the page.
- Release build: zero warnings/errors. Visual-coordinator tests: 5 passed. Full suite: 226 passed, zero failures/skips.
- Running isolated-profile smoke observed image thumbnail, file/folder native icons, three-layer batch preview, item popup metadata, and a durable popup Pin activation. Current surface evidence is build-output smoke, not installed Release qualification; real drag remains unobserved.
- Evidence: `artifacts/parity-ticket01-02-progress.json`, `artifacts/parity-ticket01-shelf.png`, `artifacts/parity-ticket01-popup.png`. Both review axes include the new untracked files and report no confirmed code findings. Changes remain uncommitted.
