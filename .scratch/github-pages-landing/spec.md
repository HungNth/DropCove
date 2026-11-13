# DropCove — GitHub Pages landing and release delivery

Status: ready-for-agent
Design confirmation: approved by the user on 2026-11-13 through `grill-with-docs`
Testing seam: the actual GitHub Actions delivery seam, exercised through a non-publishing release validation run and a browser/download smoke of the deployed public GitHub Pages surface; the first future approved version tag provides the final release-publication proof
Current release: preserve the existing stable `v0.1.0` release and `DropCove-Setup-0.1.0.zip` asset unchanged
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), [Domain Glossary](../../GLOSSARY.md), and [release research](./research.md)
Installer policy: [ADR-0001](../../docs/adr/0001-cargo-xwin-and-per-user-nsis.md) and [ADR-0004](../../docs/adr/0004-custom-nsis-and-cooperative-shutdown.md)

## Problem Statement

DropCove has a public source repository and a published Windows installer, but it has no public product website. A prospective user must read repository documentation and navigate GitHub Releases to understand what DropCove does, whether it supports their system, how to use it, and which file to download. There is no focused landing page with a direct installer download.

The current repository also has no GitHub Actions delivery workflow. The existing `v0.1.0` ZIP was published manually. Future releases therefore depend on a maintainer manually keeping the canonical version, Windows installer, ZIP filename, GitHub Release, and public download link aligned. A missed or reordered step can expose a stale link, mismatched version, incomplete release, or untested installer.

## Solution

Publish a small English-language static landing page for DropCove through GitHub Pages at `https://hungnth.github.io/DropCove/`. The page explains the product's core features and workflow, states its Windows 11 x64 and runtime requirements, gives concise installation instructions, and offers a primary button that downloads the exact ZIP asset from the current stable GitHub Release.

Keep the website independent from the Vue/Tauri application frontend and free of runtime services, tracking, forms, third-party fonts, and external asset dependencies. Its visual language uses the existing Brand Visual, DropCove palette, Segoe UI Variable, left-aligned readable copy, and an Edge Rail structural motif.

Add GitHub Actions delivery for both surfaces. Pages deployment publishes website changes from the default branch, supports a safe manual run, resolves the current published release before generating the version and direct-download link, and leaves the prior deployment live if release metadata is incomplete. Starting with the version after `v0.1.0`, a version tag validates the canonical version, runs the existing tests and cargo-xwin package build, archives the NSIS installer, publishes a stable GitHub Release, and then deploys Pages against that newly available asset.

## User Stories

