# 06: Suppress Default WebView Menu on Manage Items Rows

**Parent specification:** [DropCove 0.0.27 — Type-aware Item Actions](../spec.md), Contract Amendment — 2026-10-27

**What to build:** In Manage Items only, consume right-click on a Shelf Item row so the embedded WebView does not show its default browser menu. Do not open the Item Actions Menu, change Item Selection, or change the verified action matrix. Preserve External Reference Show in File Explorer/Copy Path, Managed Text Copy Text, Managed Image exclusion, keyboard access through More, virtualization, and the Edge Rail no-activate contract.

**Blocked by:** 03 — Manage Items Row Actions and Virtualized Lifecycle

**Status:** ready-for-human

**Implementation authorization:** Granted by the user's `implement` invocation on 2026-10-27. Implementation is complete; final candidate acceptance remains blocked on successful real clipboard verification. No Git commit is authorized.

## Acceptance criteria

- [x] The complete secondary-button sequence on any rendered Shelf Item row in Manage Items, including `pointerdown`, `contextmenu`, and release, prevents the embedded WebView's default browser menu.
- [x] Secondary-button `pointerdown` and the later `contextmenu` do not open the Item Actions Menu, dispatch an Item Action, start drag, change Item Selection, move its anchor, or alter selected/unselected row targeting.
- [x] Existing primary-button behavior remains unchanged for row selection/drag and the Pin/Unpin, More, and Remove controls.
- [x] More remains the sole Item Actions Menu entry point and retains existing pointer, keyboard, focus, ARIA, local-feedback, dismissal, and virtualization behavior.
- [x] External Reference rows still expose Show in File Explorer then Copy Path; Managed Text rows still expose Copy Text; Managed Image rows still expose no More button.
- [x] Right-click behavior remains unchanged outside Manage Items Shelf Item rows, including input/textarea editing menus, the Manage Items background, the Drop Shelf, and the Edge Rail.
- [x] The Edge Rail receives no new right-click handling and retains its no-activate, non-focus-stealing contract.
- [x] Frontend behavior tests dispatch a complete secondary-button sequence with a real cancelable `contextmenu` event. They prove prevention on ordinary and virtualized rows, unchanged selection/anchor state, no menu/action/drag dispatch, preserved primary-button and More keyboard behavior, and unaffected non-row boundaries.
- [x] The complete frontend test suite and Vue production build pass.
- [x] A new cargo-xwin Windows release build and NSIS package produce an exact 0.0.27 candidate with new recorded sizes and SHA-256 hashes.
- [ ] Installed Windows smoke covers ordinary and more-than-50-item Manage Items collections, right-click on selected and unselected External/Managed Text/Managed Image rows, unchanged selection, existing primary/keyboard interactions, existing More actions including Managed Text Copy Text, and unchanged Edge Rail behavior.
- [x] Release documentation distinguishes the 2026-10-26 pre-amendment candidate evidence from the new candidate evidence and never claims the old hashes prove this ticket.

## Testing seam

Extend the existing Manage Items component renderer and Item Selection/virtualization seams. Dispatch the complete secondary-button sequence against mounted rows: secondary-button `pointerdown`, a cancelable `contextmenu`, and release. Assert that only the WebView default menu is prevented and that selection, anchor, drag state, open-menu identity, and emitted row/item actions remain unchanged. Exercise primary-button row/control behavior and More keyboard opening/dismissal separately so scoped suppression cannot mask a pointer or accessibility regression. Repeat observable boundary checks for input/textarea controls, the Manage Items background, and unchanged Drop Shelf/Edge Rail surfaces. Use the newly packaged Windows candidate to observe that the WebView menu does not appear and that the existing More actions still produce their real clipboard/File Explorer effects.

## Demo path

1. Install the new exact 0.0.27 candidate and open Manage Items for a mixed batch containing External References, Managed Text, and a Managed Image.
2. Create a multi-item selection, then complete a physical right-click sequence on selected and unselected rows; verify no WebView menu appears, no drag/action starts, and selection plus anchor remain unchanged.
3. Repeat in a collection above 50 items before and after virtualization scroll/reuse.
4. Exercise left-click row selection/drag plus pointer and keyboard More activation; run Show in File Explorer and Copy Path on an External Reference and Copy Text on Managed Text; verify Managed Image still has no More action.
5. Verify right-click behavior outside item rows remains unchanged and the Edge Rail still does not activate or gain a right-click action path.
6. Package the candidate, record new artifact hashes, and append amendment evidence without modifying the historical Ticket 05 evidence.

## Dependencies and limits

- Reuses the existing Manage Items row event boundary, shared Item Actions Menu, Item Selection model, and virtualized row lifecycle.
- Adds no native Windows context menu, keyboard context-menu-key handling, batch action, selection-wide action, new Item Action, dependency, setting, geometry, or persistence behavior.
- Does not remove or relocate Managed Text Copy Text.
- Does not change Drop Shelf or Edge Rail pointer handling.
- Requires a new exact candidate because the verified 2026-10-26 binaries do not contain this amendment.
- No glossary change or ADR is required.

