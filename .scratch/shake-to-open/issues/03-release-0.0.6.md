# 03: Release DropCove 0.0.6 from the installed artifact

Status: resolved
Type: task
Version: 0.0.6
Parent: [Specification](../spec.md)
Blocked by: 01, 02
Implementation approval: approved by user on 2026-10-14

## What to build

Advance the verified Shake-to-open product to 0.0.6, package the Windows current-user NSIS installer, and verify the installed executable against the complete 0.0.1–0.0.6 behavior surface without disturbing ambient user state.

## Acceptance criteria

- [x] Tickets 01 and 02 are resolved with permanent tests and actual Windows evidence.
- [x] `VERSION`, package/crate/lock/Tauri metadata, PE/runtime/installed version and artifact name consistently report 0.0.6.
- [x] Fresh complete native suite and frontend build pass on the final source state.
- [x] `DropCove-Setup-0.0.6.exe` is built through cargo-xwin/NSIS and its size/SHA-256 are recorded.
- [x] Silent isolated install/uninstall and current-user registration report 0.0.6.
- [x] Installed artifact passes single-instance, tray, hotkey, Shelf lifecycle, Edge Rail, Settings, drag-in/out, fullscreen and Shake-to-open gates.
- [x] Installed Shake verification covers Low/Normal/High thresholds, 600 ms window, 500 ms cooldown, left-button requirement, near-pointer placement and restoration/promotion outcomes.
- [x] Hook enable/disable/sensitivity Save/Cancel and failure warning/retry are exercised.
- [x] Owned processes/profile/WebView/fixtures are removed; ambient executable/profile/registry/shortcuts remain byte-for-byte or value-for-value intact.
- [x] Roadmap and tracker record only exercised evidence; 0.0.7 remains unstarted.
- [x] No Git commit is created without a later explicit user instruction.

## Testing seam

Build once, pin hashes, compile and execute the complete test binary, then install the exact packaged bytes into an owned location/profile. Runtime probes must observe real HWND/OLE/hook/settings behavior and clean shutdown. Restore pre-existing DropCove registration and shortcuts exactly.

## Demo path

Install 0.0.6 into an isolated location, launch an isolated profile, create/dock content, trigger Shake-to-open under all profiles, exercise accepted and restored outcomes, change settings, restart, verify persistence, exit, uninstall, and prove ambient state restoration.

## Non-goals

0.0.7 visuals/accessibility, 0.0.8 recovery/distribution hardening, updater/signing, or creating a Git commit.

## Comments

## Answer

Release verified on native Windows from the final installed artifact; implementation complete for 0.0.6. No Git commit created.

### Final artifacts and build evidence

| Artifact | Size (bytes) | SHA-256 |
|---|---:|---|
| `build/DropCove-Setup-0.0.6.exe` | 3,203,433 | `898acd9a3993f7c618a231abcb38debe95f81bdadba219d4f4c25f2b8c2ecf41` |
| Isolated installed `dropcove.exe` | 13,202,944 | `955ca1f8ecd243b561b9e40a72705024eb054d0f6113f32f59af906a5f2df4e4` |

- Final cargo-xwin NSIS build ran `vue-tsc && vite build` successfully. Fresh native binary `dropcove_lib-7669ec7c64b55353.exe` executed with **82 passed, 0 failed**. CRT PDB linker warnings are vendor debug-symbol warnings, not source errors.
- Isolated install used `/S /D=C:\DropCove_006_TestInstall`. PE FileVersion/ProductVersion and HKCU isolated uninstall metadata reported `0.0.6`. Final artifact was reinstalled after every source change affecting packaging; the final hash above includes the Rail whole-batch busy-guard fix.

### Installed native product evidence

