# 01: Prove a reliable native OLE acceptance signal

**Parent specification:** [Reliable Native Drag Acceptance Specification](../spec.md)

**What to build:** Build a throwaway native OLE drag source and controlled destinations that prove whether a source-side result can distinguish destination acceptance from rejection, user cancellation, and a destination that stops during `DragEnter`. This ticket answers the design question; it does not change production drag-out.

**Blocked by:** None (can start immediately).

**Status:** blocked for planning — controlled native and actual DropCove WinUI matrices pass; no native advantage is established. Historical destination identity is unknown. ADR remains proposed and production cutover must not start without a planning decision.

**Testing seam:** Use a real OLE source and real controlled destination windows. Correlate one source result and one destination event stream with a unique attempt identifier. No mocked COM forwarding, file-existence inference, or later destination processing.

**Demo path:** Run accepted Copy, Reject, Cancel, and stop-during-`DragEnter` modes. Show the source result and target events for every attempt, then state whether one predicate distinguishes all outcomes.

- [x] The source offers one real filesystem path with Copy-only semantics through native OLE.
- [x] The controlled Copy destination receives `Drop`, receives the offered path, and records acceptance.
- [x] The controlled Reject destination does not satisfy the proposed accepted predicate.
- [x] User cancellation with `Esc` does not satisfy the proposed accepted predicate.
- [x] The disappearing destination stops during `DragEnter` before assigning an effect or receiving `Drop`, and does not satisfy the proposed accepted predicate. **Controlled native 5/5 returns NONE over Reject backing; actual DropCove 5/5 retains the offered reference. Historical primary-only observations cannot identify the complete accepted destination.**
- [x] At least five correlated attempts for each of the four outcomes produce the same classification; no failed attempt is discarded or averaged away. **Complete native 5×4, isolated WinUI 5×4, and real DropCove 5×4 matrices match expected acceptance with controlled Reject backing. Diagnostic/driver failures remain in evidence. Overall design gate is not approved.**
- [x] The verdict distinguishes documented API facts, measured behavior, and inference. No OLE constant or effect is called reliable without the measured correlation.
- [x] If the outcomes remain ambiguous, the ticket records the failure, leaves release blocked, and does not modify production drag-out.
- [ ] If the outcomes are distinguishable, the exact accepted predicate and evidence are recorded in this ticket and ADR 0003 is changed from proposed to accepted. **Controlled native predicate matches target oracle, but no advantage over WinUI established and installed premise unverified; ADR remains proposed pending design decision.**
- [x] Throwaway diagnostics and helper processes are removed after evidence capture; no prototype code or runtime artifact ships in the production tree.

## Comments

### 2026-10-05 — Failed native acceptance gate

See [probe verdict and reproduction](../probe-verdict.md) and [complete correlated event streams](../probe-evidence.json). Copy and stop-during-DragEnter both returned `HRESULT=0x00040100`, `effect=1` in 5/5 attempts each. Only Copy received `Drop`. Reject returned DROP/NONE in 5/5; Escape returned CANCEL/NONE in 5/5. The first driver setup failure is preserved separately, not discarded. No production drag-out changes. Tickets 02–04 remain blocked; release remains blocked pending a new design decision.

### 2026-10-05 — Controlled oracle correction

See [direct callback and WinUI comparison](../controlled-probe-verdict.md) and [all 85 correlated attempts](../controlled-probe-evidence.json). Both sources pass the complete 5×4 matrix with Reject backing. With Copy backing, both return Copy only while the backing logs real Drop acceptance; this is a successful destination, not acceptance by the failed primary. The earlier raw observations are preserved, but absence of primary Drop alone was not a complete oracle. No installed DropCove acceptance is claimed. Existing code has no profile override, and a disposable runtime check proved LOCALAPPDATA override does not redirect GetFolderPath(LocalApplicationData); safe installed reproduction requires an approved isolation seam. User profile was not modified.

### 2026-10-05 — Approved profile seam and real DropCove matrix

User approved the narrow test-only profile seam. See [real-app verdict](../installed-profile-verdict.md) and [25 correlated actual popup drags](../installed-profile-evidence.json). Cancel/Reject/primary-exit over Reject backing preserve the Temporary reference in 5/5 each. Accepted primary Copy and accepted backing Copy consume only the offered Temporary reference in 5/5 each; the pinned sentinel remains in every attempt. All offered/sentinel bytes and the live user's database/settings hashes remain unchanged, and PID 408 stays running. This is a clean self-contained side-by-side publish under the installed app directory, not an NSIS install or historical-destination reconstruction. Existing WinUI passes the same oracle as native; keep Tickets 02–04 blocked and ADR proposed pending a planning decision rather than replacing the source engine without established benefit.