## Implementation and installed evidence — 2026-10-27

- Implementation: `ShelfItemRow.vue` prevents only secondary-button (`button === 2`) `contextmenu` events. Keyboard-origin events remain untouched. Existing primary-button guards already keep right-click out of selection and drag; no new state, helper, dependency, native command, or Rail-origin branch is needed. `ShelfItemRow` is rendered only by Manage Items; its Rail-origin popup remains Manage Items, not an Edge Rail card.
- TDD: the new rendered Manage Items regression first failed because right-click was not prevented. A keyboard-origin regression then failed against unconditional suppression; conditional suppression passed both cases. Ordinary 3-item and virtualized 65-item collections preserve selected IDs and the Item Selection range anchor, proven by subsequent Shift+Space selection. Existing primary drag, Pin/Remove, menu order, keyboard, focus, and privacy regressions remain green.
- Verification: focused interaction/menu/row checks passed; `pnpm exec vue-tsc --noEmit` passed; final `pnpm test` passed **54/54**, zero failures. `pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc` passed frontend production build, Windows release compilation, and NSIS packaging. Vendor CRT missing-PDB LNK4099 warnings were non-fatal; no Rust source was changed and the native unit suite was not rerun.
- Standards and Spec reviews ran independently against the uncommitted changes: both reported correct, with no actionable findings. Their build/runtime exclusions were covered separately below.
- Installed `/S /NS /D=<owned temporary directory>` returned **0**. Embedded `http://tauri.localhost/` production assets ran from the installed executable using isolated Profile Data/WebView data. External fixtures entered through `accept_drop`; Managed Text (1,050 characters, Unicode and mixed line endings) and Managed Image fixtures were seeded into the stopped owned profile with committed Managed File records, then loaded on restart.
- Real WebView2 CDP-injected secondary-button sequences on selected and unselected ordinary rows produced `button: 2, defaultPrevented: true`, no Item Actions Menu, and identical selected IDs. Mixed External/Managed Text/Managed Image rows all suppressed the browser menu; Managed Image had no More action. Pointer and keyboard More activation exposed the existing action matrix. Show in File Explorer opened the owned fixture directory and selected the exact `item-0.txt`, observed through Windows Shell automation.
- The installed 65-item popup exercised right-click before and after virtualization scroll/remount (visible indices 35–46). Returning to the first row recovered the retained selected ID; the header still reported one selected item while it was unmounted. No item menu opened. The real Shift+F10 event had `button: -1, defaultPrevented: false`; a DOM keyboard-origin `button: 0` event was also unprevented. Background and temporary textarea boundary probes remained unprevented.
- Expanded Edge Rail card context events remained unprevented. Opening Manage Items from the Rail and right-clicking its row preserved foreground HWND **132426** before and after both interactions. That popup correctly suppresses its own row menu without changing the Edge Rail surface or activating DropCove.
- Normal uninstall returned **0** and removed the installed executable. Owned Explorer window, app processes, browser handles, database handles, fixtures, profile, WebView data, and temporary installation directory were cleaned. Captured HKCU product/uninstall registry trees were restored and compared equal; the captured DropCove Run value was restored. External fixture bytes remained unchanged before cleanup.

### Exact amended candidate

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `build/DropCove-Setup-0.0.27.exe` | 3,703,520 | `44844f2796553c39b265d7ea9c418ab92eba97d42968757553d87f1d540932dd` |
| Built Windows x64 executable | 15,287,296 | `49de31f0c2517cbe858ed78e5c6a252376f9cd1d003aeebfef6f464e4fc390cf` |
| Exact installed executable | 15,287,296 | `9e6cab2508abc71a8bcd2f9b175c3d303d11abc92326fd0bc143a208de86068a` |

### Remaining acceptance limit

Copy Path and Managed Text Copy Text were activated on the installed candidate, but Windows returned `could not open Windows clipboard: Access is denied. (0x80070005)`. Independent OLE and Windows Forms clipboard probes also could not open the clipboard; no OpenClipboard owner HWND was reported. The application displayed the local error, closed the menu, and retained items/selection. Successful clipboard writes and exact clipboard readback for this candidate are **not verified**. Historical Ticket 05 clipboard success must not be used to fill that gap. Final candidate acceptance remains blocked until Copy Path and Copy Text succeed and are read back in an interactive Windows session with clipboard access.

Input was CDP-injected at scale 1, not physical mouse hardware. Spoken screen-reader output, real Windows Contrast switching, and multi-monitor/DPI hardware were not re-exercised. No Git commit, publishing, deployment, or human candidate acceptance was performed.
