# Real DropCove test-profile verification and destination correlation

Date: 2026-10-05. User approved the narrow test-profile seam after the controlled probe comparison. No native drag cutover.

## Decision

The actual DropCove WinUI drag path passes the complete five-attempt/four-outcome matrix with a rejecting backing destination. When a backing destination genuinely accepts Drop after the primary exits, DropCove consumes only the participating Temporary Item. These measurements do not establish an advantage for a native replacement.

Keep native production Tickets 02–04 blocked and ADR 0003 proposed pending a planning decision. The precise destination that handled the historical uninstrumented attempt remains unknown; the new controlled tests must not be described as reconstructing it. Responsive-shelf keyboard, geometry, performance, destination compatibility, and final NSIS gates remain separate and unqualified.

## Approved seam

- Explicit test-only `--test-profile <absolute-directory>`; database and settings roots resolve to that directory.
- Named resident mutex and activation event derive from the normalized profile directory. Default launch retains the original folder and exact existing instance/event names.
- Startup registration never writes in test mode, including initialization, Settings save, and rollback.
- No live-profile swap, migration, fallback, production drag instrumentation, or native source-engine change.
- Test window title distinguishes test mode. Global hotkey conflict handling stays unchanged; fixtures use separate Ctrl+Alt+Shift+F11/F12 keys and disable shake in test JSON.

Implementation: `src/DropCove.Services/LaunchProfile.cs`, `src/DropCove.App/App.xaml.cs`, `src/DropCove.App/MainWindow.xaml.cs`. Invalid explicit test options fail rather than selecting the normal profile. This is trusted test-directory isolation, not an OS sandbox; filesystem aliases to the normal profile must not be used.

## Artifact and runtime surface

A clean self-contained Release publish was placed side-by-side at:

```text
C:\Users\Hung\AppData\Local\Programs\DropCove\TestProfileQualification\DropCove.exe
```

This is the actual DropCove application, not the minimal WinUI comparison source. It resides under the installed application directory but is a side-by-side publish, **not an NSIS installer run or replacement of the original running executable**. The matrix payload fingerprint is in [installed-profile-evidence.json](installed-profile-evidence.json). A final startup-only parsing refinement avoids copying the process argument array twice and rejects inline test-option syntax; final publish/smoke evidence is recorded separately.

The user's existing resident process, PID 408, remained running. An initial graceful-exit message was mistakenly directed at its Edge Rail handle; no process exit occurred. No further exit message was sent to it. All later test instances used different profile mutex/event names and were closed independently.

## Profile and OS state verification

A throwaway CLI consumer exercised the public profile/storage APIs, then seeded 27 independent fixture databases through `DropShelfManager.AcceptDropAsync` and settings through `SettingsStore.SaveAsync`:

- Missing, relative, repeated, and normal-directory test options reject before storage/instance startup.
- A normalized path containing dot/trailing-separator components shares the same identity; distinct directories have distinct mutex/event names.
- Default launch of the new executable activated the existing normal instance and exited successfully without becoming a second resident.
- Two test profiles ran concurrently with the normal process; launching the same profile with uppercase/dot path spelling exited and activated its existing instance.
- Real Settings UI toggled Start with Windows from false to true and saved only test JSON. Restart restored two persisted fixture items and the saved setting without changing Windows startup registration.

Live-profile state verification:

`settings.json` remained byte-for-byte identical (`b028dc8f448fa5fc5f0dad273f1003a4eed75627162510e76b0a4f5aafb4923e`). The HKCU Run `DropCove` value remained unchanged (`"C:\Users\Hung\AppData\Local\Programs\DropCove\DropCove.exe" --autostart`).

During test-profile verification, the live resident instance (PID 408) had checkpointed its SQLite WAL to 0 items. With explicit user approval, PID 408 was cleanly closed via WM_CLOSE/installer messages, the pre-test delta was preserved at `C:\Users\Hung\AppData\Local\Temp\DropCove-DB-Delta-Investigation-rzx36ozz`, and the 1-item snapshot `before-popup-actions.db` (containing `Ticket01.txt`) was restored to `shelf.db` (SHA-256 `5da49f96e4d8411489f581f026ea7604075ee4f757e49e39672f11e075b21d54`). This restored logical data state (1 batch, 1 item) differs in exact file bytes from the historical `ad8a57...` hash. DropCove was relaunched normally and is running as PID 26020.

## Actual popup drag matrix

