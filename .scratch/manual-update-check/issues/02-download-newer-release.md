# 02: Download newer release

**What to build:** When the Latest Release is newer than the running version, About shows `Version X.Y.Z is available.` and the action becomes `Download X.Y.Z`. Activating it opens that release's GitHub page in the default browser. The user then downloads and runs the setup through the existing installer and Cooperative Shutdown workflow. Release pages outside the DropCove repository's releases path are rejected as invalid data. See [spec](../spec.md).

**Blocked by:** 01 — Update Check results in About

**Status:** completed

- [x] A Latest Release greater than the running version yields `available` with its normalized version and `html_url`, but only when `html_url` starts with `https://github.com/{owner}/{repo}/releases/`. Otherwise the result is `failed`.
- [x] About renders the `available` state with the exact spec copy. The status line announces it politely. The state remains until About closes.
- [x] `Download X.Y.Z` opens the validated page through the existing opener plugin, as the Source link does. A browser-opening failure appears in About's existing error area, not the update status line.
- [x] The About capability's opener allow-list adds `https://github.com/HungNth/DropCove/releases/*` beside the existing repository URL and grants nothing broader.
- [x] `node:test` coverage adds: a higher release yields `available` with the correct version and page; out-of-order releases offer the highest; an `html_url` outside the releases path yields `failed`.
- [x] Demo: with a stubbed or locally served release list containing a version newer than the running build, the action becomes `Download X.Y.Z` and opens the correct GitHub release page in the default browser. (Verified by agent smoke and user testing.)
