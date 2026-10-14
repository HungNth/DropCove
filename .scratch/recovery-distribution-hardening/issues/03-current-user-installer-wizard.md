# 03: Current-User Installer Wizard

**What to build:** Deliver the approved non-elevated Windows 11 x64 setup wizard using a minimal custom NSIS template. The installer must enforce the Visual C++ prerequisite, separate program files from Profile Data, remember only successful directory choices, create only approved shortcuts, preserve conflicting user shortcuts, and provide deterministic interactive and silent behavior.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.8 — Recovery and Distribution Hardening](../spec.md)

## Acceptance criteria

- [ ] The custom installer template remains a minimal, reviewable derivative of the pinned Tauri 2.12 template and retains current-user installation without elevation.
- [ ] The interactive sequence is exactly `Welcome → Directory → Shortcut options → Installation progress → Finish`.
- [ ] First install defaults to `%LOCALAPPDATA%\Programs\DropCove`; the selected directory is registered only after successful installation.
- [ ] A registered prior location is restored for update/repair. If it is unavailable or unwritable, the Directory page requires explicit correction without silently redirecting or mutating registration on cancel/failure.
- [ ] Setup checks the 64-bit Visual C++ runtime registry contract (`Installed=1`, `Major=14`) before program-file changes.
- [ ] Missing prerequisite displays `https://aka.ms/vc14/vc_redist.x64.exe` and aborts without opening a browser, downloading, bundling, or installing it.
- [ ] The Shortcut options page communicates that the Start Menu shortcut is mandatory and presents an optional Desktop shortcut unchecked by default.
- [ ] Finish presents Launch DropCove checked by default for interactive install/repair/update.
- [ ] An existing canonical shortcut is updated only when its target proves installer ownership.
- [ ] A conflicting Start Menu shortcut aborts before program-file changes; a conflicting optional Desktop shortcut is preserved and produces a non-fatal warning.
- [ ] Silent/passive setup retains prerequisite, location, and shortcut-ownership checks but creates no Desktop shortcut and does not auto-launch.
- [ ] The template contains no user path that deletes Profile Data and is explicitly reviewed when the pinned Tauri installer template changes.

## Testing seam

Build and execute the generated installer, then assert Windows-visible pages, directories, registry values, shortcuts, process launch, and abort outcomes. Focused template-generation checks may verify successful rendering, but permanent behavior tests must not pin source text.

## Demo path

Run a clean interactive install, inspect the exact page sequence, choose a non-default directory, leave Desktop unchecked, launch from Finish, then rerun setup to observe the remembered directory. Repeat silent install and prerequisite/shortcut-conflict failure scenarios.

## Non-goals

Cooperative resident shutdown, update/uninstall cleanup, prerequisite downloading, elevation, signing, a background updater, or versioning the product as 0.0.8 before release verification.

## Answer

- Custom current-user NSIS template compiles with cargo-xwin/Tauri. Product version remained 0.0.7 during feature verification.
- Actual interactive wizard observed Welcome, Directory, separate Shortcut options, Installation progress, and Finish. Start Menu was checked/required, Desktop unchecked, and Launch DropCove checked on Finish.
- Exact setup smoke passed silent and passive installation, remembered-directory restoration without `/D`, profile-directory rejection, and unowned Start Menu collision abort (exit 2, original shortcut unchanged, no program files installed).
- Isolated Windows Sandbox ran the exact setup with x64 VC++ registration absent (`Installed=0`, `Major=0`): exit 2, no executable created. No global HKLM prerequisite state was changed. Failed debugger injections are not counted as evidence.
- Non-empty Start Menu folder build/install/uninstall proved the same folder-aware ownership path is used for creation and removal.
- Installer and shortcut fixtures were uninstalled; captured ambient registration and user resident were preserved. Final exact 0.0.8 artifact acceptance remains ticket 05.
