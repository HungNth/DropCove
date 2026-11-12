# 02: Settings UI, canonical documentation, and Windows acceptance

**Parent specification:** [Configurable Shake Threshold](../spec.md)

**What to build:** Replace the sensitivity selector with the exact Shake Threshold selector, update the TypeScript/IPC contract and canonical specification, then verify the complete workflow on Windows.

**Blocked by:** 01.

**Status:** ready-for-agent

**Testing seam:** Frontend `vue-tsc`/Vite build and existing Node tests; actual Settings surface and real Shake-to-open gesture in one Windows Resident Instance.

**Demo path:** Open Settings → General, verify threshold 5 is selected, choose 8 and Save, reopen Settings to confirm persistence, then choose 3 and verify the real gesture activates after the third accepted movement while normal text selection does not activate with the default 5.

- [ ] TypeScript snapshots and `set_shake` invocation use numeric `shakeThreshold`; no `ShakeSensitivity` type or Low/Normal/High value remains.
- [ ] Settings shows label `Shake threshold`, supporting text `Number of back-and-forth movements required to trigger`, and options 3–8 with `5 shakes (Default)`.
- [ ] The selector remains disabled when Shake is disabled or Settings is busy; failed IPC restores the authoritative numeric snapshot value.
- [ ] Save, Cancel, rollback, and reopen preserve the selected threshold exactly.
- [ ] `docs/SPECIFICATION.md` sections 7.2, 16.1, and 16.4 describe the numeric threshold, formulas, defaults, valid range, and clean-cutover JSON contract; no ADR or glossary change.
- [ ] Frontend typecheck/build and Node tests pass; native build/tests remain green.
- [ ] Real Windows inspection confirms the six options, default 5, persistence after Save/reopen, and no accidental activation during ordinary text selection with the default threshold.
