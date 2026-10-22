# DropCove 0.0.21 — Application branding and installer icon unification

Status: ready-for-agent
Version: 0.0.21
Design confirmation: approved by the user on 2026-10-22 through `grill-with-docs`
Testing seam: unified icon asset inspection (ICO layer validation and binary PE resource inspection) combined with end-to-end Windows 11 installation smoke verification across all 6 shell and resident surfaces
Delivery prerequisite: completed — DropCove 0.0.20 is accepted and committed; the canonical roadmap defines 0.0.21 as the next release
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), especially Packaging and Distribution
Installer policy: [ADR-0001](../../docs/adr/0001-cargo-xwin-and-per-user-nsis.md) and [ADR-0004](../../docs/adr/0004-custom-nsis-and-cooperative-shutdown.md) — per-user NSIS installer with Cooperative Shutdown
Domain language: [Glossary](../../GLOSSARY.md), especially **Brand Visual** and **Installer Branding**
Roadmap: [DropCove 0.0.21](../../docs/ROADMAP.md#0021--application-branding-and-installer-icon-unification)

## Problem Statement

DropCove users encounter inconsistent branding between the application and Windows installation/removal surfaces. Generic installer icons make the setup and maintenance utilities harder to recognize as DropCove. Inspection found that the existing application icon assets already match the About Surface's canonical vector mark, but NSIS packaging does not explicitly select that mark for setup and uninstaller icons. The custom template already supports both icons and registers the application's DisplayIcon; it does not need a second wiring path.

## Solution

Establish the vector Brand Visual as the canonical single source of truth for all application, resident, and installer identities. Regenerate all Windows/Tauri bundle raster and multi-resolution icon assets (`.ico`, `.png`) directly from the vector mark. Configure NSIS packaging in `tauri.conf.json` with explicit `installerIcon` and `uninstallerIcon` properties pointing to the unified Windows icon. Validate that the setup package, active setup wizard window, uninstaller, Windows Installed Apps list, desktop/start menu shortcuts, and in-app surfaces (About, Settings, Drop Shelf, and System Tray) consistently present the DropCove Brand Visual.

## User Stories

1. As a new DropCove user, I want the downloaded installer `DropCove-Setup-0.0.21.exe` in Windows File Explorer to display the official DropCove logo, so that I can immediately identify and trust the installer binary.
2. As a DropCove user running the setup wizard, I want the setup window's title bar and Windows Taskbar button to display the DropCove logo, so that the installer feels like a cohesive part of the product.
3. As a DropCove user installing DropCove, I want the setup wizard to remain minimal and clean without oversized, clipped, or low-DPI banner art, so that the installation experience is uncluttered and scales smoothly across high-DPI displays.
4. As an installed DropCove user, I want the Start Menu shortcut to display the sharp, high-resolution DropCove logo, so that I can easily discover and launch the application.
5. As an installed DropCove user, I want the optional Desktop shortcut to display the sharp DropCove logo, so that my desktop maintains clean branding.
6. As a Windows user reviewing Installed Apps in Windows Settings or Control Panel, I want DropCove to show its official icon next to its entry, so that it looks professional and distinguishable from unbranded software.
7. As an installed DropCove user navigating to `%LOCALAPPDATA%\Programs\DropCove`, I want `uninstall.exe` to display the DropCove logo, so that maintenance utilities are unmistakably branded.
8. As a DropCove user opening the About Surface, I want the rendered logo to visually match the desktop and installer icons in proportions and detail, so that brand identity is unified.
9. As a DropCove user with the application running resident in the background, I want the System Tray icon to clearly display the DropCove logo at notification-area sizes (16x16 and 24x24), so that I can spot and interact with DropCove at a glance.
10. As a DropCove user switching between windows with Alt+Tab, I want every visible DropCove window (Drop Shelf, Settings, About) to show the official logo, so that window switching is effortless.
11. As a DropCove developer, I want all Windows icon layers (16x16, 24x24, 32x32, 48x48, 64x64, 256x256) embedded in `icon.ico`, so that Windows never renders blurry or improperly scaled approximations.
12. As a DropCove developer, I want the icon generation pipeline to be automated from the canonical vector asset, so that future brand updates can be executed in a single command without manual image slicing.

## Implementation Decisions

- **Single Source of Truth**: Preserve the existing canonical vector Brand Visual, including its shape and colors. All Windows bundle icons derive from that mark; matching generated assets require no artificial changes.
- **Icon Tooling**: Use the pinned Tauri CLI icon generator with a disposable output directory, then promote only the Windows ICO and bundle PNG outputs. Leave existing mobile and macOS assets untouched rather than regenerating them in place.
- **NSIS Configuration**: Explicitly configure both installer and uninstaller icon properties to use the same branded Windows ICO. Reuse the custom template's existing MUI icon bindings without changing the installation workflow.
- **Installed DisplayIcon**: Preserve the established application-management registration pointing to the installed application executable; verify that executable's embedded icon independently of setup and uninstaller icons.
- **Minimalist Installer Visuals**: Do not add custom sidebar or header artwork. Preserve the standard wizard layout; no new claim of DPI coverage follows merely from omitting banners.
- **Manifest Synchronization**: Synchronize the authoritative release version and all package, crate/lock, and bundler metadata to 0.0.21 through the existing version-sync workflow.

## Testing Decisions

- **What Makes a Good Test**:
  - Tests must verify external behavior and observable artifacts, not internal generator intermediate files.
  - Verification must inspect binary PE headers (`RT_GROUP_ICON`) and ICO file directories rather than asserting string configuration alone.
- **Tested Modules and Artifacts**:
  - `src-tauri/icons/icon.ico`: verify presence of standard Windows dimensions (16, 24, 32, 48, 64, 256) and RGBA bit depths.
  - `DropCove-Setup-0.0.21.exe`: verify embedded `RT_GROUP_ICON` matches the brand icon rather than the Nullsoft default icon.
  - Installed `DropCove.exe` and `uninstall.exe`: verify embedded PE icon resources.
  - Registry: verify `DisplayIcon` points to the executable.
  - Visual verification across Windows Shell (Explorer, Start Menu, Settings *Installed apps*, System Tray, Alt+Tab, and About Surface).
- **Prior Art**:
  - Existing frontend Node regressions and the cargo-xwin-produced native regression executable provide supporting regression coverage; this feature's icon acceptance remains artifact and installed-surface inspection.
  - NSIS installer verification patterns documented in [ADR-0001](../../docs/adr/0001-cargo-xwin-and-per-user-nsis.md) and [ADR-0004](../../docs/adr/0004-custom-nsis-and-cooperative-shutdown.md).

## Out of Scope

- Adding custom NSIS sidebar or header banner artwork.
- Redesigning the core vector logo mark or changing the color palette of `app-icon.svg`.
- Dynamic, animated, or theme-adaptive executable or installer icons.
- End-user customizable application icons.
- Non-Windows platform icon packaging (macOS DMG or Linux packages).

## Further Notes

- Windows Explorer heavily caches shell icons in `%LOCALAPPDATA%\IconCache.db`. Verification of shortcuts and executable icons on developer workstations may require restarting `explorer.exe` or clearing the icon cache if Windows displays stale cached icons.
