# 01: Dismiss Drop Shelf control strip

**What to build:** Add the compact titleless control strip and visible `X` action that let the user Dismiss Drop Shelf without the hotkey. The action must use the Resident coordinator so an empty Shelf becomes `Hidden`, a non-empty Shelf becomes `EdgeDocked`, Manage Items closes, Shelf references remain intact, and DropCove continues running.

**Blocked by:** None. The user confirmed 0.0.13 final human acceptance on 2026-10-16.

**Status:** resolved

**Parent specification:** [Drop Shelf Window Controls](../spec.md)

**Testing seam:** Actual Windows Resident Instance and WebView2 Drop Shelf, supported by the existing Resident/native regression seam and frontend production build.

**Demo path:** Start an owned Resident with an empty Shelf and activate the `X`; verify every Shelf surface hides while the Resident remains available. Reopen, add retained content, open Manage Items, activate the `X`, observe the Rail Handle, then Open Shelf and verify the same references remain.

- [x] A titleless `28`-logical-pixel control strip is present on empty and populated Drop Shelves without restoring the removed product title or subtitle.
- [x] A top-right `24 × 24` `X` action uses the existing icon-control language, exposes the exact accessible name and native tooltip `Hide Drop Shelf`, appears before Shelf Batch/footer actions in natural Tab order, and supports Enter and Space.
- [x] The frontend invokes a resident-owned `dismiss_drop_shelf` operation; the operation accepts only the main Drop Shelf caller and rejects every other window without changing presentation state.
- [x] The `X` does not use frontend window close/hide APIs, destroy the main window, terminate the process, or duplicate display-state rules outside the Resident coordinator.
- [x] Dismissing an empty Shelf produces `Hidden`; dismissing a non-empty Shelf produces `EdgeDocked`; native close, `Esc`, hotkey, and `X` paths expose the same observable state outcomes.
- [x] Dismissal closes Manage Items through its established lifecycle, retains every Temporary and Pinned reference, preserves source files, and keeps notification-area and later activation workflows available.
- [x] A failed dismiss request leaves the Drop Shelf in its last coherent visible state and reports the failure through existing status or resident warning behavior; it never falls back to raw hiding.
- [x] The new strip is included in presentation chrome and Automatic Shelf Growth calculations without adding a persisted sizing field, changing the `191 × 180` minimum preferred outer size, or causing premature overflow for ordinary batch counts.
- [x] Empty-state content, card density, footer/status behavior, rounded transparent-frame treatment, topmost policy, Manual Height Override, and eight-direction resize behavior remain intact.
- [x] Dark, Light, Windows Contrast, hover, focus-visible, disabled/busy, and reduced-motion presentations remain legible and consistent with existing semantic tokens.
- [x] Supporting regression coverage protects the Resident display transitions and caller boundary without adding forwarding-only, source-text, CSS-class, screenshot, or frontend component-framework tests.
- [x] The frontend production build and a real Windows Resident/WebView2 smoke of the empty and non-empty demo path pass, and the canonical product documentation records the delivered control and Dismiss Drop Shelf behavior.

## Answer — implementation and observed evidence, 2026-10-16

- Frontend production typecheck/build and cargo-xwin Windows x64 release build passed. Restored the existing optional icon-size prop after typechecking caught its accidental removal; no consumer contract was changed.
- Baseline actual WebView2 had only `Drop here` and no close action; rebuilt native Resident rendered a 28px strip and a 24×24 top-right close target at x=158/y=3. A point near that button's top-left corner hit the button rather than the resize overlay. Actual screenshot inspected the empty surface.
- Actual pointer X on empty Shelf hid both surfaces and retained the owned Resident; secondary activation returned 0. With a two-item batch and open Manage Items, X closed the popup, hid Shelf, displayed the Rail Handle, and retained both references after Open Shelf.
- Real IPC from Edge Rail and Manage Items rejected `dismiss_drop_shelf` without changing presentation; main-window X and Enter activation worked. No raw window hide/close permission was added.
- Existing growth tiers were retained: their current whitespace accommodates the strip; manual sizing schema is unchanged. Final integrated theme, drag, resize, installer, and human acceptance checks remain Ticket 03 release coverage, not claimed here.
- Focused native Resident regression executable passed 17/17 cases, including hide failure, docking rollback, retained residency, popup cleanup, work-area clamping, and repeat activation. Actual Enter and Space activated X with visible focus. Dark/Light/forced-colors and reduced-motion emulation produced distinct foreground/background colors; actual Windows Contrast Theme remains unobserved, not claimed as physical theme acceptance.
- Closed a remaining existing Escape parity gap: the first Escape now closes ready Manage Items without dismissing Shelf; a distinct second Escape docks the non-empty Shelf. Native WM_CLOSE and registered WM_HOTKEY dispatch also exercised dismissal. Physical injected hotkey chords were unreliable on this host and are not claimed as hardware-input coverage.
- Real pointer resizing changed geometry through North, NorthEast, East, SouthEast, South, SouthWest, West, and NorthWest. A held native resize exposed `aria-busy=true`, disabled X, and opacity0.45; release restored controls. Actual hover exposed the semantic hover background.
- A four-row, 348px-high Shelf measured 28px strip, 48px cards, and identical 212px scrollHeight/clientHeight after dismissing the existing status alert. The initial 9px overflow occurred only while a real error/status alert occupied layout space; ordinary growth did not require changing the established tiers.
- A temporary owned-database table rename induced a real snapshot error: X preserved the visible Shelf and displayed `database error: no such table: shelf_batches`. The table was restored immediately; all owned source bytes remained intact.
