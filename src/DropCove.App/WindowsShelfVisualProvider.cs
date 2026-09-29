using System.Runtime.InteropServices.WindowsRuntime;
using DropCove.Core;
using DropCove.Native;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Storage;
using Windows.Storage.FileProperties;

namespace DropCove;

/// <summary>Loads native file icons and image thumbnails from Windows Shell.</summary>
internal sealed class WindowsShelfVisualProvider(Func<ShelfItem, IStorageItem?> getCachedItem)
    : IShelfVisualProvider<ImageSource>
{
    public async Task<ImageSource?> LoadNativeIconAsync(
        ShelfItem item,
        CancellationToken cancellationToken)
    {
        var icon = await ShellIconLoader.LoadAsync(item.Path, item.IsFolder, cancellationToken);
        return icon is null
            ? null
            : await ToImageSourceAsync(icon, cancellationToken);
    }

    public async Task<ImageSource?> LoadThumbnailAsync(
        ShelfItem item,
        CancellationToken cancellationToken)
    {
        var storageItem = await ResolveStorageItemAsync(item, cancellationToken);
        using var thumbnail = await GetThumbnailAsync(
            storageItem,
            ThumbnailMode.PicturesView,
            96,
            cancellationToken);
        return await ToImageSourceAsync(thumbnail, cancellationToken);
    }

    private async Task<IStorageItem> ResolveStorageItemAsync(
        ShelfItem item,
        CancellationToken cancellationToken)
    {
        var cached = getCachedItem(item);
        if (cached is not null)
        {
            return cached;
        }

        return item.IsFolder
            ? await StorageFolder.GetFolderFromPathAsync(item.Path).AsTask(cancellationToken)
            : await StorageFile.GetFileFromPathAsync(item.Path).AsTask(cancellationToken);
    }

    private static Task<StorageItemThumbnail?> GetThumbnailAsync(
        IStorageItem storageItem,
        ThumbnailMode mode,
        uint size,
        CancellationToken cancellationToken) => storageItem switch
        {
            StorageFile file => file.GetThumbnailAsync(
                mode,
                size,
                ThumbnailOptions.UseCurrentScale).AsTask(cancellationToken),
            StorageFolder folder => folder.GetThumbnailAsync(
                mode,
                size,
                ThumbnailOptions.UseCurrentScale).AsTask(cancellationToken),
            _ => Task.FromResult<StorageItemThumbnail?>(null),
        };

    private static async Task<ImageSource?> ToImageSourceAsync(
        ShellIconImage icon,
        CancellationToken cancellationToken)
    {
        var bitmap = new WriteableBitmap(icon.Width, icon.Height);
        await using var stream = bitmap.PixelBuffer.AsStream();
        await stream.WriteAsync(icon.BgraPixels.AsMemory(), cancellationToken);
        bitmap.Invalidate();
        return bitmap;
    }

    private static async Task<ImageSource?> ToImageSourceAsync(
        StorageItemThumbnail? thumbnail,
        CancellationToken cancellationToken)
    {
        if (thumbnail is null)
        {
            return null;
        }

        var bitmap = new BitmapImage();
        await bitmap.SetSourceAsync(thumbnail).AsTask(cancellationToken);
        return bitmap;
    }
}
