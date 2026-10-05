# 01: Prove a reliable native OLE acceptance signal

**Parent specification:** [Reliable Native Drag Acceptance Specification](../spec.md)

**What to build:** Build a throwaway native OLE drag source and controlled destinations that prove whether a source-side result can distinguish destination acceptance from rejection, user cancellation, and a destination that stops during `DragEnter`. This ticket answers the design question; it does not change production drag-out.

**Blocked by:** None (can start immediately).

**Status:** blocked — probe completed; acceptance signal is ambiguous. Production cutover must not start.

**Testing seam:** Use a real OLE source and real controlled destination windows. Correlate one source result and one destination event stream with a unique attempt identifier. No mocked COM forwarding, file-existence inference, or later destination processing.

**Demo path:** Run accepted Copy, Reject, Cancel, and stop-during-`DragEnter` modes. Show the source result and target events for every attempt, then state whether one predicate distinguishes all outcomes.

- [x] The source offers one real filesystem path with Copy-only semantics through native OLE.
- [x] The controlled Copy destination receives `Drop`, receives the offered path, and records acceptance.
- [x] The controlled Reject destination does not satisfy the proposed accepted predicate.
- [x] User cancellation with `Esc` does not satisfy the proposed accepted predicate.
- [ ] The disappearing destination stops during `DragEnter` before assigning an effect or receiving `Drop`, and does not satisfy the proposed accepted predicate. **Failed: 5/5 false positives.**
- [ ] At least five correlated attempts for each of the four outcomes produce the same classification; no failed attempt is discarded or averaged away. **Twenty controlled attempts recorded; Copy and disappearing target are indistinguishable.**
- [x] The verdict distinguishes documented API facts, measured behavior, and inference. No OLE constant or effect is called reliable without the measured correlation.
- [x] If the outcomes remain ambiguous, the ticket records the failure, leaves release blocked, and does not modify production drag-out.
- [ ] If the outcomes are distinguishable, the exact accepted predicate and evidence are recorded in this ticket and ADR 0003 is changed from proposed to accepted. **Not applicable to this failed gate; ADR remains proposed.**
- [x] Throwaway diagnostics and helper processes are removed after evidence capture; no prototype code or runtime artifact ships in the production tree.

## Comments

### 2026-10-05 — Failed native acceptance gate

See [probe verdict and reproduction](../probe-verdict.md) and [complete correlated event streams](../probe-evidence.json). Copy and stop-during-DragEnter both returned `HRESULT=0x00040100`, `effect=1` in 5/5 attempts each. Only Copy received `Drop`. Reject returned DROP/NONE in 5/5; Escape returned CANCEL/NONE in 5/5. The first driver setup failure is preserved separately, not discarded. No production drag-out changes. Tickets 02–04 remain blocked; release remains blocked pending a new design decision.
