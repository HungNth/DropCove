# 03: Drop Shelf Utility Menu for Settings and About

**What to build:** Add the always-available Drop Shelf Utility Menu so users can open the established Settings surface or the shared About Surface directly from the compact Drop Shelf. The menu must fit the minimum Drop Shelf, follow accessible menu behavior, preserve the conditional Clear action, and introduce no Edge Rail menu.

**Blocked by:** 02 — About Surface from the notification area.

**Status:** resolved

**Implementation gate:** Granted by user on 2026-10-16 after accepting 0.0.14; completion of Ticket 02 remains required. Git commit authorization remains separate.

**Parent specification:** [Drop Shelf Utility Menu and About Surface](../spec.md)

**Testing seam:** Actual Windows Resident Instance with the real WebView2 Drop Shelf, existing Settings window, shared About Surface, keyboard/focus behavior, and minimum-size layout, supported by the frontend build and native regression suite.

**Demo path:** Open an empty minimum-size Drop Shelf, activate More with pointer and keyboard, dismiss the menu through `Escape` and outside click, open Settings and verify singleton reuse, then reopen the menu and open the existing About Surface. Repeat with Temporary content and the Clear confirmation open.

- [x] An always-visible `24 × 24` bottom-right More action appears on empty and populated Drop Shelves without restoring a title/subtitle or obscuring collection, status, scrollbar, movement, or resize behavior.
- [x] The conditional Clear Temporary Items action remains bottom-left while Temporary Items exist; More remains bottom-right regardless of Shelf content.
- [x] More uses the established icon-control language, `currentColor`, semantic hover/focus/disabled states, the accessible name and native tooltip `More`, and the standard compact target size.
- [x] Activating More opens an in-WebView Drop Shelf Utility Menu above the action, approximately `132` logical pixels wide, containing exactly `Settings` followed by `About`.
- [x] More exposes menu availability and expanded state; the menu and its items expose the correct roles, names, order, and focus semantics.
- [x] Pointer, Enter, Space, Up/Down, Home/End, Tab dismissal, `Escape`, and outside click follow the approved accessible menu-button behavior.
- [x] Dismissal returns focus to More unless focus intentionally transfers to Settings or About. The menu closes before either native surface is opened or focused.
- [x] Settings uses the established resident-owned singleton opener. Opening it from the Drop Shelf shows or focuses the existing Settings window without creating a second settings lifecycle or changing Save/Cancel behavior.
- [x] About uses the exact shared resident-owned opener delivered by Ticket 02. Drop Shelf and notification-area requests show/focus the same About Surface and never create separate content or lifecycle implementations.
- [x] Frontend utility requests accept only the intended Drop Shelf caller; unrelated windows cannot use them to open privileged resident surfaces or change presentation state.
- [x] A Settings or About opening failure closes the transient menu, retains or restores coherent Drop Shelf focus, and uses existing status/resident warning behavior without a raw frontend fallback.
- [x] While the dedicated Drop Shelf Clear confirmation owns modal interaction, More and the Drop Shelf Utility Menu cannot open or invoke either utility surface.
- [x] The menu is not added to the expanded Edge Rail or Rail Handle, and existing Edge Rail actions, transient holds, no-activate behavior, and placement remain unchanged.
- [x] The footer and popover remain fully usable at the `191 × 180` minimum and representative wider/resized layouts, including empty, Temporary-only, Pinned-only, mixed, status-bearing, and scrollable Shelf states.
- [x] Dark, Light, Windows Contrast, reduced-motion, keyboard focus, screen-reader state, target sizing, and forced-color presentation reuse existing DropCove conventions.
- [x] Supporting permanent regression coverage catches incorrect item order, focus-return, singleton reuse, modal preemption, unauthorized caller, and minimum-layout regressions without CSS-class, source-text, screenshot, or forwarding-only tests.
- [x] The frontend production build, cargo-xwin Windows build, complete produced native test executable, and actual Windows Drop Shelf/Settings/About demo path pass. Canonical product and architecture documentation records the Drop Shelf Utility Menu and resident utility entry points.
- [x] No Git commit is created without separate explicit user authorization.

## Answer — implementation and exercised evidence, 2026-10-16

- Actual 191×180 Drop Shelf showed an always-present 24×24 bottom-right More action and conditional bottom-left Clear. Empty, Temporary, Pinned, mixed, and error-bearing states were exercised without replacing the existing card or resize layout.
- Actual menu roles/order and keyboard End/Home/ArrowDown/Enter/Escape behavior were exercised. Escape closed only the menu and returned focus to More. Settings opened the existing singleton and Cancel retained its normal lifecycle; About reused the shared native surface.
- Installed smoke exposed an outside-pointer focus race: native pointer default blurred the synchronously restored More focus. The pointer handler now prevents that default before closing; the final rebuilt/reinstalled candidate returned focus to More on populated and empty Shelves. Clicking an outside Pin All control still activated it and pinned both references, proving outside actions were not suppressed.
- Actual modal Clear state disabled More and backend utility navigation. Unrelated window callers were rejected. Edge Rail received no Utility Menu.
- Actual Windows native OLE Copy smoke on final installed bytes copied exact Temporary and Pinned source bytes to an owned Forms destination, consumed the Temporary reference, and retained the Pinned reference. Injected physical Escape cancellation was inconclusive and is not claimed as passed; native cancellation state regressions passed in the existing suite.
- Standards review found no actionable defect. Spec review findings for OLE Drop modal guard, Vue focus timing, and utility opener-error focus were fixed and re-reviewed; the focused outside-pointer fix was separately reviewed with no remaining finding. Full native suite passed 111/111; final frontend and cargo-xwin/NSIS builds passed. No Git commit was created.
