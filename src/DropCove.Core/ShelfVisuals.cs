using System.ComponentModel;
using System.Runtime.InteropServices;

namespace DropCove.Core;

/// <summary>Loads one Shelf Item's native icon and optional image thumbnail.</summary>
/// <typeparam name="TVisual">The visual value produced by the platform adapter.</typeparam>
public interface IShelfVisualProvider<TVisual>
    where TVisual : class
{
    /// <summary>Loads the native icon for a Shelf Item.</summary>
    /// <param name="item">The Shelf Item to represent.</param>
    /// <param name="cancellationToken">A token that cancels the shell request.</param>
    /// <returns>The native icon, or <see langword="null" /> when it is unavailable.</returns>
    Task<TVisual?> LoadNativeIconAsync(ShelfItem item, CancellationToken cancellationToken);

    /// <summary>Loads a Windows Shell thumbnail for an image Shelf Item.</summary>
    /// <param name="item">The image Shelf Item to represent.</param>
    /// <param name="cancellationToken">A token that cancels the shell request.</param>
    /// <returns>The thumbnail, or <see langword="null" /> when it is unavailable.</returns>
    Task<TVisual?> LoadThumbnailAsync(ShelfItem item, CancellationToken cancellationToken);
}

/// <summary>Reports the visual selected for one realized Shelf Item.</summary>
/// <typeparam name="TVisual">The visual value produced by the platform adapter.</typeparam>
/// <param name="Display">The thumbnail or native icon to display.</param>
/// <param name="UsedThumbnail">Whether <paramref name="Display" /> is a thumbnail.</param>
public sealed record ShelfVisualResult<TVisual>(TVisual? Display, bool UsedThumbnail)
    where TVisual : class;

/// <summary>Coordinates visible Shelf Item visual requests without retaining derived content.</summary>
/// <typeparam name="TVisual">The visual value produced by the platform adapter.</typeparam>
public sealed class ShelfVisualCoordinator<TVisual>
    where TVisual : class
{
    private readonly IShelfVisualProvider<TVisual> _provider;
    private readonly Func<ShelfItem, bool> _supportsThumbnail;

    /// <summary>Initializes a new instance of the <see cref="ShelfVisualCoordinator{TVisual}" /> class.</summary>
    /// <exception cref="ArgumentNullException">The provider or thumbnail predicate is <see langword="null" />.</exception>
    public ShelfVisualCoordinator(
        IShelfVisualProvider<TVisual> provider,
        Func<ShelfItem, bool> supportsThumbnail)
    {
        ArgumentNullException.ThrowIfNull(provider);
        ArgumentNullException.ThrowIfNull(supportsThumbnail);
        _provider = provider;
        _supportsThumbnail = supportsThumbnail;
    }

    /// <summary>Loads the native fallback and an optional thumbnail for one visible Shelf Item.</summary>
    /// <param name="item">The Shelf Item being realized.</param>
    /// <param name="cancellationToken">A token that discards work when the item leaves the visible range.</param>
    /// <returns>The visual selected for display.</returns>
    /// <exception cref="ArgumentNullException">The Shelf Item is <see langword="null" />.</exception>
    /// <exception cref="OperationCanceledException">The visible request is canceled.</exception>
    public async Task<ShelfVisualResult<TVisual>> LoadAsync(
        ShelfItem item,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(item);
        cancellationToken.ThrowIfCancellationRequested();

        var nativeTask = LoadNativeIconAsync(item, cancellationToken);
        if (!_supportsThumbnail(item))
        {
            return new ShelfVisualResult<TVisual>(
                await nativeTask.ConfigureAwait(false),
                UsedThumbnail: false);
        }

        var thumbnailTask = LoadThumbnailAsync(item, cancellationToken);
        if (await Task.WhenAny(nativeTask, thumbnailTask).ConfigureAwait(false) == thumbnailTask)
        {
            var thumbnail = await thumbnailTask.ConfigureAwait(false);
            if (thumbnail is not null)
            {
                ObserveOptionalFailure(nativeTask);
                return new ShelfVisualResult<TVisual>(thumbnail, UsedThumbnail: true);
            }
        }

        var nativeIcon = await nativeTask.ConfigureAwait(false);
        var remainingThumbnail = await thumbnailTask.ConfigureAwait(false);
        return remainingThumbnail is null
            ? new ShelfVisualResult<TVisual>(nativeIcon, UsedThumbnail: false)
            : new ShelfVisualResult<TVisual>(remainingThumbnail, UsedThumbnail: true);
    }

    private async Task<TVisual?> LoadNativeIconAsync(ShelfItem item, CancellationToken cancellationToken)
    {
        try
        {
            return await _provider.LoadNativeIconAsync(item, cancellationToken).ConfigureAwait(false);
        }
        catch (Exception exception) when (IsExpectedVisualFailure(exception) && !cancellationToken.IsCancellationRequested)
        {
            return null;
        }
    }

    private async Task<TVisual?> LoadThumbnailAsync(ShelfItem item, CancellationToken cancellationToken)
    {
        try
        {
            return await _provider.LoadThumbnailAsync(item, cancellationToken).ConfigureAwait(false);
        }
        catch (Exception exception) when (IsExpectedVisualFailure(exception) && !cancellationToken.IsCancellationRequested)
        {
            return null;
        }
    }

    private static void ObserveOptionalFailure(Task<TVisual?> task)
    {
        _ = task.ContinueWith(
            static completed => _ = completed.Exception,
            CancellationToken.None,
            TaskContinuationOptions.OnlyOnFaulted | TaskContinuationOptions.ExecuteSynchronously,
            TaskScheduler.Default);
    }

    private static bool IsExpectedVisualFailure(Exception exception) =>
        exception is COMException or IOException or UnauthorizedAccessException or Win32Exception;
}
