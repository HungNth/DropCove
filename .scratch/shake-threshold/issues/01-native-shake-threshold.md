# 01: Native Shake Threshold contract and detector

**Parent specification:** [Configurable Shake Threshold](../spec.md)

**What to build:** Replace the native Low/Normal/High sensitivity model with a validated numeric Shake Threshold and make ShakeDetector enforce the selected movement count, fixed 18 px stroke distance, and threshold-scaled detection window.

**Blocked by:** None.

**Status:** ready-for-agent

**Testing seam:** SettingsStore temporary files plus deterministic `ShakeDetector::process` pointer-event sequences.

**Demo path:** Start with defaults, hold the left button, perform four accepted alternating 18+ px strokes without a trigger, then perform the fifth stroke and observe one trigger. Repeat with thresholds 3 and 8.

- [ ] `ShakeSensitivity` is deleted; `Settings` stores `shake_threshold: u8`, default 5, and snapshots/commands/transactions carry the numeric field.
- [ ] `Settings::validate()` accepts only `3..=8`; invalid runtime writes reject atomically.
- [ ] Persisted JSON uses `ShakeThreshold`; old `ShakeSensitivity`, missing threshold, unknown fields, and out-of-range values use the established complete-default warning path without migration or fallback.
- [ ] Detector requires `threshold - 1` reversals, fixed 18 px accepted movement, and `max(800, threshold × 250)` ms.
- [ ] Existing left-button, axis reset, event overflow, cooldown, release, hook lifecycle, and Settings rollback behavior remain unchanged.
- [ ] Regression tests cover boundaries 3/5/8, default four-stroke rejection/fifth-stroke trigger, 17/18 px movement, 800/1250/2000 ms windows, invalid persisted values, and clean-cutover reset of the old contract.
- [ ] `cargo xwin build` succeeds without new warnings; native suite passes.
