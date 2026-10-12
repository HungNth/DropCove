# Use cargo-xwin and per-user NSIS installer

To develop DropCove strictly for Windows without installing, configuring, or depending on Microsoft Visual Studio or MSVC Build Tools, the project uses `cargo-xwin` targeting `x86_64-pc-windows-msvc`. The Windows installer is packaged via Tauri v2's NSIS bundler configured for `currentUser` scope, producing `DropCove-Setup-<version>.exe` without requiring administrator elevation.
