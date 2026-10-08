using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using DropCove.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Media;

namespace DropCove;

internal sealed class BatchItemPreview : INotifyPropertyChanged
{
    public event PropertyChangedEventHandler? PropertyChanged;
    public ImageSource? Icon { get; private set; }

    public void SetIcon(ImageSource? icon)
    {
        if (ReferenceEquals(Icon, icon)) return;
        Icon = icon;
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(Icon)));
    }
}

internal sealed class BatchCardViewModel : INotifyPropertyChanged
{
    private IReadOnlyList<BatchItemPreview> _previewItems;

    public event PropertyChangedEventHandler? PropertyChanged;

    public ShelfBatch Batch { get; private set; }
    public string Title { get; private set; }
    public string Subtitle { get; private set; }
    public string FallbackGlyph { get; private set; }
    public IReadOnlyList<BatchItemPreview> PreviewItems => _previewItems;
    public ImageSource? PrimaryIcon => _previewItems.ElementAtOrDefault(0)?.Icon;
    public ImageSource? SecondaryIcon => _previewItems.ElementAtOrDefault(1)?.Icon;
    public ImageSource? TertiaryIcon => _previewItems.ElementAtOrDefault(2)?.Icon;
    public bool IsSingleItem => Batch.Items.Count == 1;
    public bool IsMultiItem => Batch.Items.Count > 1;
    public Visibility SingleItemVisibility => IsSingleItem ? Visibility.Visible : Visibility.Collapsed;
    public Visibility MultiItemVisibility => IsSingleItem ? Visibility.Collapsed : Visibility.Visible;
    public ShelfItem? SingleItem => IsSingleItem ? Batch.Items[0] : null;
    public bool IsPinned => SingleItem?.IsPinned ?? false;
    public string SingleItemPinAutomationName => IsPinned ? $"Unpin {Title}" : $"Pin {Title}";
    public int PinnedItemCount { get; private set; }
    public bool? BulkPinState => PinnedItemCount == 0 ? false : PinnedItemCount == Batch.Items.Count ? true : null;
    public Visibility MixedPinVisibility => BulkPinState is null ? Visibility.Visible : Visibility.Collapsed;
    public string BulkPinSummary => $"{PinnedItemCount} of {Batch.Items.Count} pinned";
    public string BulkPinAutomationName => BulkPinState == true
        ? $"Unpin all {Batch.Items.Count} items"
        : $"Pin all {Batch.Items.Count} items{(BulkPinState is null ? $", {PinnedItemCount} currently pinned" : string.Empty)}";
    public string ItemCountLabel => $"{Batch.Items.Count} item{(Batch.Items.Count == 1 ? string.Empty : "s")}";
    public string CreatedLabel => Batch.CreatedAt.ToLocalTime().ToString("g");
    public string ManageItemsAutomationName => $"Manage {Title}";
    public string DragBatchAutomationName => $"Drag batch {Title}";
    public string RemoveBatchAutomationName => $"Remove batch {Title}";

    public BatchCardViewModel(
        ShelfBatch batch,
        string title,
        string subtitle,
        string fallbackGlyph,
        IReadOnlyList<BatchItemPreview> previewItems,
        int pinnedItemCount)
    {
        Batch = batch;
        Title = title;
        Subtitle = subtitle;
        FallbackGlyph = fallbackGlyph;
        _previewItems = previewItems;
        PinnedItemCount = pinnedItemCount;
        SubscribeToItems(_previewItems);
    }

    public void Update(ShelfBatch batch, string title, string subtitle, string fallbackGlyph, IReadOnlyList<BatchItemPreview> previewItems, int pinnedItemCount)
    {
        foreach (var item in _previewItems) item.PropertyChanged -= OnItemPropertyChanged;
        Batch = batch;
        Title = title;
        Subtitle = subtitle;
        FallbackGlyph = fallbackGlyph;
        _previewItems = previewItems;
        PinnedItemCount = pinnedItemCount;
        SubscribeToItems(_previewItems);
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(string.Empty));
    }

    private void SubscribeToItems(IReadOnlyList<BatchItemPreview> items)
    {
        foreach (var item in items)
        {
            item.PropertyChanged += OnItemPropertyChanged;
        }
    }

    private void OnItemPropertyChanged(object? sender, PropertyChangedEventArgs args)
    {
        if (args.PropertyName == nameof(BatchItemPreview.Icon))
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(PrimaryIcon)));
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(SecondaryIcon)));
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(TertiaryIcon)));
        }
    }
}

