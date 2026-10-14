# DropCove 0.0.8 — Recovery and Distribution Hardening

Status: ready-for-agent
Version: 0.0.8
Design confirmation: approved by user on 2026-10-14
Ticket decomposition: five vertical slices approved by user on 2026-10-14
Implementation approval: all five published tickets approved by user on 2026-10-14
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.8
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md)
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md)
Architecture decisions: [Recovery evidence](../../docs/adr/0003-preserve-database-evidence-and-restart-empty.md), [NSIS and Cooperative Shutdown](../../docs/adr/0004-custom-nsis-and-cooperative-shutdown.md)
Research: [Tauri v2 NSIS and Windows prerequisites](research/tauri-nsis-windows-prerequisites.md)

## Problem Statement

DropCove 0.0.7 implements the resident Shelf workflow and presentation contract, but failure recovery and distribution remain incomplete. A Shelf database that cannot be opened or fails integrity validation currently prevents the Resident Instance from starting even when the failed files could be preserved and a clean Shelf could be created. Most ordinary mutations are transactional, but the release does not yet prove every Shelf, settings, hotkey, startup, Shake-to-open, and remembered Edge Rail placement failure leaves the prior safe state visible and durable.

The stock Tauri NSIS bundle also conflicts with the required product contract. Its default install location collides with DropCove's profile location, its standard wizard does not provide the required Shortcut options page, its Desktop shortcut defaults are wrong, and its running-process path may force-terminate the Resident Instance. Update and uninstall therefore cannot yet guarantee clean teardown, bounded abort behavior, Installer-Owned Shortcut cleanup, startup-registration cleanup, or complete Profile Data preservation.

Users need a release that recovers safely when recovery is possible, fails clearly when persistence cannot be trusted, and installs, repairs, updates, or uninstalls without silently losing references, preferences, recovery evidence, or unrelated Windows state.

## Solution

Add one evidence-preserving Shelf recovery path. When an existing Shelf database has a recoverable open, format, schema, or integrity failure and the profile remains writable, DropCove moves the database and every existing WAL/SHM sidecar into one timestamped Recovery Backup, creates a clean database, starts with an empty Shelf, and warns with the backup directory. It never attempts automatic salvage. If complete preservation, rollback, or clean recreation fails, startup remains fatal rather than running without trustworthy persistence.

Complete the safe-state contract across every persistence and Windows integration path. Failed mutations retain the prior visible and durable Shelf state. Failed settings or integration changes retain the prior safe configuration when rollback succeeds; incomplete rollback reports the actual retained state and remains retryable. Malformed or unreadable settings keep their existing documented behavior: preserve the file, load defaults in memory, and warn.

Replace the stock installer behavior with a minimal custom NSIS template derived from the pinned Tauri template. Provide the exact current-user wizard `Welcome → Directory → Shortcut options → Installation progress → Finish`, validate the required x64 Visual C++ Redistributable without downloading it, use the specified install location, remember only successful directory choices, preserve ambiguous shortcuts, and retain deterministic silent/passive behavior.

Add a profile-scoped Cooperative Shutdown command used by update and uninstall. The installed executable asks the Resident Instance to exit through normal teardown, waits at most five seconds for a safe point and confirmed exit, and returns a result to setup. Setup aborts without force-killing if the resident remains active. Updates preserve Profile Data, allow repair of the same version, reject downgrades, and replace program files only after clean shutdown. Uninstall removes only installer- or application-owned artifacts and preserves the entire profile.

Verify the exact 0.0.8 installer and installed executable on Windows 11 x64 across database, integration, installer, restart, update, uninstall, single-monitor, mixed-DPI, monitor-disconnect, and prior-workflow scenarios.

## User Stories

