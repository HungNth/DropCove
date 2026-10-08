using System.Collections.ObjectModel;

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
using static DropCove.ShelfPresentation;

/// <summary>Displays and accepts drops in the resizable Drop Shelf.</summary>
public sealed partial class MainPage : Page
{
    private readonly Dictionary<string, IStorageItem> _storageItems = new(StringComparer.OrdinalIgnoreCase);
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
    private ShelfVisualLifetime? _visualLifetime;
    internal ShelfVisualCoordinator<ImageSource> VisualCoordinator =>
        _visualCoordinator ?? throw new InvalidOperationException("MainPage is not initialized.");
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
    private bool _focusPopupHeaderOnOpen;
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
        var storageItems = _storageItems;
        _visualCoordinator = new ShelfVisualCoordinator<ImageSource>(
            new WindowsShelfVisualProvider(item => storageItems.TryGetValue(item.Path, out var cached) ? cached : null),
            IsImageShelfItem);
        _visualLifetime = new ShelfVisualLifetime(_visualCoordinator);
        _dragDropService = new DragDropService(manager, confirm);
        _storageDropService = new StorageDropService(manager, storageItem => storageItems[storageItem.Path] = storageItem);
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
        var actionCount = 1 + items.Count * 2;

        var focused = FocusManager.GetFocusedElement(XamlRoot) as DependencyObject;
        var row = focused;
        while (row is not null && !ReferenceEquals(VisualTreeHelper.GetParent(row), BatchPopupItemList))
        {
            row = VisualTreeHelper.GetParent(row);
        }

        var rowIndex = row is UIElement element ? BatchPopupItemList.GetElementIndex(element) : -1;
        var currentIndex = ReferenceEquals(focused, BatchPopupBulkPin) ? 0 :
            rowIndex >= 0 ? 1 + rowIndex * 2 + (focused is ToggleButton ? 0 : 1) :
            backward ? 0 : actionCount - 1;
        var actionIndex = (currentIndex + (backward ? -1 : 1) + actionCount) % actionCount;
        if (actionIndex == 0)
        {
            BatchPopupBulkPin.Focus(FocusState.Keyboard);
            return;
        }

        // Realize only the next row; virtualized offscreen actions must remain keyboard reachable.
        var rowActionIndex = actionIndex - 1;
        var targetRow = BatchPopupItemList.GetOrCreateElement(rowActionIndex / 2);
        targetRow.UpdateLayout();
        var target = rowActionIndex % 2 == 0 ? FocusManager.FindFirstFocusableElement(targetRow) :
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

    private async void OnBulkPinClicked(object sender, RoutedEventArgs e)
    {
        if (sender is not ToggleButton { Tag: BatchCardViewModel card } toggle || _manager is null) return;

        var target = card.BulkPinState != true;
        toggle.IsChecked = card.BulkPinState;
        var fromPopup = ReferenceEquals(toggle, BatchPopupBulkPin);
        if (!fromPopup) CloseBatchPopup();
        bool exists;
        try
        {
            exists = await _manager.SetAllItemsPinnedAsync(card.Batch.Id, target);
        }
        catch (Exception exception) when (exception is Microsoft.Data.Sqlite.SqliteException or IOException or InvalidDataException)
        {
            if (!ReferenceEquals(toggle.Tag, card)) return;
            ShowDropMessage("Couldn’t update pinning. Nothing changed.", InfoBarSeverity.Error);
            toggle.IsChecked = card.BulkPinState;
            if (fromPopup && BatchPopup.IsOpen) toggle.Focus(FocusState.Programmatic);
            return;
        }

        if (exists) await RefreshAfterMutationAsync();
        if (!ReferenceEquals(toggle.Tag, card)) return;
        toggle.IsChecked = card.BulkPinState;
        if (fromPopup && BatchPopup.IsOpen) toggle.Focus(FocusState.Programmatic);
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
            ShelfConfirmationText.ClearTemporaryTitle,
            ShelfConfirmationText.ClearTemporaryMessage);

        if (confirmed)
        {
            try
            {
                var count = await _manager.ClearTemporaryItemsAsync();
                await RefreshAfterMutationAsync();
                ShowDropMessage($"Cleared {count} temporary items.");
            }
            catch (Exception exception) when (exception is Microsoft.Data.Sqlite.SqliteException or IOException or InvalidDataException)
            {
                ShowDropMessage("Couldn’t clear temporary items. Nothing changed.", InfoBarSeverity.Error);
            }
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

        ToggleBatchPopup(batchVm, button, fromKeyboard: false);
    }

    private void OnManageItemsKeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (e.Key is not (Windows.System.VirtualKey.Enter or Windows.System.VirtualKey.Space) ||
            sender is not FrameworkElement { Tag: BatchCardViewModel card } button || card.IsSingleItem) return;

        ToggleBatchPopup(card, button, fromKeyboard: true);
        e.Handled = true;
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
        _focusPopupHeaderOnOpen = fromKeyboard;
        UpdatePopupHeader(batchVm);

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
            if (_focusPopupHeaderOnOpen) BatchPopupBulkPin.Focus(FocusState.Keyboard);
        });
    }

    private void UpdatePopupHeader(BatchCardViewModel card)
    {
        BatchPopupPinCount.Text = card.BulkPinSummary;
        BatchPopupBulkPin.Tag = card;
        BatchPopupBulkPin.Content = card;
        BatchPopupBulkPin.IsChecked = card.BulkPinState;
        Microsoft.UI.Xaml.Automation.AutomationProperties.SetName(BatchPopupBulkPin, card.BulkPinAutomationName);
        ToolTipService.SetToolTip(BatchPopupBulkPin, card.BulkPinAutomationName);
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
        foreach (var item in _popupItemRealizations.Values) _visualLifetime?.Release(item);
        _popupItemRealizations.Clear();
        _popupItemViewModels = null;
        BatchPopupBulkPin.Tag = null;
        BatchPopupBulkPin.Content = null;
    }


    internal Task RefreshAsync() => RefreshCardsAsync(animate: false);
    internal void SetResizeHover(bool isHovered) =>
        ResizeAffordance.Visibility = isHovered ? Visibility.Visible : Visibility.Collapsed;

    internal void ReleasePresentation()
    {
        CloseBatchPopup();
        BatchList.ItemsSource = null;
        _visualLifetime?.Clear();
        _popupItemRealizations.Clear();
        _batchPreviews.Clear();
        _storageItems.Clear();
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
                foreach (var preview in previousPreviews) _visualLifetime?.Release(preview);
            }

            CreateBatchCard(batch, card);
            if (element is FrameworkElement { Tag: BatchCardViewModel realizedCard } && ReferenceEquals(realizedCard, card))
            {
                _batchPreviews[element] = card.PreviewItems;
                for (var previewIndex = 0; previewIndex < card.PreviewItems.Count; previewIndex++)
                    _visualLifetime?.Realize(card.PreviewItems[previewIndex], batch.Items[previewIndex]);
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
        foreach (var card in _cards)
        {
            if (card.Batch.Id != batch.Id) continue;
            UpdatePopupHeader(card);
            break;
        }

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
            _visualLifetime?.Realize(previewItems[index], batch.Batch.Items[index]);
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
                _visualLifetime?.Release(item);
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
        _visualLifetime?.Realize(item, item.Item);
    }

    private void OnPopupItemElementClearing(ItemsRepeater sender, ItemsRepeaterElementClearingEventArgs args)
    {
        if (_popupItemRealizations.Remove(args.Element, out var item))
        {
            _visualLifetime?.Release(item);
        }
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



}

