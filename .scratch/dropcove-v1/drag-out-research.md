# DropCove native drag-out research

## Recommendation

Use the WinUI XAML drag source API on the **individual Shelf Item element**:

1. Set `CanDrag="True"` on the item root (not the whole batch card).
2. Handle `DragStarting` in the `DragDropService` seam. Put the existing filesystem object into `args.Data` with `SetStorageItems`, set both `args.AllowedOperations` and `args.Data.RequestedOperation` to `DataPackageOperation.Copy`, and use a deferral while resolving the path asynchronously.
3. Handle `DropCompleted` on the same source element. Treat `DropResult == DataPackageOperation.Copy` as an accepted copy and all other results as non-successful for shelf-state purposes.
4. Call `DropShelfManager.CompleteItemDrag(item.Id, DragOutOutcome.AcceptedCopy)` only for `Copy`; map every non-copy completion to a retaining outcome. Do not delete, move, rename, or overwrite the source filesystem object.

`StartDragAsync` is available but is not required for the normal `CanDrag` gesture. Use it only if the UI later implements custom pointer/gesture detection; its returned `DataPackageOperation` is documented as the same result supplied by `DropCompleted`, so the adapter should choose one completion path rather than committing twice.

## Evidence labels

- **Documented fact** means the linked Microsoft API/reference or overview explicitly states the behavior.
- **[INFERENCE]** means the recommendation is derived by applying those facts to DropCove’s existing item identity, Copy-only contract, and manager seam. The note calls out the important inferences explicitly rather than presenting them as platform guarantees.

The API facts below are documented. The choices to attach drag behavior to the item root, resolve one path immediately before drag, capture the item ID, normalize an unqualified non-Copy result to safe retention, and avoid duplicate completion commits are **[INFERENCE]** from those facts plus the repository contract.

