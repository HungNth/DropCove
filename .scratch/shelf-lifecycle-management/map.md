# DropCove 0.0.3 — Shelf lifecycle and management delivery map

Version: 0.0.3
Implementation approval: approved by the user on 2026-10-13
Parent: [Specification](spec.md)
Progress source: [Roadmap](../../docs/ROADMAP.md), version 0.0.3

## Notes

- [01: Single-item Pinning](issues/01-single-item-pinning.md) — `resolved`; native/core evidence in ticket Comments.
- [02: Manage Items pointer workflow](issues/02-manage-items-pointer-workflow.md) — `resolved`; native/core evidence in ticket Comments.
- [03: Bulk Pinning and keyboard management](issues/03-bulk-pinning-and-keyboard-management.md) — `ready-for-agent`; dependency 02 resolved, now unblocked.
- [04: Confirmed Clear Temporary Items](issues/04-confirmed-clear-temporary.md) — `ready-for-agent`; dependency 02 resolved, now unblocked.
- [05: Native sizing intent](issues/05-native-sizing-intent.md) — `ready-for-agent`; dependency 02 resolved, now unblocked.
- [06: Release 0.0.3](issues/06-release-0.0.3.md) — `ready-for-agent`; blocked by 03, 04, and 05.

## Decisions-so-far

- Reuse the existing Core Shelf State/real-SQLite permanent testing seam and native Windows smoke for presentation, focus, resize, and OLE behavior.
- Preserve the closed 0.0.2 spec/tickets and delivered artifact. Version metadata advances at the 0.0.3 release gate, not at an intermediate ticket.
- Existing `is_pinned` schema and Temporary-consumption predicate provide the foundation only; Pin/Unpin and verified pinned Copy retention are ticket 01 work. Reuse existing sizing/transactional state; introduce no batch pin field or alternative durable owner.
- Split Manage Items into two complete vertical workflows: native pointer/item management, then bulk/keyboard integration. Do not split core and frontend into horizontal implementation tickets or ship no-op controls.
- Existing Remove Batch is regression coverage only, not new feature work.
- Clear and sizing need the working management surface but do not depend on Bulk Pinning implementation; their blocker is 02, not 03.
- Use an actual native popup to satisfy the 280-pixel management width independently of the 180-pixel Shelf.
- Scope stays within the roadmap's 0.0.3 capabilities. Resident dismissal/hidden states, Edge Rail, Shake-to-open, visual hardening, and recovery remain later versions.
- Ticket 01 resolved with 14 passing native core tests, a successful frontend build, and embedded-app native Copy/rollback/restore proof. Its owned smoke data and throwaway script were removed.
- Ticket 02 resolved with 20 passing native tests, final frontend/embedded-release builds, and actual management row Copy, rollback, bounded placement, and lifecycle smoke. Owned resources and diagnostic scaffolds were removed; ticket 03 is the next staged slice.

## Fog

- No new product discovery question is open. The user approved the published 0.0.3 spec/tickets; staged delivery starts with ticket 01.
- Pointer versus keyboard activation and native user-resize intent require actual Windows surface evidence; core tests cannot substitute for it.
- Cleanup is restricted to explicitly owned test data/processes. Ambient per-user profile and recovery files must remain untouched.
