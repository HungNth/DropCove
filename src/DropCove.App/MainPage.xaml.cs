using System.ComponentModel;
using System.Runtime.InteropServices;

using DropCove.Core;
using DropCove.Native;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage;

namespace DropCove;

/// <summary>Displays and accepts drops into the bounded unified Drop Shelf.</summary>
public sealed partial class MainPage : Page
{
    private readonly Dictionary<string, IStorageItem> _storageItems = new(StringComparer.OrdinalIgnoreCase);
    private readonly Dictionary<ShelfItemViewModel, VisualRequest> _visualRequests = [];
    private readonly Dictionary<UIElement, ShelfItemViewModel> _itemRealizations = [];
    private readonly Dictionary<UIElement, IReadOnlyList<ShelfItemViewModel>> _batchPreviews = [];
    private DropShelfManager? _manager;
    private DragDropService? _dragDropService;
    private ShelfVisualCoordinator<ImageSource>? _visualCoordinator;
    private Func<Task>? _dismissShelf;
    private Action? _showSettings;
    private Action? _beginWindowMove;
    private Action<int, int>? _resizeWindow;
    private Func<string, string, Task<bool>>? _confirm;
    private DispatcherTimer? _statusTimer;
    /// <summary>Initializes the page.</summary>
    public MainPage()
    {
        InitializeComponent();
        Loaded += (_, _) => Focus(FocusState.Programmatic);
    }

    internal async Task InitializeAsync(
        DropShelfManager manager,
        Func<Task> dismissShelf,
        Action showSettings,
        Action beginWindowMove,
        Action<int, int> resizeWindow,
        Func<string, string, Task<bool>> confirm)
    {
        _manager = manager;
        _visualCoordinator = new ShelfVisualCoordinator<ImageSource>(
            new WindowsShelfVisualProvider(TryGetCachedStorageItem),
            IsImageShelfItem);
        _dragDropService = new DragDropService(manager, confirm);
        _dismissShelf = dismissShelf;
        _showSettings = showSettings;
        _beginWindowMove = beginWindowMove;
        _resizeWindow = resizeWindow;
        _confirm = confirm;
        await RebuildBatchCardsAsync();
        ResizeForBatchCount(manager.Batches.Count);
    }

    private void OnDragRegionPointerPressed(object sender, PointerRoutedEventArgs e)
    {
        if (e.GetCurrentPoint(DragRegion).Properties.IsLeftButtonPressed)
        {
            _beginWindowMove?.Invoke();
        }
    }

    private void OnSettingsClicked(object sender, RoutedEventArgs e) => _showSettings?.Invoke();

    private async void OnCloseClicked(object sender, RoutedEventArgs e)
    {
        if (_dismissShelf is not null)
        {
            await _dismissShelf();
        }
    }

