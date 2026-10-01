# 17: Prepare self-contained MSIX for Microsoft Store

**What to build:** After the EXE-installed application is stable, give maintainers a reproducible x64 self-contained MSIX/MSIXBundle submission path for Microsoft Store, with development identity and production Partner Center identity kept distinct.

**Blocked by:** 16: Add motion, accessibility, and full multi-monitor polish.

**Status:** ready-for-agent

- [x] Release automation produces a versioned x64 self-contained MSIX/MSIXBundle from a clean checkout.
- [x] Development builds use a self-signed certificate that matches the development manifest identity and is never presented as publicly trusted.
- [x] Production automation requires Partner Center package identity and publisher values before generating a Store submission package.
- [ ] The submission artifact is checked for its Store package identity, version, architecture, self-contained runtime behavior, Store submission requirements, and Windows App Certification Kit results.
- [x] Microsoft Store submission metadata and package validation are reproducible without storing private keys or secrets in the repository.
- [ ] Installation, upgrade, and uninstall are exercised against a Store-signed release candidate on supported Windows 11.
- [x] No GitHub Releases, WinGet, Scoop, unpackaged, portable, x86, or ARM64 public artifact is produced.
- [x] The application adds no telemetry, background updater, or release-check network request.

## Comments

- 2026-10-01: Added the separate `scripts/build-msix.ps1` pipeline and `packaging/msix/AppxManifest.template.xml`. Development output is signed with an ephemeral self-signed `CN=DropCove Development` certificate; the PFX is created only under the OS temporary directory and removed before package validation. The public `.cer` is emitted for local test trust.
- 2026-10-01: Development build `0.1.0.0` produced and validated `DropCove-0.1.0.0-x64.msix` and `DropCove-0.1.0.0-x64.msixbundle`. Reports confirmed `DropCove.Development`, `CN=DropCove Development`, version `0.1.0.0`, x64 architecture, packaged-classic-app entry point, `runFullTrust`, and self-contained runtime files. Package and bundle contained no private-key material.
- 2026-10-01: Production generation fails closed before publishing when `-IdentityName` or `-Publisher` is absent, and rejects the development identity. Production artifacts are explicitly marked `UnsignedStoreSubmission`; Microsoft Store signing remains external.
- 2026-10-01: Production version guards reject `0.1.0.0` and `1.0.0.1` before output creation; valid Store versions require a nonzero major component and revision `0`. The same `-StoreSubmission` rule is forwarded to both MSIX and MSIXBundle validation reports.
- 2026-10-01: Store-signed release-candidate installation, upgrade, and uninstall remain unverified because no Partner Center-signed candidate is available in this environment.
- 2026-10-01: Final Production builds now fail closed if Windows App Certification Kit is unavailable. When available, the pipeline runs `appcert.exe reset` and `test` against both the x64 MSIX and x64 MSIXBundle, parses both XML reports for explicit pass/fail outcomes, and records both report names/statuses. This environment has no `appcert.exe`, so Store-certification validation remains unchecked; earlier custom-validation-only Production artifacts are stale.
- 2026-10-01: Development package install smoke imported the emitted certificate into `CurrentUser\TrustedPeople` and confirmed the certificate was present immediately before `Add-AppxPackage`; this host still rejected the signed bundle with `0x800B0109` (untrusted root). No package or certificate was left installed. Runtime launch was therefore not claimed.