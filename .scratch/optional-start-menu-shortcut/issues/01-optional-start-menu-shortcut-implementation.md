# 01: Optional Start Menu shortcut implementation and canonical spec update

**Parent specification:** `.scratch/optional-start-menu-shortcut/spec.md`

**Required behavior:** Make the Start Menu shortcut optional in the NSIS installer wizard. Enable the checkbox for user interaction, keep it checked by default, set initial focus on it, and update copy to reflect optional creation without mandatory assertions. Manage selection via explicit state (`StartMenuShortcutRequested`) initialized to enabled for interactive and unattended (`/S`/`/P`) modes, and disabled when `/NS` is passed or unchecked by the user. Guard shortcut conflict checks and creation so that unchecking or using `/NS` bypasses conflicts and skips creation without removing pre-existing shortcuts on update. Update `docs/SPECIFICATION.md` to canonically describe the optional Start Menu shortcut contract across interactive, silent, passive, `/NS`, update-preserve, and uninstallation behaviors.

**Blocked by:** None (can start immediately)

Status: resolved

## Acceptance criteria

- [x] Checkbox "Create Start Menu shortcut" is interactive, checked by default, and receives initial wizard page focus.
- [x] Wizard page label and checkbox copy are updated to communicate optional shortcut selection without `(required)` or mandatory assertions.
- [x] Explicit request state is enabled by default for interactive, `/S`, and `/P` runs, and disabled for `/NS` runs or unchecked interactive selections.
- [x] When Start Menu shortcut creation is not requested or `/NS` is passed, the installer skips the Start Menu conflict check and does not create or modify the Start Menu shortcut.
- [x] When updating or reinstalling with Start Menu shortcut creation unchecked, existing Start Menu shortcuts are preserved.
- [x] `docs/SPECIFICATION.md` reflects the optional Start Menu shortcut contract across interactive, silent, passive, `/NS`, update, and uninstall behaviors.

## Testing seam

- Source template review and syntax compilation via `cargo xwin build` / Tauri bundle generation.

## Demo path

- Inspect modified `installer.nsi` and `docs/SPECIFICATION.md` diffs, followed by successful packaging compilation without NSIS script errors.