internal sealed class ShelfItemViewModel : INotifyPropertyChanged
{
    private ImageSource? _icon;
    private bool _usedThumbnail;
    private bool _isBusy;
    private static readonly PropertyChangedEventArgs AllPropertiesChanged = new(string.Empty);
    private static readonly PropertyChangedEventArgs ActionsEnabledChanged = new(nameof(ActionsEnabled));

    public ShelfItemViewModel(ShelfItem item, string name, string type, string fallbackGlyph)
    {
        Item = item;
        Name = name;
        Type = type;
        FallbackGlyph = fallbackGlyph;
    }

    public event PropertyChangedEventHandler? PropertyChanged;

    public ShelfItem Item { get; private set; }
    public string Name { get; }
    public string Type { get; }
    public string Path => Item.Path;
    public string AvailabilityText => Item.Availability switch
    {
        ItemAvailability.Available => "Available",
        ItemAvailability.Missing => "Missing",
        ItemAvailability.Unavailable => "Unavailable",
        _ => "Unknown",
    };
    public string DetailsLabel => $"{AvailabilityText} • {(IsPinned ? "Pinned" : "Temporary")}";
    public string FallbackGlyph { get; }
    public ImageSource? Icon => _icon;
    public bool IsThumbnail => _usedThumbnail;
    public bool IsPinned => Item.IsPinned;
    public string VisualAutomationName => $"{Name} {(IsThumbnail ? "thumbnail" : "native icon")}";
    public string DragAutomationName => $"Drag {Name}";
    public string PinAutomationName => IsPinned ? $"Unpin {Name}" : $"Pin {Name}";
    public string RemoveAutomationName => $"Remove {Name}";
    public bool ActionsEnabled => !_isBusy;

    public void Update(ShelfItem item)
    {
        if (ReferenceEquals(Item, item)) return;
        Item = item;
        PropertyChanged?.Invoke(this, AllPropertiesChanged);
    }

    public void SetBusy(bool isBusy)
    {
        if (_isBusy == isBusy) return;
        _isBusy = isBusy;
        PropertyChanged?.Invoke(this, ActionsEnabledChanged);
    }

    public void SetIcon(ImageSource? icon, bool usedThumbnail)
    {
        if (ReferenceEquals(_icon, icon) && _usedThumbnail == usedThumbnail)
        {
            return;
        }

        _icon = icon;
        _usedThumbnail = usedThumbnail;
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(Icon)));
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(IsThumbnail)));
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(VisualAutomationName)));
    }
}

internal static class ShelfPresentation
{
    public static BatchCardViewModel CreateBatchCard(ShelfBatch batch, BatchCardViewModel? existing = null)
    {
        var previewCount = Math.Min(3, batch.Items.Count);
        IReadOnlyList<BatchItemPreview> previewItems;
        if (existing is not null && existing.PreviewItems.Count == previewCount)
        {
            previewItems = existing.PreviewItems;
        }
        else
        {
            var newPreviews = new BatchItemPreview[previewCount];
            for (var index = 0; index < previewCount; index++)
            {
                newPreviews[index] = new BatchItemPreview();
            }
            previewItems = newPreviews;
        }

        var first = batch.Items[0];
        var pinnedCount = batch.Items.Count(item => item.IsPinned);
        string subtitle;
        if (batch.Items.Count == 1)
        {
            subtitle = first.IsPinned ? $"Pinned • {GetItemType(first)}" : GetItemType(first);
        }
        else
        {
            var n1 = batch.Items[0].Name;
            var n2 = batch.Items[1].Name;
            subtitle = pinnedCount > 0 ? $"{n1}, {n2} • {pinnedCount} pinned" : $"{n1}, {n2}";
        }

        var title = batch.Items.Count == 1 ? first.Name : $"{batch.Items.Count} items";
        var fallbackGlyph = first.IsFolder ? "\uE8B7" : "\uE8A5";

        if (existing is not null)
        {
            existing.Update(batch, title, subtitle, fallbackGlyph, previewItems, pinnedCount);
            return existing;
        }

        return new BatchCardViewModel(
            batch,
            title,
            subtitle,
            fallbackGlyph,
            previewItems,
            pinnedCount);
    }