- Hidden Start retained data with both surfaces hidden. Ordinary/secondary launch activated the resident Shelf without a second state owner; native hotkey dismissal returned non-empty content to the Rail.
- Injected Windows left-button/movement events reached the real `WH_MOUSE_LL` observer. Low rejected 15-pixel/two-reversal motion and accepted 28-pixel/four-reversal motion; High accepted 8-pixel reversals; Normal accepted its documented reversal workflow. Pure permanent detector tests exercise exact thresholds, equal-axis ties, axis switches, baseline behavior, cooldown, expiry, and bounded-queue reset.
- Native input with no left button did not trigger. A second gesture inside the cooldown was rejected; a direction sequence exceeding 600 ms expired. Stable owned non-fullscreen foreground probes established no-activate reveal and exact Hidden/EdgeDocked restoration without conflating fullscreen hiding with a missing Rail.
- Actual OLE `DoDragDrop` with filesystem data produced Copy, a new durable batch, and retained Visible placement. Duplicate-only data restored prior presentation without adding a batch. Unsupported text returned None. Escape/DragLeave cancellation restored prior presentation; cancellation probes did not drop copies onto the user's desktop.
- Native pointer drag-out from the actual Rail cards, not an IPC-only simulated drag start, proved Copy destination acceptance, Temporary consumption, Pinned retention, and owned-drag Shake rejection. The final packaged whole-batch drag delivered both `alpha.txt` and `beta.txt` and retained only the separately pinned reference afterward.
- Manage Items rendered real item rows after owner-thread subclass installation. Actual hover/collapse retained 280/16-pixel widths; fullscreen coverage `[0,0,1920,1080]` hid Rail and restored only its collapsed handle. Display transitions reevaluate fullscreen after docking/Shake restoration.
- Actual Settings controls exposed Enable plus Low/Normal/High. Draft/Cancel preserved disk settings; Save persisted seven required fields; disabling prevented gesture activation and enabling reinstalled it. Footer bottom was 519 within a 520-pixel window, with scrollable settings content.
- Clear Temporary retained Pinned data; removing the final item hid both surfaces. Owned resident Exit returned zero.

### Failure evidence — precise injection boundaries

- Deterministic settings transaction tests force registration/application and rollback failures against the real temporary atomic JSON store, preserving prior hotkey, startup, Rail, Shake, and file state or reporting retained state.
- Native `HOOK_FAILED` dispatch in an owned installed process produced a new observed Windows notification containing `Shake-to-open pointer hook stopped; tray, hotkey, Shelf and Edge Rail remain available`. Resident launch/hotkey/Shelf/Rail remained usable, desired enabled/Normal settings remained visible, and unchanged explicit Save reinstalled the hook, verified by a subsequent native gesture.
- For installation-specific coverage, an LLDB breakpoint stopped **one** `SetWindowsHookExW` call in a separate owned installed process while enabling the draft. Changing its x64 hook-type argument from 14 to an invalid value produced actual Windows rejection `Invalid hook procedure type. (0x80070592)` through the production installation/error path. The real Settings alert and a Windows notification contained that error; the original disabled JSON remained unchanged, the resident process survived, and after removing the breakpoint an explicit Save succeeded and a real gesture/hotkey worked. The owned process exited zero. No system DLL/code bytes, application files, or global OS configuration were modified by the fault injection.
- This is deliberate native-call fault injection, **not** evidence that a host security policy spontaneously rejected valid `WH_MOUSE_LL`, and **not** a separate startup-policy-denial run. Startup retains the existing nonfatal catch/warning path in `settings::initialize`.

### Cleanup and limits

- Owned NSIS uninstaller ran successfully. All `DropCove_006_*` test install/profile/WebView/fixture directories, `C:\DropCove_006_QA` (including fault profile), helper windows, and resident processes were removed; `tasklist` reported no `dropcove.exe` processes.
- HKCU Run values matched the captured snapshot. Original 0.0.3 HKCU uninstall values were restored exactly. Desktop and Start Menu shortcut targets were restored to the ambient executable; shortcut-byte identity was not claimed.
- Ambient executable remained SHA-256 `7583ab748d4d84736babe696daafffd276bb95d821f5ea1f748188f5e3d826ab`. Ambient profile was not used by the tests; all app launches supplied isolated `--profile` paths.
- Native pointer smoke used injected Windows input and dual 96-DPI displays. Physical device polling-rate variation, mixed-DPI hardware, and spontaneous host-policy installation denial remain unobserved limits. No 0.0.7 work or commit was created.
