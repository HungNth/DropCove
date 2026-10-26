# 02: Packaged installer end-to-end acceptance verification

**Parent specification:** `.scratch/optional-start-menu-shortcut/spec.md`

**Required behavior:** Build the production installer package and verify end-to-end Windows shortcut behavior against an isolated target directory across interactive, silent (`/S`), passive (`/P`), `/NS`, conflict, update-preserve, and uninstallation paths. Verify that created shortcuts set the canonical `AppUserModelId` (`DropCove`). Verify that workstation state (registry and existing user shortcuts) is cleanly restored afterward.

**Blocked by:** None (Ticket 01 resolved; can start immediately)

Status: resolved

## Acceptance criteria

- [x] Installer package builds cleanly via `cargo xwin build` / Tauri bundler.
- [x] Interactive setup smoke verifies that Start Menu shortcut creation can be unchecked, and installing with it unchecked creates no Start Menu shortcut.
- [x] Unchecked setup or `/NS` execution succeeds (exit 0) without creating a shortcut, even when an unowned conflicting `DropCove.lnk` exists in the Start Menu.
- [x] Checked interactive setup, standard silent setup (`/S`), and passive setup (`/P`) create the Start Menu shortcut with canonical AUMID.
- [x] Passive setup with `/P /NS` succeeds (exit 0) without creating a shortcut or aborting on conflicts.
- [x] Checked interactive setup and standard silent setup (`/S`) abort (exit 2) when an unowned conflicting shortcut exists.
- [x] Reinstalling with Start Menu shortcut creation unchecked preserves an existing Start Menu shortcut without deleting or modifying it.
- [x] Uninstallation removes the Start Menu shortcut only when it is verified as installer-owned, and completes cleanly when no shortcut exists.
- [x] Workstation registry and shortcut baselines are fully restored after verification.

## Testing seam

- Direct execution of the packaged NSIS installer (`DropCove-Setup-<version>.exe`) with explicit command-line flags (`/S`, `/P`, `/NS`, `/D=...`) and automated Windows UI interaction against isolated installation and profile directories, evaluating process exit codes and Windows Shell shortcut metadata.

## Demo path

- Run isolated setup passes covering:
  1. Interactive unchecked install: verify exit 0, no `.lnk` created in `$SMPROGRAMS`.
  2. Silent `/S` and passive `/P` install: verify exit 0, `.lnk` created with AUMID.
  3. Silent `/S /NS` and passive `/P /NS` with conflicting unowned `.lnk`: verify exit 0, unowned `.lnk` preserved.
  4. Silent `/S` with conflicting unowned `.lnk`: verify exit 2, unowned `.lnk` preserved.
  5. Reinstall unchecked: verify existing `.lnk` preserved.
  6. Uninstall: verify owned `.lnk` cleaned, then restore baseline registry.