    public static ShelfItemViewModel CreateShelfItemViewModel(ShelfItem item) => new(
        item,
        item.Name,
        GetItemType(item),
        item.IsFolder ? "\uE8B7" : "\uE8A5");

    public static bool IsImageShelfItem(ShelfItem item)
    {
        if (item.IsFolder)
        {
            return false;
        }

        return Path.GetExtension(item.Name).ToUpperInvariant() switch
        {
            ".AVIF" or ".BMP" or ".GIF" or ".HEIC" or ".JPEG" or ".JPG" or ".PNG" or ".TIF" or ".TIFF" or ".WEBP" => true,
            _ => false,
        };
    }

    public static string GetItemType(ShelfItem item)
    {
        if (item.IsFolder)
        {
            return "Folder";
        }

        var extension = Path.GetExtension(item.Name);
        return string.IsNullOrEmpty(extension) ? "File" : $"{extension.TrimStart('.').ToUpperInvariant()} file";
    }
}

internal sealed class ShelfVisualLifetime
{
    private readonly ShelfVisualCoordinator<ImageSource> _coordinator;
    private readonly Dictionary<object, VisualRequest> _visualRequests = [];

    public ShelfVisualLifetime(ShelfVisualCoordinator<ImageSource> coordinator)
    {
        _coordinator = coordinator ?? throw new ArgumentNullException(nameof(coordinator));
    }

    public void Realize(object projection, ShelfItem item)
    {
        if (_visualRequests.TryGetValue(projection, out var existing))
        {
            existing.RealizationCount++;
            return;
        }

        var request = new VisualRequest();
        _visualRequests[projection] = request;
        _ = LoadVisualAsync(projection, item, request);
    }

    public void Release(object projection)
    {
        if (!_visualRequests.TryGetValue(projection, out var request))
        {
            return;
        }

        request.RealizationCount--;
        if (request.RealizationCount > 0)
        {
            return;
        }

        _visualRequests.Remove(projection);
        request.IsActive = false;
        SetVisual(projection, null, usedThumbnail: false);
        request.Cancellation.Cancel();
        request.Cancellation.Dispose();
    }

    public void Clear()
    {
        foreach (var (projection, request) in _visualRequests)
        {
            request.IsActive = false;
            SetVisual(projection, null, usedThumbnail: false);
            request.Cancellation.Cancel();
            request.Cancellation.Dispose();
        }

        _visualRequests.Clear();
    }

    private async Task LoadVisualAsync(object projection, ShelfItem item, VisualRequest request)
    {
        try
        {
            var result = await _coordinator.LoadAsync(item, request.Cancellation.Token);
            if (request.IsActive && !request.Cancellation.IsCancellationRequested)
            {
                SetVisual(projection, result.Display, result.UsedThumbnail);
            }
        }
        catch (OperationCanceledException)
        {
        }
        catch (Exception exception) when (IsExpectedVisualFailure(exception))
        {
            // The fallback glyph remains visible when an expected shell visual failure occurs.
        }
    }

    private static bool IsExpectedVisualFailure(Exception exception) =>
        exception is COMException or IOException or UnauthorizedAccessException or Win32Exception;

    private static void SetVisual(object projection, ImageSource? icon, bool usedThumbnail)
    {
        if (projection is BatchItemPreview preview)
        {
            preview.SetIcon(icon);
        }
        else if (projection is ShelfItemViewModel itemVm)
        {
            itemVm.SetIcon(icon, usedThumbnail);
        }
    }

    private sealed class VisualRequest
    {
        public CancellationTokenSource Cancellation { get; } = new();
        public bool IsActive { get; set; } = true;
        public int RealizationCount { get; set; } = 1;
    }
}