- [Microsoft: Windows App SDK 2.0 stable release notes](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/release-notes/windows-app-sdk-2-0?pivots=stable#version-251) — identifies 2.5.1 as the current stable release in the 2.0 family; the API references above cover the drag/drop surface used here.

## Scope and current API availability

The app targets `net10.0-windows10.0.22000.0` and references `Microsoft.WindowsAppSDK` **2.5.1** in [`DropCove.csproj`](../../src/DropCove.App/DropCove.csproj). Microsoft lists Windows App SDK 2.5.1 as the current stable 2.0 release, and the WinUI API reference lists `CanDrag`, `DragStarting`, `DropCompleted`, and `StartDragAsync` across the documented Windows App SDK 0.8–2.0 monikers. These are therefore existing WinUI 3 APIs for this project, not a preview-only design.

Sources:

- [Microsoft: latest Windows App SDK downloads](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/downloads) — lists stable 2.5.1.
- [Microsoft NuGet: Microsoft.WindowsAppSDK 2.5.1](https://www.nuget.org/packages/Microsoft.WindowsAppSDK/2.5.1).
- [UIElement.CanDrag](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.uielement.candrag?view=windows-app-sdk-2.0).
- [UIElement.DragStarting](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.uielement.dragstarting?view=windows-app-sdk-2.0).
- [UIElement.DropCompleted](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.uielement.dropcompleted?view=windows-app-sdk-2.0).
- [UIElement.StartDragAsync](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.uielement.startdragasync?view=windows-app-sdk-2.0).

## API facts

### `CanDrag`

`UIElement.CanDrag` is a Boolean, defaulting to `false`; setting it to `true` enables the element as drag data. Microsoft’s overview says the element and, for collection controls, contained elements can become draggable. The `DragStarting` and `DropCompleted` events are raised only when `CanDrag` is `true`.

For DropCove, the item template’s root should own `CanDrag`; putting it on a batch container would make the batch—not the requested one item—the drag source. The current shelf rendering is in [`MainPage.xaml`](../../src/DropCove.App/MainPage.xaml), and the native adapter boundary is the documented `DragDropService` seam in [`spec.md`](spec.md#implementation-decisions).

### `DragStarting`

`DragStarting` is a routed `UIElement` event raised when the drag begins. `DragStartingEventArgs` supplies:

- `Data`: the `Windows.ApplicationModel.DataTransfer.DataPackage` to populate.
- `AllowedOperations`: the source’s permitted operations.
- `Cancel`: set to `true` to cancel the drag before it proceeds.
- `GetDeferral()`: keeps the event active while asynchronous payload preparation completes.

Minimal handler shape (illustrative; product code is intentionally unchanged):

```csharp
private async void OnItemDragStarting(UIElement sender, DragStartingEventArgs args)
{
    var item = GetShelfItem(sender);
    var deferral = args.GetDeferral();
    try
    {
        IStorageItem storageItem = item.IsFolder
            ? await StorageFolder.GetFolderFromPathAsync(item.Path)
            : await StorageFile.GetFileFromPathAsync(item.Path);

        args.Data.SetStorageItems([storageItem]);
        args.AllowedOperations = DataPackageOperation.Copy;
        args.Data.RequestedOperation = DataPackageOperation.Copy;
    }
    catch (Exception)
    {
        args.Cancel = true;
        // Keep the Shelf Item reference; payload construction did not succeed.
    }
    finally
    {
        deferral.Complete();
    }
}
```

The `try/finally` is required: `Complete()` tells the platform that the asynchronous data package is ready. `StorageFile.GetFileFromPathAsync` and `StorageFolder.GetFolderFromPathAsync` accept absolute filesystem paths and can fail with missing-file, unauthorized-access, or invalid-path exceptions. Resolving the item immediately before the drag avoids relying only on stale cached `IStorageItem` instances.

Sources:

- [DragStartingEventArgs](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.dragstartingeventargs?view=windows-app-sdk-2.0).
- [DragStartingEventArgs.Data](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.dragstartingeventargs.data?view=windows-app-sdk-2.0).
- [DragStartingEventArgs.AllowedOperations](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.dragstartingeventargs.allowedoperations?view=windows-app-sdk-2.0).
- [DragStartingEventArgs.Cancel](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.dragstartingeventargs.cancel?view=windows-app-sdk-2.0).
- [DragStartingEventArgs.GetDeferral](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.dragstartingeventargs.getdeferral?view=windows-app-sdk-2.0).
- [DragOperationDeferral.Complete](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.dragoperationdeferral.complete?view=windows-app-sdk-2.0).
- [StorageFile.GetFileFromPathAsync](https://learn.microsoft.com/en-us/uwp/api/windows.storage.storagefile.getfilefrompathasync?view=winrt-26100).
- [StorageFolder.GetFolderFromPathAsync](https://learn.microsoft.com/en-us/uwp/api/windows.storage.storagefolder.getfolderfrompathasync?view=winrt-26100).

### `DataPackage.SetStorageItems`

`DataPackage.SetStorageItems(IEnumerable<IStorageItem>)` sets files and folders in the package. The overload with `readOnly` controls whether the files/folders are read-only; it does **not** select Copy versus Move. The standard format is `StandardDataFormats.StorageItems`, defined as the file/folder storage-item format.

Use the existing `ShelfItem.Path` and `ShelfItem.IsFolder` fields to resolve exactly one `StorageFile` or `StorageFolder`, then pass a one-element collection to `SetStorageItems`. This preserves path-reference semantics: DropCove offers the source object to the destination and does not copy source bytes into DropCove.

Sources:

- [DataPackage.SetStorageItems](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.datatransfer.datapackage.setstorageitems?view=winrt-26100).
- [StandardDataFormats.StorageItems](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.datatransfer.standarddataformats.storageitems?view=winrt-26100).
- [DataPackage](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.datatransfer.datapackage?view=winrt-26100).

### Copy-only operation semantics

`DataPackageOperation.Copy` means the target copies content; `Move` means the target copies content and deletes the original. Therefore Copy-only must prevent Move from being offered by the source:

```csharp
args.AllowedOperations = DataPackageOperation.Copy;
args.Data.RequestedOperation = DataPackageOperation.Copy;
```

`AllowedOperations` declares what the source allows; `RequestedOperation` declares the source’s desired operation. Microsoft notes that XAML drag/drop uses the latter for multiple flags, while the source’s `AllowedOperations` is exposed to potential targets. For this ticket there is no reason to offer multiple flags: only Copy should be present. This also prevents Explorer or a modifier-key choice from turning the operation into Move/Link through the source contract.

Sources:

- [DataPackageOperation](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.datatransfer.datapackageoperation?view=winrt-26100).
- [DataPackage.RequestedOperation](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.datatransfer.datapackage.requestedoperation?view=winrt-26100).
- [DragStartingEventArgs.AllowedOperations](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.dragstartingeventargs.allowedoperations?view=windows-app-sdk-2.0).

### `DropCompleted` and `DropResult`

`DropCompleted` is raised on the source after the drag operation ends. Microsoft explicitly says it fires after `DragStarting` whether the drop succeeded or the drag was canceled. `DropCompletedEventArgs.DropResult` is a `DataPackageOperation` value indicating the operation type and whether it succeeded. `StartDragAsync` returns the same value.

Use the following application mapping:

| Native observation | Shelf outcome | Manager action |
|---|---|---|
| `DropResult == DataPackageOperation.Copy` | AcceptedCopy | `CompleteItemDrag(item.Id, DragOutOutcome.AcceptedCopy)`; remove a Temporary Item, retain a Pinned Item, and remove an empty batch. |
| `DropResult == DataPackageOperation.None` | Canceled/rejected/failed for state purposes | Retain the participating reference; map to `Canceled`, `Rejected`, or `Failed` only when an independent adapter observation justifies that distinction. |
| Drag payload resolution fails before the operation starts | Failed | Set `args.Cancel = true`; do not mutate the manager. |
| `StartDragAsync` throws or cannot start | Failed | Catch at the native adapter boundary; do not mutate the manager. |

**Important limitation:** the XAML completion API does not expose a separate reason field for user cancellation, target rejection, and native failure. `DropCompleted` exposes an operation result, not a cancellation/rejection diagnostic. Therefore the API can reliably distinguish accepted Copy from non-Copy, but cannot by itself prove which of canceled, rejected, or failed produced `None`. The existing `DropOutOutcome` enum is still useful at the application seam for explicit tests and future adapter diagnostics, but a normal event-only implementation should map an unqualified non-Copy result to “unsuccessful; retain reference.” This is an API limitation, not a reason to remove the safe retention behavior.

Also, `Copy` means the destination accepted/performed the Copy operation reported by the drag system. It does not prove that Explorer has completed all later internal processing. The DropCove contract should commit its own reference removal after the native Copy result, while preserving the documented at-least-once crash window if the process terminates before that local commit.

Sources:

- [UIElement.DropCompleted](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.uielement.dropcompleted?view=windows-app-sdk-2.0).
- [DropCompletedEventArgs](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.dropcompletedeventargs?view=windows-app-sdk-2.0).
- [DropCompletedEventArgs.DropResult](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.dropcompletedeventargs.dropresult?view=windows-app-sdk-2.0).
- [Microsoft drag-and-drop overview](https://learn.microsoft.com/en-us/windows/apps/develop/data/drag-and-drop) — states that the target processes the package and returns the operation it performed, and that XAML drag/drop works app-to-app and app-to-desktop.

## `StartDragAsync`

`UIElement.StartDragAsync(PointerPoint)` programmatically initiates a drag and returns `IAsyncOperation<DataPackageOperation>`. Microsoft documents it for custom gesture detection and says it raises `DragStarting`; the returned value indicates Copy/Move/Link and success and is the same value provided by `DropCompletedEventArgs.DropResult`.

It is not the minimal path for a normal Shelf Item. `CanDrag="True"` lets the XAML input system initiate the standard gesture and then raises `DragStarting`. Introducing custom pointer tracking solely to call `StartDragAsync` would duplicate native gesture logic. If a later custom shelf interaction requires programmatic initiation, pass the current `PointerPoint`, wire `DragStarting` first, and use either the awaited return or `DropCompleted` as the single completion signal.

The API reference has one material caveat: `StartDragAsync` is not supported when the user runs the app elevated as administrator. DropCove is designed to run unelevated, so this is compatible with the product boundary; installed smoke tests should still include the documented same-integrity limitation rather than testing an elevated helper.

Source: [UIElement.StartDragAsync](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.uielement.startdragasync?view=windows-app-sdk-2.0).

## Explorer interoperability

Microsoft’s drag-and-drop overview states that the XAML API works between apps, with Win32 desktop apps, and in the app-to-desktop direction. `StorageItems` is the standard files/folders format and is the same format already used by DropCove’s incoming Explorer drop path in [`MainPage.xaml.cs`](../../src/DropCove.App/MainPage.xaml.cs). Consequently, Explorer interoperability should use the same native package format: one `StorageFile` or `StorageFolder`, not a custom path string or application-private format.

The source only offers Copy. Explorer may accept the item and perform the file-system copy; DropCove must never implement a second copy or delete/move the source. The actual installed-EXE Explorer scenario remains required validation because Microsoft’s conceptual interoperability statement does not replace an end-to-end test of this app’s unpackaged runtime, item template, and window/input behavior.

## Mapping to `DropShelfManager`

The existing manager is already the correct application seam:

- [`ShelfItem`](../../src/DropCove.Core/ShelfModels.cs) provides the path, folder flag, per-occurrence ID, and Temporary/Pinned state.
- [`CompleteItemDrag`](../../src/DropCove.Core/DropShelfManager.cs) already consumes a Temporary Item only for `DragOutOutcome.AcceptedCopy`; a Pinned Item remains.
- `RemoveItem` removes the empty batch after the last item is consumed.

The native `DragDropService` should own UI event registration, `StorageFile`/`StorageFolder` resolution, `DataPackage` construction, and translation from `DropResult` to the application outcome. The item view/view-model should only expose the item identity and bind `CanDrag`; it should not own OLE/data-transfer logic. On `DropCompleted`, pass the captured item ID—not a path lookup—because identity is per Shelf Item occurrence and the same path can occur in multiple batches.

No persistence design is needed for this ticket. The service should call the manager after the native completion observation; persistence remains the manager/application state layer’s responsibility.

## Lifecycle caveats and risks

1. **Async payload preparation:** path-to-storage-item resolution is asynchronous. Always call `GetDeferral()` and `Complete()` in `finally`; otherwise the platform can start with an incomplete package or remain in an invalid lifecycle state.
2. **Source disappearance or access loss:** `GetFileFromPathAsync`/`GetFolderFromPathAsync` can fail immediately before drag. Cancel and retain the reference. This is distinct from a successful native Copy result.
3. **Result granularity:** `DropCompleted` does not separately identify cancellation, rejection, and failure. Safe state behavior is to retain all non-Copy results; only classify the three separately when the adapter has an independent, evidence-backed signal.
4. **Copy is not a later-processing receipt:** a Copy result is the shell drag operation’s accepted result, not a guarantee that every destination-side asynchronous action has finished.
5. **No duplicate commits:** if `StartDragAsync` is later used, its return and `DropCompleted.DropResult` represent the same result. Do not apply `CompleteItemDrag` twice.
6. **Crash window:** if the process ends after Explorer accepts Copy but before DropCove commits local removal, restart may show the Temporary reference again. This is the specified at-least-once behavior and is safer than preemptively removing it.
7. **Elevated process:** `StartDragAsync` is documented as unsupported when running as administrator. Keep the app unelevated; do not add an elevated drag helper.
8. **UI ownership:** `DropCompleted` is a routed source event. Attach it to the actual item drag root and capture the item ID at drag start; do not assume the routed `OriginalSource` is the declared item element because templates can make it a template part.
9. **Native validation:** test the packaged/unpackaged installed EXE against Explorer for accepted Copy and cancellation. API documentation establishes the data contract, not this app’s runtime packaging, focus, DPI, or visual behavior.

## Sources

Primary Microsoft sources used above:

- [Drag and drop overview](https://learn.microsoft.com/en-us/windows/apps/develop/data/drag-and-drop)
- [UIElement.CanDrag](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.uielement.candrag?view=windows-app-sdk-2.0)
- [UIElement.DragStarting](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.uielement.dragstarting?view=windows-app-sdk-2.0)
- [DragStartingEventArgs](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.dragstartingeventargs?view=windows-app-sdk-2.0)
- [UIElement.DropCompleted](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.uielement.dropcompleted?view=windows-app-sdk-2.0)
- [DropCompletedEventArgs.DropResult](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.dropcompletedeventargs.dropresult?view=windows-app-sdk-2.0)
- [UIElement.StartDragAsync](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.uielement.startdragasync?view=windows-app-sdk-2.0)
- [DataPackage.SetStorageItems](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.datatransfer.datapackage.setstorageitems?view=winrt-26100)
- [DataPackage.RequestedOperation](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.datatransfer.datapackage.requestedoperation?view=winrt-26100)
- [DataPackageOperation](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.datatransfer.datapackageoperation?view=winrt-26100)
- [StorageFile.GetFileFromPathAsync](https://learn.microsoft.com/en-us/uwp/api/windows.storage.storagefile.getfilefrompathasync?view=winrt-26100)
- [StorageFolder.GetFolderFromPathAsync](https://learn.microsoft.com/en-us/uwp/api/windows.storage.storagefolder.getfolderfrompathasync?view=winrt-26100)
- [Latest Windows App SDK downloads](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/downloads)
