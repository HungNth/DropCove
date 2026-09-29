# 17: Prepare self-contained MSIX for Microsoft Store

**What to build:** After the EXE-installed application is stable, give maintainers a reproducible x64 self-contained MSIX/MSIXBundle submission path for Microsoft Store, with development identity and production Partner Center identity kept distinct.

**Blocked by:** 16: Add motion, accessibility, and full multi-monitor polish.

**Status:** ready-for-agent

- [ ] Release automation produces a versioned x64 self-contained MSIX/MSIXBundle from a clean checkout.
- [ ] Development builds use a self-signed certificate that matches the development manifest identity and is never presented as publicly trusted.
- [ ] Production automation requires Partner Center package identity and publisher values before generating a Store submission package.
- [ ] The submission artifact is checked for its Store package identity, version, architecture, self-contained runtime behavior, and Store submission requirements.
- [ ] Microsoft Store submission metadata and package validation are reproducible without storing private keys or secrets in the repository.
- [ ] Installation, upgrade, and uninstall are exercised against a Store-signed release candidate on supported Windows 11.
- [ ] No GitHub Releases, WinGet, Scoop, unpackaged, portable, x86, or ARM64 public artifact is produced.
- [ ] The application adds no telemetry, background updater, or release-check network request.