1. As a user whose Shelf database is corrupt, I want DropCove to preserve the failed files before continuing, so that recovery evidence is not destroyed.
2. As a user whose Shelf database cannot be parsed, I want DropCove to create a clean Shelf when safe recovery is possible, so that I can continue using the application.
3. As a user after database recovery, I want the Shelf to start empty, so that corrupted records are not presented as trustworthy state.
4. As a user after database recovery, I want a warning containing the Recovery Backup path, so that I can locate the preserved files.
5. As a user launched normally after recovery, I want the Shelf to follow normal launch visibility, so that recovery does not change activation behavior.
6. As a user using Hidden Start after recovery, I want the Resident Instance to remain hidden, so that sign-in startup stays non-disruptive.
7. As a user whose recovery notification cannot be delivered, I want the same warning available when I open the Shelf, so that the backup path is not lost.
8. As a user whose database or sidecar is locked, I want startup to fail clearly rather than delete or ignore evidence, so that persistence remains trustworthy.
9. As a user whose profile is not writable or has no space, I want a fatal explanation rather than an in-memory Shelf that appears durable, so that I am not misled.
10. As a user, I do not want DropCove to guess which corrupt records can be salvaged, so that partial reconstruction cannot masquerade as complete state.
11. As a user removing, pinning, clearing, or dragging Shelf references, I want failed persistence to retain the prior visible state, so that the UI never claims an uncommitted mutation.
12. As a user accepting a new Shelf Batch, I want a failed insertion to leave no partial batch or reordered state, so that the Shelf remains coherent.
13. As a user changing settings, I want failed persistence or integration to retain the prior safe configuration, so that Save is atomic from my perspective when rollback succeeds.
14. As a user encountering incomplete settings rollback, I want a warning describing the actual retained configuration, so that I can retry from reality rather than the requested state.
15. As a user with malformed settings, I want the original file preserved and documented defaults used temporarily, so that DropCove does not overwrite possible recovery evidence.
16. As a user changing the global hotkey, I want registration failure to leave the prior working hotkey active, so that access is not silently lost.
17. As a user changing Start with Windows, I want registry failure to retain the prior startup behavior, so that Settings and Windows agree.
18. As a user changing Shake-to-open, I want hook failure to retain the prior detector state, so that Settings does not report a false configuration.
19. As a user moving the Edge Rail between monitors, I want a failed remembered-placement write to retain the prior durable placement, so that restart behavior is predictable.
20. As a Windows 11 x64 user, I want a current-user installer that does not require elevation, so that DropCove remains a per-user utility.
21. As a user without the required x64 Visual C++ Redistributable, I want setup to stop before installation and show the official Microsoft remediation URL, so that I do not receive a non-runnable installation.
22. As a user installing for the first time, I want `%LOCALAPPDATA%\Programs\DropCove` selected by default, so that program files remain separate from Profile Data.
23. As a returning user, I want setup to remember the last successfully installed directory, so that update and repair target the existing installation.
24. As a user whose remembered directory is unavailable, I want setup to keep me on the Directory page with a clear error, so that it does not silently install a second copy.
25. As a user installing DropCove, I want a Start Menu shortcut created, so that the application is discoverable.
26. As a user, I want the Desktop shortcut to be optional and disabled by default, so that setup does not add desktop clutter without consent.
27. As a user finishing interactive setup, I want Launch DropCove enabled by default, so that I can start the installed application immediately.
28. As a user with an unrelated shortcut named DropCove, I want setup to preserve it, so that installation cannot overwrite my shortcut.
29. As a user running silent or passive setup, I want all safety checks retained without Desktop shortcut creation or auto-launch, so that automation has deterministic side effects.
30. As a user updating DropCove while it is running, I want setup to request a clean exit and wait briefly, so that program files are not replaced under a live Resident Instance.
31. As a user performing an update during a short Shelf operation, I want the Resident Instance to exit at the next safe point within the deadline, so that setup need not cancel work or fail immediately.
32. As a user whose Resident Instance cannot exit within five seconds, I want setup to abort and tell me to close DropCove, so that it never force-kills the application.
33. As a user running a newer setup, I want the existing installation updated in its remembered directory, so that one installation remains authoritative.
34. As a user repairing the same version, I want setup to reinstall the program files, so that a damaged installation can be restored.
35. As a user running an older setup, I want downgrade blocked, so that an older binary cannot consume newer Profile Data.
36. As a user whose program-file replacement fails, I want setup to abort without touching Profile Data, so that rerunning setup can repair the application without data loss.
37. As a user uninstalling DropCove, I want setup to close the Resident Instance cooperatively before removing files, so that teardown and persistence complete normally.
38. As a user uninstalling DropCove, I want only Installer-Owned Shortcuts and the owned Start with Windows registration removed, so that unrelated Windows state remains intact.
39. As a user uninstalling DropCove, I want all Profile Data and Recovery Backups preserved, so that reinstall or manual recovery remains possible.
40. As a user reinstalling after uninstall, I want setup to use the default directory unless a current installation registration exists, so that removed installer-location state does not linger.
41. As a user with multiple monitors at different DPI values, I want Shelf and Edge Rail placement, resizing, dragging, and restart restoration to remain correct, so that hardening does not regress existing display behavior.
42. As a user disconnecting and reconnecting a remembered monitor, I want DropCove to use the documented fallback and restore behavior, so that unavailable display state remains safe.
43. As a user upgrading to 0.0.8, I want every workflow delivered in 0.0.2 through 0.0.7 to continue working, so that hardening does not replace product behavior.
44. As a maintainer, I want final evidence tied to the exact installer and installed executable bytes, so that release claims apply to the distributed artifact.

