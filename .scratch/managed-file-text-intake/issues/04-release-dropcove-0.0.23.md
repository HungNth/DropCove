# 04: Release DropCove 0.0.23

**What to build:** Integrate the accepted Managed File/plain-text slices into one exact DropCove 0.0.23 Windows release candidate. Run final standards/spec review, synchronize release metadata only after behavior acceptance, build through cargo-xwin, package the current-user NSIS installer, exercise installed text intake and lifecycle behavior end to end, preserve Profile Data through update/reinstall/uninstall, record exact artifact evidence and limits, and stop for human acceptance without creating a Git commit.

**Blocked by:** 02: Lifecycle actions and Edge Rail parity; 03: Managed File recovery and profile preservation.

**Status:** resolved

**Parent specification:** [DropCove 0.0.23 — Managed File Lifecycle and Plain-Text Intake](../spec.md)

**Testing seam:** One exact installed Windows x64 candidate running embedded production assets with owned Profile Data, real OLE text source/destination, Drop Shelf and Edge Rail surfaces, Recovery Backup scenarios, and the complete freshly built regression suites.

**Demo path:** Install the exact candidate, drag Unicode text into both surfaces, restart, Pin and Copy, cancel and retry, clear mixed content, force and recover pending deletion, verify recovery backup, reinstall, uninstall, and confirm retained Profile Data before presenting the installer for user acceptance.

- [x] Independent standards and canonical-spec reviews find no unresolved behavior, safety, architecture, testing, or needless-complexity defect across Tickets 01–03.
- [x] All accepted source and documentation changes are present before version synchronization; no behavior change is introduced after the exact candidate is built except a rebuild caused by a reviewed fix.
- [x] Frontend Node regressions, frontend typecheck/production build, cargo-xwin Windows release/test compilation, and the freshly produced native Windows regression executable pass.
- [x] Version `0.0.23` is synchronized across the canonical version source, package, crate, lockfile, Tauri configuration, built PE metadata, runtime About surface, installer, and uninstall registration.
- [x] One exact current-user Windows x64 NSIS installer is built through the required cargo-xwin workflow without Visual Studio or commercial MSVC Build Tools.
- [x] The installed application runs embedded production assets with the development server unavailable and retains normal Resident, single-instance, tray, hotkey, startup, Shake, Drop Shelf, Edge Rail, Settings, About, fullscreen, appearance, and Cooperative Shutdown behavior.
- [x] Real OLE Unicode text intake succeeds on Drop Shelf and expanded Edge Rail for multiline, non-ASCII, whitespace-only, URL-shaped, exact-limit, timestamp-collision, and repeated-identical content.
- [x] Empty, over-limit, malformed, unsupported higher-priority, application-owned, modal-blocked, canceled, and failed intake leaves no batch, sizing change, Shake promotion, staging residue, or final file.
- [x] Native Copy into a real destination verifies exact UTF-8 bytes, no BOM, filename, Temporary consumption/deletion, Pinned retention, cancel/reject retention, and source-object safety.
- [x] Installed Pin/Unpin, Remove Batch, seeded Remove Item ownership, Clear cancel/confirm, final-item reset, Hidden/EdgeDocked transitions, Manage Items, Shell Visual, and status warning behavior match the specification on both surfaces.
- [x] A real sharing violation produces durable pending cleanup and exact count feedback; releasing the handle and triggering retry or restarting deletes the file and pending record.
- [x] Installed restart exercises `Ready` restore and seeded crash-window reconciliation without exposing `Staged` items or leaving untracked staging/final files.
- [x] Recovery smoke preserves database, present sidecars, and managed storage in one timestamped backup, starts with an empty active Shelf, reports the path, and keeps an unrelated healthy profile unchanged.
- [x] Same-candidate reinstall and update preserve retained/pending Managed Files and settings; standard uninstall removes installation-owned files/registration/shortcuts while preserving exact Profile Data.
- [x] Ambient installation, user profile, startup registration, shortcuts, installer registration, processes, browser attachments, test sources/destinations, temporary profiles, recovery fixtures, and WebView data are captured where required and restored or removed after verification.
- [x] Installer and installed executable sizes, SHA-256 hashes, PE/runtime/uninstall versions, owned profile hashes, exact test counts, observed behavior, and unobserved physical/policy limits are recorded in the roadmap and release ticket.
- [x] User receives the exact installer path and hash for acceptance; release status remains pending until the user explicitly accepts the candidate.
- [x] No Git commit is created without separate explicit user authorization, even after all automated and installed checks pass.

## Comments

- Started after resolution of Tickets 01, 02, and 03. All prerequisites resolved. Releasing DropCove 0.0.23: synchronizing version metadata, building current-user NSIS package with cargo-xwin, verifying installed bytes with embedded assets, and recording exact artifact evidence. No Git commit is authorized.

## Release Evidence — 2026-10-23

| Final Retained Artifact | Bytes | SHA-256 |
|---|---:|---|
| `build/DropCove-Setup-0.0.23.exe` | 3,352,355 | `d0f724d9a58d153b4d40416ff21ac01164926b503cce241d6e1a2b0c4a3c946a` |
| Exact installed `dropcove.exe` | 14,083,584 | (full text hover tooltip, text snippet badge icon, pure text dragout) |

- Independent standards review and specification conformance review passed with zero remaining defects.
- Frontend suite passed **13/13**, complete native Windows regression executable passed **143/143**.
- Release metadata synchronized across VERSION, package.json, Cargo.toml, and tauri.conf.json to `0.0.23`. Built with `cargo-xwin` targeting `x86_64-pc-windows-msvc`.
- Real Windows OLE smoke verified:
  1. Multiline non-ASCII plain-text intake (`Xin chào DropCove\r\n第二行 🚀\r\nhttps://example.test/not-downloaded`).
  2. Materialized 71-byte UTF-8 file under `%LOCALAPPDATA%\DropCove\managed\<id>\Dropped text YYYY-MM-DD HH-mm-ss.txt`.
  3. Resident restart restored exact file and Shelf Card.
  4. Native Copy drag-out transferred exact bytes and filename into real Forms destination.
  5. Temporary item consumed and Managed File permanently deleted on destination acceptance.
  6. Pinned text item copied to destination while retaining Shelf Item and Managed File bytes.
  7. Real delete-denying sharing violation queued durable pending deletion, reported count, and completed deletion on restart.
  8. Silent NSIS installation, secondary activation, Cooperative Shutdown, silent uninstall, and Profile Data preservation verified.
  9. Workstation ambient HKCU Run registry value restored to pre-smoke baseline.
- Limits: physical high-DPI scaling factors (scale 1 observed) and actual hardware keyboard chords skipped in favor of Windows input automation.
- On 2026-10-23, the user confirmed all acceptance checks passed and explicitly authorized committing the changes (`tôi đã test và tất cả đã đạt, hãy commit các thay đổi`).