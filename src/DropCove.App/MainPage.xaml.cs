using System.Collections.ObjectModel;
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

/// <summary>Displays and accepts drops in the resizable Drop Shelf.</summary>
public sealed partial class MainPage : Page
{
    private readonly Dictionary<string, IStorageItem> _storageItems = new(StringComparer.OrdinalIgnoreCase);
    private readonly Dictionary<object, VisualRequest> _visualRequests = [];
    private readonly Dictionary<UIElement, ShelfItemViewModel> _popupItemRealizations = [];
    private readonly Dictionary<UIElement, IReadOnlyList<BatchItemPreview>> _batchPreviews = [];
    private readonly HashSet<Guid> _pendingBatchAnimations = [];
    private HashSet<Guid> _knownBatchIds = [];
    private readonly ObservableCollection<BatchCardViewModel> _cards = [];
    private DropShelfManager? _manager;
    private nint _windowHandle;
    private DragDropService? _dragDropService;
    private StorageDropService? _storageDropService;
    private ShelfVisualCoordinator<ImageSource>? _visualCoordinator;
    private Func<Task>? _dismissShelf;
    private Action? _showSettings;
    private Func<string, string, Task<bool>>? _confirm;
    private Action? _shakeDragCanceled;
    private Func<bool, Task>? _shakeDropCompleted;
    private Action? _shakeDragStarted;
    private DispatcherTimer? _statusTimer;
    private Guid? _openPopupBatchId;
    private UIElement? _openPopupAnchor;
    private IReadOnlyList<ShelfItemViewModel>? _popupItemViewModels;
    private bool _focusFirstPopupItemOnOpen;
    internal bool IsItemDragInProgress { get; private set; }
    public MainPage()
    {
        InitializeComponent();
        AddHandler(PointerReleasedEvent, new PointerEventHandler(OnPagePointerReleased), handledEventsToo: true);
        Loaded += (_, _) => Focus(FocusState.Programmatic);
    }

    internal async Task InitializeAsync(
        DropShelfManager manager,
        nint windowHandle,
        Func<Task> dismissShelf,
        Action showSettings,
        Func<string, string, Task<bool>> confirm,
        Action shakeDragStarted,
        Action shakeDragCanceled,
        Func<bool, Task> shakeDropCompleted)
    {
        ArgumentNullException.ThrowIfNull(shakeDragStarted);
        ArgumentNullException.ThrowIfNull(shakeDragCanceled);
        ArgumentNullException.ThrowIfNull(shakeDropCompleted);
        _manager = manager;
        _windowHandle = windowHandle;
        _visualCoordinator = new ShelfVisualCoordinator<ImageSource>(
            new WindowsShelfVisualProvider(TryGetCachedStorageItem),
            IsImageShelfItem);
        _dragDropService = new DragDropService(manager, confirm);
        _storageDropService = new StorageDropService(manager, storageItem => _storageItems[storageItem.Path] = storageItem);
        _dismissShelf = dismissShelf;
        _showSettings = showSettings;
        _confirm = confirm;
        _shakeDragCanceled = shakeDragCanceled;
        _shakeDropCompleted = shakeDropCompleted;
        _shakeDragStarted = shakeDragStarted;
        await RefreshCardsAsync(animate: false);
    }
    internal event Action? ShelfBatchAccepted;

    internal async Task<DropAcceptance> AcceptStorageDropAsync(DataPackageView dataView)
    {
        var outcome = await (_storageDropService ?? throw new InvalidOperationException("MainPage is not initialized.")).AcceptAsync(dataView);
        if (outcome.Batch is not null)
        {
            ShelfBatchAccepted?.Invoke();
        }
        return outcome;
    }

    private void OnContentSizeChanged(object sender, SizeChangedEventArgs args)
    {
        BatchGridLayout.MaximumRowsOrColumns = ShelfSizingPolicy.ColumnCount(args.NewSize.Width);
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
        if (e.Key == Windows.System.VirtualKey.Escape && IsItemDragInProgress) return;
        if (e.Key == Windows.System.VirtualKey.Tab && BatchPopup.IsOpen)
        {
            var shift = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(Windows.System.VirtualKey.Shift);
            MovePopupFocus((shift & Windows.UI.Core.CoreVirtualKeyStates.Down) != 0);
            e.Handled = true;
            return;
        }

        if (e.Key == Windows.System.VirtualKey.Escape)
        {
            if (BatchPopup.IsOpen)
            {
                CloseBatchPopup();
                e.Handled = true;
                return;
            }

            if (_dismissShelf is not null)
            {
                await _dismissShelf();
            }

            e.Handled = true;
        }
    }