1. As a prospective DropCove user, I want a dedicated product page, so that I can understand DropCove without reading development documentation.
2. As a prospective DropCove user, I want the opening copy to explain that DropCove is a temporary drag-and-drop workspace, so that I can quickly decide whether it solves my problem.
3. As a prospective DropCove user, I want to know that DropCove is not a file manager, clipboard history, or cloud-sync product, so that I do not install it with the wrong expectations.
4. As a cautious user, I want the page to state that DropCove never moves, alters, or deletes original External References, so that I understand the source-safety guarantee.
5. As a Windows user, I want the page to state that DropCove supports Windows 11 x64, so that I can check compatibility before downloading.
6. As a Windows user, I want the minimum Windows build stated, so that I can determine whether my installation is supported.
7. As a Windows user, I want the Microsoft Visual C++ Redistributable requirement stated and linked, so that I can satisfy the runtime prerequisite.
8. As a prospective user, I want to learn that DropCove accepts files and folders, so that I understand its basic intake workflow.
9. As a prospective user, I want to learn that DropCove accepts Website Image Virtual Files, so that I know browser images can become usable files without a source path.
10. As a prospective user, I want to learn that DropCove accepts non-empty text as Managed Files, so that I know text snippets can be carried between applications.
11. As a prospective user, I want to understand that one drop creates a Shelf Batch, so that grouped items and their order make sense.
12. As a prospective user, I want to understand the Drop Shelf, so that I know where accepted content appears.
13. As a prospective user, I want to understand the Edge Rail, so that I know retained Shelf content remains available after the Drop Shelf is dismissed.
14. As a prospective user, I want to learn about Shake-to-open, so that I know the Shelf can appear during an active drag.
15. As a prospective user, I want to learn that whole Shelf Batches and selected Shelf Items can be dragged out, so that I understand the main completion workflow.
16. As a prospective user, I want to learn the difference between Temporary Items and Pinned Items, so that I understand when references are consumed or retained.
17. As a prospective user, I want to know that Shelf state survives application and system restarts, so that I understand the resident and persistent behavior.
18. As a prospective user, I want the page to explain the workflow in a short real sequence, so that I can learn the product before installing it.
19. As a first-time installer, I want the instructions to say `Download ZIP`, `Extract`, and `Run the installer`, so that I know exactly what to do with the downloaded asset.
20. As a first-time installer, I want to know that installation is per-user and does not require administrator elevation, so that I can anticipate the setup experience.
21. As a prospective user, I want one obvious Download action, so that I do not have to choose among source archives and unrelated GitHub assets.
22. As a prospective user, I want the Download action to start the official ZIP download directly, so that I do not have to navigate an intermediate release page.
23. As a prospective user, I want the Download action to identify Windows x64 and the published version, so that I know which binary I am receiving.
24. As a prospective user, I want the displayed version and direct-download URL to describe the same published release, so that the page never mislabels an installer.
25. As a prospective user, I want the direct-download link published only after its asset exists, so that I do not encounter a newly advertised broken download.
26. As a prospective user, I want access to the source repository and Apache-2.0 license from a small footer, so that I can inspect the project without distracting from the primary download flow.
27. As a keyboard user, I want every interactive element reachable with visible focus, so that I can navigate and download without a pointer.
28. As a screen-reader user, I want semantic headings, landmarks, links, and meaningful Brand Visual alternative text, so that the page structure and actions are understandable.
29. As a Windows Contrast Theme user, I want content, focus, dividers, and actions to remain distinguishable, so that the page stays usable under forced colors.
30. As a user who prefers reduced motion, I want the page to avoid nonessential motion, so that it respects my system preference.
31. As a user visiting from a narrow or zoomed viewport, I want the content to reflow without horizontal scrolling or clipped actions, so that I can read the page comfortably.
32. As a privacy-conscious visitor, I want no analytics, cookies, forms, third-party fonts, CDNs, or client-side release requests, so that the site exposes no unnecessary data or runtime dependency.
33. As a visitor on a slow connection, I want a small static page with local assets, so that the product information and download action appear quickly.
34. As a maintainer, I want the website source separated from the Vue/Tauri frontend, so that landing-page work cannot alter embedded application surfaces.
35. As a maintainer, I want the landing page to require no additional package or static-site framework, so that dependency installation and upgrades remain unchanged.
36. As a maintainer, I want website changes on the default branch to deploy automatically, so that the public page follows accepted source changes.
37. As a maintainer, I want a safe manual Pages deployment trigger, so that I can recover or republish without changing source.
38. As a maintainer, I want Pages generation to resolve the repository's current stable release at deployment time, so that ordinary content edits do not advertise an unreleased canonical version.
39. As a maintainer, I want Pages generation to require the correctly named ZIP asset, so that missing or mismatched release packaging fails before public deployment.
40. As a maintainer, I want a pushed `vMAJOR.MINOR.PATCH` tag to be the release signal, so that publishing is explicit and auditable.
41. As a maintainer, I want a release tag to match the canonical `VERSION`, so that package metadata and GitHub Release identity cannot drift.
42. As a maintainer, I want malformed or mismatched tags rejected before build publication, so that accidental tags never create official releases.
43. As a maintainer, I want the existing frontend tests to pass before publishing, so that known user-facing regressions block a release.
44. As a maintainer, I want the official cargo-xwin Windows x64 package build to pass before publishing, so that the released installer comes from the required toolchain.
45. As a maintainer, I want the release ZIP to contain only the matching NSIS installer, so that users receive the same simple archive shape as `v0.1.0`.
46. As a maintainer, I want a normal stable GitHub Release created only after tests, build, and archive checks succeed, so that the Update Check can treat it as a Latest Release.
47. As a maintainer, I want Pages redeployed only after the new release asset is available, so that the website changes atomically from the prior working download to the new one.
48. As a maintainer, I want failed release or Pages jobs to leave the existing release and deployed site intact, so that automation failure does not remove a working download.
49. As a maintainer, I want a non-publishing release validation run on the real GitHub Actions runner, so that runner setup, tests, cargo-xwin packaging, and ZIP shape can be proven without creating or replacing a release.
50. As a maintainer, I want workflow permissions restricted to each job's required repository, release, Pages, and identity operations, so that delivery automation has no unnecessary authority.
51. As a maintainer, I want concurrent Pages runs serialized with stale runs canceled, so that an older deployment cannot overwrite newer accepted content.
52. As a maintainer, I want the existing `v0.1.0` tag, release, and asset left unchanged, so that adding automation does not rewrite already published history.
53. As a maintainer, I want the public homepage metadata and repository documentation to identify the Pages site, so that users and tools can discover the canonical product page.
54. As a maintainer, I want canonical documentation to stop claiming that no public product site exists, so that product documentation remains factual after launch.

