using DropCove.Core;
using Microsoft.UI.Xaml;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage;

namespace DropCove;

/// <summary>Adapts Windows storage-item drops to the shared Shelf acceptance seam.</summary>
internal sealed class StorageDropService
{
    private readonly DropShelfManager _manager;
    private readonly Action<IStorageItem> _cacheStorageItem;

    public StorageDropService(DropShelfManager manager, Action<IStorageItem> cacheStorageItem)
    {
        ArgumentNullException.ThrowIfNull(manager);
        ArgumentNullException.ThrowIfNull(cacheStorageItem);
        _manager = manager;
        _cacheStorageItem = cacheStorageItem;
    }

    public async Task<DropAcceptance> AcceptAsync(DataPackageView dataView)
    {
        ArgumentNullException.ThrowIfNull(dataView);

        var storageItems = await dataView.GetStorageItemsAsync();
        var incomingItems = new List<IncomingShelfItem>(storageItems.Count);
        var skipped = 0;

        foreach (var storageItem in storageItems)
        {
            if (string.IsNullOrWhiteSpace(storageItem.Path))
            {
                skipped++;
                continue;
            }

            _cacheStorageItem(storageItem);
            incomingItems.Add(new IncomingShelfItem(
                storageItem.Path,
                storageItem.Name,
                storageItem is StorageFolder));
        }

        return await _manager.AcceptDropAsync(incomingItems, skipped);
    }

    public static bool CanCopyStorageItems(DragEventArgs e) =>
        e.DataView.Contains(StandardDataFormats.StorageItems) &&
        (e.AllowedOperations & DataPackageOperation.Copy) != 0;
}