    private async void OnPageKeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (e.Key == Windows.System.VirtualKey.Escape)
        {
            if (_dismissShelf is not null)
            {
                await _dismissShelf();
            }

            e.Handled = true;
        }
    }

    private void OnDragEnter(object sender, DragEventArgs e) => UpdateDragFeedback(e);

    private void OnDragOver(object sender, DragEventArgs e) => UpdateDragFeedback(e);

    private void OnDragLeave(object sender, DragEventArgs e) => DragOverlay.Visibility = Visibility.Collapsed;

    private void UpdateDragFeedback(DragEventArgs e)
    {
        var acceptsCopy = CanCopyStorageItems(e);
        e.AcceptedOperation = acceptsCopy ? DataPackageOperation.Copy : DataPackageOperation.None;
        e.DragUIOverride.Caption = acceptsCopy ? "Add to DropCove" : "Filesystem items with Copy support only";
        e.DragUIOverride.IsCaptionVisible = true;
        e.DragUIOverride.IsContentVisible = false;
        DragOverlay.Visibility = acceptsCopy ? Visibility.Visible : Visibility.Collapsed;
    }

    private async void OnDrop(object sender, DragEventArgs e)
    {
        DragOverlay.Visibility = Visibility.Collapsed;
        if (_manager is null || !CanCopyStorageItems(e))
        {
            ShowDropMessage("No supported filesystem paths were found.", InfoBarSeverity.Warning);
            return;
        }

        var deferral = e.GetDeferral();
        try
        {
            var storageItems = await e.DataView.GetStorageItemsAsync();
            var incomingItems = new List<IncomingShelfItem>(storageItems.Count);
            var skipped = 0;

            foreach (var storageItem in storageItems)
            {
                if (string.IsNullOrWhiteSpace(storageItem.Path))
                {
                    skipped++;
                    continue;
                }

                _storageItems[storageItem.Path] = storageItem;
                incomingItems.Add(new IncomingShelfItem(
                    storageItem.Path,
                    storageItem.Name,
                    storageItem is StorageFolder));
            }

            var outcome = await _manager.AcceptDropAsync(incomingItems, skipped);
            if (outcome.Batch is null)
            {
                ShowDropMessage("No supported filesystem paths were found.", InfoBarSeverity.Warning);
                return;
            }

            await RebuildBatchCardsAsync();
            ResizeForBatchCount(_manager.Batches.Count);
            BatchScroller.ChangeView(null, 0, null, true);

            if (outcome.SkippedUnsupportedCount > 0)
            {
                ShowDropMessage(
                    $"Added {outcome.AcceptedCount}; skipped {outcome.SkippedUnsupportedCount} unsupported item(s).",
                    InfoBarSeverity.Warning);
            }
            else
            {
                HideStatus();
            }
        }
        catch (Exception exception)
        {
            ShowDropMessage($"Drop failed: {exception.Message}", InfoBarSeverity.Error);
        }
        finally
        {
            deferral.Complete();
        }
    }

    private async void OnItemDragStarting(UIElement sender, DragStartingEventArgs e)
    {
        if (sender is not FrameworkElement { DataContext: ShelfItemViewModel item } ||
            _dragDropService is null)
        {
            e.Cancel = true;
            return;
        }

        var error = await _dragDropService.PrepareItemDragAsync(item.Item, e);
        if (error is not null)
        {
            ShowDropMessage($"Could not drag {item.Name}: {error}", InfoBarSeverity.Warning);
        }
    }

    private async void OnItemDropCompleted(UIElement sender, DropCompletedEventArgs e)
    {
        if (sender is FrameworkElement { DataContext: ShelfItemViewModel item } &&
            _dragDropService is not null &&
            await _dragDropService.CompleteItemDragAsync(item.Item.Id, e.DropResult))
        {
            await RefreshAfterMutationAsync();
        }
    }

    private async void OnBatchDragStarting(UIElement sender, DragStartingEventArgs e)
    {
        if (sender is not FrameworkElement { DataContext: BatchCardViewModel batchVm } ||
            _dragDropService is null)
        {
            e.Cancel = true;
            return;
        }

        var (error, missingRemoved, unavailableRetained) = await _dragDropService.PrepareBatchDragAsync(batchVm.Batch, e);
        if (missingRemoved > 0)
        {
            if (e.Cancel)
            {
                await RefreshAfterMutationAsync();
            }

            ShowDropMessage($"Removed {missingRemoved} missing items from batch.", InfoBarSeverity.Informational);
        }

        if (error is not null)
        {
            ShowDropMessage($"Could not drag batch: {error}", InfoBarSeverity.Warning);
        }
        else if (unavailableRetained > 0)
        {
            ShowDropMessage($"Dragging available items ({unavailableRetained} unavailable items retained).", InfoBarSeverity.Informational);
        }
    }

    private async void OnBatchDropCompleted(UIElement sender, DropCompletedEventArgs e)
    {
        if (sender is FrameworkElement { DataContext: BatchCardViewModel batchVm } &&
            _dragDropService is not null)
        {
            await _dragDropService.CompleteBatchDragAsync(batchVm.Batch.Id, e.DropResult);
            await RefreshAfterMutationAsync();
        }
    }
    private async void OnPinClicked(object sender, RoutedEventArgs e)
    {
        if (sender is ToggleButton { DataContext: ShelfItemViewModel item } toggle &&
            _manager is not null &&
            await _manager.SetPinnedAsync(item.Item.Id, toggle.IsChecked == true))
        {
            await RefreshAfterMutationAsync();
        }
    }

    private async void OnRemoveItemClicked(object sender, RoutedEventArgs e)
    {
        if (sender is Button { DataContext: ShelfItemViewModel item } &&
            _manager is not null &&
            await _manager.RemoveItemAsync(item.Item.Id))
        {
            await RefreshAfterMutationAsync();
        }
    }

    private async void OnClearTemporaryClicked(object sender, RoutedEventArgs e)
    {
        if (_manager is null || _manager.Batches.Count == 0)
        {
            return;
        }

        var confirmed = _confirm is not null && await _confirm(
            "Clear Temporary Items",
            "Remove all temporary item references from DropCove?\n\nSource files on disk will not be touched.");

        if (confirmed)
        {
            var count = await _manager.ClearTemporaryItemsAsync();
            await RefreshAfterMutationAsync();
            ShowDropMessage($"Cleared {count} temporary items.");
        }
    }

    private async void OnRemoveBatchClicked(object sender, RoutedEventArgs e)
    {
        if (sender is Button { DataContext: BatchCardViewModel batchVm } &&
            _manager is not null &&
            await _manager.RemoveBatchAsync(batchVm.Batch.Id))
        {
            await RefreshAfterMutationAsync();
        }
    }

    private void OnToggleExpandClicked(object sender, RoutedEventArgs e)
    {
        if (sender is Button { DataContext: BatchCardViewModel batchVm })
        {
            batchVm.IsExpanded = !batchVm.IsExpanded;
        }
    }

    private async Task RefreshAfterMutationAsync()
    {
        HideStatus();
        await RebuildBatchCardsAsync();
        ResizeForBatchCount(_manager!.Batches.Count);
    }

    private Task RebuildBatchCardsAsync()
    {
        var manager = _manager!;
        CancelVisualRequests();
        var cards = manager.Batches.Select(CreateBatchCard).ToArray();
        BatchList.ItemsSource = cards;
        EmptyState.Visibility = cards.Length == 0 ? Visibility.Visible : Visibility.Collapsed;
        BatchScroller.Visibility = cards.Length == 0 ? Visibility.Collapsed : Visibility.Visible;

        var overflow = Math.Max(0, cards.Length - 4);
        OverflowLabel.Text = $"+{overflow} batch{(overflow == 1 ? string.Empty : "es")}";
        OverflowLabel.Visibility = overflow == 0 ? Visibility.Collapsed : Visibility.Visible;
        return Task.CompletedTask;
    }

    private void OnBatchElementPrepared(ItemsRepeater sender, ItemsRepeaterElementPreparedEventArgs args)
    {
        if (args.Element is not FrameworkElement { DataContext: BatchCardViewModel batch })
        {
            return;
        }

        var previewItems = batch.Items.Take(3).ToArray();
        _batchPreviews[args.Element] = previewItems;
        foreach (var item in previewItems)
        {
            RealizeVisual(item);
        }
    }

    private void OnBatchElementClearing(ItemsRepeater sender, ItemsRepeaterElementClearingEventArgs args)
    {
        if (_batchPreviews.Remove(args.Element, out var previewItems))
        {
            foreach (var item in previewItems)
            {
                ReleaseVisual(item);
            }
        }

        if (args.Element is FrameworkElement { DataContext: BatchCardViewModel batch })
        {
            foreach (var realization in _itemRealizations
                         .Where(pair => batch.Items.Contains(pair.Value))
                         .ToArray())
            {
                _itemRealizations.Remove(realization.Key);
                ReleaseVisual(realization.Value);
            }
        }
    }

    private void OnItemElementPrepared(ItemsRepeater sender, ItemsRepeaterElementPreparedEventArgs args)
    {
        if (args.Element is not FrameworkElement { DataContext: ShelfItemViewModel item })
        {
            return;
        }

        _itemRealizations[args.Element] = item;
        RealizeVisual(item);
    }

    private void OnItemElementClearing(ItemsRepeater sender, ItemsRepeaterElementClearingEventArgs args)
    {
        if (_itemRealizations.Remove(args.Element, out var item))
        {
            ReleaseVisual(item);
        }
    }

    private void RealizeVisual(ShelfItemViewModel item)
    {
        if (_visualCoordinator is null)
        {
            return;
        }

        if (_visualRequests.TryGetValue(item, out var existing))
        {
            existing.RealizationCount++;
            return;
        }

        var request = new VisualRequest();
        _visualRequests[item] = request;
        _ = LoadVisualAsync(item, request);
    }

    private void ReleaseVisual(ShelfItemViewModel item)
    {
        if (!_visualRequests.TryGetValue(item, out var request))
        {
            return;
        }

        request.RealizationCount--;
        if (request.RealizationCount > 0)
        {
            return;
        }

        _visualRequests.Remove(item);
        request.IsActive = false;
        item.SetIcon(null, usedThumbnail: false);
        request.Cancellation.Cancel();
        request.Cancellation.Dispose();
    }

    private async Task LoadVisualAsync(ShelfItemViewModel item, VisualRequest request)
    {
        try
        {
            var result = await _visualCoordinator!.LoadAsync(item.Item, request.Cancellation.Token);
            if (request.IsActive && !request.Cancellation.IsCancellationRequested)
            {
                item.SetIcon(result.Display, result.UsedThumbnail);
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

    private void CancelVisualRequests()
    {
        foreach (var (item, request) in _visualRequests)
        {
            request.IsActive = false;
            item.SetIcon(null, usedThumbnail: false);
            request.Cancellation.Cancel();
            request.Cancellation.Dispose();
        }

        _visualRequests.Clear();
        _itemRealizations.Clear();
        _batchPreviews.Clear();
    }

    private BatchCardViewModel CreateBatchCard(ShelfBatch batch)
    {
        var items = batch.Items.Select(CreateShelfItemViewModel).ToArray();
        var first = items[0];
        var pinnedCount = items.Count(item => item.IsPinned);
        var subtitle = items.Length == 1
            ? $"{(first.IsPinned ? "Pinned • " : string.Empty)}{first.Type}"
            : $"{string.Join(", ", items.Take(2).Select(item => item.Name))}{(pinnedCount > 0 ? $" • {pinnedCount} pinned" : string.Empty)}";

        return new BatchCardViewModel(
            batch,
            items.Length == 1 ? first.Name : $"{items.Length} items",
            subtitle,
            first.FallbackGlyph,
            items.Length > 1 ? Visibility.Visible : Visibility.Collapsed,
            items.Length > 2 ? Visibility.Visible : Visibility.Collapsed,
            items);
    }

    private static ShelfItemViewModel CreateShelfItemViewModel(ShelfItem item) => new(
        item,
        item.Name,
        GetItemType(item),
        item.IsFolder ? "\uE8B7" : "\uE8A5");

    private IStorageItem? TryGetCachedStorageItem(ShelfItem item) =>
        _storageItems.TryGetValue(item.Path, out var storageItem) ? storageItem : null;

    private static bool IsImageShelfItem(ShelfItem item)
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

    private sealed class VisualRequest
    {
        public CancellationTokenSource Cancellation { get; } = new();
        public bool IsActive { get; set; } = true;
        public int RealizationCount { get; set; } = 1;
    }

    private void ResizeForBatchCount(int batchCount)
    {
        var height = batchCount switch
        {
            <= 1 => 180,
            2 => 236,
            3 => 292,
            _ => 348,
        };
        _resizeWindow?.Invoke(180, height);
    }

    private static bool CanCopyStorageItems(DragEventArgs e) =>
        e.DataView.Contains(StandardDataFormats.StorageItems) &&
        (e.AllowedOperations & DataPackageOperation.Copy) != 0;

    private void ShowDropMessage(string message, InfoBarSeverity? severity = null)
    {
        StatusText.Text = message;
        StatusBar.Visibility = Visibility.Visible;

        _statusTimer?.Stop();
        _statusTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(3.5) };
        _statusTimer.Tick += (_, _) =>
        {
            _statusTimer?.Stop();
            HideStatus();
        };
        _statusTimer.Start();
    }

    private void OnDismissStatusClicked(object sender, RoutedEventArgs e) => HideStatus();

    private void HideStatus()
    {
        _statusTimer?.Stop();
        StatusBar.Visibility = Visibility.Collapsed;
    }



    private static string GetItemType(ShelfItem item)
    {
        if (item.IsFolder)
        {
            return "Folder";
        }

        var extension = Path.GetExtension(item.Name);
        return string.IsNullOrEmpty(extension) ? "File" : $"{extension.TrimStart('.').ToUpperInvariant()} file";
    }
}

internal sealed class BatchCardViewModel : System.ComponentModel.INotifyPropertyChanged
{
    private bool _isExpanded;

    public event System.ComponentModel.PropertyChangedEventHandler? PropertyChanged;

    public ShelfBatch Batch { get; }
    public string Title { get; }
    public string Subtitle { get; }
    public string FallbackGlyph { get; }
    public ImageSource? PrimaryIcon => Items.ElementAtOrDefault(0)?.Icon;
    public ImageSource? SecondaryIcon => Items.ElementAtOrDefault(1)?.Icon;
    public ImageSource? TertiaryIcon => Items.ElementAtOrDefault(2)?.Icon;
    public Visibility SecondaryVisibility { get; }
    public Visibility TertiaryVisibility { get; }
    public IReadOnlyList<ShelfItemViewModel> Items { get; }

    public BatchCardViewModel(
        ShelfBatch batch,
        string title,
        string subtitle,
        string fallbackGlyph,
        Visibility secondaryVisibility,
        Visibility tertiaryVisibility,
        IReadOnlyList<ShelfItemViewModel> items)
    {
        Batch = batch;
        Title = title;
        Subtitle = subtitle;
        FallbackGlyph = fallbackGlyph;
        SecondaryVisibility = secondaryVisibility;
        TertiaryVisibility = tertiaryVisibility;
        Items = items;
        foreach (var item in items)
        {
            item.PropertyChanged += OnItemPropertyChanged;
        }
    }

    public bool IsExpanded
    {
        get => _isExpanded;
        set
        {
            if (_isExpanded != value)
            {
                _isExpanded = value;
                PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(IsExpanded)));
                PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(ItemsVisibility)));
                PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(ExpandGlyph)));
            }
        }
    }

    public Visibility ItemsVisibility => IsExpanded ? Visibility.Visible : Visibility.Collapsed;
    public string ExpandGlyph => IsExpanded ? "\uE70E" : "\uE70D";
    public string DragBatchAutomationName => $"Drag batch {Title}";
    public string RemoveBatchAutomationName => $"Remove batch {Title}";
    public string ToggleExpandAutomationName => IsExpanded ? $"Collapse {Title}" : $"Expand {Title}";

    private void OnItemPropertyChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs args)
    {
        if (args.PropertyName != nameof(ShelfItemViewModel.Icon))
        {
            return;
        }

        PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(PrimaryIcon)));
        PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(SecondaryIcon)));
        PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(TertiaryIcon)));
    }
}

