# DropCove 0.0.20 — Manual update check

Status: ready-for-agent
Version: 0.0.20
Design confirmation: approved by the user on 2026-10-22 through `grill-with-docs`
Testing seam: one pure frontend Update Check module exercised with `node:test` against stubbed GitHub release responses, plus exact installed Windows WebView2 About Surface smoke against the live GitHub API; approved by the user on 2026-10-22
Delivery prerequisite: completed — DropCove 0.0.19 is accepted and committed; the canonical roadmap defines 0.0.20 as the next release
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), especially Updating and About Surface
Update policy: [ADR-0004](../../docs/adr/0004-custom-nsis-and-cooperative-shutdown.md) — installer-based updates with Cooperative Shutdown remain the only update path
Domain language: [Glossary](../../GLOSSARY.md), especially **About Surface**, **Update Check**, and **Latest Release**
Roadmap: [DropCove 0.0.20](../../docs/ROADMAP.md#0020--manual-update-check)

## Problem Statement

The About Surface shows a disabled **Check for updates** action labelled **Coming soon**. A DropCove user has no way to learn from inside the application whether a newer release exists. They must remember to visit the repository, find the releases page, and compare version numbers by hand. As a result, users keep running outdated versions without knowing it.

## Solution

**Check for updates** becomes a working, user-requested Update Check. When activated, DropCove asks GitHub for the repository's published releases, determines the Latest Release, and compares it with the running version. A single status line reports the result.

When a newer release exists, the action becomes **Download X.Y.Z**. It opens that release's GitHub page in the default browser, where the user downloads and runs the versioned setup as today. DropCove never downloads or installs anything itself, never checks in the background, and never checks at startup. The existing installer and Cooperative Shutdown workflow remains the only way to update.

## User Stories

1. As a DropCove user, I want the About Surface's **Check for updates** action to be enabled, so that I can ask whether a newer DropCove exists.
2. As a DropCove user, I want the check to run only when I activate the action, so that DropCove makes no network requests I did not ask for.
3. As a DropCove user, I want the action to show **Checking…** and become temporarily disabled while a check runs, so that I know it is working and cannot start duplicate checks.
4. As a DropCove user, I want to see **You're up to date.** when my running version matches the Latest Release, so that I can stop wondering.
5. As a DropCove developer running a local build newer than any release, I want the check to report **You're up to date.**, so that a development build is never offered an older release.
6. As a DropCove user, I want to see **Version X.Y.Z is available.** when a newer release exists, so that I know exactly which version I would get.
7. As a DropCove user, I want the action to become **Download X.Y.Z** when a newer release exists, so that the next step is one obvious click.
8. As a DropCove user, I want **Download X.Y.Z** to open the release's GitHub page in my default browser, so that I can download the official setup and see its release assets.
9. As a DropCove user, I want to update by running the downloaded setup exactly as before, so that my Profile Data and Shelf survive through the existing Cooperative Shutdown workflow.
10. As a DropCove user, I want draft releases ignored, so that unpublished work is never offered to me.
11. As a DropCove user, I want pre-releases ignored, so that I am only offered stable versions.
12. As a DropCove user, I want release tags that are not semantic versions (such as `nightly` or `test`) ignored, so that one unusual tag does not break the check.
13. As a DropCove user, I want tags written with or without a leading `v` (for example `v0.0.20` and `0.0.20`) treated the same, so that tag style does not affect the result.
14. As a DropCove user, I want the highest semantic version chosen even if releases were published out of order, so that I am always offered the true Latest Release.
15. As a DropCove user, I want to see **No releases published yet.** when the repository has no qualifying release, so that I understand there is nothing to compare against rather than seeing an error.
16. As a DropCove user, I want to see **Couldn't check for updates. Try again later.** when the network is unavailable, so that I know the check failed and can retry.
17. As a DropCove user, I want a check that receives no response within `10 s` to stop and report the same failure, so that the action never hangs indefinitely.
18. As a DropCove user, I want an unexpected or malformed GitHub response reported as the same failure, so that bad data never produces a misleading result.
19. As a DropCove user, I want to see **Update check limit reached. Try again later.** when GitHub rate-limits the request, so that I know retrying immediately will not help.
20. As a DropCove user, I want the action to return to **Check for updates** after any non-available result, so that I can retry whenever I choose.
21. As a DropCove user, I want each activation to perform a fresh check, so that I can confirm a release that was published moments ago.
22. As a DropCove user, I want the check result discarded when I close About, so that reopening About starts from a clean state.
23. As a DropCove user relying on a screen reader, I want each result announced politely, so that I learn the outcome without moving focus.
24. As a keyboard user, I want to activate **Check for updates** and **Download X.Y.Z** with Enter or Space, so that the feature works without a pointer.
25. As a keyboard user, I want focus to remain visible on the action across state changes, so that I never lose my place.
26. As a DropCove user, I want every status message to fit on one line inside the fixed `400 × 460` About Surface, so that the layout never shifts or clips.
27. As a DropCove user using Dark, Light, or Windows Contrast Theme, I want the action and status line to follow the active appearance, so that the feature looks consistent with the rest of About.
28. As a DropCove user who prefers reduced motion, I want state changes to occur without animation, so that the feature respects my system preference.
29. As a security-conscious user, I want DropCove to open only release pages inside the DropCove repository's releases path, so that a tampered response cannot send my browser to an arbitrary site.
30. As a security-conscious user, I want the About Surface's browser-opening permission limited to the repository and its releases path, so that the WebView cannot open other URLs.
31. As a maintainer, I want the GitHub owner and repository derived from the Cargo package repository metadata, so that the check follows the canonical source without a second frontend copy.
32. As a maintainer, I want publishing a normal GitHub release with a semantic version tag to be enough for users to discover it, so that releasing requires no extra manifest or signing step.
33. As a maintainer, I want failures to open the browser still reported in About's existing error area, so that system failures remain distinguishable from update results.

## Implementation Decisions

- **Update Check module.** Add one pure frontend module that performs the whole Update Check. Its input is the running version and the canonical repository URL. It uses the platform `fetch` and returns one discriminated result. The About Surface is its only consumer. The result shape:

  ```ts
  type UpdateCheckResult =
    | { kind: "up-to-date" }
    | { kind: "available"; version: string; url: string }
    | { kind: "no-release" }
    | { kind: "rate-limited" }
    | { kind: "failed" };
  ```

- **Request contract.** `GET https://api.github.com/repos/{owner}/{repo}/releases` with `Accept: application/vnd.github+json`, so the browser sends no CORS preflight. Owner and repository come from the GitHub repository URL already returned by the About metadata command, which reads it from Cargo package metadata. A repository URL that is not a GitHub repository URL yields `failed`. The request is aborted after `10 s`. Only the default first page is requested.
- **Response interpretation.**
  - HTTP 403 or 429 yields `rate-limited`.
  - Any other non-success status, network error, abort, non-array body, or release missing a required field yields `failed`.
  - Required fields per release: `tag_name`, `draft`, `prerelease`, `html_url`.
- **Latest Release selection.**
  - Exclude drafts and pre-releases.
  - Ignore tags not matching `v?MAJOR.MINOR.PATCH`, where each part is a non-negative integer, and strip an optional leading `v`.
  - Compare versions numerically by major, then minor, then patch, and choose the highest.
  - No qualifying release yields `no-release`. A Latest Release that is not greater than the running version yields `up-to-date`. A higher one yields `available`.
- **Release page validation.** An `available` result is produced only when the Latest Release's `html_url` starts with the repository's releases path (`https://github.com/{owner}/{repo}/releases/`). Any other URL yields `failed`.
- **About Surface states.** The existing action and its described-by status line replace the disabled placeholder and **Coming soon** copy. The status line is a polite live region.

  | State | Action | Status |
  | --- | --- | --- |
  | Idle | `Check for updates` | empty |
  | Checking | `Checking…`, `aria-disabled` so keyboard focus stays on the action | empty |
  | Up to date | `Check for updates` | `You're up to date.` |
  | Newer release | `Download X.Y.Z` | `Version X.Y.Z is available.` |
  | No release | `Check for updates` | `No releases published yet.` |
  | Failure or invalid data | `Check for updates` | `Couldn't check for updates. Try again later.` |
  | HTTP 403 or 429 | `Check for updates` | `Update check limit reached. Try again later.` |

  The action uses the existing About button styling in its enabled form and keeps the focus-ring tokens. When a newer release is available, the action renders as a primary button with accent fill and text, and the status line renders as an accent-subtle callout with an accent border, semibold text, and an inline download icon. Update results appear only in the status line. A browser-opening failure uses About's existing error area.
- **Download action.** Activating **Download X.Y.Z** opens the validated release page through the existing opener plugin, the same way the **Source** link opens the repository. The available state remains until About closes.
- **State lifetime.** Update Check state lives only in the About Surface component. Closing About destroys it. Nothing is persisted or cached by DropCove. Browser caching of the response follows GitHub's headers.
- **Permissions.** The About capability's opener allow-list gains `https://github.com/HungNth/DropCove/releases/*` beside the existing repository URL. No new Tauri command, backend module, crate, npm package, updater plugin, Settings field, or CSP change is added.
- **Contingency.** If the exact installed WebView2 blocks the GitHub request, stop and request user approval before moving the request into a Rust command. Do not silently change the design.
- **Documentation.** The canonical specification already describes Updating and the About Surface behavior above, and the glossary defines **Update Check** and **Latest Release**. Keep both synchronized with the delivered copy. Manifests are synchronized to `0.0.20` in the release gate.

## Testing Decisions

- A good test asserts only the observable contract: given a running version, a repository URL, and a stubbed HTTP response, the module returns the expected discriminated result. Tests do not assert internal helpers, request headers beyond what changes behavior, or component markup.
- **Seam:** the Update Check module, exercised with `node:test` and a stubbed `globalThis.fetch`. It is imported directly from TypeScript source like the existing Shell Visual composable test. The full Node suite runs through `pnpm test`.
- **Behavior coverage:**
  - empty list yields `no-release`;
  - drafts-only, pre-releases-only, and malformed-tags-only lists yield `no-release`;
  - out-of-order tags select the highest version;
  - `v`-prefixed and bare tags compare equally;
  - multi-digit parts compare numerically (`0.0.10` > `0.0.9`);
  - equal and higher running versions yield `up-to-date`;
  - a higher release yields `available` with its version and page;
  - an `html_url` outside the releases path yields `failed`;
  - 403 and 429 yield `rate-limited`;
  - other non-success status, network rejection, abort/timeout, non-array body, and missing fields yield `failed`;
  - a non-GitHub repository URL yields `failed`.
- **Prior art:** the existing Node suite under the repository test directory, especially the Shell Visual composable test that imports TypeScript source and stubs platform globals.
- **Installed smoke:**
  - From the exact installed `0.0.20` candidate with the dev server stopped, activate the action against the live GitHub API and observe the real current state. Today that state is `No releases published yet.`
  - Exercise network failure and keyboard activation. Exercise screen-reader announcement where available.
  - Measure that every status message fits one line in Dark, Light, and emulated forced colors.
  - The available-release path requires a published release newer than the candidate during human acceptance.
- Existing native and frontend checks run unchanged: the full native Windows suite, the frontend production build, and cargo-xwin release/NSIS packaging.

## Out of Scope

- Downloading, verifying, or launching setup from DropCove.
- `tauri-plugin-updater`, signed update manifests, or any installer template change.
- Startup, periodic, or background release checks, notifications about updates, and any Settings preference for update checking.
- Pre-release or beta channels.
- Displaying release notes, changelogs, asset sizes, or publication dates.
- Authenticated GitHub requests or pagination beyond the first page of releases.
- Changing About Surface geometry, metadata rows, or the notification-area menu.

## Further Notes

- GitHub was observed on 2026-10-22 to answer both the simple request and its preflight for origin `http://tauri.localhost` with `Access-Control-Allow-Origin: *`. Unauthenticated requests are limited to 60 per hour per IP address. Actual WebView2 behavior remains to be observed in the installed smoke.
- The repository currently has no published releases, so `No releases published yet.` is the expected live result until the first release exists.
- The next workflow step is `to-tickets`, followed by explicit user approval of the ticket set before implementation.