Every attempt starts with a new isolated database containing a two-item batch: one Temporary Item offered by the drag and one pinned sentinel that is not offered. The real batch Manage button opens the real popup. Real SendInput left-button movement crosses the Windows threshold and continues to the controlled destination. Primary and backing are directly registered IDropTarget implementations with separate attempt-correlated event streams.

| Case | Attempts | Primary Drop | Backing Drop | Offered Temporary reference afterward |
|---|---:|---|---|---:|
| Accepted Copy; backing Reject | 5/5 | Yes: exact path, S_OK/COPY | No | 0 |
| Explicit Reject; backing Reject | 5/5 | No | No | 1 |
| Esc cancel over Copy-capable primary; backing Reject | 5/5 | No | No | 1 |
| Primary exits inside DragEnter before effect assignment; backing Reject | 5/5 | No | No | 1 |
| Primary exits inside DragEnter; backing Copy | 5/5 | No | Yes: exact path, S_OK/COPY | 0 |

The sentinel remained present in all 25 attempts. Cancel/Reject/primary-exit-with-Reject-backing retained the open popup in all 15 attempts. Accepted Copy leaves a one-item batch; that batch has no Manage chevron, and the popup was observed closed in the corrected contiguous driver runs. These results do not qualify every popup lifetime/accessibility/keyboard criterion.

No raw production DropCompleted/DoDragDrop result was logged or injected. The real-app outcome is the persisted exact Shelf Item identity before/after, correlated against the target events and returned acceptance. Accepted backing Copy is a genuine successful destination, **not acceptance by the failed primary**. Copied-file existence and later processing are not oracles.

Full event streams, fixture identities, fingerprints, before/after counts, popup observations, and setup notes: [installed-profile-evidence.json](installed-profile-evidence.json).

## Driver corrections retained

The first Copy attempt had setup retries, all retained in its log/note: a popup closed between tool calls, untyped ctypes SetWindowPos did not place windows, and the initial input drive failed to reach DragEnter. The corrected driver declares pointer-sized SetWindowPos parameters, asserts placement, opens/drags in one contiguous operation, checks the HWND's PID under the pointer, and uses relative SendInput movement for the initial threshold before absolute destination movement. No controlled acceptance failure was discarded or averaged away.

Helpers and PowerShell inspection run with CREATE_NO_WINDOW. Both destination windows are positioned at the same rectangle; backing is created/placed first, primary second. For the failure case, primary exit does not assign an effect; backing events establish whether any other destination accepts Drop. For the accepting cases, the exact fixture path and S_OK/COPY callback return are required in the accepting destination's log.

## Remaining scope

No Explorer/browser/editor/chat compatibility matrix, Edge Rail drag-out matrix, folder/multi-path matrix, 100-batch/1,000-item resource window, keyboard qualification, or final installer qualification is claimed. No native cutover is warranted solely by these measurements. A planning decision must determine whether to retain WinUI and retire the conditional native effort or investigate an additional precisely identified failure.

## Final verification and cleanup

- Final side-by-side Release publish succeeded without new warnings/errors. [Final executable smoke](final-profile-smoke.json) exercised inline-option rejection through the real native error dialog (exit 1), test `--autostart` hidden launch, same-profile activation, restored two-item popup, and unchanged live files/Run value. The initial UIA Invoke did not dismiss the native OK pane; a real click did, with no replacement invalid launch. The final startup-only refinement's binary hash is recorded separately from the matrix payload.
- `dotnet test tests/DropCove.Tests/DropCove.Tests.csproj -c Release --no-restore --filter FullyQualifiedName~SettingsStoreTests`: 4 passed, 0 failed/skipped, 125 ms.
- Complete suite, run once after source stabilization: `dotnet test tests/DropCove.Tests/DropCove.Tests.csproj -c Release --no-restore`: 155 passed, 0 failed/skipped, 411 ms.
- Independent Standards and Spec reviews each reported zero findings. No new permanent wiring/implementation tests were added; actual launch/Settings/drag surfaces and the throwaway boundary/storage consumer supplied the new seam proof.
- All test-profile instances and native helpers exited. Disposable fixtures, backup copies, source/build harnesses, and their temporary workspace were removed. Initial removal hit retained probe SQLite reader locks; those readers were explicitly closed before completing deletion, without forced GC or working-set trimming. The side-by-side real application binary remains available for the approved test-only option; PID 408 and its original profile remain preserved. [Verification summary](profile-verification-summary.json). These follow-up/profile changes remain uncommitted pending explicit approval.
