# 02: Persistence and Integration Safe State

**What to build:** Complete and prove the failure contract across Shelf mutations, settings persistence, global hotkey registration, Start with Windows, Shake-to-open, and remembered Edge Rail placement. Every failed operation must leave the prior durable and visible state intact, or report the actual retained state when native rollback cannot be completed.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.8 — Recovery and Distribution Hardening](../spec.md)

## Acceptance criteria

- [x] Failed Shelf Batch intake leaves no partial batch, item, ordering, sizing, or presentation mutation.
- [x] Failed item/batch pinning, removal, Clear Temporary, and successful-drag cleanup retain the prior visible and durable Shelf snapshot.
- [x] Failed preferred-size or remembered Edge Rail monitor persistence does not claim a value that was not durably stored; restart reflects the actual durable value.
- [x] Missing settings continue to load defaults silently; malformed or unreadable settings remain untouched, load defaults in memory, and warn without creating a Recovery Backup.
- [x] Failed settings-file replacement retains the prior persisted settings and prior active integrations when clean rollback succeeds.
- [x] Hotkey, Start with Windows, Shake-to-open, and rail-placement failures retain the prior safe configuration when possible.
- [x] Incomplete native rollback reports the actual retained configuration, keeps Save retryable, and never presents the requested candidate as fully applied.
- [x] Retry starts from the actual retained state rather than the failed candidate.
- [x] Failure-injection coverage includes each durable Shelf mutation, settings-file replacement, hotkey registration, startup registry update, Shake hook transition, rail placement, and representative combined failures.
- [x] Cross-layer tests compare the authoritative visible snapshot immediately after failure and the durable snapshot after restart.
- [x] No second transaction framework, compatibility layer, or speculative integration abstraction is introduced.

## Testing seam

Reuse the serialized Shelf store, existing persistence failure injection, settings transaction coordinator, and native integration adapter. Assert user-visible state and restart state; do not test mock forwarding or internal call order.

## Demo path

Inject one persistence failure and one incomplete integration rollback. Attempt the corresponding Shelf or Settings operation, observe prior state or the explicit actual-state warning, restart DropCove, and verify the durable state matches what was shown.

## Non-goals

Database corruption recovery, installer behavior, new Settings fields, integration telemetry, or changing successful-path product behavior.

## Answer

- Existing transaction/coordinator and malformed-settings behavior retained; added failure coverage for batch intake/removal, batch drag consumption, remembered rail monitor, and combined Shake application plus rail restoration failure. Reused existing settings-file tests instead of adding duplicate boundary coverage.
- Native targeted verification: eight Shelf failure-path tests passed; nine settings transaction tests passed, including incomplete rollback and retry from retained state.
- Actual embedded Windows application smoke (`tauri.localhost`, native IPC): injected SQLite triggers caused intake, item/batch pinning, item/batch removal, Clear Temporary, and sizing Save to fail. Every operation retained the prior authoritative snapshot; clean shutdown/restart restored the same durable snapshot.
- This is ticket-level proof. The full exact-artifact acceptance matrix remains assigned to ticket 05; no 0.0.8 release completion is claimed.