## Implementation Decisions

- **Repository and hosting.** The landing page remains in the existing public DropCove repository and is published as its GitHub Pages project site at `https://hungnth.github.io/DropCove/`. No second repository, publishing branch, Jekyll setup, or custom domain is introduced.
- **Static website module.** Add one dependency-free static website module, separate from the Vue/Tauri application frontend. It consists of semantic HTML, CSS, the existing Brand Visual, and only local assets. Runtime JavaScript is omitted unless an accessibility behavior cannot be expressed with HTML and CSS; release/version resolution happens before deployment, not in the visitor's browser.
- **Content contract.** The page contains one Hero, Features, How it works, Requirements, a direct Download call to action, and a small source/license footer. Copy is English, factual, active-voice, and grounded in the canonical product specification and glossary. It does not invent testimonials, usage metrics, security guarantees, or unsupported capabilities.
- **Product language.** Public copy uses the established terms Shelf, Drop Shelf, Edge Rail, Shelf Batch, Shelf Item, External Reference, Managed File, Temporary Item, Pinned Item, Shake-to-open, Brand Visual, Cooperative Shutdown, and Latest Release where their precision is useful. Plain introductory wording may explain those terms but must not redefine them.
- **Visual system.** Reuse the application palette: dark canvas `#0c0c0c`, surface `#161616`, primary text `#f5f5f5`, muted text `#929292`, accent/focus `#7aa2ff`, and structural border `#3a3a3a`. Use Segoe UI Variable with system fallbacks, left-aligned copy with readable line lengths, and one Edge Rail-inspired structural motif. Avoid decorative gradients, repeated generic SaaS cards, remote fonts, fake application UI, and section-by-section entrance animation.
- **Responsive and accessible behavior.** The layout works from narrow mobile viewports through wide desktops, at browser zoom, with keyboard-only navigation, visible focus, semantic landmarks and heading order, useful alternative text, forced colors, and reduced motion. These are baseline behavior, not optional modes.
- **Distribution copy.** The primary action clearly identifies Windows 11 x64 and the resolved release version. Requirements state Windows 11 build 22000 or later and the x64 Microsoft Visual C++ Redistributable. Installation copy is the approved three-step sequence: `Download ZIP`, `Extract`, `Run the installer`. The current-user, no-elevation installer behavior may be stated. The page does not imply that DropCove auto-updates.
- **Published release interface.** Before producing a Pages artifact, deployment resolves the repository's current stable GitHub Release and normalizes its semantic version. It requires an asset named `DropCove-Setup-<version>.zip` whose release tag is `v<version>`, then injects that version and browser download URL into the static output. Missing, draft, pre-release, malformed, mismatched, or asset-less metadata fails the deployment; the last successful Pages deployment remains live.
- **Pages delivery module.** Use GitHub's supported Pages artifact and deployment actions with the `github-pages` environment. The module's interface supports default-branch website changes, an explicit manual run, and invocation after successful release publication. All callers use the same generation and deployment implementation so release-triggered and content-triggered deployments cannot drift.
- **Project-site paths.** Website asset and navigation URLs remain valid under the `/DropCove/` project-site base path. The source must also be locally previewable without hard-coding the GitHub host.
- **Release signal and version invariant.** Automated stable release publication begins after the existing `v0.1.0` release. A pushed tag must match exactly `vMAJOR.MINOR.PATCH`; its normalized version must equal the trimmed canonical `VERSION`. The synchronized package, crate, lockfile, Tauri, PE, installer, archive, and release identities must remain aligned with that version.
- **Release validation mode.** The release delivery module also supports an explicit manual validation run that executes setup, version synchronization checks, tests, cargo-xwin packaging, archive creation, and archive inspection on the real hosted runner. Validation mode uploads only a temporary Actions artifact; it never creates or edits a GitHub Release and never deploys Pages.
- **Build and test gate.** Release delivery installs the locked JavaScript dependencies with pnpm, runs the existing frontend regression suite, and invokes the established cargo-xwin/Tauri/NSIS build path for `x86_64-pc-windows-msvc`. It does not substitute the Visual Studio compiler flow or introduce another packaging command.
- **Archive contract.** The freshly built `DropCove-Setup-<version>.exe` is compressed as `DropCove-Setup-<version>.zip`. Archive inspection must prove it contains exactly that one installer at the archive root before publication.
- **GitHub Release contract.** After all gates pass, create a non-draft, non-prerelease release for the existing pushed tag and attach the versioned ZIP. Do not replace an existing release or asset with the same tag/name; a collision fails safely. GitHub release notes may be generated from the repository history. Only after the asset upload succeeds may release delivery invoke Pages delivery.
- **Failure and concurrency.** Any validation, test, build, archive, upload, or Pages generation failure stops downstream publication. Pages deployments use one concurrency group, cancel stale in-progress deployments, and never delete the last successful deployment. Release jobs do not cancel one another after publication work begins.
- **Permissions.** Default workflow permissions are read-only. Release publication receives only repository-content write access. Pages deployment receives only repository-content read, Pages write, and identity-token write access. Untrusted pull-request code never receives publication permissions and never deploys.
- **Current release preservation.** The existing `v0.1.0` release and its `DropCove-Setup-0.1.0.zip` asset are inputs to the first Pages deployment and are never recreated, retagged, replaced, or deleted by this work.
- **Product metadata and documentation.** The public Pages URL becomes the canonical product homepage metadata and is linked from repository documentation. Canonical documentation is updated where it currently says no public product site exists. The About Surface remains unchanged: **Source** continues to open the repository, and no Website row or automatic update behavior is added.
- **No domain-model addition.** GitHub Pages, workflow triggers, tags, and release assets are delivery implementation language, not DropCove product-domain terms. No glossary entry or ADR is added; the hosting choice is inexpensive to reverse and unsurprising from repository configuration.

