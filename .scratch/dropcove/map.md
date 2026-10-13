# DropCove 0.0.2 Delivery Map

## Notes

Parent specification: [`spec.md`](spec.md)

- [`01-single-item-round-trip.md`](issues/01-single-item-round-trip.md) — `resolved`; single-item workflow verified.
- [`02-batch-intake-and-layout.md`](issues/02-batch-intake-and-layout.md) — `resolved`; native intake, duplicates, layout, and sizing verified.
- [`03-batch-availability-drag.md`](issues/03-batch-availability-drag.md) — `resolved`; availability, confirmation, and native Copy verified.
- [`04-release-0.0.2.md`](issues/04-release-0.0.2.md) — `resolved`; installer, sync, and packaged smoke verified.

## Decisions-so-far

- `0.0.1` is the accepted bootstrap baseline; `0.0.2` is the first product workflow.
- Delivery advances through sequential `0.0.x` versions, one approved version per cycle.
- `0.0.2` delivers the floating Drop Shelf round trip: drag-in, durable restore, item drag-out, availability-aware whole-batch drag-out, and Temporary Item consumption after accepted `Copy`.
- Completion requires real Windows 11 x64 smoke evidence and `DropCove-Setup-0.0.2.exe`.
- Pinning, Manage Items, resident integrations, Edge Rail, Shake-to-open, final visual/accessibility work, and recovery hardening remain deferred to later roadmap versions.
- Ticket 01 completed: SQLite-backed single-item intake, restore, reference removal, and native Copy-only drag-out; five native core checks and live Windows smoke passed. [Evidence](issues/01-single-item-round-trip.md#comments).
- Ticket 02 completed: native file/folder and mixed Shell intake, ordinal duplicate suppression, exact grid thresholds, and sizing restore; eight native core checks passed. [Evidence](issues/02-batch-intake-and-layout.md#comments).
- Ticket 03 completed: eleven core checks plus actual embedded UI/native Copy smoke, Missing cleanup, cancellation, no-Available handling, and restored-root drag. [Evidence](issues/03-batch-availability-drag.md#comments).
- Ticket 04 completed: `DropCove-Setup-0.0.2.exe` built, installed, and validated against parent demo path; version synced and 11 core checks rerun. [Evidence](issues/04-release-0.0.2.md#comments).

## Fog

- All planned vertical slices for `0.0.2` are complete and verified. Next step is user review before `0.0.3`.
