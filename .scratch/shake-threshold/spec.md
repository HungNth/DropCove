# Configurable Shake Threshold

Status: ready-for-agent
Design confirmation: approved by the user through `grill-with-docs` on 2026-11-12 (Q1–Q7)
Testing seam: existing native ShakeDetector unit seam and SettingsStore temporary-file seam; frontend `vue-tsc`/Vite build; real Settings surface and Shake-to-open gesture in one Windows Resident Instance
Baseline: 198 native tests passed, 0 failed before implementation
Release assignment: none — no version bump or installer requested
Ticket decomposition: two sequential vertical tickets `01 → 02`: [01 — Native threshold contract and detector](issues/01-native-shake-threshold.md), [02 — Settings UI, canonical documentation, and Windows acceptance](issues/02-settings-ui-and-acceptance.md)
Implementation authorization: separate user gate after ticket approval

## Problem Statement

Shake-to-open currently triggers too easily during ordinary left-button drag gestures such as selecting text. The default Normal profile accepts movement samples of only 10 px and triggers after two direction reversals—three accepted movement strokes—inside 600 ms. A small involuntary back-and-forth movement can therefore show the Drop Shelf.

The Settings UI exposes abstract Low, Normal, and High sensitivity profiles rather than the actual number of movements needed. Users cannot directly choose the trade-off between deliberate activation and accidental triggering.

## Solution

Replace `ShakeSensitivity` with a direct integer `ShakeThreshold`. The threshold is the number of accepted movement strokes required to trigger Shake-to-open:

- first accepted stroke establishes direction;
- each subsequent opposite-direction stroke counts one reversal;
- trigger after `ShakeThreshold - 1` reversals;
- every accepted stroke must travel at least 18 px on one dominant axis;
- all required strokes must complete within `max(800 ms, 250 ms × ShakeThreshold)`;
- trigger cooldown remains 500 ms.

The valid threshold range is `3..=8`; the default is 5 strokes (4 reversals, 1250 ms). Settings exposes the exact value through a select control.

## Persisted Contract

Clean cutover:

```json
{
  "ShakeEnabled": true,
  "ShakeThreshold": 5
}
```

`ShakeSensitivity` is removed from Rust, TypeScript, IPC, tests, and canonical documentation. `Settings` continues to use `#[serde(deny_unknown_fields)]`; therefore a previous file containing `ShakeSensitivity`, a file missing `ShakeThreshold`, or a threshold outside `3..=8` is malformed and loads the complete current defaults through the established SettingsStore warning path. No migration, alias, fallback, dual-write, or compatibility field is introduced.

`Settings::validate()` rejects values outside `3..=8`, covering both loaded data and runtime writes. The field remains a `u8` in Rust and a numeric value in TypeScript/IPC.

## Settings UI

Replace the sensitivity selector with:

- label: **Shake threshold**;
- supporting text: **Number of back-and-forth movements required to trigger**;
- options: `3 shakes`, `4 shakes`, `5 shakes (Default)`, `6 shakes`, `7 shakes`, `8 shakes`.

The control remains disabled when Shake-to-open is disabled or Settings is busy. Failed IPC restores the authoritative snapshot value exactly as today.

## Decisions

1. One shake means one accepted directional movement stroke. Five strokes require four reversals.
2. Replace Low/Normal/High with an exact user-selectable threshold from 3 through 8; default 5.
3. Detection window is `max(800, 250 × threshold)` milliseconds.
4. Minimum movement is fixed at 18 px for every threshold.
5. Persist `ShakeThreshold: u8`; remove `ShakeSensitivity`; old contracts reset to defaults without migration.
6. Use the approved English Settings labels and six select options.
7. Update `docs/SPECIFICATION.md` sections 7.2, 16.1, and 16.4; no ADR.

## Acceptance Criteria

- Default Settings contain `shake_enabled = true` and `shake_threshold = 5`.
- Threshold validation accepts 3 and 8, rejects 0–2 and 9–255, and invalid persisted input loads complete defaults with the established warning.
- Three selected strokes trigger only at threshold 3; five selected strokes trigger only at threshold 5; eight selected strokes trigger only at threshold 8.
- Four strokes do not trigger the default threshold; an accepted fifth stroke does.
- Movement below 18 px is ignored.
- Threshold 3 allows 800 ms; threshold 5 allows 1250 ms; threshold 8 allows 2000 ms. Expired sequences reset without triggering.
- Axis changes still reset the reversal run; left-button requirement, cooldown, overflow reset, release behavior, and Shake session restoration remain unchanged.
- Settings Save/Cancel and rollback apply threshold atomically with the other settings.
- The built Settings UI shows the six approved options and persists a changed value after Save/reopen.
- `cargo xwin build`, native tests, frontend typecheck/build, and frontend tests pass with no new warnings.

## Out of Scope

- Proving that another application is performing a file drag.
- Per-axis thresholds, distance customization, timing customization, acceleration/velocity analysis, telemetry, or automatic calibration.
- Keeping Low/Normal/High as aliases or adding a Custom profile.
- Version bump, release, installer, glossary term, or ADR.
