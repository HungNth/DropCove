# GitHub Pages and release research

## Current repository state

- `HungNth/DropCove` is public, uses `main` as the default branch, and does not currently have GitHub Pages enabled. Source: [GitHub repository API](https://api.github.com/repos/HungNth/DropCove).
- Release `v0.1.0` is already published with `DropCove-Setup-0.1.0.zip`. Its direct browser download URL is `https://github.com/HungNth/DropCove/releases/download/v0.1.0/DropCove-Setup-0.1.0.zip`. Source: [GitHub releases API](https://api.github.com/repos/HungNth/DropCove/releases).
- The repository currently has no `.github/` directory, so no Pages or release workflow is present in this checkout.
- `VERSION` is the canonical project version and currently contains `0.1.0`. `scripts/sync-version.cjs` copies it into `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
- `make build` uses Tauri with `cargo-xwin`, then `scripts/copy-installer.cjs` creates `build/DropCove-Setup-<version>.exe`. No checked-in script currently creates the release ZIP or uploads a GitHub Release asset.

## GitHub Pages facts

- GitHub Pages hosts static HTML, CSS, and JavaScript from a repository. A project site for this repository uses the default URL `https://hungnth.github.io/DropCove/`. Source: [What is GitHub Pages?](https://docs.github.com/en/pages/getting-started-with-github-pages/what-is-github-pages).
- Pages can publish from a branch/folder or from a custom GitHub Actions workflow. GitHub recommends a custom workflow when a custom build process is required. Source: [Configuring a publishing source](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site).
- A custom Pages workflow checks out the repository, prepares the static site, uploads a Pages artifact, then deploys it with `actions/deploy-pages`. The deploy job needs `pages: write` and `id-token: write`; the standard environment name is `github-pages`. Source: [Using custom workflows with GitHub Pages](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).

## Release facts

- GitHub Releases associate a tag with release notes and downloadable binary assets. Releases can be created from GitHub Actions through the GitHub API or GitHub CLI. Source: [Managing releases in a repository](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository).
- A workflow may trigger on a pushed version tag. Adding such a workflow after `v0.1.0` was already pushed does not retroactively run it; it applies to a future tag unless invoked manually or the existing release/tag is deliberately replaced.

## Settled design

- Keep the landing page in `HungNth/DropCove` and publish it at `https://hungnth.github.io/DropCove/`; no custom domain.
- Keep the existing `v0.1.0` release and its direct `DropCove-Setup-0.1.0.zip` URL unchanged. Release automation begins with the next version.
- Store a dependency-free English static site under `website/`, separate from the Vue/Tauri frontend.
- Include only Hero, Features, How it works, Requirements, a direct Download CTA, and a small source/license footer. The first version uses the real Brand Visual but no product screenshots.
- Deploy Pages automatically after website changes reach `main`, allow manual deployment, and redeploy after a successful future release.
- Trigger future releases from `vX.Y.Z` tags, require the tag to match `VERSION`, run existing tests and the cargo-xwin package build, ZIP only the NSIS installer, upload it, then update Pages to that exact release URL.
- Use the existing DropCove dark palette, Segoe UI Variable, left-aligned readable text, and an Edge Rail structural motif. Avoid gradients, generic repeated cards, and decorative motion.
- Show `Download ZIP → Extract → Run the installer`, Windows 11 x64 support, and the Microsoft Visual C++ Redistributable requirement.
- Add no analytics, cookies, forms, third-party fonts, CDNs, or external runtime dependencies. Responsive layout, keyboard focus, forced-colors support, and reduced-motion behavior remain baseline quality requirements.

No glossary change is warranted: GitHub Pages, workflows, tags, and release assets are delivery implementation terms, not DropCove product-domain language. No ADR is warranted because this static hosting choice is inexpensive to reverse and unsurprising from the repository configuration.

Shared understanding confirmed by the user on 2026-11-13; no design-tree decisions remain open.
