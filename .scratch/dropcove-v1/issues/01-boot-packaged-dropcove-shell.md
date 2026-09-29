# 01: Boot the CLI-installed DropCove shell

**What to build:** Deliver an installable unpackaged x64 self-contained DropCove shell through a CLI-built EXE installer, proving the selected WinUI, four-module architecture, and test foundations on a supported Windows 11 system.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [x] The solution contains the selected application/UI, core domain, services, and native interop modules with only the dependencies needed to launch.
- [x] The project publishes an unpackaged x64 self-contained WinUI application with no package identity and no separately installed .NET runtime requirement.
- [x] A CLI-built EXE installer installs the complete publish output and accounts for the x64 Visual C++ Redistributable prerequisite.
- [x] Launching the installed application displays a real WinUI Drop Shelf surface rather than a placeholder console, mock, or non-functional scaffold.
- [x] The EXE installer uninstalls DropCove cleanly without leaving a running process.
- [x] The automated test project discovers and passes a minimal test, and is ready to host application-seam behavior tests.
- [x] An installed-EXE smoke run is documented and exercised for install, launch, close, and uninstall.

## Comments

- 2026-09-28: CLI-only validation passed: official WinUI template, restore, build, and `dotnet run` all worked without Visual Studio workloads, standalone MSBuild, or .NET workloads.
- 2026-09-28: [Historical note] Initial development MSIX build passed, but actual local installation required elevated LocalMachine certificate trust. The project switched to an unpackaged self-contained NSIS EXE installer as the approved first-install path; MSIX/Store packaging is moved to Ticket 17.
- 2026-09-28: The non-commercial Inno Setup compiler was removed. NSIS 3.12 was installed after its zlib/libpng commercial-use license was verified; its `makensis.exe` built the self-contained installer.
- 2026-09-28: NSIS installer smoke passed: clean install, visible installed `DropCove` window, uninstall while the window was open, process closure, and install-directory removal. The installer was then reinstalled and DropCove remains open for inspection.
- 2026-09-28: `dotnet test` now discovers and passes one test-runner smoke test. Application-seam behavior tests begin in Ticket 03.
- 2026-09-29: User found that DropCove was absent from Windows Installed apps, so the existing uninstall acceptance was not actually satisfied by the prior installer.
- 2026-09-29: NSIS now writes a per-user Windows uninstall entry with display metadata and uninstall commands. A fresh installer build/install exposed the registration, and silent uninstall removed the registration, process, and install directory.