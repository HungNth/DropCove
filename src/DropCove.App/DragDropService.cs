using DropCove.Core;
using Microsoft.UI.Xaml;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage;

namespace DropCove;

internal sealed class DragDropService(
    DropShelfManager manager,
    Func<string, string, Task<bool>> confirm)
{
    public async Task<string?> PrepareItemDragAsync(ShelfItem item, DragStartingEventArgs args)
    {
        var deferral = args.GetDeferral();
        try
        {
            IStorageItem storageItem = item.IsFolder
                ? await StorageFolder.GetFolderFromPathAsync(item.Path)
                : await StorageFile.GetFileFromPathAsync(item.Path);

            args.Data.SetStorageItems([storageItem], readOnly: true);
            args.Data.RequestedOperation = DataPackageOperation.Copy;
            args.AllowedOperations = DataPackageOperation.Copy;
            return null;
        }
        catch (Exception exception)
        {
            args.Cancel = true;
            return exception.Message;
        }
        finally
        {
            deferral.Complete();
        }
    }

    public async Task<(string? Error, int MissingRemoved, int UnavailableRetained)> PrepareBatchDragAsync(
        ShelfBatch batch,
        DragStartingEventArgs args)
    {
        var deferral = args.GetDeferral();
        BatchDragPreparation? prep = null;
        try
        {
            prep = await manager.PrepareBatchForDragAsync(batch.Id, CheckAvailability);
            if (prep.AvailableItems.Count == 0)
            {
                args.Cancel = true;
                return (prep.UnavailableItems.Count > 0
                    ? "All remaining files in this batch are currently unavailable."
                    : "All files in this batch were missing and have been removed.",
                    prep.MissingItemsRemovedCount,
                    prep.UnavailableItems.Count);
            }

            if (prep.UnavailableItems.Count > 0 && !await confirm(
                "Drag available items?",
                $"{prep.UnavailableItems.Count} item(s) are unavailable and will remain in DropCove. Drag only the {prep.AvailableItems.Count} available item(s)?"))
            {
                args.Cancel = true;
                return ("Partial batch drag canceled.", prep.MissingItemsRemovedCount, prep.UnavailableItems.Count);
            }

            var storageItems = new List<IStorageItem>(prep.AvailableItems.Count);
            foreach (var item in prep.AvailableItems)
            {
                IStorageItem storageItem = item.IsFolder
                    ? await StorageFolder.GetFolderFromPathAsync(item.Path)
                    : await StorageFile.GetFileFromPathAsync(item.Path);
                storageItems.Add(storageItem);
            }

            args.Data.SetStorageItems(storageItems, readOnly: true);
            args.Data.RequestedOperation = DataPackageOperation.Copy;
            args.AllowedOperations = DataPackageOperation.Copy;
            return (null, prep.MissingItemsRemovedCount, prep.UnavailableItems.Count);
        }
        catch (Exception exception)
        {
            args.Cancel = true;
            return (exception.Message, prep?.MissingItemsRemovedCount ?? 0, prep?.UnavailableItems.Count ?? 0);
        }
        finally
        {
            deferral.Complete();
        }
    }

    public static ItemAvailability CheckAvailability(string path)
    {
        try
        {
            _ = File.GetAttributes(path);
            return ItemAvailability.Available;
        }
        catch (FileNotFoundException)
        {
            return ItemAvailability.Missing;
        }
        catch (DirectoryNotFoundException)
        {
            var root = Path.GetPathRoot(path);
            return !string.IsNullOrEmpty(root) && Directory.Exists(root)
                ? ItemAvailability.Missing
                : ItemAvailability.Unavailable;
        }
        catch
        {
            return ItemAvailability.Unavailable;
        }
    }

    public Task<bool> CompleteBatchDragAsync(Guid batchId, DataPackageOperation result) =>
        manager.CompleteBatchDragAsync(
            batchId,
            result == DataPackageOperation.Copy
                ? DragOutOutcome.AcceptedCopy
                : DragOutOutcome.Rejected);

    public Task<bool> CompleteItemDragAsync(Guid itemId, DataPackageOperation result) =>
        manager.CompleteItemDragAsync(
            itemId,
            result == DataPackageOperation.Copy
                ? DragOutOutcome.AcceptedCopy
                : DragOutOutcome.Rejected);
}
