# 00: Isolate real-application verification from the user's profile

**Parent specification:** [Reliable Native Drag Acceptance Specification, approved test-profile amendment](../spec.md#approved-test-profile-verification-amendment--2026-10-05)

**What to build:** Provide an explicit test-only launch profile so actual DropCove drag behavior can be verified without replacing live data, colliding with the resident normal instance, or writing Windows startup registration. This prerequisite does not change the source drag engine or approve native cutover.

**Blocked by:** None; user explicitly approved this scope.

**Status:** ready-for-human — implementation and scoped runtime verification complete; native design/release gates remain separate.

**Testing seam:** Actual launch/activation and Settings UI, storage through the existing manager/settings APIs, and actual popup drags correlated to instrumented primary/backing destinations. Existing automated suite remains the regression gate.

**Demo path:** Launch `DropCove.exe --test-profile <absolute-directory>` alongside the normal app. Save test settings, restart, and relaunch the same normalized profile; then run Copy/Reject/Escape/primary-exit cases with controlled backing destinations.

- [x] Explicit test launch uses a separate database/settings directory; invalid or normal-profile selection does not silently fall back to live storage.
- [x] Same normalized test profile shares its resident mutex/activation event; different test profiles coexist with each other and the normal instance. Default launch retains original paths and names.
- [x] Test startup, Settings save, and rollback route through a policy that prevents Windows startup-registration writes; saved test settings survive restart.
- [x] Real popup drag-out uses the existing WinUI engine and correlates exact offered path/item identity with primary and backing events; successful backing acceptance is not attributed to the failed primary.
- [x] Live profile bytes and startup Run value remain unchanged, the normal resident process stays running, and automated regressions pass. No NSIS, resource, keyboard, full destination-matrix, or native release qualification is inferred.
## Comments

### Approved implementation and verification
See [real-app verdict](../installed-profile-verdict.md), [25 attempt event streams](../installed-profile-evidence.json), and [final executable startup smoke](../final-profile-smoke.json). Release build/publish succeeded without new warnings/errors. Added 8 permanent `LaunchProfileTests` covering default legacy fallback, normalization, case-insensitivity, and fail-closed argument validation. Complete suite: 163 passed, 0 failed, 0 skipped, 439 ms. Independent Standards and Spec reviews each reported zero findings. Side-by-side test publish folder `TestProfileQualification` removed after verification. Native Tickets 02–04 and ADR 0003 still require a planning decision.