    private void MovePopupFocus(bool backward)
    {
        if (_popupItemViewModels is not { Count: > 0 } items) return;
        var actionCount = items.Count * 2;

        var focused = FocusManager.GetFocusedElement(XamlRoot) as DependencyObject;
        var row = focused;
        while (row is not null && !ReferenceEquals(VisualTreeHelper.GetParent(row), BatchPopupItemList))
        {
            row = VisualTreeHelper.GetParent(row);
        }

        var rowIndex = row is UIElement element ? BatchPopupItemList.GetElementIndex(element) : -1;
        var actionIndex = rowIndex < 0 ? (backward ? actionCount - 1 : 0) :
            rowIndex * 2 + (focused is ToggleButton ? 0 : 1) + (backward ? -1 : 1);
        actionIndex = (actionIndex + actionCount) % actionCount;

        // Realize only the next row; virtualized offscreen actions must remain keyboard reachable.
        var targetRow = BatchPopupItemList.GetOrCreateElement(actionIndex / 2);
        targetRow.UpdateLayout();
        var target = actionIndex % 2 == 0 ? FocusManager.FindFirstFocusableElement(targetRow) :
            FocusManager.FindLastFocusableElement(targetRow);
        if (target is Control control)
        {
            control.StartBringIntoView();
            control.Focus(FocusState.Keyboard);
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

        IsItemDragInProgress = true;
        var error = await _dragDropService.PrepareItemDragAsync(item.Item, e);
        if (e.Cancel) IsItemDragInProgress = false;
        if (error is not null)
        {
            ShowDropMessage($"Could not drag {item.Name}: {error}", InfoBarSeverity.Warning);
        }
    }
    private async void OnBatchDragStarting(UIElement sender, DragStartingEventArgs e)
    {
        CloseBatchPopup();

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
            await RefreshAfterMutationAsync();
            ShelfMotion.Reset(sender);
        }
    }

