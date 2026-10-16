# 01: Dedicated Drop Shelf Clear Temporary Items confirmation

**What to build:** Move the Drop Shelf Clear Temporary Items decision into a readable native confirmation window without changing the established Clear mutation. From the user’s perspective, the bottom-left trash action opens one complete confirmation outside the compact Drop Shelf; cancellation is harmless, confirmation removes only current Temporary Items, failures retain prior state, and the Edge Rail workflow remains unchanged.

**Blocked by:** None. User confirmed final 0.0.14 human acceptance on 2026-10-16.

**Status:** resolved

**Implementation gate:** Granted by user on 2026-10-16 after accepting 0.0.14. Git commit authorization remains separate.

**Parent specification:** [Drop Shelf Utility Menu and About Surface](../spec.md)

**Testing seam:** Actual Windows Resident Instance with the real WebView2 Drop Shelf and dedicated native confirmation, supported by the existing Shelf/native regression suite and frontend production build.

**Demo path:** Start an owned Resident with a minimum-size Drop Shelf containing a mixed Temporary/Pinned batch. Open Clear, inspect the complete copy and actions, cancel through `Escape`, reopen and confirm, then verify only Temporary references leave the Shelf and every source file remains unchanged. Repeat a request while the window is open and exercise a persistence failure.

- [x] Activating the Drop Shelf’s conditional Clear Temporary Items action opens one decorated native confirmation outside the Drop Shelf viewport; no in-shelf Clear dialog remains for this action.
- [x] The confirmation uses the approved `360 × 200` logical size, standard `DropCove` title bar, fixed sizing, no minimize/maximize controls, taskbar-hidden presentation, always-on-top behavior, Drop Shelf ownership, and initial centering on the Drop Shelf monitor work area.
- [x] The complete approved content is visible at the Drop Shelf’s `191 × 180` minimum: heading `Clear temporary items?`, body `Pinned items stay. Source files and folders aren’t deleted.`, and actions `Cancel` and `Clear`.
- [x] `Cancel` receives initial focus. Cancel, `Escape`, and the confirmation close action close the window, clear modal state, preserve every Shelf Item, and restore focus coherently when the Drop Shelf remains active.
- [x] Repeated Clear requests refresh current availability and show, raise, and focus the existing confirmation instead of creating another window.
- [x] Only the Drop Shelf can request this confirmation. Requests from About, Settings, Manage Items, Edge Rail, or another window are rejected without presentation or Shelf-state changes.
- [x] While the confirmation is open, Dismiss Drop Shelf, native movement, resize initiation, Shelf drag-out, Drop Shelf utility actions, duplicate Clear mutation, and competing Drop Shelf transactions cannot begin.
- [x] Explicit Exit DropCove and Cooperative Shutdown remain authoritative, close the confirmation, release modal state, and complete normal teardown without deadlock or orphaned native state.
- [x] Confirm re-checks authoritative Temporary Item availability immediately before mutation rather than trusting the snapshot that opened the window.
- [x] If no Temporary Items remain at confirmation time, the window closes safely, publishes current Shelf state, and performs no removal of unrelated items.
- [x] Confirm delegates to the existing serialized, atomic Clear Temporary Items operation; it removes Temporary Items across Shelf Batches, preserves Pinned Items, and never deletes source filesystem objects.
- [x] Persistence failure leaves visible and durable Shelf state unchanged, closes the accepted confirmation as specified, and reports the failure through existing Shelf status or resident warning behavior without fallback deletion.
- [x] The existing Edge Rail Clear confirmation, transient hold, fullscreen cancellation, collapse behavior, and action placement remain unchanged and never open this Drop Shelf confirmation window.
- [x] The confirmation receives only the capabilities required for its view and close/focus lifecycle; it receives no Shelf mutation, filesystem, drag, resize, or unrelated resident permissions.
- [x] Dark, Light, Windows Contrast, keyboard focus, screen-reader names, disabled/busy states, and reduced-motion presentation use existing DropCove conventions and remain legible.
- [x] Supporting permanent regression coverage catches stale confirmation, duplicate-window, unauthorized-caller, modal-guard, retained-Pinned, atomic-failure, and source-safety regressions without forwarding-only, source-text, CSS-class, or screenshot tests.
- [x] The frontend production build, cargo-xwin Windows build, complete produced native test executable, and actual Windows demo path pass. Canonical product and architecture documentation records the delivered Drop Shelf confirmation ownership and preserved Edge Rail behavior.
- [x] No Git commit is created without separate explicit user authorization.

## Answer — implementation and exercised evidence, 2026-10-16

- Reproduced the original clipping on the real 191×180 WebView: dialog client height 158 px versus scroll height 183 px; the Clear action extended below its visible frame. Dedicated native confirmation measured 360×200 with all content visible, Cancel initially focused, and buttons fully within the client area.
- Actual Cancel, Escape, and native X preserved references. Native X returned foreground activation to the enabled Drop Shelf. Repeated request reused the confirmation; modal owner disablement and IPC guards blocked Hide, Move, Settings, and About.
- Actual confirmed Clear removed Temporary references while retaining Pinned references and unchanged source bytes. An owned SQLite delete-failure trigger preserved both visible and durable state and produced a visible error. A stale confirmation with all items pinned closed without removing references.
- Both monitors were exercised at scale 1: complete Clear outer frame centered in the active Drop Shelf monitor work area. Explicit Cooperative Shutdown with confirmation open exited 0, then restart restored the unconfirmed Temporary and Pinned references with no modal state.
- Existing Windows native suite passed 111/111. Added production-helper geometry and caller authorization coverage; cross-window lifecycle is verified at the approved actual-Windows seam, not through forwarding or source-text tests. Frontend production build and cargo-xwin release build passed.
- Physical high/mixed-DPI, physical Windows Contrast Theme, and screen-reader output remain human-acceptance limits. Edge Rail source lifecycle was retained, with native regression support; physical fullscreen combinations were not re-measured. No Git commit was created.
