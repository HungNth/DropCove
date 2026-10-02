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

/// <summary>Displays and accepts drops in the Compact and Expanded Drop Shelf presentations.</summary>
public sealed partial class MainPage : Page
{
    private readonly Dictionary<string, IStorageItem> _storageItems = new(StringComparer.OrdinalIgnoreCase);
    private readonly Dictionary<ShelfItemViewModel, VisualRequest> _visualRequests = [];
    private readonly Dictionary<UIElement, ShelfItemViewModel> _itemRealizations = [];
    private readonly Dictionary<UIElement, IReadOnlyList<ShelfItemViewModel>> _batchPreviews = [];
    private readonly HashSet<Guid> _pendingBatchAnimations = [];
    private HashSet<Guid> _knownBatchIds = [];
    private DropShelfManager? _manager;
    private DragDropService? _dragDropService;
    private StorageDropService? _storageDropService;
    private ShelfVisualCoordinator<ImageSource>? _visualCoordinator;
    private Func<Task>? _dismissShelf;
    private Action? _showSettings;
    private Action? _beginWindowMove;
    private Action<ShelfDisplayState>? _resizeForPresentation;
    private Func<string, string, Task<bool>>? _confirm;
    private Action? _shakeDragCanceled;
    private Func<bool, Task>? _shakeDropCompleted;
    private Action? _shakeDragStarted;
    private DispatcherTimer? _statusTimer;
    private ShelfDisplayState _presentation = ShelfDisplayState.Compact;
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
        Action<ShelfDisplayState> resizeForPresentation,
        Func<string, string, Task<bool>> confirm,
        Action shakeDragStarted,
        Action shakeDragCanceled,
        Func<bool, Task> shakeDropCompleted)
    {
        ArgumentNullException.ThrowIfNull(resizeForPresentation);
        ArgumentNullException.ThrowIfNull(shakeDragStarted);
        ArgumentNullException.ThrowIfNull(shakeDragCanceled);
        ArgumentNullException.ThrowIfNull(shakeDropCompleted);
        _manager = manager;
        _presentation = manager.DisplayState == ShelfDisplayState.Expanded
            ? ShelfDisplayState.Expanded
            : ShelfDisplayState.Compact;
        _visualCoordinator = new ShelfVisualCoordinator<ImageSource>(
            new WindowsShelfVisualProvider(TryGetCachedStorageItem),
            IsImageShelfItem);
        _dragDropService = new DragDropService(manager, confirm);
        _storageDropService = new StorageDropService(manager, storageItem => _storageItems[storageItem.Path] = storageItem);
        _dismissShelf = dismissShelf;
        _showSettings = showSettings;
        _beginWindowMove = beginWindowMove;
        _resizeForPresentation = resizeForPresentation;
        _confirm = confirm;
        _shakeDragCanceled = shakeDragCanceled;
        _shakeDropCompleted = shakeDropCompleted;
        _shakeDragStarted = shakeDragStarted;
        await RefreshCardsAsync(animate: false);
    }
    internal Task<DropAcceptance> AcceptStorageDropAsync(DataPackageView dataView) =>
        (_storageDropService ?? throw new InvalidOperationException("MainPage is not initialized.")).AcceptAsync(dataView);

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

    private void OnDragEnter(object sender, DragEventArgs e)
    {
        _shakeDragStarted?.Invoke();
        UpdateDragFeedback(e);
    }

    private void OnDragOver(object sender, DragEventArgs e) => UpdateDragFeedback(e);

    private void OnDragLeave(object sender, DragEventArgs e)
    {
        DragOverlay.Visibility = Visibility.Collapsed;
        _shakeDragCanceled?.Invoke();
    }

    private void UpdateDragFeedback(DragEventArgs e)
    {
        var acceptsCopy = StorageDropService.CanCopyStorageItems(e);
        e.AcceptedOperation = acceptsCopy ? DataPackageOperation.Copy : DataPackageOperation.None;
        e.DragUIOverride.Caption = acceptsCopy ? "Add to DropCove" : "Filesystem items with Copy support only";
        e.DragUIOverride.IsCaptionVisible = true;
        e.DragUIOverride.IsContentVisible = false;
        DragOverlay.Visibility = acceptsCopy ? Visibility.Visible : Visibility.Collapsed;
    }

    private async void OnDrop(object sender, DragEventArgs e)
    {
        var deferral = e.GetDeferral();
        var shakeOutcomeReported = false;
        try
        {
            _shakeDragStarted?.Invoke();
            DragOverlay.Visibility = Visibility.Collapsed;
            if (_manager is null || !StorageDropService.CanCopyStorageItems(e))
            {
                shakeOutcomeReported = true;
                await (_shakeDropCompleted?.Invoke(false) ?? Task.CompletedTask);
                ShowDropMessage("No supported filesystem paths were found.", InfoBarSeverity.Warning);
                return;
            }

            var outcome = await AcceptStorageDropAsync(e.DataView);
            if (outcome.Batch is null)
            {
                shakeOutcomeReported = true;
                await (_shakeDropCompleted?.Invoke(false) ?? Task.CompletedTask);
                ShowDropMessage("No supported filesystem paths were found.", InfoBarSeverity.Warning);
                return;
            }

            shakeOutcomeReported = true;
            await (_shakeDropCompleted?.Invoke(true) ?? Task.CompletedTask);
            await RefreshCardsAsync();
            if (_presentation == ShelfDisplayState.Expanded)
            {
                ExpandedBatchScroller.ChangeView(null, 0, null, true);
            }
            else
            {
                CompactBatchScroller.ChangeView(null, 0, null, true);
            }

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
            if (!shakeOutcomeReported)
            {
                await (_shakeDropCompleted?.Invoke(false) ?? Task.CompletedTask);
            }

            ShowDropMessage($"Drop failed: {exception.Message}", InfoBarSeverity.Error);
        }
        finally
        {
            deferral.Complete();
        }
    }

    private async void OnItemDragStarting(UIElement sender, DragStartingEventArgs e)
    {
        if (sender is not FrameworkElement element || _dragDropService is null)
        {
            e.Cancel = true;
            return;
        }

        var item = element.Tag as ShelfItemViewModel ?? element.DataContext as ShelfItemViewModel;
        if (item is null)
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
    private async void OnBatchDragStarting(UIElement sender, DragStartingEventArgs e)
    {
        if (sender is not FrameworkElement element || _dragDropService is null)
        {
            e.Cancel = true;
            return;
        }

        var batchVm = element.Tag as BatchCardViewModel ?? element.DataContext as BatchCardViewModel;
        if (batchVm is null)
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
        if (sender is not FrameworkElement element || _dragDropService is null)
        {
            return;
        }

        var batchVm = element.Tag as BatchCardViewModel ?? element.DataContext as BatchCardViewModel;
        if (batchVm is null)
        {
            return;
        }

        var removesWholeBatch = batchVm.Batch.Items.All(item => !item.IsPinned);
        var consumed = await _dragDropService.CompleteBatchDragAsync(batchVm.Batch.Id, e.DropResult);
        try
        {
            if (consumed && removesWholeBatch)
            {
                await ShelfMotion.PlayExitAsync(sender);
            }
        }
        finally
        {
            await RefreshAfterMutationAsync(autoHideWhenEmpty: consumed);
            ShelfMotion.Reset(sender);
        }
    }

    private async void OnItemDropCompleted(UIElement sender, DropCompletedEventArgs e)
    {
        if (sender is not FrameworkElement element || _dragDropService is null)
        {
            return;
        }

        var item = element.Tag as ShelfItemViewModel ?? element.DataContext as ShelfItemViewModel;
        if (item is not null && await _dragDropService.CompleteItemDragAsync(item.Item.Id, e.DropResult))
        {
            try
            {
                await ShelfMotion.PlayExitAsync(sender);
            }
            finally
            {
                await RefreshAfterMutationAsync(autoHideWhenEmpty: true);
                ShelfMotion.Reset(sender);
            }
        }
    }



    private async void OnPinClicked(object sender, RoutedEventArgs e)
    {
        if (sender is ToggleButton toggle && _manager is not null)
        {
            var itemId = (toggle.Tag as ShelfItemViewModel)?.Item.Id
                ?? (toggle.Tag as BatchCardViewModel)?.SingleItem?.Item.Id
                ?? toggle.DataContext switch
                {
                    ShelfItemViewModel item => item.Item.Id,
                    BatchCardViewModel { SingleItem: { } single } => single.Item.Id,
                    _ => (Guid?)null
                };

            if (itemId.HasValue && await _manager.SetPinnedAsync(itemId.Value, toggle.IsChecked == true))
            {
                await RefreshAfterMutationAsync();
            }
        }
    }

    private async void OnRemoveItemClicked(object sender, RoutedEventArgs e)
    {
        if (sender is not Button button || _manager is null)
        {
            return;
        }

        var item = button.Tag as ShelfItemViewModel ?? button.DataContext as ShelfItemViewModel;
        if (item is null)
        {
            return;
        }

        var animationTarget = FindAnimationTarget(sender);
        if (await _manager.RemoveItemAsync(item.Item.Id))
        {
            try
            {
                if (animationTarget is not null)
                {
                    await ShelfMotion.PlayExitAsync(animationTarget);
                }
            }
            finally
            {
                await RefreshAfterMutationAsync(autoHideWhenEmpty: true);
                if (animationTarget is not null)
                {
                    ShelfMotion.Reset(animationTarget);
                }
            }
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
            await RefreshAfterMutationAsync(autoHideWhenEmpty: true);
            ShowDropMessage($"Cleared {count} temporary items.");
        }
    }

    private async void OnRemoveBatchClicked(object sender, RoutedEventArgs e)
    {
        if (sender is not Button button || _manager is null)
        {
            return;
        }

        var batchVm = button.Tag as BatchCardViewModel ?? button.DataContext as BatchCardViewModel;
        if (batchVm is null)
        {
            return;
        }

        var animationTarget = FindAnimationTarget(sender);
        if (await _manager.RemoveBatchAsync(batchVm.Batch.Id))
        {
            try
            {
                if (animationTarget is not null)
                {
                    await ShelfMotion.PlayExitAsync(animationTarget);
                }
            }
            finally
            {
                await RefreshAfterMutationAsync(autoHideWhenEmpty: true);
                if (animationTarget is not null)
                {
                    ShelfMotion.Reset(animationTarget);
                }
            }
        }
    }

    private void OnToggleExpandClicked(object sender, RoutedEventArgs e)
    {
        if (sender is not Button button)
        {
            return;
        }

        var batchVm = button.Tag as BatchCardViewModel ?? button.DataContext as BatchCardViewModel;
        if (batchVm is not null)
        {
            batchVm.IsExpanded = !batchVm.IsExpanded;
        }
    }

    private async void OnCompactModeClicked(object sender, RoutedEventArgs e) => await SetPresentationModeAsync(ShelfDisplayState.Compact);

    private async void OnExpandedModeClicked(object sender, RoutedEventArgs e) => await SetPresentationModeAsync(ShelfDisplayState.Expanded);

    private async void OnShowExpandedClicked(object sender, RoutedEventArgs e) => await SetPresentationModeAsync(ShelfDisplayState.Expanded);

    private async Task SetPresentationModeAsync(ShelfDisplayState presentation)
    {
        if (_manager is null || presentation is not (ShelfDisplayState.Compact or ShelfDisplayState.Expanded))
        {
            return;
        }

        _manager.ShowShelf(presentation);
        await RefreshCardsAsync();
        if (presentation == ShelfDisplayState.Expanded)
        {
            ExpandedBatchScroller.ChangeView(null, 0, null, true);
        }
    }

    internal Task RefreshAsync() => RefreshCardsAsync(animate: false);

    internal void ReleasePresentation()
    {
        CompactBatchList.ItemsSource = null;
        ExpandedBatchList.ItemsSource = null;
        CancelVisualRequests();
        _storageItems.Clear();
        _knownBatchIds.Clear();
        _pendingBatchAnimations.Clear();
        HideStatus();
    }

    internal void PlayShelfAppearance() => ShelfMotion.PlayEntrance(ContentSurface);

    private Task RefreshCardsAsync(bool animate = true)
    {
        HideStatus();
        CancelVisualRequests();
        var batches = _manager!.Batches;
        var currentBatchIds = batches.Select(batch => batch.Id).ToHashSet();
        _pendingBatchAnimations.Clear();
        if (animate)
        {
            _pendingBatchAnimations.UnionWith(currentBatchIds.Except(_knownBatchIds));
        }

        _knownBatchIds = currentBatchIds;
        _presentation = _manager.DisplayState == ShelfDisplayState.Expanded
            ? ShelfDisplayState.Expanded
            : ShelfDisplayState.Compact;
        var cards = batches.Select(CreateBatchCard).ToArray();
        if (_presentation == ShelfDisplayState.Compact)
        {
            CompactBatchList.ItemsSource = cards;
            ExpandedBatchList.ItemsSource = null;
        }
        else
        {
            CompactBatchList.ItemsSource = null;
            ExpandedBatchList.ItemsSource = cards;
        }

        UpdatePresentationVisibility(batches.Count > 0);
        if (_manager.DisplayState is ShelfDisplayState.Compact or ShelfDisplayState.Expanded)
        {
            _resizeForPresentation?.Invoke(_presentation);
        }

        if (animate && batches.Count > 0 && _pendingBatchAnimations.Count == 0)
        {
            ShelfMotion.PlayStateChange(
                _presentation == ShelfDisplayState.Expanded
                    ? ExpandedSurface
                    : CompactSurface);
        }

        return Task.CompletedTask;
    }

    private void UpdatePresentationVisibility(bool hasBatches)
    {
        var isExpanded = _presentation == ShelfDisplayState.Expanded;
        CompactSurface.Visibility = hasBatches && !isExpanded ? Visibility.Visible : Visibility.Collapsed;
        ExpandedSurface.Visibility = hasBatches && isExpanded ? Visibility.Visible : Visibility.Collapsed;
        EmptyState.Visibility = hasBatches ? Visibility.Collapsed : Visibility.Visible;
        ExpandModeButton.Visibility = isExpanded ? Visibility.Collapsed : Visibility.Visible;
        CompactModeButton.Visibility = isExpanded ? Visibility.Visible : Visibility.Collapsed;
        ClearTemporaryButton.Visibility = hasBatches ? Visibility.Visible : Visibility.Collapsed;
    }

    private async Task RefreshAfterMutationAsync(bool autoHideWhenEmpty = false)
    {
        await RefreshCardsAsync();
        if (autoHideWhenEmpty && _manager?.Batches.Count == 0 && _dismissShelf is not null)
        {
            await _dismissShelf();
        }
    }
    private static UIElement? FindAnimationTarget(object sender)
    {
        if (sender is not DependencyObject current)
        {
            return null;
        }

        while (current is not null)
        {
            if (current is Border border)
            {
                return border;
            }

            current = VisualTreeHelper.GetParent(current);
        }

        return sender as UIElement;
    }

    private void OnBatchElementPrepared(ItemsRepeater sender, ItemsRepeaterElementPreparedEventArgs args)
    {
        if (args.Element is not FrameworkElement { Tag: BatchCardViewModel batch })
        {
            return;
        }

        var previewItems = batch.PreviewItems;
        _batchPreviews[args.Element] = previewItems;
        foreach (var item in previewItems)
        {
            RealizeVisual(item);
        }
        if (_pendingBatchAnimations.Remove(batch.Batch.Id))
        {
            ShelfMotion.PlayInsertion(args.Element);
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

        if (args.Element is FrameworkElement { Tag: BatchCardViewModel batch })
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
        if (args.Element is not FrameworkElement { Tag: ShelfItemViewModel item })
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
        var previewItems = new ShelfItemViewModel[Math.Min(3, batch.Items.Count)];
        for (var index = 0; index < previewItems.Length; index++)
        {
            previewItems[index] = CreateShelfItemViewModel(batch.Items[index]);
        }

        var first = previewItems[0];
        var pinnedCount = batch.Items.Count(item => item.IsPinned);
        var subtitle = batch.Items.Count == 1
            ? $"{(first.IsPinned ? "Pinned • " : string.Empty)}{first.Type}"
            : $"{string.Join(", ", batch.Items.Take(2).Select(item => item.Name))}{(pinnedCount > 0 ? $" • {pinnedCount} pinned" : string.Empty)}";

        var isExpanded = _presentation == ShelfDisplayState.Expanded;
        return new BatchCardViewModel(
            batch,
            batch.Items.Count == 1 ? first.Name : $"{batch.Items.Count} items",
            subtitle,
            first.FallbackGlyph,
            previewItems,
            () => CreateShelfItemViews(batch, previewItems),
            isExpanded);
    }

    private static IReadOnlyList<ShelfItemViewModel> CreateShelfItemViews(
        ShelfBatch batch,
        IReadOnlyList<ShelfItemViewModel> previewItems)
    {
        var items = new ShelfItemViewModel[batch.Items.Count];
        for (var index = 0; index < items.Length; index++)
        {
            items[index] = index < previewItems.Count
                ? previewItems[index]
                : CreateShelfItemViewModel(batch.Items[index]);
        }

        return items;
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
    private readonly IReadOnlyList<ShelfItemViewModel> _previewItems;
    private readonly Func<IReadOnlyList<ShelfItemViewModel>> _createItems;
    private IReadOnlyList<ShelfItemViewModel>? _items;
    private bool _isExpanded;

    public event System.ComponentModel.PropertyChangedEventHandler? PropertyChanged;

    public ShelfBatch Batch { get; }
    public string Title { get; }
    public string Subtitle { get; }
    public string FallbackGlyph { get; }
    public IReadOnlyList<ShelfItemViewModel> PreviewItems => _previewItems;
    public ImageSource? PrimaryIcon => _previewItems.ElementAtOrDefault(0)?.Icon;
    public ImageSource? SecondaryIcon => _previewItems.ElementAtOrDefault(1)?.Icon;
    public ImageSource? TertiaryIcon => _previewItems.ElementAtOrDefault(2)?.Icon;
    public bool IsSingleItem => Batch.Items.Count == 1;
    public Visibility SingleItemVisibility => IsSingleItem ? Visibility.Visible : Visibility.Collapsed;
    public Visibility MultiItemVisibility => IsSingleItem ? Visibility.Collapsed : Visibility.Visible;
    public ShelfItemViewModel? SingleItem => _previewItems.ElementAtOrDefault(0);
    public bool IsPinned => SingleItem?.IsPinned ?? false;
    public string SingleItemPinAutomationName => IsPinned ? $"Unpin {Title}" : $"Pin {Title}";
    public string ItemCountLabel => $"{Batch.Items.Count} item{(Batch.Items.Count == 1 ? string.Empty : "s")}";
    public string CreatedLabel => Batch.CreatedAt.ToLocalTime().ToString("g");
    public IReadOnlyList<ShelfItemViewModel> Items => _items ?? _previewItems;

    public bool IsExpanded
    {
        get => _isExpanded;
        set
        {
            if (_isExpanded != value)
            {
                if (value)
                {
                    EnsureItems();
                }
                else
                {
                    ReleaseItems();
                }

                _isExpanded = value;
                PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(IsExpanded)));
                PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(Items)));
                PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(ItemsVisibility)));
                PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(ExpandGlyph)));
                PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(ToggleExpandAutomationName)));
            }
        }
    }

    public Visibility ItemsVisibility => (!IsSingleItem && IsExpanded) ? Visibility.Visible : Visibility.Collapsed;
    public string ExpandGlyph => IsExpanded ? "\uE70E" : "\uE70D";
    public string ToggleExpandAutomationName => IsExpanded ? $"Collapse {Title}" : $"Expand {Title}";

    public BatchCardViewModel(
        ShelfBatch batch,
        string title,
        string subtitle,
        string fallbackGlyph,
        IReadOnlyList<ShelfItemViewModel> previewItems,
        Func<IReadOnlyList<ShelfItemViewModel>> createItems,
        bool isExpanded)
    {
        Batch = batch;
        Title = title;
        Subtitle = subtitle;
        FallbackGlyph = fallbackGlyph;
        _previewItems = previewItems;
        _createItems = createItems;
        _isExpanded = isExpanded;
        SubscribeToItems(_previewItems);
        if (isExpanded)
        {
            EnsureItems();
        }
    }

    public string DragBatchAutomationName => $"Drag batch {Title}";
    public string RemoveBatchAutomationName => $"Remove batch {Title}";

    private void EnsureItems()
    {
        if (_items is not null)
        {
            return;
        }

        _items = _createItems();
        for (var index = _previewItems.Count; index < _items.Count; index++)
        {
            _items[index].PropertyChanged += OnItemPropertyChanged;
        }
    }

    private void ReleaseItems()
    {
        if (_items is null)
        {
            return;
        }

        for (var index = _previewItems.Count; index < _items.Count; index++)
        {
            _items[index].PropertyChanged -= OnItemPropertyChanged;
        }

        _items = null;
    }

    private void SubscribeToItems(IReadOnlyList<ShelfItemViewModel> items)
    {
        foreach (var item in items)
        {
            item.PropertyChanged += OnItemPropertyChanged;
        }
    }


    private void OnItemPropertyChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs args)
    {
        if (args.PropertyName == nameof(ShelfItemViewModel.Icon))
        {
            PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(PrimaryIcon)));
            PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(SecondaryIcon)));
            PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(TertiaryIcon)));
        }
        else if (args.PropertyName == nameof(ShelfItemViewModel.IsPinned))
        {
            PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(IsPinned)));
            PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(SingleItemPinAutomationName)));
        }
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
