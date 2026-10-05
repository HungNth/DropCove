# 03: Manage multi-item Shelf Batches in an anchored popup

**Parent specification:** [Responsive Resizable Drop Shelf Specification](../spec.md)

**What to build:** Replace inline expansion of multi-item Shelf Batches with one non-modal, accessible popup anchored to the selected card. Keep the Drop Shelf's preferred size stable while providing complete item inspection, management, and drag-out behavior outside the shelf bounds when space permits.

**Blocked by:** 02: Reflow Shelf Batches in a responsive grid.

**Status:** needs-triage

**Testing seam:** Reuse the existing manager and visual-coordinator behavior seams for item lifecycle and deferred visuals. Use installed Release UI Automation and direct interaction for popup geometry, focus, outside-click dismissal, and real drag initiation.

**Demo path:** Open a multi-item batch by pointer and keyboard, switch to another batch, manage and drag individual items, start a whole-batch drag, resize the shelf, remove the owning batch, and exercise `Esc` precedence and focus return.

- [x] Inline multi-item expansion/collapse and its retained item projections are removed from the Drop Shelf.
- [ ] Single-item Shelf Batches have no popup trigger and retain direct drag, Pin/Unpin, and Remove behavior on the card.
- [ ] A multi-item batch chevron opens and closes a non-modal popup anchored to that card; `Enter` and `Space` provide equivalent keyboard activation.
- [x] Only one popup can be open. Opening another batch replaces the current popup, and clicking outside closes it.
- [ ] Placement preference is right, left, below, then above the card, followed by clamping inside the current work area with a `16` logical-pixel margin where possible.
- [ ] Popup width sizes to content between `320` and `480` logical pixels; height sizes to content up to `480`; a smaller work area may override minimums to keep the popup reachable.
- [ ] Popup overflow scrolls vertically only. Long names and paths are visually truncated while complete values remain available to tooltips and accessibility names.
- [ ] Each popup item exposes icon or thumbnail, name, path, availability, pinned state, Pin/Unpin, Remove Item, and individual drag-out.
- [ ] Remove Batch and whole-batch drag remain on the card, and Clear Temporary Items remains in the footer.
- [x] A keyboard-opened popup moves focus to the first available item or action; `Tab` traverses all controls; closing restores focus to the originating chevron when it still exists. Verified on current published Release with isolated test profile: Enter/Space open to `Pin Item_01.txt`; Tab and Shift+Tab traverse all 20 actions including virtualized items; bidirectional wrap verified; Esc restores focus to `Manage 10 items`. See [keyboard evidence](qualification-keyboard-evidence.json).
- [x] `Esc` closes the popup before shelf dismissal. A subsequent `Esc` follows the existing Drop Shelf dismissal behavior. Verified: first Esc closes popup and restores owner focus; second Esc dismisses shelf to Edge Rail.
- [ ] Hiding the shelf, deleting the owner batch, removing the final item, beginning shelf resize, starting whole-batch drag, or otherwise invalidating the owner closes the popup without leaving an orphaned surface.
- [ ] Starting an individual item drag keeps the popup open. Canceled, rejected, and failed drags preserve references; successful drag-out applies existing Temporary/Pinned semantics and updates or closes the popup according to the remaining batch state.
- [ ] Full item view models and visual requests are created only for the open popup and are released when it closes, its owner recycles, or the shelf hides; compact cards retain only bounded preview data.
- [ ] Opening, closing, updating, or scrolling the popup never changes the Drop Shelf's preferred or actual window size.
- [ ] Installed Release smoke verifies placement fallback, work-area clamping, size limits, one-at-a-time lifecycle, pointer and keyboard dismissal, focus restoration, all item actions, item and batch drag, resize closure, owner deletion, and accessibility names.

## Comments

### Installed implementation evidence — incomplete; product decision required

