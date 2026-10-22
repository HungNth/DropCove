# 01: Save and restore Appearance Mode

**Parent specification:** [DropCove 0.0.19 — Appearance modes and palette refresh](../spec.md)

**What to build:** Let a user open Settings, choose System, Dark, or Light under a new Appearance tab, save that choice through the existing Settings transaction, and see the saved appearance applied to Settings, the Drop Shelf, and their native frames after Save and restart. Introduce the strict 0.0.19 settings cutover safely: preserve a pre-feature file, warn, load all defaults, and write the complete new schema only after explicit successful Save.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] Settings remains a decorated, fixed `420 × 520` logical-pixel singleton and presents horizontal General and Appearance tabs below its title.
- [x] General contains every existing setting; Appearance contains one System/Dark/Light selector; Settings opens on General and does not persist the selected tab.
- [x] Both tabs share the existing draft and one Save/Cancel footer; switching tabs does not discard any draft field.
- [x] The Appearance selector is one accessible radio group with Monitor, Moon, and Sun icons from the installed icon dependency, visible selection, arrow-key movement, and standard Tab/Shift+Tab entry and exit.
- [x] `AppearanceMode` is a required persisted field with exactly `System`, `Dark`, and `Light`; System is the default and unknown or missing values remain invalid.
- [x] A pre-0.0.19 settings file without `AppearanceMode` remains byte-for-byte untouched, loads the complete active defaults with a warning, and is replaced only after explicit successful Save.
- [x] The clean cutover resets the in-memory hotkey, startup, Rail, fullscreen, Shake, and Appearance values together; it does not partially recover old fields or introduce migration, compatibility, browser-local, registry, database, or second-file storage.
- [x] Successful Save commits `AppearanceMode` atomically with every other Settings field through the existing transaction and persisted JSON contract.
- [x] After successful Save, Settings and the Drop Shelf render the selected System, Dark, or Light appearance and their native frames match it.
- [x] Restart restores the saved mode before Settings or the Drop Shelf becomes visibly presented; no wrong-theme core frame is shown first.
- [x] System resolves the current Windows application Light/Dark preference; explicit Dark and Light remain fixed for the delivered core surfaces.
- [x] Closing and reopening Settings selects the persisted mode and starts on General.
- [x] Deterministic regressions cover defaults, valid and invalid values, complete-schema round trip, old-file preservation/warning, explicit-save replacement, and restart restoration.
- [x] A running Windows Resident/WebView2 smoke demonstrates Save and restart for System, Dark, and Light on Settings and the Drop Shelf with matching native frames.
- [x] Existing hotkey, startup, Rail, fullscreen, Shake, Shelf state, source-file safety, placement, and Resident lifecycle behavior remains unchanged.
- [x] Version metadata remains at 0.0.18 and no Git commit is created without separate explicit authorization.

## Comments

- 2026-10-22: Parent observed the old seven-field schema regression fail before the required `AppearanceMode` field and pass afterward. A separate transaction regression failed on retained `System` instead of `Dark`, then the complete transaction module passed **10/10** after appearance application and rollback were integrated. Settings Store module passed **9/9**, including every mode and rejected `Automatic` values with original-byte preservation.
- Frontend vue-tsc/Vite and cargo-xwin debug build passed. Actual owned Windows/WebView2 UI demonstrated General/Appearance tabs, accessible radios, Save Light/Dark/System, matching native frame attributes, and restart restoration. Settings viewport measured `420 × 520`; native Light frame attribute was `0`, Dark was `1`. Screenshot inspected on the real Settings WebView.
- One earlier focused transaction execution encountered native `ReplaceFileW` error `1175`; no retry/fallback was added. Direct native replacement probes succeeded, the complete transaction module passed, and actual application Save succeeded. This transient observation is not hidden.
- Smoke uses owned Profile Data and captures/restores the ambient startup Run value. Auxiliary-surface synchronization and final installed-candidate acceptance remain subsequent tickets; metadata remains `0.0.18` and changes are uncommitted.
