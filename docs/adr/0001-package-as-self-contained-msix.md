# Deliver the first DropCove build with an unpackaged EXE installer

The first installable DropCove build will be an unpackaged x64 self-contained WinUI application installed by an NSIS EXE. This creates a CLI-first path to a working application without a Partner Center identity, production signing, Microsoft Store submission, or Visual Studio tooling; MSIX/Microsoft Store delivery follows after the app is stable.

## Considered Options

- Packaged MSIX now: deferred because the actual Partner Center identity is unavailable and local signed-package installation requires certificate trust.
- Inno Setup: rejected because the installed compiler identifies itself as non-commercial use only, which does not meet the commercial-use toolchain policy.
- WiX EXE/MSI: deferred because the current official binary-license terms add an Open Source Maintenance Fee for revenue-generating users above its threshold.
- Framework-dependent deployment: rejected because the first installer must not require a separately installed .NET runtime.

## Consequences

- The initial app uses `WindowsPackageType=None`, a self-contained .NET publish, and a self-contained Windows App SDK publish. It has no package identity.
- NSIS is the installer compiler. Its prebuilt `makensis.exe` CLI runs independently of Visual Studio and its zlib/libpng license permits commercial use.
- The installer must account for the x64 Visual C++ Redistributable target prerequisite.
- Partner Center package name and publisher remain required before later MSIX/Microsoft Store submission. The development manifest identity is not a production identity.
- x64 is the initial architecture. ARM64 can be added after native integration is verified there.