## Testing Decisions

- A good test crosses the public delivery interface and asserts behavior a visitor or maintainer can observe: deployed content, accessible interaction, resolved version, exact download target, successful archive shape, release state, and failure-safe ordering. Tests do not assert CSS selector names, HTML source formatting, workflow YAML text, action step names, internal helper boundaries, or incidental generated markup.
- **Primary seam — GitHub Actions delivery and public Pages surface.** Exercise the actual hosted workflows rather than introducing mocks for GitHub Pages or GitHub Releases. The first Pages deployment uses the already published stable `v0.1.0` release. A manual non-publishing release validation run exercises the same hosted runner setup, existing tests, cargo-xwin build, and archive path used by a tag release.
- **Pages deployment proof.** A successful default-branch or manual Pages run must resolve `v0.1.0`, find `DropCove-Setup-0.1.0.zip`, upload one Pages artifact, deploy through the `github-pages` environment, and report `https://hungnth.github.io/DropCove/` as the public URL.
- **Browser surface proof.** Open the deployed site in a real browser and verify the Hero, Features, How it works, Requirements, direct Download action, and source/license footer. Check desktop and narrow viewports, zoom/reflow, keyboard order, visible focus, heading/landmark semantics, Brand Visual alternative text, forced colors, and reduced motion. Confirm there are no runtime console errors, third-party network requests, cookies, forms, horizontal overflow, or broken local assets.
- **Download proof.** Activate the primary Download action from the deployed page. Confirm the browser reaches the exact `v0.1.0` release asset URL, downloads a valid ZIP, and that the archive contains exactly `DropCove-Setup-0.1.0.exe` at its root. The displayed version, accessible action name, downloaded ZIP name, and contained installer name must agree.
- **Pages failure proof.** In a non-deploying validation context, exercise release metadata with a missing matching ZIP, malformed version tag, draft/pre-release result, and mismatched asset name. Each case must stop artifact deployment rather than emit a fallback or stale newly generated link. Do not alter the live `v0.1.0` release to create these cases.
- **Release validation proof.** Manually dispatch release validation on the hosted runner. It must run the existing `pnpm test` suite, the established cargo-xwin Windows package build, produce the expected versioned installer and ZIP, inspect the one-file archive shape, and upload only a temporary Actions artifact. Confirm that it creates no tag, release, release asset, or Pages deployment.
- **Tag validation proof.** The non-publishing validation interface must demonstrate that an exact synthetic `v<canonical-version>` value is accepted and malformed or version-mismatched values fail before package publication. This tests the observable version invariant without rewriting the existing tag.
- **Future release proof.** The first future user-approved stable version tag is the final end-to-end proof of release publication: one successful gated build creates one normal GitHub Release and one correctly named ZIP asset, then Pages redeploys with the same version and direct URL. Until that event is observed, reports must identify release creation and post-release Pages invocation as configured but not yet exercised.
- **Regression checks.** The complete existing frontend suite and production frontend build run unchanged. The release path uses the established cargo-xwin/Tauri/NSIS packaging seam and preserves current-user installation, Cooperative Shutdown, and Profile Data contracts; this feature does not require repeating unrelated installed-app interaction matrices solely for a website-only change.
- **Prior art.** Follow the repository's existing exact-candidate release gates: canonical version synchronization, freshly built cargo-xwin installer, exact artifact naming, archive inspection, and evidence that distinguishes observed behavior from unobserved limits. For actual-surface verification, follow existing release tickets that require real WebView/browser behavior rather than source-text assertions. The repository has no prior GitHub Actions or Pages module to reuse.

