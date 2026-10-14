# DropCove 0.0.5 — Edge Rail workflow delivery map

Version: 0.0.5
Implementation approval: approved by user on 2026-10-13
Parent: [Specification](spec.md)
Progress source: [Roadmap](../../docs/ROADMAP.md), version 0.0.5
Placement decision: [ADR-0002](../../docs/adr/0002-edge-rail-placement-ownership.md)

## Notes

- [01: EdgeDocked Rail Handle and Placement Settings](issues/01-edge-docked-handle-placement-settings.md) — `resolved`; 71 tests, cargo-xwin build and actual native handle/settings/rollback smoke passed.
- [02: Expanded Rail and Storage Intake](issues/02-expanded-rail-storage-intake.md) — `resolved`; focused TDD and native timer/OLE/scroll/projection evidence passed.
- [03: Rail Management and Copy Drag-Out](issues/03-rail-management-copy-drag-out.md) — `resolved`; 74 tests, cargo-xwin build and native outbound OLE/management/hold smoke passed.
- [04: Fullscreen, Display Reflow and Settings Transaction](issues/04-fullscreen-reflow-settings-transaction.md) — `resolved`; 75 tests, cargo-xwin build and native fullscreen/settings/drag-survival smoke passed.
- [05: Release 0.0.5](issues/05-release-0.0.5.md) — `resolved`; installer packaged, 75 tests passed, installed release smoke verified, ambient state restored.

Implementation dependency graph: `01 → 02 → 03 → 04 → 05`. Read-only research and disposable geometry/native prototypes may run in parallel when they do not mutate shared source or weaken a ticket gate; implementation tickets remain linear because each slice consumes the working surface and coordinator behavior delivered by its predecessor.

## Decisions-so-far

- The Resident Application Coordinator remains the highest product seam and gains `EdgeDocked`; Core Shelf State remains the only durable Shelf Item/lifecycle owner.
- Ordinary non-empty dismissal docks; empty dismissal and final-item removal hide every shelf surface. Sign-in startup may retain content while `Hidden`.
- The Edge Rail is a separate no-activate presentation. Keyboard access remains global hotkey → Drop Shelf; no rail-specific global keyboard hook is added.
- Open Shelf uses the monitor where the rail is currently docked, including temporary primary-display fallback, rather than stale remembered placement.
- Settings own `RailEdge`, `RailMonitorId` and `ShowRailOverFullscreen`; shelf state owns only the remembered actual dismissal monitor.
- The 0.0.5 JSON schema is a clean cutover with exactly five required top-level fields. Valid 0.0.4 settings follow the existing malformed-input path until explicit successful Save; no migration or compatibility shim is added.
- Ticket 01 persists the fullscreen field at its default but does not expose an actionable fullscreen control. Ticket 04 ships that runtime capability and then exposes the control.
- Fullscreen hiding closes rail Manage Items and unconfirmed Clear prompts, restores only the collapsed handle, and does not cancel an already-running native drag.
- Rail layout constants, timers and action set come from the canonical product specification; tickets do not invent alternative geometry.
- Permanent tests use the Resident Application Coordinator with real temporary Shelf/Settings stores and deterministic native adapters/manual clock. Actual focus, OLE, fullscreen, monitor/DPI and installer behavior require Windows smoke.
- No ADR beyond ADR-0002 is needed: the new decisions refine its placement ownership and reversible presentation behavior rather than selecting a new hard-to-reverse architecture.

## Fog

- Available hardware may not provide mixed-DPI displays or safe physical monitor disconnect/reconnect. Release evidence must state exact observed monitor/DPI limits and must not infer missing coverage.
- Windows fullscreen coverage, no-activate focus, OLE storage drag and notification behavior require native observation; DOM-only events or process existence are not substitutes.
- Visual polish is bounded to functional Edge Rail geometry, accessible names/states and theme-safe resources. Final icon/thumbnail resources, full accessibility conformance and Shake-to-open remain later work.
- Version metadata remains 0.0.4 until ticket 05. No 0.0.6 work starts during this effort.