- Clean self-contained Release was published to `%LOCALAPPDATA%/Programs/DropCove` after removing every temporary drag/coordinate diagnostic and reverting the ineffective layout experiment. Source search for `DEBUG-item-drag`, `DEBUG-popup-bounds`, both diagnostic paths, and `WindowFromPoint` returned no matches. No native PopupHost interception was added.
- Clean installed item-drag smoke: genuine SendInput cancellation preserved the reference and open popup; rejection preserved both; an actual Copy target copied a pinned item and retained its reference, and copied a temporary item and removed its reference. Manual item removal kept the remaining popup open; deleting its owner closed it. One click on Settings opened the real Settings window and closed the popup. Shelf bounds remained `180 × 180` throughout these mutations.
- Whole-batch acceptance was correlated to new target events only, with the main shelf first confirmed visible after the real `Ctrl+Shift+Space` chord. The target received and copied all 10 paths with effect `Copy` and zero copy errors; all 10 temporary references were removed. An earlier apparent failure read an old single-item Drop event after canceled batch drag had dismissed the shelf; it was a probe-selection error, not a demonstrated batch-consumption defect.
- Clean installed right/left/below/above content placement was exercised on 96-DPI DISPLAY2. Tiny `280 × 200` work-area probing calculated content bounds `(1936,16)–(2184,184)`, satisfying the 16-pixel margin. The native host shadow extended to y=202; strict whole-host containment was not proved and is not substituted for the deterministic content-placement contract. No HWND-discovery workaround is authorized. Owned AppBars were reset/removed and saved user window placements restored after every probe.
- Earlier installed keyboard traversal covered all 20 actions, bidirectional wrap, scroll reset and Esc focus return. A later clean keyboard replay lost the intended native focus and produced no popup-focused actions; it is not passing evidence. Desktop input stopped when the user resumed interaction. Remaining final keyboard/tooltip/accessibility and closure coverage must not be inferred from the passing unit suite.
- **Blocking contract gap:** Two correlated disappearing-target attempts exited inside DragEnter before assigning an effect or receiving Drop; WinUI nevertheless reported completed Copy and consumed the temporary item. `RequestedOperation=None` preserved failed/rejected references but prevented legitimate Copy/Drop, so it was reverted. Manager mapping and file-existence heuristics cannot establish destination acceptance. Failed-target retention remains red; no reliable native/OLE acceptance signal is established. Escalate for a product decision before changing this approved requirement or qualifying Ticket 04.
- Referenced-source integrity was checked for all 1,000 canonical fixture paths across clean accepted copies and manual removal: hashes unchanged. Fixture snapshots/probes remain isolated under the existing temporary smoke directory for resumption; they are not live user data. The latest original profile, `original-before-resumed-smoke`, was restored before relaunch, not the older snapshot.
- Full suite after these corrections: `dotnet test tests/DropCove.Tests/DropCove.Tests.csproj --no-restore` — 155 passed, 0 failed, 0 skipped, 507 ms. The final clean installed app responded to WM_NULL within 2 seconds on the restored original profile. No final NSIS/release qualification claim.
- Separate Standards and Spec reviews of the newest pointer-release/item-drag/activation lifetime changes returned zero new findings; this is not whole-ticket qualification. Before delivery, both user-cited `Batch001-Item02.txt` and `Batch001-Item04.txt` still existed, zero native probe helper processes remained, and the clean original-profile app was still running. No further synthetic desktop input was sent.
- **Clean verification remains blocked:** the required latest keyboard replay produced `keyboard_action_order = [None × 21]`, did not establish popup focus, and failed the action-order assertion. Passing automated tests, earlier keyboard evidence, and scoped reviews do not complete this required installed smoke path. The verification todo must remain blocked until a correctly focused clean installed replay passes; no replay is attempted during user desktop interaction.

### 2026-10-05 — Controlled destination follow-up

The user-approved test-only profile seam enabled actual DropCove popup drags from a clean self-contained Release published side-by-side under the installed app directory. Five attempts each for Copy, Reject, Escape, and primary exit during DragEnter over Reject backing matched reference lifecycle requirements; a further five primary-exit attempts over Copy backing proved genuine acceptance by that backing target. Exact target paths/callback returns and persisted item IDs were correlated. The earlier “verified API mismatch” inference did not instrument the destination exposed underneath the failed primary and is not established by that primary-only oracle. Historical destination identity remains unknown; native cutover awaits a planning decision. See [verdict](../../reliable-drag-acceptance/installed-profile-verdict.md) and [full evidence](../../reliable-drag-acceptance/installed-profile-evidence.json). No whole-ticket, keyboard, geometry, compatibility, resource, or NSIS release qualification is added by this scoped follow-up.

### 2026-10-05 — Keyboard traversal passed; geometry blocked by normal app crash

- Focused keyboard replay on current published Release with isolated test profile passed all 20 actions, bidirectional wrap, Enter/Space activation, and Esc precedence. Verified foreground and focused element for every physical key stroke; no foreign input delivered. See [keyboard evidence](qualification-keyboard-evidence.json) and captured popup states (`keyboard-open.png`, `keyboard-last-row.png`).
- Real Pin/Unpin updated accessible action name while preserving 180×180 bounds. Full path is exposed through accessible properties. Moving the owner window closed the popup.
- Geometry fallback verification on DISPLAY2 was interrupted: normal installed instance PID 26020 crashed (Event 1000: CoreMessagingXP.dll, exception 0xc000027b; Event 1001: combase.dll, 0x80004005). Owned DISPLAY2 AppBars were immediately reset and helper exited. Stopped all desktop input. Normal app was NOT restarted and its profile was NOT altered. See [environment diagnosis](qualification-environment-change.json).
- Ticket 03 remains blocked pending resolution of the crash cause and explicit restart/recovery direction.
