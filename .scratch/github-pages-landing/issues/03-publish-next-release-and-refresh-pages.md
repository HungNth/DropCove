# 03: Publish the next stable release and refresh Pages

**Parent specification:** [GitHub Pages landing and release delivery](../spec.md)

**What to build:** For the next separately approved stable DropCove version, turn its explicit version tag into one gated GitHub Release containing the exact validated Windows installer ZIP, then refresh the public landing page only after that asset exists. Users must see and download the new version atomically while the existing `v0.1.0` release remains unchanged.

**Blocked by:** 01 — Publish the v0.1.0 DropCove landing page; 02 — Validate future release packaging on GitHub Actions.

**Status:** ready-for-human

- [ ] Actual publication waits for separate human acceptance of the next stable DropCove candidate and explicit authorization for its commit, tag, and release; this ticket does not choose or auto-increment the version.
- [ ] A pushed tag matching exactly `vMAJOR.MINOR.PATCH` starts stable release delivery, and its normalized version must equal the canonical `VERSION` before publication work begins.
- [ ] Malformed tags, version mismatches, unsynchronized committed metadata, failed tests, failed cargo-xwin packaging, or invalid archive contents stop the release with no partial public result.
- [ ] The tag run uses the same proven dependency installation, frontend regression, cargo-xwin Tauri/NSIS build, ZIP creation, and one-installer archive checks as non-publishing validation.
- [ ] The freshly built package, crate, lockfile, Tauri, PE, installer, ZIP, tag, and release identities all report the same approved version.
- [ ] After every gate succeeds, the workflow creates one normal non-draft, non-prerelease GitHub Release for the already pushed tag and uploads `DropCove-Setup-<version>.zip`.
- [ ] The workflow refuses to replace or overwrite an existing release or asset with the same tag or filename; collisions fail safely for human review.
- [ ] The published ZIP contains exactly `DropCove-Setup-<version>.exe` at its root and is the same artifact produced by the successful gated run.
- [ ] Only after GitHub confirms the release asset upload does release delivery invoke the established Pages delivery interface.
- [ ] The resulting public site displays the same new version and its exact direct asset URL; it never points at an asset before that asset exists.
- [ ] A real-browser smoke activates the updated Download action, retrieves the new ZIP, and confirms the displayed version, URL, ZIP name, contained installer name, and archive bytes belong to the approved release.
- [ ] The new normal stable release is discoverable as the Latest Release by the existing Update Check without changing About Surface behavior.
- [ ] A failed release job leaves the previous release and Pages deployment usable; a failed post-release Pages deployment leaves the prior site live and reports the release/site mismatch for correction instead of deleting release history.
- [ ] Release publication and Pages deployment use separate least-required permissions; untrusted pull-request code receives neither authority.
- [ ] Concurrent Pages runs cannot let older content overwrite the release-triggered deployment, and publication work is not canceled after a public release begins.
- [ ] The existing `v0.1.0` tag, release, asset, and direct URL remain unchanged throughout the new release.
- [ ] Final evidence records the approved version and tag, test and cargo-xwin results, installer and ZIP identities, archive contents, release URL, asset digest where GitHub exposes it, Pages deployment run, public URL, and browser/download observations.

## Comments

Release workflow tag gating, collision check, `gh release create`, and post-release `pages.yml` call are implemented in `.github/workflows/release.yml`. Actual execution awaits completion of Tickets 01 and 02 on GitHub and a future user-approved version tag.