    private async void OnItemDropCompleted(UIElement sender, DropCompletedEventArgs e)
    {
        IsItemDragInProgress = false;
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
                await RefreshAfterMutationAsync();
                ShelfMotion.Reset(sender);
            }
        }
    }



    private async void OnPinClicked(object sender, RoutedEventArgs e)
    {
        if (sender is ToggleButton toggle && _manager is not null)
        {
            var itemVm = toggle.Tag as ShelfItemViewModel ?? toggle.DataContext as ShelfItemViewModel;
            var singleVm = (toggle.Tag as BatchCardViewModel)?.SingleItem
                ?? (toggle.DataContext as BatchCardViewModel)?.SingleItem;

            var itemId = itemVm?.Item.Id ?? singleVm?.Id;
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
                await RefreshAfterMutationAsync();
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
            await RefreshAfterMutationAsync();
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
                await RefreshAfterMutationAsync();
                if (animationTarget is not null)
                {
                    ShelfMotion.Reset(animationTarget);
                }
            }
        }
    }
    private void OnPagePointerReleased(object sender, PointerRoutedEventArgs e)
    {
        if (!BatchPopup.IsOpen || IsItemDragInProgress) return;
        var element = e.OriginalSource as DependencyObject;
        while (element is not null)
        {
            if (ReferenceEquals(element, BatchPopupBorder) || element is FrameworkElement { Name: "ManageItemsButton" }) return;
            element = VisualTreeHelper.GetParent(element);
        }

        CloseBatchPopup();
    }

    private void OnManageItemsClicked(object sender, RoutedEventArgs e)
    {
        if (sender is not FrameworkElement button)
        {
            return;
        }

        var batchVm = button.Tag as BatchCardViewModel ?? button.DataContext as BatchCardViewModel;
        if (batchVm is null || batchVm.IsSingleItem)
        {
            return;
        }

        var focusFirst = false;
        if (FocusManager.GetFocusedElement(XamlRoot) is DependencyObject focusedElement &&
            (ReferenceEquals(focusedElement, button) || VisualTreeHelper.GetParent(focusedElement) != null))
        {
            focusFirst = true;
        }

        ToggleBatchPopup(batchVm, button, focusFirst);
    }

    internal void CloseBatchPopup()
    {
        if (!BatchPopup.IsOpen && _openPopupBatchId is null)
        {
            return;
        }

        var returnTarget = _openPopupAnchor;
        _openPopupBatchId = null;
        _openPopupAnchor = null;
        BatchPopup.IsOpen = false;
        ReleasePopupProjections();

        if (returnTarget is UIElement element && element.XamlRoot is not null)
        {
            if (element is Control control)
            {
                control.Focus(FocusState.Programmatic);
            }
            else
            {
                _ = FocusManager.TryFocusAsync(element, FocusState.Programmatic);
            }
        }
    }

    private void OnBatchPopupClosed(object? sender, object e)
    {
        CloseBatchPopup();
    }

    private void ToggleBatchPopup(BatchCardViewModel batchVm, FrameworkElement anchor, bool fromKeyboard)
    {
        if (BatchPopup.IsOpen && _openPopupBatchId == batchVm.Batch.Id)
        {
            CloseBatchPopup();
            return;
        }

        OpenBatchPopup(batchVm, anchor, fromKeyboard);
    }

    private void OpenBatchPopup(BatchCardViewModel batchVm, FrameworkElement anchor, bool fromKeyboard)
    {
        if (_windowHandle == 0 || XamlRoot is null)
        {
            return;
        }

        ReleasePopupProjections();

        _openPopupBatchId = batchVm.Batch.Id;
        _openPopupAnchor = anchor;
        _focusFirstPopupItemOnOpen = fromKeyboard;

        var fullItems = new ShelfItemViewModel[batchVm.Batch.Items.Count];
        for (var i = 0; i < fullItems.Length; i++)
        {
            fullItems[i] = CreateShelfItemViewModel(batchVm.Batch.Items[i]);
        }
        _popupItemViewModels = fullItems;
        BatchPopupItemList.ItemsSource = _popupItemViewModels;

        UpdatePopupPlacement(anchor, new Windows.Foundation.Size(320, 480));
        BatchPopup.IsOpen = true;
        BatchPopupBorder.Width = double.NaN;
        BatchPopupBorder.Height = double.NaN;
        BatchPopupBorder.MaxWidth = 480;

        DispatcherQueue.TryEnqueue(() =>
        {
            if (!BatchPopup.IsOpen || _openPopupBatchId != batchVm.Batch.Id) return;
            BatchPopupBorder.UpdateLayout();
            if (BatchPopupScroller.VerticalOffset != 0)
            {
                BatchPopupScroller.ChangeView(null, 0, null, true);
                BatchPopupBorder.UpdateLayout();
            }
            UpdatePopupPlacement(anchor);
            if (_focusFirstPopupItemOnOpen && FocusManager.FindFirstFocusableElement(BatchPopupBorder) is { } firstFocusable)
            {
                _ = FocusManager.TryFocusAsync(firstFocusable, FocusState.Programmatic);
            }
        });
    }

    private void UpdatePopupPlacement(FrameworkElement anchor, Windows.Foundation.Size? contentSize = null)
    {
        if (_windowHandle == 0 || XamlRoot is null)
        {
            return;
        }

        FrameworkElement cardElement = anchor;
        var currentParent = VisualTreeHelper.GetParent(anchor);
        while (currentParent is not null)
        {
            if (currentParent is Border b && b.Tag is BatchCardViewModel)
            {
                cardElement = b;
                break;
            }
            currentParent = VisualTreeHelper.GetParent(currentParent);
        }

        var scale = XamlRoot.RasterizationScale;
        if (scale <= 0)
        {
            scale = 1.0;
        }

        var transform = cardElement.TransformToVisual(null);
        var cardTopLeft = transform.TransformPoint(new Windows.Foundation.Point(0, 0));
        var cardSize = cardElement.RenderSize;

        var windowBounds = WindowInterop.GetWindowBounds(_windowHandle);
        var dpi = WindowInterop.GetWindowDpi(_windowHandle);
        var workArea = WindowInterop.GetMonitorWorkAreaForWindow(_windowHandle);

        var cardPhysicalLeft = windowBounds.Left + (int)Math.Round(cardTopLeft.X * scale);
        var cardPhysicalTop = windowBounds.Top + (int)Math.Round(cardTopLeft.Y * scale);
        var cardPhysicalWidth = (int)Math.Round(cardSize.Width * scale);
        var cardPhysicalHeight = (int)Math.Round(cardSize.Height * scale);

        var anchorBounds = new WindowBounds(
            cardPhysicalLeft,
            cardPhysicalTop,
            cardPhysicalLeft + cardPhysicalWidth,
            cardPhysicalTop + cardPhysicalHeight);

        var desired = contentSize ?? BatchPopupBorder.DesiredSize;
        var reqLogicalWidth = Math.Clamp((int)Math.Ceiling(desired.Width), 320, 480);
        var reqLogicalHeight = Math.Clamp((int)Math.Ceiling(desired.Height), 1, 480);

        var popupPhysicalBounds = WindowInterop.PlaceAnchoredPopup(
            anchorBounds,
            workArea,
            reqLogicalWidth,
            reqLogicalHeight,
            dpi);

        var popupLogicalWidth = popupPhysicalBounds.Width / scale;
        var popupLogicalHeight = popupPhysicalBounds.Height / scale;

        BatchPopupBorder.Width = popupLogicalWidth;
        BatchPopupBorder.Height = popupLogicalHeight;
        BatchPopupBorder.MaxHeight = 480;

        var offsetLogicalX = (popupPhysicalBounds.Left - windowBounds.Left) / scale;
        var offsetLogicalY = (popupPhysicalBounds.Top - windowBounds.Top) / scale;

        BatchPopup.HorizontalOffset = offsetLogicalX;
        BatchPopup.VerticalOffset = offsetLogicalY;
    }

    private void ReleasePopupProjections()
    {
        BatchPopupItemList.ItemsSource = null;
        foreach (var item in _popupItemRealizations.Values) ReleaseVisual(item);
        _popupItemRealizations.Clear();
        _popupItemViewModels = null;
    }


    internal Task RefreshAsync() => RefreshCardsAsync(animate: false);
    internal void SetResizeHover(bool isHovered) =>
        ResizeAffordance.Visibility = isHovered ? Visibility.Visible : Visibility.Collapsed;

    internal void ReleasePresentation()
    {
        CloseBatchPopup();
        BatchList.ItemsSource = null;
        CancelVisualRequests();
        _cards.Clear();
        _storageItems.Clear();
        _knownBatchIds.Clear();
        _pendingBatchAnimations.Clear();
        HideStatus();
    }
    internal void PlayShelfAppearance() => ShelfMotion.PlayEntrance(ContentSurface);

    private Task RefreshCardsAsync(bool animate = true)
    {
        HideStatus();
        var batches = _manager!.Batches;
        var currentBatchIds = batches.Select(batch => batch.Id).ToHashSet();
        _pendingBatchAnimations.Clear();
        if (animate)
        {
            _pendingBatchAnimations.UnionWith(currentBatchIds.Except(_knownBatchIds));
        }

        _knownBatchIds = currentBatchIds;
        for (var index = _cards.Count - 1; index >= 0; index--)
        {
            if (!currentBatchIds.Contains(_cards[index].Batch.Id)) _cards.RemoveAt(index);
        }

        for (var index = 0; index < batches.Count; index++)
        {
            var batch = batches[index];
            var existingIndex = index;
            while (existingIndex < _cards.Count && _cards[existingIndex].Batch.Id != batch.Id) existingIndex++;
            if (existingIndex == _cards.Count)
            {
                _cards.Insert(index, CreateBatchCard(batch));
                continue;
            }

            if (existingIndex != index) _cards.Move(existingIndex, index);
            var card = _cards[index];
            if (ReferenceEquals(card.Batch, batch)) continue;
            var element = BatchList.TryGetElement(index);
            if (element is not null && _batchPreviews.Remove(element, out var previousPreviews))
            {
                foreach (var preview in previousPreviews) ReleaseVisual(preview);
            }

            CreateBatchCard(batch, card);
            if (element is FrameworkElement { Tag: BatchCardViewModel realizedCard } && ReferenceEquals(realizedCard, card))
            {
                _batchPreviews[element] = card.PreviewItems;
                for (var previewIndex = 0; previewIndex < card.PreviewItems.Count; previewIndex++)
                    RealizeVisual(card.PreviewItems[previewIndex], batch.Items[previewIndex]);
            }
        }

        if (BatchList.ItemsSource is null) BatchList.ItemsSource = _cards;
        UpdatePresentationVisibility(batches.Count > 0);

        if (animate && batches.Count > 0 && _pendingBatchAnimations.Count == 0)
        {
            ShelfMotion.PlayStateChange(BatchSurface);
        }

        return Task.CompletedTask;
    }

    private void UpdatePresentationVisibility(bool hasBatches)
    {
        BatchSurface.Visibility = hasBatches ? Visibility.Visible : Visibility.Collapsed;
        EmptyState.Visibility = hasBatches ? Visibility.Collapsed : Visibility.Visible;
        ClearTemporaryButton.Visibility = hasBatches ? Visibility.Visible : Visibility.Collapsed;
    }

    private async Task RefreshAfterMutationAsync()
    {
        await RefreshCardsAsync();

        if (_openPopupBatchId.HasValue)
        {
            var currentBatch = _manager?.Batches.FirstOrDefault(b => b.Id == _openPopupBatchId.Value);
            if (currentBatch is null || currentBatch.Items.Count <= 1)
            {
                CloseBatchPopup();
            }
            else
            {
                RefreshOpenPopup(currentBatch);
            }
        }

        if (_manager?.DisplayState == ShelfDisplayState.Hidden && _manager.Batches.Count == 0 && _dismissShelf is not null)
        {
            await _dismissShelf();
        }
    }

    private void RefreshOpenPopup(ShelfBatch batch)
    {
        ReleasePopupProjections();
        var fullItems = new ShelfItemViewModel[batch.Items.Count];
        for (var i = 0; i < fullItems.Length; i++)
        {
            fullItems[i] = CreateShelfItemViewModel(batch.Items[i]);
        }
        _popupItemViewModels = fullItems;
        BatchPopupItemList.ItemsSource = _popupItemViewModels;

        if (_openPopupAnchor is FrameworkElement anchor && anchor.XamlRoot is not null)
        {
            BatchPopupBorder.Width = double.NaN;
            BatchPopupBorder.Height = double.NaN;
            DispatcherQueue.TryEnqueue(() =>
            {
                if (!BatchPopup.IsOpen || _openPopupBatchId != batch.Id || anchor.XamlRoot is null) return;
                BatchPopupBorder.UpdateLayout();
                UpdatePopupPlacement(anchor);
            });
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
        for (var index = 0; index < previewItems.Count; index++)
        {
            RealizeVisual(previewItems[index], batch.Batch.Items[index]);
        }
        if (_pendingBatchAnimations.Remove(batch.Batch.Id))
        {
            ShelfMotion.PlayInsertion(args.Element);
        }
    }

    private void OnBatchElementClearing(ItemsRepeater sender, ItemsRepeaterElementClearingEventArgs args)
    {
        if (args.Element is FrameworkElement { Tag: BatchCardViewModel batch })
        {
            if (_openPopupBatchId == batch.Batch.Id)
            {
                CloseBatchPopup();
            }
        }

        if (_batchPreviews.Remove(args.Element, out var previewItems))
        {
            foreach (var item in previewItems)
            {
                ReleaseVisual(item);
            }
        }
    }

    private void OnPopupItemElementPrepared(ItemsRepeater sender, ItemsRepeaterElementPreparedEventArgs args)
    {
        if (args.Element is not FrameworkElement { Tag: ShelfItemViewModel item })
        {
            return;
        }

        _popupItemRealizations[args.Element] = item;
        RealizeVisual(item, item.Item);
    }

    private void OnPopupItemElementClearing(ItemsRepeater sender, ItemsRepeaterElementClearingEventArgs args)
    {
        if (_popupItemRealizations.Remove(args.Element, out var item))
        {
            ReleaseVisual(item);
        }
    }

    private void RealizeVisual(object projection, ShelfItem item)
    {
        if (_visualCoordinator is null)
        {
            return;
        }

        if (_visualRequests.TryGetValue(projection, out var existing))
        {
            existing.RealizationCount++;
            return;
        }

        var request = new VisualRequest();
        _visualRequests[projection] = request;
        _ = LoadVisualAsync(projection, item, request);
    }

    private void ReleaseVisual(object item)
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
        SetVisual(item, null, usedThumbnail: false);
        request.Cancellation.Cancel();
        request.Cancellation.Dispose();
    }

    private async Task LoadVisualAsync(object projection, ShelfItem item, VisualRequest request)
    {
        try
        {
            var result = await _visualCoordinator!.LoadAsync(item, request.Cancellation.Token);
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

    private void CancelVisualRequests()
    {
        foreach (var (item, request) in _visualRequests)
        {
            request.IsActive = false;
            SetVisual(item, null, usedThumbnail: false);
            request.Cancellation.Cancel();
            request.Cancellation.Dispose();
        }

        _visualRequests.Clear();
        _popupItemRealizations.Clear();
        _batchPreviews.Clear();
    }

    private BatchCardViewModel CreateBatchCard(ShelfBatch batch, BatchCardViewModel? existing = null)
    {
        var previewItems = new BatchItemPreview[Math.Min(3, batch.Items.Count)];
        for (var index = 0; index < previewItems.Length; index++)
        {
            previewItems[index] = new BatchItemPreview();
        }

        var first = batch.Items[0];
        var pinnedCount = batch.Items.Count(item => item.IsPinned);
        var subtitle = batch.Items.Count == 1
            ? $"{(first.IsPinned ? "Pinned • " : string.Empty)}{GetItemType(first)}"
            : $"{string.Join(", ", batch.Items.Take(2).Select(item => item.Name))}{(pinnedCount > 0 ? $" • {pinnedCount} pinned" : string.Empty)}";

        if (existing is not null)
        {
            existing.Update(batch, batch.Items.Count == 1 ? first.Name : $"{batch.Items.Count} items", subtitle, first.IsFolder ? "\uE8B7" : "\uE8A5", previewItems);
            return existing;
        }

        return new BatchCardViewModel(
            batch,
            batch.Items.Count == 1 ? first.Name : $"{batch.Items.Count} items",
            subtitle,
            first.IsFolder ? "\uE8B7" : "\uE8A5",
            previewItems);
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

    private static void SetVisual(object projection, ImageSource? icon, bool usedThumbnail)
    {
        if (projection is BatchItemPreview preview) preview.SetIcon(icon);
        else ((ShelfItemViewModel)projection).SetIcon(icon, usedThumbnail);
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

internal sealed class BatchCardViewModel : System.ComponentModel.INotifyPropertyChanged
{
    private IReadOnlyList<BatchItemPreview> _previewItems;

    public event System.ComponentModel.PropertyChangedEventHandler? PropertyChanged;

    public ShelfBatch Batch { get; private set; }
    public string Title { get; private set; }
    public string Subtitle { get; private set; }
    public string FallbackGlyph { get; private set; }
    public IReadOnlyList<BatchItemPreview> PreviewItems => _previewItems;
    public ImageSource? PrimaryIcon => _previewItems.ElementAtOrDefault(0)?.Icon;
    public ImageSource? SecondaryIcon => _previewItems.ElementAtOrDefault(1)?.Icon;
    public ImageSource? TertiaryIcon => _previewItems.ElementAtOrDefault(2)?.Icon;
    public bool IsSingleItem => Batch.Items.Count == 1;
    public Visibility SingleItemVisibility => IsSingleItem ? Visibility.Visible : Visibility.Collapsed;
    public Visibility MultiItemVisibility => IsSingleItem ? Visibility.Collapsed : Visibility.Visible;
    public ShelfItem? SingleItem => IsSingleItem ? Batch.Items[0] : null;
    public bool IsPinned => SingleItem?.IsPinned ?? false;
    public string SingleItemPinAutomationName => IsPinned ? $"Unpin {Title}" : $"Pin {Title}";
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
        IReadOnlyList<BatchItemPreview> previewItems)
    {
        Batch = batch;
        Title = title;
        Subtitle = subtitle;
        FallbackGlyph = fallbackGlyph;
        _previewItems = previewItems;
        SubscribeToItems(_previewItems);
    }

    public void Update(ShelfBatch batch, string title, string subtitle, string fallbackGlyph, IReadOnlyList<BatchItemPreview> previewItems)
    {
        foreach (var item in _previewItems) item.PropertyChanged -= OnItemPropertyChanged;
        Batch = batch;
        Title = title;
        Subtitle = subtitle;
        FallbackGlyph = fallbackGlyph;
        _previewItems = previewItems;
        SubscribeToItems(_previewItems);
        PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(string.Empty));
    }

    private void SubscribeToItems(IReadOnlyList<BatchItemPreview> items)
    {
        foreach (var item in items)
        {
            item.PropertyChanged += OnItemPropertyChanged;
        }
    }

    private void OnItemPropertyChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs args)
    {
        if (args.PropertyName == nameof(BatchItemPreview.Icon))
        {
            PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(PrimaryIcon)));
            PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(SecondaryIcon)));
            PropertyChanged?.Invoke(this, new System.ComponentModel.PropertyChangedEventArgs(nameof(TertiaryIcon)));
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
