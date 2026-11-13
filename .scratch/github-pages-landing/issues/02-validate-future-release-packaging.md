# 02: Validate future release packaging on GitHub Actions

**Parent specification:** [GitHub Pages landing and release delivery](../spec.md)

**What to build:** Give maintainers a safe manual validation run on the real GitHub Actions runner that proves the future DropCove release path through version validation, existing regressions, cargo-xwin Windows packaging, ZIP creation, and artifact inspection. The run must produce a temporary Actions artifact for inspection while making no public release or Pages change.

**Blocked by:** None (can start immediately).

**Status:** ready-for-human

- [ ] A maintainer can explicitly start one non-publishing validation run on the hosted GitHub Actions runner.
- [x] Validation accepts only an effective `vMAJOR.MINOR.PATCH` release tag whose normalized version equals the trimmed canonical `VERSION`.
- [x] The canonical synthetic tag is accepted, while malformed and version-mismatched values fail before any package can be published.
- [x] Committed package, crate, lockfile, Tauri, and canonical version metadata must already be synchronized; validation fails rather than publishing source changes produced by synchronization or build steps.
- [x] JavaScript dependencies install from the committed pnpm lockfile and the complete existing frontend regression suite passes.
- [ ] The established Tauri/NSIS build runs with cargo-xwin for `x86_64-pc-windows-msvc`; it does not switch to a Visual Studio compiler workflow.
- [ ] The freshly built installer identity and filename agree with the canonical version.
- [x] Validation creates `DropCove-Setup-<version>.zip` containing exactly `DropCove-Setup-<version>.exe` at the archive root and rejects any missing, stale, additional, or incorrectly named entry.
- [ ] The versioned ZIP is uploaded only as a temporary GitHub Actions artifact that can be downloaded and inspected from the validation run.
- [ ] Validation mode never creates or changes a Git tag, GitHub Release, release asset, Pages artifact, Pages deployment, or public download link.
- [x] The workflow defaults to read-only repository permissions and receives no release or Pages publication authority in validation mode.
- [ ] A repeated validation run is independent of prior local or hosted build output and proves the artifact came from that run's checked-out revision.
- [ ] Verification records the Actions run, exact test result, cargo-xwin package result, installer and ZIP names, archive contents, artifact size and digest where GitHub exposes them, and confirms that public releases and Pages remained unchanged.

## Comments

Local release verification and ZIP packaging/inspection (`scripts/release-check.mjs`, `tests/releaseCheck.test.mjs`) passed on 2026-11-13, alongside `pnpm build` and `pnpm test` (`94/94`). Hosted GitHub Actions `workflow_dispatch` execution remains pending human commit/push authorization.