## Out of Scope

- A custom domain, DNS records, redirects from another domain, or a separate website repository.
- Jekyll, another static-site generator, a publishing branch, or reuse of the application's Vue, Vite, Tailwind, and Tauri frontend as the website build.
- Product screenshots, recordings, animated demos, fake application mockups, testimonials, download counts, or usage metrics.
- Additional pages, routing, blog posts, pricing, accounts, forms, newsletters, comments, support portals, or localization.
- Analytics, advertising, cookies, consent banners, remote fonts, CDNs, or visitor-side GitHub API requests.
- Automatic semantic-version selection, automatic version bumps, changelog policy, release scheduling, or publishing without an explicit approved version tag.
- Rebuilding, deleting, replacing, retagging, or otherwise changing the existing `v0.1.0` release and asset.
- Draft, pre-release, nightly, beta, multi-channel, macOS, Linux, ARM64, Microsoft Store, or portable distributions.
- Code signing, SBOM generation, checksum/signature assets, vulnerability scanning, dependency update automation, or release attestation beyond GitHub's normal asset metadata.
- Any change to Shelf behavior, Drop Shelf, Edge Rail, Settings, About Surface interactions, Update Check behavior, installer UI, Cooperative Shutdown, Profile Data, or the running application's network behavior.
- A Website action in the About Surface, in-application downloads, an automatic updater, background release checks, or automatic installer launch.

## Further Notes

- Repository state observed during specification: `HungNth/DropCove` is public with `main` as its default branch; GitHub Pages is not enabled; no GitHub Actions directory exists; `VERSION` is `0.1.0`; and stable release `v0.1.0` already contains the required ZIP asset.
- Enabling **Settings → Pages → Build and deployment → Source: GitHub Actions** is a one-time repository-admin action. An implementation agent may prepare and validate the workflow source, but public deployment remains blocked until an authenticated repository administrator enables that source if it is not already enabled.
- GitHub Pages infrastructure records visitor IP addresses under GitHub's own privacy policy even though DropCove adds no analytics or tracking.
- The release workflow is intentionally not retroactive. Adding it after `v0.1.0` does not cause that tag to run again.
- The next workflow step is `to-tickets`, followed by explicit user approval of the ticket set before implementation.
