# 02: Qualify Duplicate Shelf Batch suppression in the installed Release

**Parent specification:** [Duplicate Shelf Batch Suppression Specification](../spec.md)

**What to build:** Complete the installed Release qualification gate for Duplicate Shelf Batch suppression. Prove with real Windows drops that exact duplicates are silent no-ops across the Drop Shelf, Edge Rail, persistence, shake restoration, geometry, and established scale conditions, while non-duplicate drops keep their current behavior.

**Blocked by:** 01: Silently suppress Duplicate Shelf Batches; inherited Native Residency gate; reliable native-drop input for installed qualification.

**Status:** needs-info

**Testing seam:** Use the self-contained installed Release with isolated test profiles and real Explorer or Desktop drag sources. Use the existing automated suite for manager, SQLite, and shake contracts; installed interaction and UI Automation are authoritative for native drop routing, status, viewport, window geometry, and Edge Rail behavior.

**Demo path:** Install the qualified Release, create a retained multi-item Shelf Batch, and repeat it in the same order, reversed order, and different casing on the Drop Shelf and Edge Rail. Demonstrate unchanged cards, status, viewport, bounds, and persistence; restore Hidden and EdgeDocked after duplicate shake drops; then add a partial-overlap batch and show normal creation. Restart and repeat the duplicate before exercising the established scale scenario.

- [x] The full automated suite passes with obsolete duplicate-permitting expectations removed and no unrelated contract weakened.
- [x] The qualified artifact is a self-contained win-x64 Release installed through the current installer flow, and the installed executable is verified against the publish output before smoke testing.
- [ ] Real Drop Shelf drops prove that exact single-item and multi-item duplicates, reversed-order duplicates, and casing-only duplicates create no new card or persisted Shelf Batch.
- [ ] A Drop Shelf duplicate leaves current status content, card realization, batch ordering, viewport position, window position, and window bounds unchanged.
- [ ] A duplicate at an Automatic Shelf Growth row boundary does not grow or otherwise resize the shelf; a subsequent non-duplicate batch still follows the established growth policy.
- [ ] Real Edge Rail drops prove that an exact duplicate creates no row, notification, refresh-visible disturbance, expansion, resize, or placement change.
- [ ] Shake-to-open qualification starts from Hidden and EdgeDocked, drops an exact duplicate, restores the precise prior state, and shows no unsupported-input warning.
- [ ] A successful non-duplicate shake drop continues to leave the Drop Shelf near the cursor for inspection.
- [ ] Restart qualification proves that a restored retained batch suppresses an exact duplicate while preserving batch identity, creation time, item identity, pin state, and order.
- [ ] Current-content qualification removes one item and proves comparison uses the remaining Shelf Items; removing the entire batch permits the same path set to be added again.
- [ ] A partially overlapping drop creates the complete incoming Shelf Batch in first-seen order, including paths already present in another batch.
- [ ] A non-duplicate payload mixing supported and unsupported entries retains the existing added/skipped reporting, while a duplicate path set with skipped unsupported entries remains silent.
- [ ] Existing duplicate batches seeded before launch remain present and unchanged; qualification performs no retroactive cleanup or merge.
- [ ] Source files and folders remain untouched throughout all duplicate, partial-overlap, restart, and shake scenarios.
- [ ] The established `100` Shelf Batch / `1,000` Shelf Item scenario remains responsive and does not regress the current working-set, idle CPU, interaction, or shelf-show latency gates.
- [ ] Qualification records the exact installed artifact, executable hash comparison, isolated profile inputs, exercised scenarios, observed outcomes, performance measurements, and environmental limitations.
- [ ] Any in-scope defect found during qualification is fixed and the affected automated and installed scenarios are rerun before the ticket is complete.
- [x] Temporary fixtures, diagnostic hooks, seeded profiles, and smoke scaffolding are removed or kept outside the shipped application before completion.
- [ ] Product documentation and the domain glossary remain consistent with the qualified runtime behavior.

## Comments

### 2026-10-09 — Installed payload verified; inherited residency gate still fails

- NSIS installation completed with exit code 0. Installed EXE and application/Core/Services/Native DLL SHA-256 values match the self-contained publish; installed application DLL SHA-256 is `05C8570A2C0356CE61C1379241FB4DE8B29DB1D9B633C649036776E64EFAAC15`.
- The new installed 100-batch/1,000-item, ten-PNG-preview, 30-show/dismiss-cycle run observed visible WorkingSet64 153.90 MB, post-dismissal 167.69 MB, visible and hidden CPU 0.0%, and native-hotkey-message-to-visible p95 44.24 ms. No forced GC, working-set trimming, or UI Automation was used in that measurement.
- The new artifact still fails the inherited <160 MB post-dismissal residency gate. The earlier artifact's failing residency measurement is historical context, not a pass for this artifact; no memory fix or threshold relaxation is included in this feature.
- [Scale evidence](../../../artifacts/duplicate-batches-scale.json) records artifact hashes, samples, measurement limits, responsiveness, and owned profile/process cleanup. Native-message-to-visible latency is not physical-keyboard or first-frame timing.
- Native qualification is blocked: the Explorer non-duplicate positive control did not create a batch, and the supplementary OLE source timed out after 240 seconds. No duplicate, overlap, status/scroll/geometry, or shake pass is inferred from unchanged database counts.
- [Qualification evidence](../qualification.json) records verified build/tests/reviews/installation, both blockers, unobserved configurations, and cleanup. Disposable native drivers and fixture profiles were removed; no production probe or alternate drag-in path was introduced.
- Resume: obtain a reliable real native-drop positive control, complete the unchecked native scenarios, and resolve the independent residency gate without weakening this specification. The user explicitly approved committing the implementation while retaining blocked qualification; this is not release approval.