## Implementation Decisions

- Keep the existing Windows-only, x64, per-user architecture and one Resident Instance per profile. Do not add a service, updater daemon, elevated helper, or second state owner.
- Treat database detection and recovery as separate responsibilities. Existing integrity validation remains the detection authority; a startup recovery coordinator owns preservation, clean recreation, and the recovered-startup result.
- A Recovery Backup is one directory under the profile recovery area named `shelf-corrupt-<UTC timestamp>[-N]`. It preserves the original names of the main database and any existing WAL/SHM sidecars. One timestamp identifies the entire recovery episode; a numeric suffix resolves collisions.
- Missing WAL or SHM files are normal. Every artifact that does exist must move successfully before a clean database is created. On move failure, restore previously moved artifacts and abort. If rollback cannot completely restore them, abort with both source and Recovery Backup locations.
- Recovery applies only when an existing failed database can be preserved and the profile can create a clean database. Permission errors, locks, exhausted storage, backup failures, recreation failures, and backup rollback failures remain fatal startup errors.
- Recovery creates an empty Shelf and performs no automatic salvage, import, record filtering, or schema guessing.
- Preserve normal launch versus Hidden Start visibility. Publish one startup recovery warning through the notification-area channel after resident infrastructure is ready. If delivery fails, retain the warning for the Shelf status surface until shown.
- Preserve the existing settings-file contract. Missing settings load defaults silently. Malformed or unreadable settings remain untouched, load defaults in memory, and warn. Do not create a settings Recovery Backup or rewrite malformed settings during startup.
- Complete failure behavior using the existing serialized Shelf store, transactions, settings transaction coordinator, and integration adapters. Add no second transaction framework.
- Every failed Shelf mutation retains its prior durable and visible snapshot. Every failed settings/integration mutation either restores the prior safe configuration or exposes the actual retained state with an actionable warning. Retry starts from that actual state.
- Include remembered Edge Rail monitor persistence in the safe-state matrix; it must not claim a placement that was not durably stored.
- Maintain a minimal custom NSIS template derived from the pinned Tauri 2.12 template. Preserve upstream structure where it satisfies requirements and document/review the local diff when Tauri changes.
- The interactive wizard sequence is exactly `Welcome → Directory → Shortcut options → Installation progress → Finish`. The Shortcut options page presents the mandatory Start Menu shortcut and an unchecked optional Desktop shortcut. The Finish page presents checked-by-default Launch DropCove.
- Installation remains `currentUser`, non-elevated, and x64. The first-install directory is `%LOCALAPPDATA%\Programs\DropCove`.
- Remember install location only after successful installation. A registered prior location wins over the default. An invalid or unwritable remembered location remains visible for explicit correction; setup does not silently redirect or mutate registration on cancel/failure.
- Detect the Visual C++ Redistributable in the 64-bit registry view at `HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64`, requiring `Installed=1` and `Major=14`. On failure, show `https://aka.ms/vc14/vc_redist.x64.exe` and abort without opening a browser, downloading, or bundling the prerequisite.
- A canonical shortcut may be updated or removed only when its target proves installer ownership. A conflicting Start Menu shortcut aborts before program-file changes. A conflicting optional Desktop shortcut remains untouched and produces a non-fatal warning.
- Silent/passive setup uses the remembered or default install directory and retains prerequisite, version, shutdown, ownership, and profile-preservation checks. It does not create the optional Desktop shortcut or auto-launch DropCove.
- Add a pre-UI `dropcove.exe --shutdown` mode. It locates the default profile's Resident Instance through the existing profile-scoped receiver, requests normal resident exit, waits up to five seconds for the profile lease to disappear, and returns a stable process exit status. No resident is success.
- Cooperative Shutdown never force-cancels a drag, resize, mutation, or integration operation. A busy resident reaches its existing safe point and accepts a retried request within the deadline; otherwise setup aborts.
- Newer setup versions update, the same version repairs, and older versions abort. Update performs Cooperative Shutdown before replacing files in the remembered install directory. There is no custom transactional rollback for program files; a failed installation preserves Profile Data and is repaired by rerunning setup.
- Uninstall performs Cooperative Shutdown before deleting program files. Timeout aborts before removal. It removes installed files, uninstall/location registration, and only owned shortcuts and startup registration. A same-named startup value with unexpected target or arguments is preserved and warned.
- Update and uninstall never offer or perform Profile Data deletion. The complete per-user profile, including unknown future files and Recovery Backups, remains untouched.
- The release remains version `0.0.7` during implementation. Version metadata changes to `0.0.8` only in the release ticket after all behavior is verified.
- Record the recovery policy in ADR-0003 and the custom NSIS/Cooperative Shutdown policy in ADR-0004.

