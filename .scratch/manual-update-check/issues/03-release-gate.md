# 03: 0.0.20 release gate

**What to build:** An exact installed DropCove `0.0.20` candidate whose About Surface Update Check works against the live GitHub API in the packaged WebView2. Verification evidence is recorded in the roadmap, and the user gives human acceptance. See [spec](../spec.md) and the [roadmap entry](../../../docs/ROADMAP.md#0020--manual-update-check).

**Blocked by:** 01 — Update Check results in About; 02 — Download newer release

**Status:** ready-for-agent

- [ ] Every project manifest and the running About version report `0.0.20`.
- [ ] `pnpm test`, the frontend production build, the full native Windows suite, and cargo-xwin release/NSIS packaging pass. The installer path, size, and SHA-256 are recorded.
- [ ] From the exact installed candidate with the dev server stopped, activating the action reaches the live GitHub API and shows the real current state, today `No releases published yet.`. If WebView2 blocks the request, stop and request user approval before moving the request into a Rust command.
- [ ] Network failure, keyboard activation, focus visibility, and (where available) screen-reader announcement are exercised on the installed candidate.
- [ ] Every status message, including the longest, fits on one line inside the fixed `400 × 460` About Surface in Dark, Light, and emulated forced colors.
- [ ] Specification §3.3/§5.4 and the glossary match the delivered copy. ROADMAP `0.0.20` status and observed evidence are updated, with unobserved limits stated.
- [ ] Human acceptance: the user confirms the candidate, including the available-release path once a release newer than the candidate is published. Git commit remains a separate explicit authorization.
