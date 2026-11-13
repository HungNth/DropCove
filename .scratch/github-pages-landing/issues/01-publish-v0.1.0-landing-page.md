# 01: Publish the v0.1.0 DropCove landing page

**Parent specification:** [GitHub Pages landing and release delivery](../spec.md)

**What to build:** Publish the approved English DropCove landing page at the repository's GitHub Pages project URL. A prospective user can understand the product, its supported workflows and requirements, then download the existing official `v0.1.0` ZIP directly. The public site, current release metadata, accessibility behavior, repository homepage metadata, and documentation must agree end to end without changing the existing release.

**Blocked by:** None (can start immediately).

**Status:** ready-for-human

- [ ] The public project site loads successfully at `https://hungnth.github.io/DropCove/` through the repository's GitHub Pages environment.
- [x] The page contains only the approved Hero, Features, How it works, Requirements, direct Download action, and small source/license footer.
- [x] English copy accurately explains DropCove as a temporary drag-and-drop workspace and uses established Shelf, Drop Shelf, Edge Rail, Shelf Batch, Shelf Item, External Reference, Managed File, Temporary Item, Pinned Item, and Shake-to-open language where precision is useful.
- [x] Copy states the Windows 11 build 22000 or later, x64, Microsoft Visual C++ Redistributable, current-user installation, and no-administrator-elevation requirements without implying automatic updates.
- [x] Installation guidance is exactly the approved sequence: `Download ZIP`, `Extract`, `Run the installer`.
- [x] Pages generation resolves the current stable release before deployment, normalizes `v0.1.0`, and requires the matching `DropCove-Setup-0.1.0.zip` asset.
- [x] The primary action identifies Windows x64 and version `0.1.0`, then starts the exact existing GitHub Release asset download without an intermediate page.
- [x] Activating the deployed Download action retrieves a valid ZIP containing exactly `DropCove-Setup-0.1.0.exe` at its root.
- [x] Missing, malformed, draft, pre-release, mismatched, or asset-less release metadata stops a new deployment without replacing the last successful site or emitting a fallback link.
- [ ] Website changes accepted on the default branch deploy automatically; a safe manual deployment is available; the same Pages delivery interface can be invoked after a future release.
- [ ] Pages deployment uses the standard GitHub Pages environment, least-required permissions, and concurrency that prevents stale runs from overwriting newer accepted content.
- [x] The site uses the real Brand Visual, approved DropCove palette, Segoe UI Variable with system fallbacks, left-aligned readable copy, and one Edge Rail-inspired structural motif.
- [x] The site contains no screenshots, fake application UI, decorative gradients, generic repeated card grid, or nonessential entrance animation.
- [x] A real-browser smoke proves responsive reflow without horizontal overflow, useful heading and landmark structure, meaningful Brand Visual alternative text, keyboard order, visible focus, forced-colors usability, reduced-motion behavior, and working local assets.
- [x] Browser inspection finds no runtime errors, analytics, cookies, forms, third-party fonts, CDNs, client-side GitHub API calls, or other external runtime dependencies.
- [x] The source repository and Apache-2.0 license remain available from a small footer without competing with the primary Download action.
- [x] The canonical product homepage metadata and repository documentation identify the public Pages URL, and canonical documentation no longer claims that DropCove has no public product site.
- [x] The About Surface remains unchanged: **Source** still opens the repository and no Website row or automatic-update behavior is added.
- [x] The existing `v0.1.0` tag, release, and asset remain byte-for-byte and metadata-identical; this ticket never recreates, replaces, retags, or deletes them.
- [ ] Verification records the Pages workflow run, deployed URL, browser surface observations, exact download URL, archive contents, and any repository-admin action needed to enable GitHub Actions as the Pages source.

## Comments

Local implementation, unit tests (`tests/buildPages.test.mjs`), live release resolution (`v0.1.0` -> `DropCove-Setup-0.1.0.zip`), archive root inspection (`DropCove-Setup-0.1.0.exe`), and headless Chrome verification (`1280x900`, `375x812`, keyboard focus, `forced-colors`, `prefers-reduced-motion`, zero external requests/console errors) passed on 2026-11-13. Remote GitHub Pages deployment remains pending human commit/push authorization and repository admin configuration (**Settings -> Pages -> Source: GitHub Actions**).
