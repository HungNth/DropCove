# 02: Preview and roll back Appearance drafts

**Parent specification:** [DropCove 0.0.19 — Appearance modes and palette refresh](../spec.md)

**What to build:** Make Appearance selection an immediate but reversible Settings preview. The user sees System, Dark, or Light on Settings, the Drop Shelf, and their native frames before Save; Save promotes the preview to committed state, while Cancel, native close, teardown, or failed Save restores the last committed appearance and the previous complete Settings state.

**Blocked by:** 01: Save and restore Appearance Mode.

**Status:** resolved

- [x] Selecting System, Dark, or Light immediately previews that mode on Settings, the Drop Shelf, and their native frames without writing settings bytes.
- [x] The focused tab or Appearance option retains focus while preview updates; applying a preview does not remount Settings or replace its control tree.
- [x] Switching General and Appearance retains the complete shared draft and current preview.
- [x] The Resident Instance remains the single appearance authority; preview is a projection of the existing Settings draft, not a second committed state owner.
- [x] The last committed Appearance Mode is captured as rollback authority before preview starts; the already-previewed candidate is never mistaken for the previous committed state.
- [x] Successful Save commits the complete Settings candidate, leaves the candidate appearance active, and clears preview ownership without a visual revert/reapply flash.
- [x] Cancel restores every delivered core WebView and native frame to the committed mode and leaves persisted bytes unchanged.
- [x] Native Settings close behaves like Cancel for Appearance and every other unsaved draft field.
- [x] Settings teardown during Resident exit cannot leave preview state as the next startup state.
- [x] A failed Save restores the committed appearance, previous integration state, active hotkey state, startup state, Rail state, Shake state, and original settings bytes through the existing rollback contract.
- [x] If rollback cannot fully restore a live integration, the warning reports the actual retained safe state rather than claiming a complete rollback.
- [x] A core surface destroyed during fan-out is treated as no longer live; failure to update a still-live owned core surface is not silently ignored.
- [x] Reopening Settings after Cancel, native close, failed Save, or restart selects the committed mode rather than the discarded preview.
- [x] Deterministic coordinator and transaction regressions cover preview-without-write, Save, Cancel, native close, successful rollback, incomplete rollback reporting, and locked or rejected settings-file replacement.
- [x] Actual Windows Resident/WebView2 smoke observes immediate preview and restoration on Settings, Drop Shelf, and matching native frames while preserving focus and window bounds.
- [x] Ticket 01 persistence and cutover behavior remains unchanged; auxiliary-surface fan-out is completed by Ticket 03.
- [x] Version metadata remains at 0.0.18 and no Git commit is created without separate explicit authorization.

## Comments

- 2026-10-22: Parent observed the committed-baseline regression fail on restored `Dark` instead of committed `System`, then pass after production rollback reads the Settings session's committed mode rather than the active native preview. Complete transaction module: **12/12**, including retained `appearance=Light` reporting when native restoration fails.
- Real owned Windows/WebView2 smoke used native radio ArrowLeft selection and observed immediate Light preview while focus remained on the Light radio. General/Appearance switching retained the draft. Persisted settings bytes stayed unchanged through preview and Cancel.
- Real native settings-file share locking made Save fail with Win32 error `32`; the Settings error was visible, both core native frame attributes returned to Dark (`1`), the committed `System` radio was restored, and original file bytes were unchanged.
- A native `WM_CLOSE` restored committed System. Successful Save committed a Light preview; a later unsaved Dark preview was present during Cooperative Shutdown, and restart restored Light with byte-identical committed settings. Owned shutdown exited `0`; ambient startup Run bytes were restored. Auxiliary fan-out remains Ticket 03.
