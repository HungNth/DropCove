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
- [ ] A keyboard-opened popup moves focus to the first available item or action; `Tab` traverses all controls; closing restores focus to the originating chevron when it still exists.
- [ ] `Esc` closes the popup before shelf dismissal. A subsequent `Esc` follows the existing Drop Shelf dismissal behavior.
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