## Testing Decisions

- Use three primary seams: startup against an owned temporary profile, existing deterministic persistence/integration failure adapters, and the exact generated NSIS/installed artifact. Do not create a test-only parallel architecture.
- Recovery tests exercise corrupt/non-database/schema-invalid input with no sidecars, WAL only, SHM only, and both sidecars. Assert byte-for-byte preservation, one Recovery Backup path, clean empty Shelf state, warning content, and unchanged launch visibility.
- Recovery failure tests exercise backup-name collision, locked artifact, denied directory, exhausted or failed writes, partial move failure, rollback failure, and clean-database creation failure. Permanent tests assert preservation/fatal outcomes, not private helper sequencing.
- Safe-state tests inject failures across intake, pinning, removal, Clear Temporary, drag completion, sizing/placement persistence, settings-file replacement, hotkey registration, startup registration, Shake integration, and rollback. Assert the authoritative visible snapshot and post-restart durable snapshot.
- Reuse the existing settings integration adapter to distinguish clean rollback from incomplete rollback. Verify warning state reports reality and retry starts from the retained configuration.
- Installer contract tests inspect behavior through built setup execution and Windows-visible outcomes, not source-template strings. Focused native tests may cover CLI shutdown exit status and receiver behavior.
- Interactive installer smoke verifies exact page sequence, first-install default, remembered directory, invalid-directory validation, prerequisite failure, mandatory Start Menu shortcut, optional unchecked Desktop shortcut, checked Finish launch, conflict handling, update, repair, downgrade rejection, and timeout abort.
- Silent/passive smoke verifies no Desktop shortcut and no auto-launch while retaining prerequisite, shutdown, version, location, and preservation rules.
- Update/uninstall smoke runs the exact installed binary, proves five-second bounded Cooperative Shutdown without forced termination, verifies owned artifact cleanup, preserves ambiguous shortcut/registry state, and compares Profile Data byte-for-byte before and after.
- Display acceptance covers one 100% monitor; two monitors at 100% and 150% with each acting as primary; and secondary disconnect/reconnect. Exercise Shelf placement, Edge Rail, drag/drop, resize, and restart restoration on each relevant configuration.
- Release regression exercises the documented end-to-end example and every target behavior assigned to versions 0.0.2 through 0.0.8.
- Build once, pin installer and installed executable byte sizes and SHA-256 hashes, then install and exercise those exact bytes. Restore ambient profiles, registry values, shortcuts, processes, monitor configuration, and test fixtures.
- Good permanent tests catch consumer-visible data loss, partial mutation, misleading state, missing evidence, destructive cleanup, forced shutdown, wrong version policy, or packaging behavior. They do not assert source text, mock forwarding, private helper call order, or incidental NSIS layout beyond the approved wizard contract.

## Out of Scope

- Automatic salvage, repair, import, merge, or inspection of records from a Recovery Backup.
- Backup, rewrite, or migration of malformed `settings.json` during startup.
- Cloud backup, synchronization, accounts, telemetry, crash reporting, or a recovery UI.
- Background updater, update service, signing, differential patches, download hosting, or automatic prerequisite download/install.
- Forced process termination, operation cancellation for setup, or an elevated helper.
- Transactional rollback of partially replaced program files; rerunning setup is the repair path.
- A user-facing uninstall option to delete Profile Data.
- Support for Windows versions before Windows 11 build 22000, x86, ARM64, macOS, or Linux.
- Frontend dependency modernization assigned to 0.0.9 and UI refinement assigned to 0.0.10.
- Advancing to 0.1.0 or 1.0.0.
- Creating a Git commit without separate explicit user authorization.

## Further Notes

- The current source already validates SQLite integrity, uses WAL and foreign keys, and transactionally protects most Shelf mutations. Version 0.0.8 adds recovery and closes observable failure gaps; it does not replace the persistence model.
- Stock Tauri NSIS behavior is intentionally not accepted as the product contract because its install location, wizard flow, Desktop default, running-app termination, and app-data UI differ from the approved behavior.
- The user approved implementation of all five published tickets; Git commit approval remains a separate gate.