internal sealed class ShelfItemViewModel : System.ComponentModel.INotifyPropertyChanged
{
    private ImageSource? _icon;
    private bool _usedThumbnail;

    public ShelfItemViewModel(ShelfItem item, string name, string type, string fallbackGlyph)
    {
        Item = item;
        Name = name;
        Type = type;
        FallbackGlyph = fallbackGlyph;
    }

    public event System.ComponentModel.PropertyChangedEventHandler? PropertyChanged;

    public ShelfItem Item { get; }
    public string Name { get; }
    public string Type { get; }
    public string FallbackGlyph { get; }
    public ImageSource? Icon => _icon;
    public bool IsThumbnail => _usedThumbnail;
    public bool IsPinned => Item.IsPinned;
    public string VisualAutomationName => $"{Name} {(IsThumbnail ? "thumbnail" : "native icon")}";
    public string DragAutomationName => $"Drag {Name}";
    public string PinAutomationName => IsPinned ? $"Unpin {Name}" : $"Pin {Name}";
    public string RemoveAutomationName => $"Remove {Name}";

    public void SetIcon(ImageSource? icon, bool usedThumbnail)
    {
        if (ReferenceEquals(_icon, icon) && _usedThumbnail == usedThumbnail)
        {
            return;
        }

        _icon = icon;
        _usedThumbnail = usedThumbnail;
        PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(Icon)));
        PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(IsThumbnail)));
        PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(VisualAutomationName)));
    }
}
