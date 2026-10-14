# DropCove 0.0.6 — Shake-to-open delivery map

Version: 0.0.6
Implementation approval: approved by user on 2026-10-14
Parent: [Specification](spec.md)
Progress source: [Roadmap](../../docs/ROADMAP.md), version 0.0.6

## Notes

- [01: Gesture Hook, Detector, Session and Settings](issues/01-gesture-hook-detector-session-settings.md) — `resolved`; 82 tests, cargo-xwin build and live Windows hook/profile/settings smoke passed.
- [02: Storage Outcome Restoration and Failure Resilience](issues/02-storage-outcome-restoration-failure-resilience.md) — `resolved`; real OLE `DoDragDrop` acceptance, duplicate-only, unsupported, and `DragLeave` restoration passed.
- [03: Release DropCove 0.0.6](issues/03-release-0.0.6.md) — `resolved`; final 82-test suite, NSIS installation, native product/failure/retry verification, uninstallation, and ambient restoration passed.

Implementation dependency graph: `01 → 02 → 03`. The first slice delivers a working gesture end to end; the second closes every OLE/restoration/failure boundary; the third advances metadata only after behavior is verified.

## Decisions-so-far

- The canonical roadmap/specification is approved scope; no additional product decision is required before implementation.
- One Windows adapter owns the low-level hook, bounded queue, detector, and native lifecycle on a dedicated event-driven message-loop thread. Only detector results reach the existing main-window UI receiver; Resident/Window/Settings operations never run on the hook thread. No polling or dependency is introduced.
- The Resident Application Coordinator owns the transient Shake session; Core Shelf State remains the only intake/lifecycle owner.
- Gesture presentation does not activate during the in-progress drag. Accepted intake promotes to normal Visible state only after the drop result is known.
- The 0.0.6 settings schema is a strict seven-field clean cutover with `ShakeEnabled: true` and `ShakeSensitivity: Normal` defaults.
- Existing settings transaction gains explicit Shake apply/restore operations rather than a second transaction or generic integration registry.
- No ADR is required; implementation follows the settled module boundaries and reversible adapter design.

## Fog

- API contracts verified against Microsoft Learn; corrected unsupported claims about a 50-microsecond deadline, mandatory injection filtering, and zero-risk thread behavior in the research note.
- Native pointer evidence uses injected input; physical mouse polling-rate variation and mixed-DPI hardware remain unobserved limits. Deterministic timestamp/geometry tests cover the exercised contracts without claiming those hardware runs.
- Metadata is now 0.0.6. Final installer/installed hashes and precise fault-injection evidence live in ticket 03. No 0.0.7 implementation or Git commit was started.
