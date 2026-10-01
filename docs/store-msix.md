# DropCove MSIX and Microsoft Store pipeline

DropCove has two independent release paths:

- **Unpackaged EXE:** `installer/DropCove.nsi` remains the local EXE installer path.
- **MSIX / Microsoft Store:** `scripts/build-msix.ps1` is the separate Store-oriented path. It never changes the unpackaged project deployment model.

The MSIX pipeline targets **x64 only** and publishes the complete self-contained `DropCove.App` output. It does not create GitHub Releases, WinGet, Scoop, portable, x86, or ARM64 artifacts.

## Development package

Development builds use a distinct identity and an ephemeral self-signed certificate:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-msix.ps1 `
  -Configuration Development `
  -Version 0.1.0.0
```

The default development identity is:

```text
Name:      DropCove.Development
Publisher: CN=DropCove Development
```

The script creates the PFX only under the operating-system temporary directory, signs the MSIX and MSIXBundle, writes the public `.cer` certificate beside the artifacts, and deletes the private PFX before validation completes. The certificate is self-signed and is never publicly trusted.

The following is the documented local-test recipe for a supported Windows test host. Run it from an **elevated PowerShell** session and trust only the emitted public certificate on that test machine:

```powershell
$artifactRoot = '.\artifacts\msix\development\0.1.0.0'
$certificate = Join-Path $artifactRoot 'DropCove-0.1.0.0-development.cer'
$package = Join-Path $artifactRoot 'DropCove-0.1.0.0-x64.msix'

Import-Certificate -FilePath $certificate -CertStoreLocation Cert:\LocalMachine\TrustedPeople
Add-AppxPackage -Path $package
```

The certificate must be present in `LocalMachine\TrustedPeople` immediately before `Add-AppxPackage`; this is local test trust, not public trust. The available non-elevated host was tested with the earlier `CurrentUser\TrustedPeople` sequence and still returned `0x800B0109` (untrusted root). The corrected elevated path was not exercised in this session, so package installation and runtime launch remain unclaimed. Remove the test package and certificate after any successful smoke run. Do not publish the development package.

## Production Store submission package

Production generation fails closed unless both Partner Center values are supplied at build time and the Windows App Certification Kit is available. Partner Center values are intentionally not stored in the repository. The script auto-resolves `appcert.exe` from PATH or the standard Windows SDK location; use `-CertificationKitPath` when it is installed elsewhere.

AppCertKit is required because Microsoft’s Store package guidance recommends Windows App Certification Kit validation before submission. It remains an external Windows SDK tool: the repository does not add a NuGet dependency, install it, redistribute it, or store it. The applicable [Microsoft Windows SDK license terms](https://download.microsoft.com/download/0/F/F/0FF2B061-47DD-4F55-89B6-FD1D8C44F14D/preview_sdk_license.rtf) permit SDK installation and use to design, develop, test, compile, build, verify, and archive programs; this pipeline invokes the tool only for Production validation.

Run the production command from an elevated PowerShell session with an active interactive user session:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-msix.ps1 `
  -Configuration Production `
  -Version 1.0.0.0 `
  -IdentityName '<Partner-Center-package-name>' `
  -Publisher '<Partner-Center-publisher>' `
  -PublisherDisplayName 'DropCove'
```

The pipeline runs `appcert.exe reset`, then `appcert.exe test` once for the x64 MSIX and once for the x64 MSIXBundle, writing `wack-msix-report.xml` and `wack-msixbundle-report.xml`. It fails if the kit is missing, either command fails, either report is missing, or either XML report lacks an explicit passing result or contains a failed result. Microsoft documents the [Windows App Certification Kit command-line flow](https://learn.microsoft.com/en-us/windows/uwp/debug-test-perf/windows-app-certification-kit).

The production output is an **unsigned Store submission artifact**. Microsoft Store signs the package during submission; the local pipeline does not substitute the development certificate for the Partner Center identity. The build metadata records `SignatureState: UnsignedStoreSubmission`, both AppCertKit report names and pass results, identity, version, architecture, self-contained status, artifact hashes, and custom validation report names.

Partner Center supplies the production package identity and publisher after reserving the app name. Microsoft documents the manual `MakeAppx.exe` packaging shape and the Store signing hand-off:

- [Generating MSIX package components](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-manual-conversion)
- [Sign your MSIX package](https://learn.microsoft.com/en-us/windows/msix/package/sign-msix-package-guide)
- [Create an app submission for MSIX apps](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/create-app-submission)

## Validation performed by the pipeline

Each build creates and validates both:

```text
DropCove-<version>-x64.msix
DropCove-<version>-x64.msixbundle
```

Validation unpacks the artifact and checks:

- Partner Center/development identity, publisher, and four-part version;
- `x64` package architecture and `Windows.Desktop` target;
- `DropCove.exe` packaged-classic-app entry point at medium integrity;
- `runFullTrust` capability;
- self-contained .NET and Windows App SDK runtime files;
- package visual assets;
- absence of private-key material from the package and bundle;
- no x86 or ARM64 package is included;
- Windows App Certification Kit production validation and its retained report.

Reports are emitted as `validation-msix.json`, `validation-msixbundle.json`, and, for Production, `wack-msix-report.xml` plus `wack-msixbundle-report.xml` under the versioned artifact directory. All generated output is ignored by Git.

## Store lifecycle gate

Installation, upgrade, and uninstall against a **Store-signed release candidate** remain an external Partner Center gate. The repository provides the reproducible package and validation path, but it cannot claim that gate until Microsoft Store has signed and returned a release candidate for supported Windows 11 smoke testing.

DropCove adds no telemetry, background updater, or application-owned release-check request. Store listing metadata, screenshots, pricing, age rating, and privacy-policy declarations remain Partner Center submission fields rather than repository secrets.
