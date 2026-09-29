using DropCove.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage;
using Windows.Storage.FileProperties;

namespace DropCove;

/// <summary>Displays and accepts drops into the bounded unified Drop Shelf.</summary>
public sealed partial class MainPage : Page
{
    private readonly Dictionary<string, IStorageItem> _storageItems = new(StringComparer.OrdinalIgnoreCase);
    private readonly Dictionary<string, ImageSource?> _iconCache = new(StringComparer.OrdinalIgnoreCase);
    private DropShelfManager? _manager;
    private DragDropService? _dragDropService;
    private Action? _hideShelf;
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
        Action hideShelf,
        Action showSettings,
        Action beginWindowMove,
        Action<int, int> resizeWindow,
        Func<string, string, Task<bool>> confirm)
    {
        _manager = manager;
        _dragDropService = new DragDropService(manager, confirm);
        _hideShelf = hideShelf;
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

    private void OnCloseClicked(object sender, RoutedEventArgs e) => _hideShelf?.Invoke();

    private void OnPageKeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (e.Key == Windows.System.VirtualKey.Escape)
        {
            _hideShelf?.Invoke();
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

    private async Task RebuildBatchCardsAsync()
    {
        var manager = _manager!;
        var tasks = manager.Batches.Select(CreateBatchCardAsync).ToArray();
        var cards = await Task.WhenAll(tasks);
        BatchList.ItemsSource = cards;
        EmptyState.Visibility = cards.Length == 0 ? Visibility.Visible : Visibility.Collapsed;
        BatchScroller.Visibility = cards.Length == 0 ? Visibility.Collapsed : Visibility.Visible;

        var overflow = Math.Max(0, cards.Length - 4);
        OverflowLabel.Text = $"+{overflow} batch{(overflow == 1 ? string.Empty : "es")}";
        OverflowLabel.Visibility = overflow == 0 ? Visibility.Collapsed : Visibility.Visible;
    }

    private async Task<BatchCardViewModel> CreateBatchCardAsync(ShelfBatch batch)
    {
        var itemTasks = batch.Items.Select(CreateShelfItemViewModelAsync).ToArray();
        var items = await Task.WhenAll(itemTasks);
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
            items.ElementAtOrDefault(0)?.Icon,
            items.ElementAtOrDefault(1)?.Icon,
            items.ElementAtOrDefault(2)?.Icon,
            items.Length > 1 ? Visibility.Visible : Visibility.Collapsed,
            items.Length > 2 ? Visibility.Visible : Visibility.Collapsed,
            items);
    }

    private async Task<ShelfItemViewModel> CreateShelfItemViewModelAsync(ShelfItem item) => new(
        item,
        item.Name,
        GetItemType(item),
        item.IsFolder ? "\uE8B7" : "\uE8A5",
        await LoadIconAsync(item));

    private async Task<ImageSource?> LoadIconAsync(ShelfItem item)
    {
        if (_iconCache.TryGetValue(item.Path, out var cached))
        {
            return cached;
        }

        try
        {
            IStorageItem storageItem;
            if (!_storageItems.TryGetValue(item.Path, out storageItem!))
            {
                storageItem = item.IsFolder
                    ? await StorageFolder.GetFolderFromPathAsync(item.Path)
                    : await StorageFile.GetFileFromPathAsync(item.Path);
            }

            using var thumbnail = storageItem switch
            {
                StorageFile file => await file.GetThumbnailAsync(
                    ThumbnailMode.ListView,
                    32,
                    ThumbnailOptions.UseCurrentScale),
                StorageFolder folder => await folder.GetThumbnailAsync(
                    ThumbnailMode.ListView,
                    32,
                    ThumbnailOptions.UseCurrentScale),
                _ => null,
            };
            if (thumbnail is null)
            {
                _iconCache[item.Path] = null;
                return null;
            }

            var bitmap = new BitmapImage();
            await bitmap.SetSourceAsync(thumbnail);
            _iconCache[item.Path] = bitmap;
            return bitmap;
        }
        catch
        {
            _iconCache[item.Path] = null;
            return null;
        }
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
    public ImageSource? PrimaryIcon { get; }
    public ImageSource? SecondaryIcon { get; }
    public ImageSource? TertiaryIcon { get; }
    public Visibility SecondaryVisibility { get; }
    public Visibility TertiaryVisibility { get; }
    public IReadOnlyList<ShelfItemViewModel> Items { get; }

    public BatchCardViewModel(
        ShelfBatch batch,
        string title,
        string subtitle,
        string fallbackGlyph,
        ImageSource? primaryIcon,
        ImageSource? secondaryIcon,
        ImageSource? tertiaryIcon,
        Visibility secondaryVisibility,
        Visibility tertiaryVisibility,
        IReadOnlyList<ShelfItemViewModel> items)
    {
        Batch = batch;
        Title = title;
        Subtitle = subtitle;
        FallbackGlyph = fallbackGlyph;
        PrimaryIcon = primaryIcon;
        SecondaryIcon = secondaryIcon;
        TertiaryIcon = tertiaryIcon;
        SecondaryVisibility = secondaryVisibility;
        TertiaryVisibility = tertiaryVisibility;
        Items = items;
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
}

internal sealed record ShelfItemViewModel(
    ShelfItem Item,
    string Name,
    string Type,
    string FallbackGlyph,
    ImageSource? Icon)
{
    public bool IsPinned => Item.IsPinned;
    public string DragAutomationName => $"Drag {Name}";
    public string PinAutomationName => IsPinned ? $"Unpin {Name}" : $"Pin {Name}";
    public string RemoveAutomationName => $"Remove {Name}";
}
