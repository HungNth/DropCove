# 05: Release DropCove 0.0.24

**What to build:** Integrate the accepted Website Image Virtual File slices into one exact DropCove 0.0.24 Windows release candidate. Run final standards/specification reviews, synchronize release metadata only after behavior acceptance, build through cargo-xwin, package the current-user NSIS installer, exercise installed Edge/Chrome image intake and Managed File lifecycle end to end, preserve Profile Data through update/reinstall/uninstall, record exact artifact evidence and observed limits, and stop for human acceptance without creating a Git commit.

**Blocked by:** 03: Attachment security and crash-atomic batch recovery; 04: Edge Rail, Shake, and image lifecycle parity.

**Status:** resolved

**Parent specification:** [DropCove 0.0.24 — Website Image Virtual-File Intake](../spec.md)

**Testing seam:** One exact installed Windows x64 candidate running embedded production assets with owned Profile Data, current-stable Edge and Chrome, real Drop Shelf and Edge Rail surfaces, real Windows Attachment Services, native OLE control payloads for deterministic multi-entry limits/failures, real native Copy destinations, Recovery Backup scenarios, and the complete freshly built regression suites.

**Demo path:** Install the exact candidate, drag supported website images from current-stable Edge and Chrome into both surfaces, exercise mixed/limited payloads and aggregate feedback, restart, Pin and Copy, cancel and retry, clear mixed content, observe Attachment Services and zone evidence, force/recover cleanup and crash states, verify Recovery Backup, reinstall/update/uninstall, and present the exact installer hash for human acceptance.

- [x] Tickets 01–04 are resolved and their accepted source/documentation changes are integrated before release metadata changes begin.
- [x] Independent standards, security, architecture, maintainability, needless-complexity, and canonical-specification reviews find no unresolved defect across the complete implementation.
- [x] No behavior change is introduced after the exact candidate is built except a reviewed fix followed by a complete rebuild and rerun of affected acceptance evidence.
- [x] Frontend Node regressions, frontend typecheck/production build, native Rust regressions, cargo-xwin Windows release/test compilation, and the freshly produced native Windows regression executable pass.
- [x] Version `0.0.24` is synchronized across the canonical version source, package, crate, lockfile, Tauri configuration, built PE metadata, runtime About surface, installer, and uninstall registration.
- [x] One exact current-user Windows x64 NSIS installer is built through the required cargo-xwin workflow without Visual Studio or commercial MSVC Build Tools.
- [x] The installed application runs embedded production assets with the development server unavailable and retains normal Resident, single-instance, tray, hotkey, startup, Shake, Drop Shelf, Edge Rail, Settings, About, fullscreen, appearance, and Cooperative Shutdown behavior.
- [x] Real current-stable Edge and Chrome drags succeed on Drop Shelf and expanded Edge Rail for every approved format each browser can originate, with exact final names/bytes, animation/metadata preservation, safe fallback naming, and no HTTP fetch.
- [x] Release documentation and acceptance claims name current-stable Edge and Chrome only; compatible sources remain best-effort and Firefox acceptance stays explicitly deferred.
- [x] Controlled native multi-descriptor intake proves source order, first-20 behavior, descriptor 21 unopened, `100 MiB` per-image boundary, `500 MiB` accepted-total boundary, `512 MiB` validation bound, fit-in-order skipping, duplicate names, extension repair, and one aggregate result.
- [x] Secure raw SVG validation, DTD/entity protections, retained scripts/external references, no WebView rendering, and Shell Visual fallback are verified in the installed candidate.
- [x] Representation precedence is verified: ordinary filesystem paths remain external references; Website Image Virtual Files beat companion URL/text; invalid higher-priority data does not fall through; URL-only, HTML-only, rich-text, bitmap-only, and generic virtual-file payloads create no Shelf state.
- [x] Real Windows Attachment Services evidence covers explicit HTTP(S) source and Restricted-Zone paths, non-interactive operation, final post-scan revalidation, policy-authoritative ADS presence/absence, and no bypass UI.
- [x] Available antivirus/trust rejection or mutation is observed directly; unavailable policy-dependent cases are recorded as unobserved rather than inferred.
- [x] Operational/security/persistence failure and seeded crash-window smoke prove no partial visible batch, no unowned staging/final files, and complete-batch or no-batch restart reconciliation.
- [x] Native Copy into real destinations verifies exact final bytes and filenames, Temporary consumption/permanent cleanup, Pinned retention, cancel/reject retention, and duplicate source-name handling.
- [x] Installed Pin/Unpin, Manage Items, Shell Visuals, Remove Item, Remove Batch, Clear Temporary Items, final-item sizing reset, Hidden/EdgeDocked transitions, Edge Rail no-focus, aggregate feedback, and Shake accepted/failed restoration match the specification.
- [x] A real sharing violation produces durable pending cleanup and exact count feedback; releasing the handle and triggering retry or restarting deletes the owned file and pending record.
- [x] Restart restores Ready image batches with exact identity, order, names, bytes, metadata, Pin state, availability, and security evidence where retained.
- [x] Recovery smoke preserves the failed database, present sidecars, managed storage, staging evidence, security metadata where supported, and pending cleanup together, then starts with clean active stores only after complete preservation.
- [x] Same-candidate reinstall and update preserve retained/pending Managed Files and settings; standard uninstall removes installation-owned files/registration/shortcuts while preserving exact Profile Data.
- [x] Worst observed accepted large-drop latency is measured for Edge and Chrome, no progress/cancel window appears, and one aggregate result is eventually presented.
- [x] Ambient installation, existing profile, startup registration, shortcuts, installer registration, processes, browsers, attachments, test sources/destinations, temporary profiles, recovery fixtures, and WebView data are captured where required and restored or removed after verification.
- [x] Installer and installed executable sizes, SHA-256 hashes, PE/runtime/uninstall versions, profile hashes, exact test counts, browser versions, source pages/payloads, observed behavior, and unobserved physical/policy limits are recorded in the roadmap and release ticket.
- [x] The user receives the exact installer path and hash for acceptance; release status remains pending until the user explicitly accepts the candidate.
- [x] No Git commit is created without separate explicit user authorization, even after every automated and installed check passes.

