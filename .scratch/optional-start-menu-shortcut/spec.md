# Specification: Optional Start Menu Shortcut in Installer

Status: ready-for-agent

## Problem Statement

Currently, the DropCove setup wizard enforces Start Menu shortcut creation as mandatory: the "Create Start Menu shortcut" checkbox is disabled and fixed as `(required)`. Users who prefer a minimal environment without Start Menu additions cannot opt out during interactive setup. Furthermore, when the `/NS` (no shortcuts) flag is passed or when an unrelated shortcut named `DropCove.lnk` exists in the Start Menu, setup unconditionally aborts with exit code 2 even if the user does not want setup to touch the Start Menu.

## Solution

Make the Start Menu shortcut optional in the setup wizard:
1. Enable the "Create Start Menu shortcut" checkbox on the Shortcut options page, keeping it checked by default.
2. Update the page instructional copy and checkbox label to communicate that shortcut creation is optional and user-configurable.
3. Skip the Start Menu conflict validation check if the user unchecks the option or if the `/NS` command-line switch is passed.
4. Maintain default shortcut creation for unattended installations (`/S` and `/P`) unless suppressed by `/NS`.
5. When updating or reinstalling with the option unchecked, leave any existing Start Menu shortcut untouched.

## User Stories

1. As a Windows user running the DropCove setup wizard, I want the "Create Start Menu shortcut" checkbox to be interactive, so that I can decide whether to add DropCove to my Start Menu.
2. As a Windows user running the DropCove setup wizard, I want the "Create Start Menu shortcut" checkbox to be checked by default, so that standard installation adds discoverability without extra clicks.
3. As a Windows user who prefers a clean Start Menu, I want to uncheck "Create Start Menu shortcut", so that setup installs DropCove without creating an Installer-Owned Shortcut in `%APPDATA%\Microsoft\Windows\Start Menu\Programs`.
4. As a Windows user with an existing unrelated file or shortcut named `DropCove.lnk` in my Start Menu, I want to uncheck "Create Start Menu shortcut" and complete setup successfully, so that an unrelated shortcut conflict does not block me from installing DropCove.
5. As an IT administrator deploying DropCove silently with `/S`, I want the Start Menu shortcut created by default, so that unattended installations remain discoverable for standard desktop users.
6. As an IT administrator deploying DropCove silently with `/S /NS`, I want setup to skip Start Menu shortcut creation and conflict checks, so that automated deployments into specialized or restricted environments succeed without shortcut creation.
7. As a Windows user running passive setup with `/P`, I want the Start Menu shortcut created by default unless `/NS` is passed, so that passive installations match silent installation behavior.
8. As a Windows user reinstalling or updating DropCove with "Create Start Menu shortcut" unchecked, I want any pre-existing Start Menu shortcut preserved as-is, so that setup does not delete my existing shortcut.
9. As a Windows user installing DropCove with "Create Start Menu shortcut" checked (default), I want setup to verify shortcut ownership before installing, so that setup never overwrites an unowned shortcut.
10. As a Windows user uninstalling DropCove, I want the uninstaller to remove the Start Menu shortcut only if it was created and owned by the installer, so that uninstallation cleans up cleanly without removing user-managed shortcuts.
11. As a keyboard-only Windows user navigating the Shortcut options wizard page, I want the initial focus placed on the first interactive shortcut checkbox, so that I can toggle shortcut choices easily using the spacebar and tab key.

## Implementation Decisions

- **State Management**:
  - Introduce an explicit state variable for Start Menu shortcut requests.
  - Initialize the state to requested (enabled) during installer initialization so that silent and passive modes create the Start Menu shortcut by default.
  - When the `/NS` command-line switch is present, set the state to unrequested (disabled).
  - On the interactive Shortcut options page, initialize the checkbox from this state, allow the user to toggle it, and persist the checkbox state back to the state variable on page leave.

- **UI Copy & Presentation**:
  - Update the instructional label on the Shortcut options page from the mandatory assertion to a neutral prompt: `"Select the shortcuts you want Setup to create for ${PRODUCTNAME}:"`.
  - Update the Start Menu checkbox label from `"Create Start Menu shortcut (required)"` to `"Create Start Menu shortcut"`.
  - Remove the window-disable directive on the Start Menu checkbox so it accepts user clicks and keyboard spacebar toggles.
  - Set initial control focus to the Start Menu checkbox.

- **Conflict Validation Guard**:
  - Guard the Start Menu ownership and collision verification check so it only executes when the Start Menu shortcut is actively requested and `/NS` is not active.
  - If the Start Menu shortcut is not requested or `/NS` is specified, skip collision verification completely, allowing installation to proceed even if an unowned `DropCove.lnk` exists.

- **Creation & Reinstall Guard**:
  - Guard Start Menu shortcut creation and AppUserModelId registration so they execute only when the state variable indicates it was requested and `/NS` is not active.
  - If not requested, do not create, overwrite, or delete any Start Menu shortcut file.

- **Uninstaller Continuity**:
  - Maintain the existing uninstaller behavior: inspect whether the Start Menu shortcut exists and proves installer ownership; if so, delete it; otherwise, leave it untouched.

## Testing Decisions

- **Primary Seam**: The generated NSIS setup package executable (`DropCove-Setup-<version>.exe`) executed against isolated target directories and temporary registry profiles, inspecting Windows-visible outcomes (process exit codes, file existence in `$SMPROGRAMS`, and file content preservation).
- **Good Test Criteria**:
  - Tests verify user-observable behavior and process exit status, never template source text or internal label positioning.
  - Tests verify that default interactive and silent execution creates the expected shortcut.
  - Tests verify that unchecked interactive execution and `/NS` flag execution skip creation and exit with code 0 even in the presence of an unowned shortcut collision.
  - Tests verify that checked interactive execution and `/S` without `/NS` abort with exit code 2 when an unowned shortcut collision exists.
  - Tests verify clean uninstallation in both paths (shortcut present vs shortcut skipped).
- **Prior Art**:
  - NSIS installer contract verification established in `docs/adr/0001-cargo-xwin-and-per-user-nsis.md`, `docs/adr/0004-custom-nsis-and-cooperative-shutdown.md`, and past release smokes (`.scratch/recovery-distribution-hardening/issues/03-current-user-installer-wizard.md`).

## Out of Scope

- Adding custom Start Menu folder selection or subfolder path customization.
- Pinning to Start or Taskbar via shell automation (disallowed by modern Windows guidelines).
- Altering the Desktop shortcut default (remains optional and unchecked by default).
- Altering the Finish page launch option (remains checked by default).

## Further Notes

- Synchronization: Once implemented, `docs/SPECIFICATION.md` must be updated to align item 5 (`offers an optional Start Menu shortcut, enabled by default;`).
