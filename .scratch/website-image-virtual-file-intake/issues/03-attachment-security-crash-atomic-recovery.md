# 03: Attachment security and crash-atomic batch recovery

**What to build:** Make the complete multi-format image batch fail safely under real Windows trust policy, I/O, persistence, and process interruption. A valid post-scan batch becomes visible exactly once; a blocked, deleted, corrupted, partially promoted, or durably failed batch leaves no partial Shelf state and no unowned files. Restart and Recovery Backup preserve the same all-or-nothing ownership boundary.

**Blocked by:** 02: Bounded multi-format Website Image batch intake.

**Status:** resolved

**Parent specification:** [DropCove 0.0.24 — Website Image Virtual-File Intake](../spec.md)

**Testing seam:** The confirmed installed Windows Resident seam with real Attachment Services evidence, supported by injectable native trust/failure behavior and Core Shelf State temporary-profile SQLite/filesystem crash-window and Recovery Backup regressions.

**Demo path:** Accept a multi-image batch after a trust service mutates one image into another supported type, then repeat with one candidate rejected/deleted and prove no batch remains; terminate intake at grouped staging/promotion/Ready boundaries, restart, and observe either the complete ordered Ready batch or no visible batch with durable cleanup and preserved recovery evidence.

- [x] Ticket 02's supported formats, limits, names, ordering, partial validation skips, aggregate feedback, and exact-byte behavior remain intact.
- [x] Every accepted Website Image Virtual File uses mandatory non-interactive `IAttachmentExecute::Save`; there is no setting, prompt, allowlist, keep-anyway action, retry-without-scan path, or alternate acceptance route.
- [x] Only explicit HTTP(S) `CFSTR_INETURLW/A` supplies source evidence; unsupported/malformed schemes and URL-shaped text/HTML do not, and missing evidence uses Restricted Zone.
- [x] Successful Windows Attachment Services policy is authoritative when `Zone.Identifier` is present or absent under filesystem/enterprise policy; DropCove preserves existing security metadata and never synthesizes a replacement.
- [x] A trust service may alter bytes or image type. A final supported file within all limits is revalidated, keeps exact final bytes/animation/metadata, and receives a repaired extension.
- [x] Trust rejection, required interaction, file deletion, unsupported mutation, invalid mutation, or over-limit mutation rejects the complete drop and removes all candidates rather than preserving pre-scan bytes.
- [x] Descriptor/content read failure, zero-progress/short stream failure, staging write or flush failure, Attachment Services failure, post-scan read failure, database failure, promotion failure, and Ready failure each produce one categorized non-modal warning and no partial visible batch.
- [x] Failure before durable ownership removes every staging file and creates no Shelf, Managed File, pending-deletion, sizing, scroll, or Shake state.
- [x] The grouped Staged transaction creates one batch and every participating item/ownership row together; no subset can commit as a separate user-visible action.
- [x] Promotion failure for any participant compensates the complete unprojected batch, records durable cleanup responsibility for every relevant staging/final path, restores empty-Shelf sizing when applicable, and attempts immediate cleanup.
- [x] The grouped Ready transition is atomic across every participant; a Ready commit failure removes the complete unprojected batch and retains cleanup responsibility.
- [x] Startup reconciliation operates at Shelf Batch scope. It promotes and marks every recoverable participant Ready together or removes the entire unprojected batch; it never restores successful siblings while dropping a failed sibling.
- [x] Crash-window regressions cover termination before ownership, after grouped Staged commit, during multi-file promotion, after all promotions, and before grouped Ready commit.
- [x] Every crash window restores either one complete ordered Ready Shelf Batch with exact names/bytes/metadata/security evidence or no visible batch and no untracked staging/final file.
- [x] `Staged` image rows remain excluded from every projection before, during, and after reconciliation.
- [x] Recovery Backup preserves the failed database, present sidecars, complete managed root, complete staging root, security metadata where the filesystem retains it, and pending cleanup together before starting with clean active stores.
- [x] Incomplete Recovery Backup creation, preservation, rollback, clean-store creation, or startup reconciliation remains fatal and leaves original evidence intact.
- [x] Pending Managed File deletion remains durable across security/antivirus sharing failures and restart without restoring a removed Shelf Item.
- [x] Mixed external/Managed File failure scenarios prove that ownership relation—not browser identity, path prefix, filename, extension, source URL, Pin state, or batch membership—controls deletion.
- [x] The native trust adapter is injectable only at the established testing boundary; production behavior always uses Windows Attachment Services and permanent tests assert final state rather than COM helper ordering.
- [x] Actual Windows smoke observes HTTP(S) source and Restricted-Zone paths, policy-authoritative ADS presence/absence, and at least the real trust behavior available on the workstation; unavailable antivirus/policy cases are recorded as unobserved rather than inferred.
- [x] Native regressions, Recovery Backup regressions, frontend regressions, frontend production build, and cargo-xwin Windows build pass while release metadata remains `0.0.23`.
- [x] No Git commit is created without separate explicit user authorization.

## Verification evidence

- Tested injectable trust failures: second-image rejection drops the entire batch and leaves zero staging files; postscan file deletion produces classified error; mutation to JPEG repairs extension and keeps exact bytes; invalid and over-limit (>100 MiB) mutations reject the drop immediately.
- Missing `CFSTR_FILECONTENTS` index returns classified `Image I/O failed` error, cleans staging, and leaves prior Shelf untouched.
- Tested HTTP(S) provenance via wide and ANSI formats passing through to `SetSource`; unsupported schemes default to Restricted Zone.
- Core Shelf State crash-window regressions verify: staged transaction failure cleans up staging; sibling promotion failure compensates whole batch atomically; ready transaction failure rolls back cleanly; startup reconciliation recovers complete batches or rolls back if any sibling is corrupt; Recovery Backup preserves managed bytes and NTFS alternate data streams.
- Unavailable staging file I/O error fails startup safely without discarding unowned files.
- Full test suites passed: **172/172 native tests** and **13/13 frontend tests**; frontend production build and cargo-xwin compile succeeded. Version remains `0.0.23`.
