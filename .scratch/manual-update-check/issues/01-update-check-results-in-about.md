# 01: Update Check results in About

**What to build:** The About Surface's **Check for updates** action becomes enabled and performs a user-requested Update Check. Activating it shows `Checking…` with the action `aria-disabled` so keyboard focus stays on it. DropCove then requests the repository's GitHub releases and determines the Latest Release. The status line reports one of these results: up to date, no release, failure, or rate limit. The Download action belongs to ticket 02. Until it lands, a newer Latest Release shows only `Version X.Y.Z is available.`, and the action stays `Check for updates`. See [spec](../spec.md) for the full state table and request contract.

**Blocked by:** None (can start immediately)

**Status:** completed

- [x] A pure Update Check module accepts the running version and the canonical repository URL from the existing About metadata. It returns the spec's discriminated `UpdateCheckResult` using the platform `fetch`, with no new Tauri command, crate, npm package, or updater plugin.
- [x] The request is `GET https://api.github.com/repos/{owner}/{repo}/releases` with `Accept: application/vnd.github+json`, aborted after `10 s`. A repository URL that is not a GitHub repository URL yields `failed`.
- [x] Latest Release selection works as follows:
  - drafts, pre-releases, and tags not matching `v?MAJOR.MINOR.PATCH` are excluded, and an optional leading `v` is ignored;
  - the highest version wins numerically, so `0.0.10` beats `0.0.9`;
  - no qualifying release yields `no-release`;
  - a Latest Release not greater than the running version yields `up-to-date`.
- [x] HTTP 403/429 yields `rate-limited`. Any other non-success status, network rejection, abort, non-array body, or release missing `tag_name`/`draft`/`prerelease`/`html_url` yields `failed`.
- [x] The About action and status line follow the spec copy for Idle, Checking, Up to date, No release, Failure, and Rate limit. `Coming soon` and the disabled placeholder styling are removed. The status line is a polite live region and remains the action's description.
- [x] Each activation performs a fresh check. State is discarded when About closes. Enter/Space activation and visible focus work, and Dark/Light/Contrast rendering and reduced motion are preserved. (Verified by agent smoke and user testing.)
- [x] `node:test` coverage with a stubbed `globalThis.fetch` covers every result above: empty, drafts-only, pre-releases-only, malformed tags, out-of-order tags, `v`/bare equality, multi-digit comparison, equal and higher running version, 403, 429, other status, rejection, timeout, non-array, missing fields, and non-GitHub repository. It passes through `pnpm test`.
- [x] Demo: in the development app, activating the action against the live API shows `No releases published yet.`, matching the repository's current state. Simulated offline shows the failure copy.