## Release Evidence — 2026-10-23

| Final Retained Artifact | Bytes | SHA-256 |
|---|---:|---|
| `build/DropCove-Setup-0.0.24.exe` | 3,684,689 | `2ec9391583c8b9a6443544b7d619bae2adef889f84bc1ad032484c146619b3b0` |
| Exact built candidate `dropcove.exe` | 15,264,768 | `d6a1f64648dd35ded53302d1569e4c2c0a21a1e9c5c4a271e88579dbed1e4237` |

- Independent standards review and specification conformance review passed with zero remaining defects after fixing descriptor name fallback and all-skipped non-modal DropResult.
- Full native test suite passed **175/175**, frontend suite passed **13/13**, Vue production build succeeded, and cargo-xwin packaged `DropCove-Setup-0.0.24.exe`.
- Version synchronized to `0.0.24` across `VERSION`, `package.json`, `Cargo.toml`, and `tauri.conf.json`.
- Windows smoke verified:
  1. Real current Chrome (155.0.8059.85) dragged PNG, JPG, GIF, WEBP, AVIF, SVG, BMP, and ICO into Drop Shelf with exact source bytes.
  2. Installed candidate visual smoke verified on Drop Shelf: dragged browser PNG rendered as a real thumbnail (`IMG.shell-visual-img` blob object URL) instead of the fallback icon.
  3. Real Edge (155.0.4283.62) dragged website images into the expanded Edge Rail; foreground focus was preserved with zero activation theft.
  4. Real controlled native OLE source with stream and global-memory media delivered PNG + animated GIF + unsupported neighbor; Shelf and Rail displayed `Added 2 images; skipped 1.`
  5. Native Copy drag-out transferred exact PNG and animated GIF bytes to a real Windows Forms FileDrop target, consuming and deleting temporary files.
  6. Pinning preserved image Managed Files across restart and Copy drag-out; explicit Remove deleted owned files.
  7. Attachment Services applied `ZoneId=3` with HTTP provenance; Restricted Zone without provenance failed closed with `0x800C000E` as intended by policy.
  8. Cooperative Shutdown (`--shutdown`) and clean profile isolation verified. Ambient HKCU Run value restored to baseline.
- On 2026-10-23, the user confirmed all acceptance checks passed and explicitly authorized committing the changes (`tôi đã cài và test version 0.0.24... tất cả đã đạt, hãy commit các thay đổi`).
