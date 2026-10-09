using System.Collections.ObjectModel;
using DropCove.Core;
using DropCove.Native;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Windows.ApplicationModel.DataTransfer;

namespace DropCove;

/// <summary>Hosts the bounded collapsed or expanded Edge Rail overlay.</summary>
public sealed partial class EdgeRailWindow : Window
{
    private const int CollapsedWidth = EdgeRailSizingPolicy.HandleWidth;
    private const int CollapsedHeight = EdgeRailSizingPolicy.HandleHeight;
    private const int ExpandedWidth = EdgeRailSizingPolicy.ExpandedWidth;
    // The native region only shapes the input silhouette; a tighter radius keeps it from clipping the XAML corner's antialiased fringe.
    private const int RegionCornerRadius = 5;
    private int _expandedHeight;
    private readonly nint _windowHandle;
    private nint _inputWindowHandle;
    private readonly Action _openShelf;
    private readonly Action<string> _notify;
    private readonly Func<Task> _refreshAfterMutation;
    private readonly Func<DataPackageView, Task<DropAcceptance>> _acceptStorageDrop;
    private readonly DragDropService _dragDropService;
    private readonly DropShelfManager _manager;
    private readonly ShelfVisualLifetime _rowVisuals;
    private readonly ShelfVisualLifetime _itemVisuals;
    private readonly ObservableCollection<RailBatchSummary> _rows = [];
    private readonly HashSet<Guid> _batchIds = [];
    private bool _sourceDragInProgress;
    private readonly Dictionary<SelectorItem, RailBatchSummary> _realizedRows = [];
    private readonly Func<string, string, Task<bool>> _confirm;
    private readonly DispatcherQueueTimer _expandTimer;
    private readonly DispatcherQueueTimer _collapseTimer;
    private readonly WindowMessageHook _windowMessageHook;
    private WindowMessageHook? _inputMessageHook;
    private ForegroundWindowHook? _foregroundWindowHook;
    private readonly DispatcherQueue _dispatcherQueue;
    private readonly DispatcherQueueHandler _completePointerExit;
    private bool _pointerExitPending;
    private ShelfRailPlacement? _placement;
    private Flyout? _openFlyout;
    private bool _isExpanded;
    private bool _pointerInside;
    private bool _dragInside;
    private bool _expandPending;
    private bool _flyoutOpen;
    private bool _clearInProgress;
    private readonly HashSet<Guid> _pendingMutations = [];
    private bool _requestedVisible;
    private bool _showOverFullscreen;
    private (int Width, int Height, int Radius, bool DockLeft) _cornerGeometry;

    /// <summary>Creates an Edge Rail for the current Shelf Batches.</summary>
    /// <param name="manager">The shared shelf lifecycle manager.</param>
    /// <param name="confirm">Confirms partial batch drag-out and Clear Temporary Items.</param>
    /// <param name="acceptStorageDrop">Accepts storage-item drops through the shared Shelf seam.</param>
    /// <param name="refreshAfterMutation">Refreshes or hides the rail after a mutation.</param>
    /// <param name="openShelf">Opens and activates the unified Drop Shelf.</param>
    /// <param name="notify">Reports drag preparation and drop warnings to the resident application.</param>
    /// <param name="visualCoordinator">The shared thumbnail and native-icon pipeline.</param>
    public EdgeRailWindow(
        DropShelfManager manager,
        Func<string, string, Task<bool>> confirm,
        Func<DataPackageView, Task<DropAcceptance>> acceptStorageDrop,
        Func<Task> refreshAfterMutation,
        Action openShelf,
        Action<string> notify,
        ShelfVisualCoordinator<Microsoft.UI.Xaml.Media.ImageSource> visualCoordinator)
    {
        ArgumentNullException.ThrowIfNull(manager);
        ArgumentNullException.ThrowIfNull(confirm);
        ArgumentNullException.ThrowIfNull(acceptStorageDrop);
        ArgumentNullException.ThrowIfNull(refreshAfterMutation);
        ArgumentNullException.ThrowIfNull(openShelf);
        ArgumentNullException.ThrowIfNull(notify);
        ArgumentNullException.ThrowIfNull(visualCoordinator);

        InitializeComponent();
        var managementStyle = (Style)RailSurface.Resources["RailManagementPresenterStyle"];
        managementStyle.Setters.Add(new Setter(FrameworkElement.WidthProperty, (double)ExpandedWidth));
        managementStyle.Setters.Add(new Setter(FrameworkElement.MinWidthProperty, 0d));
        managementStyle.Setters.Add(new Setter(FrameworkElement.MaxWidthProperty, (double)ExpandedWidth));
        _windowHandle = WinRT.Interop.WindowNative.GetWindowHandle(this);
        WindowInterop.MakeBorderless(_windowHandle);
        var initialDpi = WindowInterop.GetWindowDpi(_windowHandle);
        AppWindow.Resize(new Windows.Graphics.SizeInt32(
            WindowInterop.ScaleLogicalPixels(CollapsedWidth, initialDpi),
            WindowInterop.ScaleLogicalPixels(CollapsedHeight, initialDpi)));
        WindowInterop.MakeNoActivate(_windowHandle);
        _windowMessageHook = new WindowMessageHook(_windowHandle,
            (message, wParam, lParam) => HandleWindowMessage(_windowHandle, message, wParam, lParam));
        WindowInterop.EnableTransparentSurface(_windowHandle);
        SystemBackdrop = new TransparentBackdrop();
        _dispatcherQueue = DispatcherQueue.GetForCurrentThread();
        _completePointerExit = CompletePointerExit;
        _expandTimer = _dispatcherQueue.CreateTimer();
        _expandTimer.Interval = TimeSpan.FromMilliseconds(200);
        _expandTimer.Tick += OnExpandTimerTick;
        _collapseTimer = _dispatcherQueue.CreateTimer();
        _collapseTimer.Interval = TimeSpan.FromMilliseconds(300);
        _collapseTimer.Tick += OnCollapseTimerTick;
        _dragDropService = new DragDropService(manager, confirm);
        _manager = manager;
        _rowVisuals = new ShelfVisualLifetime(visualCoordinator);
        _itemVisuals = new ShelfVisualLifetime(visualCoordinator);
        _confirm = confirm;
        _acceptStorageDrop = acceptStorageDrop;
        _refreshAfterMutation = refreshAfterMutation;
        _openShelf = openShelf;
        _notify = notify;
        Closed += (_, _) => DisposeNativeResources();
        UpdateBatches(manager.Batches);
    }


    /// <summary>Gets the native rail window handle.</summary>
    public nint WindowHandle => _windowHandle;

    /// <summary>Gets whether the rail is requested to remain available.</summary>
    public bool IsRequestedVisible => _requestedVisible;


    /// <summary>Rebuilds the compact batch summaries.</summary>
    /// <param name="batches">The current Shelf Batches.</param>
    public void UpdateBatches(IReadOnlyList<ShelfBatch> batches)
    {
        ArgumentNullException.ThrowIfNull(batches);
        _batchIds.Clear();
        foreach (var batch in batches) _batchIds.Add(batch.Id);
        if (_managementRow is not null && !_batchIds.Contains(_managementRow.Batch.Id)) CloseManagementFlyout(false);
        for (var index = _rows.Count - 1; index >= 0; index--)
            if (!_batchIds.Contains(_rows[index].Batch.Id)) _rows.RemoveAt(index);
        for (var index = 0; index < batches.Count; index++)
        {
            var batch = batches[index];
            var existingIndex = index;
            while (existingIndex < _rows.Count && _rows[existingIndex].Batch.Id != batch.Id) existingIndex++;
            if (existingIndex == _rows.Count)
            {
                _rows.Insert(index, new RailBatchSummary(batch, index < batches.Count - 1));
                continue;
            }
            if (existingIndex != index) _rows.Move(existingIndex, index);
            var row = _rows[index];
            var reload = PreviewReferencesChanged(row.Batch, batch);
            if (reload)
                foreach (var realized in _realizedRows.Values)
                    if (ReferenceEquals(realized, row))
                        foreach (var preview in row.Presentation.PreviewItems) _rowVisuals.Release(preview);
            row.Update(batch, index < batches.Count - 1);
            if (reload)
                foreach (var realized in _realizedRows.Values)
                    if (ReferenceEquals(realized, row))
                        for (var preview = 0; preview < row.Presentation.PreviewItems.Count; preview++)
                            _rowVisuals.Realize(row.Presentation.PreviewItems[preview], batch.Items[preview]);
        }
        RefreshManagementItems();
        if (_isExpanded)
        {
            _expandedHeight = EdgeRailSizingPolicy.HeightAfterMutation(_expandedHeight, batches.Count);
            ApplyVisibility();
        }
        if (_requestedVisible)
        {
            ShelfMotion.PlayStateChange(BatchList);
        }
    }

    private static bool PreviewReferencesChanged(ShelfBatch previous, ShelfBatch current)
    {
        var count = Math.Min(3, previous.Items.Count);
        if (count != Math.Min(3, current.Items.Count)) return true;
        for (var index = 0; index < count; index++)
        {
            var first = previous.Items[index];
            var second = current.Items[index];
            if (first.Id != second.Id || first.Availability != second.Availability || first.IsFolder != second.IsFolder ||
                !string.Equals(first.Path, second.Path, StringComparison.OrdinalIgnoreCase)) return true;
        }
        return false;
    }

    /// <summary>Reapplies the current monitor, DPI, fullscreen, and visibility state.</summary>
    public void Reposition() => ApplyVisibility();

    /// <summary>Positions and shows the rail without activating DropCove.</summary>
    /// <param name="placement">The selected monitor and edge.</param>
    /// <param name="showOverFullscreen">Whether fullscreen foreground windows may remain covered.</param>
    public void Show(ShelfRailPlacement placement, bool showOverFullscreen)
    {
        ArgumentNullException.ThrowIfNull(placement);
        _placement = placement;
        OpenShelfButton.HorizontalAlignment = placement.Edge == ShelfRailEdge.Left ? HorizontalAlignment.Right : HorizontalAlignment.Left;
        var cornerRadius = placement.Edge == ShelfRailEdge.Left ? new CornerRadius(0, 8, 8, 0) : new CornerRadius(8, 0, 0, 8);
        RailRoot.CornerRadius = cornerRadius;
        RailSurface.CornerRadius = cornerRadius;
        RailSurface.BorderThickness = placement.Edge == ShelfRailEdge.Left ? new Thickness(0, 1, 1, 1) : new Thickness(1, 1, 0, 1);
        _showOverFullscreen = showOverFullscreen;
        _requestedVisible = true;
        if (showOverFullscreen)
        {
            _foregroundWindowHook?.Dispose();
            _foregroundWindowHook = null;
        }
        else
        {
            _foregroundWindowHook ??= new ForegroundWindowHook(() => _dispatcherQueue.TryEnqueue(ApplyVisibility));
        }
        _isExpanded = false;
        ClearTemporaryButton.Visibility = Visibility.Collapsed;
        OpenShelfButton.Visibility = Visibility.Collapsed;
        BatchList.Visibility = Visibility.Collapsed;
        ReleaseRowVisuals();
        CloseManagementFlyout(false);
        _pointerInside = false;
        _dragInside = false;
        _expandPending = false;
        _expandTimer.Stop();
        _collapseTimer.Stop();
        ApplyVisibility();
    }

    /// <summary>Hides the rail window without destroying it.</summary>
    public void Hide()
    {
        _requestedVisible = false;
        _foregroundWindowHook?.Dispose();
        ReleaseRowVisuals();
        _foregroundWindowHook = null;
        _pointerInside = false;
        _dragInside = false;
        _expandPending = false;
        CloseManagementFlyout(false);
        _expandTimer.Stop();
        _collapseTimer.Stop();
        WindowInterop.Hide(_windowHandle);
    }

    /// <summary>Keeps the rail expanded while a later ticket's flyout is open.</summary>
    /// <param name="isOpen">Whether a rail flyout remains open.</param>
    public void SetFlyoutOpen(bool isOpen)
    {
        _flyoutOpen = isOpen;
        if (isOpen)
        {
            _collapseTimer.Stop();
        }
        else
        {
            StartCollapseTimer();
        }
    }

    private void OnRailPointerEntered(nint sourceWindow = 0)
    {
        _pointerInside = true;
        var inputWindow = sourceWindow == 0 ? _inputWindowHandle : sourceWindow;
        if (inputWindow != 0) WindowInterop.TrackMouseLeave(inputWindow);
        _collapseTimer.Stop();
        BeginExpandDelay();
    }

    private void OnRailPointerExited()
    {
        // A recycled child can exit before its replacement is hit-testable in this layout turn.
        if (_pointerExitPending) return;
        _pointerExitPending = true;
        if (!_dispatcherQueue.TryEnqueue(DispatcherQueuePriority.Low, _completePointerExit)) _pointerExitPending = false;
    }

    private void CompletePointerExit()
    {
        _pointerExitPending = false;
        if (!_requestedVisible || WindowInterop.IsPointerOverWindow(_windowHandle)) return;
        _pointerInside = false;
        if (!_dragInside)
        {
            CancelExpandDelay();
            StartCollapseTimer();
        }
    }

    private void OnRailPointerMoved(object sender, PointerRoutedEventArgs e) => OnRailPointerEntered();

    private void OnRailPointerExited(object sender, PointerRoutedEventArgs e) => OnRailPointerExited();

    private void OnRailDragEnter(object sender, DragEventArgs e)
    {
        _dragInside = true;
        _collapseTimer.Stop();
        BeginExpandDelay();
        UpdateDragFeedback(e);
    }

    private void OnRailDragOver(object sender, DragEventArgs e)
    {
        _dragInside = true;
        UpdateDragFeedback(e);
    }

    private void OnRailDragLeave(object sender, DragEventArgs e) => EndDragInside();

    private void EndDragInside()
    {
        _dragInside = false;
        if (!_pointerInside)
        {
            CancelExpandDelay();
            StartCollapseTimer();
        }
    }

    private void BeginExpandDelay()
    {
        if (!_isExpanded && !_expandPending)
        {
            _expandPending = true;
            _expandTimer.Start();
        }
    }

    private void CancelExpandDelay()
    {
        _expandPending = false;
        _expandTimer.Stop();
    }

    private static void UpdateDragFeedback(DragEventArgs e)
    {
        var acceptsCopy = StorageDropService.CanCopyStorageItems(e);
        e.AcceptedOperation = acceptsCopy ? DataPackageOperation.Copy : DataPackageOperation.None;
        e.DragUIOverride.Caption = acceptsCopy ? "Add to DropCove" : "Filesystem items with Copy support only";
        e.DragUIOverride.IsCaptionVisible = true;
        e.DragUIOverride.IsContentVisible = false;
    }

    private void OnExpandTimerTick(DispatcherQueueTimer sender, object args)
    {
        sender.Stop();
        _expandPending = false;
        if ((_pointerInside || _dragInside) && !_isExpanded)
        {
            SetExpanded(true);
        }
    }

    private void OnCollapseTimerTick(DispatcherQueueTimer sender, object args)
    {
        sender.Stop();
        if (!_pointerInside && !_dragInside && !_sourceDragInProgress && !_flyoutOpen && !_clearInProgress && _pendingMutations.Count == 0 && _isExpanded)
        {
            SetExpanded(false);
        }
    }

    private void StartCollapseTimer()
    {
        if (_requestedVisible && !_pointerInside && !_dragInside && !_sourceDragInProgress && !_flyoutOpen && !_clearInProgress && _pendingMutations.Count == 0 && _isExpanded)
        {
            _collapseTimer.Start();
        }
    }

    private void SetExpanded(bool expanded)
    {
        if (_isExpanded == expanded)
        {
            return;
        }

        _isExpanded = expanded;
        if (expanded) _expandedHeight = EdgeRailSizingPolicy.TargetExpandedHeight(_manager.Batches.Count);
        if (expanded) BatchList.ItemsSource = _rows;
        else ReleaseRowVisuals();
        OpenShelfButton.Visibility = expanded ? Visibility.Visible : Visibility.Collapsed;
        BatchList.Visibility = expanded ? Visibility.Visible : Visibility.Collapsed;
        ClearTemporaryButton.Visibility = expanded ? Visibility.Visible : Visibility.Collapsed;
        UpdateScrollMode();
        ApplyVisibility();
        ShelfMotion.PlayRailTransition(RailRoot, expanded);
    }

    private void UpdateScrollMode() =>
        ScrollViewer.SetVerticalScrollBarVisibility(BatchList, _isExpanded
            ? ScrollBarVisibility.Auto
            : ScrollBarVisibility.Hidden);

    private void ApplyVisibility()
    {
        if (!_requestedVisible || _placement is null)
        {
            WindowInterop.Hide(_windowHandle);
            return;
        }

        try
        {
            var monitorId = WindowInterop.ResolveMonitorId(_placement.MonitorId, _windowHandle);
            if (!_showOverFullscreen && WindowInterop.IsForegroundWindowFullscreen(monitorId))
            {
                _pointerInside = false;
                _expandPending = false;
                _expandTimer.Stop();
                _collapseTimer.Stop();
                WindowInterop.Hide(_windowHandle);
                return;
            }

            WindowInterop.PositionEdgeRail(
                _windowHandle,
                monitorId,
                _placement.Edge == ShelfRailEdge.Left,
                _isExpanded ? ExpandedWidth : CollapsedWidth,
                _isExpanded ? _expandedHeight : CollapsedHeight);
            var size = AppWindow.Size;
            var geometry = (Width: size.Width, Height: size.Height, Radius: WindowInterop.ScaleLogicalPixels(RegionCornerRadius, WindowInterop.GetWindowDpi(_windowHandle)), DockLeft: _placement.Edge == ShelfRailEdge.Left);
            if (_cornerGeometry != geometry)
            {
                WindowInterop.SetEdgeRailCorners(_windowHandle, geometry.Width, geometry.Height, geometry.Radius, geometry.DockLeft);
                _cornerGeometry = geometry;
            }
            WindowInterop.ShowNoActivate(_windowHandle);
            EnsureInputWindowHook();
            _dispatcherQueue.TryEnqueue(EnsureInputWindowHook);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[ApplyVisibility] Handled transient display reconfiguration error: {ex}");
        }
    }

    private void EnsureInputWindowHook()
    {
        if (_inputMessageHook is not null)
        {
            return;
        }

        var inputWindowHandle = WindowInterop.GetFirstChildWindow(_windowHandle);
        if (inputWindowHandle == 0)
        {
            return;
        }

        _inputWindowHandle = inputWindowHandle;
        _inputMessageHook = new WindowMessageHook(inputWindowHandle,
            (message, wParam, lParam) => HandleWindowMessage(inputWindowHandle, message, wParam, lParam));
    }

    private WindowMessageResult HandleWindowMessage(nint sourceWindow, uint message, nuint wParam, nint lParam)
    {
        if (message == WindowInterop.EraseBackgroundMessage)
        {
            return WindowInterop.ClearTransparentBackground(_windowHandle, (nint)wParam);
        }

        if (message is WindowInterop.DisplayChangeMessage or WindowInterop.DpiChangedMessage or WindowInterop.DeviceChangeMessage)
        {
            _dispatcherQueue.TryEnqueue(ApplyVisibility);
        }

        if (message == WindowInterop.MouseMoveMessage)
        {
            OnRailPointerEntered(sourceWindow);
        }
        else if (message == WindowInterop.MouseLeaveMessage)
        {
            OnRailPointerExited();
        }

        return WindowInterop.PreventMouseActivation(message);
    }


    private async void OnRailDrop(object sender, DragEventArgs e)
    {
        if (!StorageDropService.CanCopyStorageItems(e))
        {
            _notify("No supported filesystem paths were found.");
            EndDragInside();
            return;
        }

        var deferral = e.GetDeferral();
        try
        {
            var outcome = await _acceptStorageDrop(e.DataView);
            switch (outcome.Disposition)
            {
                case DropDisposition.Duplicate:
                    return;
                case DropDisposition.Unsupported:
                    _notify("No supported filesystem paths were found.");
                    return;
                case DropDisposition.Added:
                    break;
                default:
                    throw new InvalidOperationException($"Unknown drop disposition: {outcome.Disposition}");
            }

            await _refreshAfterMutation();
            if (outcome.SkippedUnsupportedCount > 0)
            {
                _notify($"Added {outcome.AcceptedCount}; skipped {outcome.SkippedUnsupportedCount} unsupported item(s).");
            }
        }
        catch (Exception exception)
        {
            _notify($"Drop failed: {exception.Message}");
        }
        finally
        {
            EndDragInside();

            deferral.Complete();
        }
    }


    private async void OnRailBatchDragStarting(UIElement sender, DragStartingEventArgs e)
    {
        if (sender is not FrameworkElement { DataContext: RailBatchSummary summary })
        {
            e.Cancel = true;
            return;
        }

        _sourceDragInProgress = true;
        _collapseTimer.Stop();
        try
        {
            var (error, missingRemoved, unavailableRetained) = await _dragDropService.PrepareBatchDragAsync(summary.Batch, e);
            if (missingRemoved > 0 && e.Cancel) await _refreshAfterMutation();
            if (error is not null) _notify(error);
            else if (unavailableRetained > 0) _notify($"Dragging available items ({unavailableRetained} unavailable item(s) retained).");
        }
        finally
        {
            if (e.Cancel)
            {
                _sourceDragInProgress = false;
                StartCollapseTimer();
            }
        }
    }

    private async void OnRailBatchDropCompleted(UIElement sender, DropCompletedEventArgs e)
    {
        if (sender is not FrameworkElement { DataContext: RailBatchSummary summary })
        {
            return;
        }

        try
        {
            CloseManagementFlyout(false);
            await _dragDropService.CompleteBatchDragAsync(summary.Batch.Id, e.DropResult);
            await _refreshAfterMutation();
        }
        finally
        {
            _sourceDragInProgress = false;
            StartCollapseTimer();
        }
    }


    private async void OnRailPinClicked(object sender, RoutedEventArgs e)
    {
        if (sender is not ToggleButton { Tag: RailBatchSummary row } toggle) return;
        var card = row.Presentation;
        toggle.IsChecked = card.IsSingleItem ? card.IsPinned : card.BulkPinState;
        var keyboardFocused = toggle.FocusState == FocusState.Keyboard;
        if (!BeginMutation(row)) return;
        try
        {
            var updated = card.SingleItem is { } item
                ? await _manager.SetPinnedAsync(item.Id, !item.IsPinned)
                : await _manager.SetAllItemsPinnedAsync(card.Batch.Id, card.BulkPinState != true);
            if (updated) await _refreshAfterMutation();
            else _notify("Couldn’t update pinning. Nothing changed.");
        }
        catch (Exception exception) when (exception is Microsoft.Data.Sqlite.SqliteException or IOException or InvalidDataException)
        {
            _notify("Couldn’t update pinning. Nothing changed.");
        }
        finally
        {
            EndMutation(row);
            if (keyboardFocused && toggle.IsLoaded && toggle.IsEnabled) toggle.Focus(FocusState.Keyboard);
        }
    }

    private async void OnRailRemoveClicked(object sender, RoutedEventArgs e)
    {
        if (sender is not Button { Tag: RailBatchSummary row }) return;
        if (!BeginMutation(row)) return;
        var single = row.Presentation.SingleItem;
        try
        {
            var removed = single is not null
                ? await _manager.RemoveItemAsync(single.Id)
                : await _manager.RemoveBatchAsync(row.Batch.Id);
            if (removed) await _refreshAfterMutation();
            else _notify(single is not null ? "Couldn’t remove item. Nothing changed." : "Couldn’t remove batch. Nothing changed.");
        }
        catch (Exception exception) when (exception is Microsoft.Data.Sqlite.SqliteException or IOException or InvalidDataException)
        {
            _notify(single is not null ? "Couldn’t remove item. Nothing changed." : "Couldn’t remove batch. Nothing changed.");
        }
        finally
        {
            EndMutation(row);
        }
    }

    private bool BeginMutation(RailBatchSummary row)
    {
        if (!_pendingMutations.Add(row.Batch.Id)) return false;
        row.SetBusy(true);
        SetManagementBusy(row, true);
        _collapseTimer.Stop();
        return true;
    }

    private void EndMutation(RailBatchSummary row)
    {
        _pendingMutations.Remove(row.Batch.Id);
        row.SetBusy(false);
        SetManagementBusy(row, false);
        StartCollapseTimer();
    }

    private async void OnClearTemporaryClicked(object sender, RoutedEventArgs e)
    {
        if (_manager.Batches.Count == 0 || _clearInProgress) return;
        _clearInProgress = true;
        _collapseTimer.Stop();
        try
        {
            if (!await _confirm(ShelfConfirmationText.ClearTemporaryTitle, ShelfConfirmationText.ClearTemporaryMessage)) return;
            await _manager.ClearTemporaryItemsAsync();
            await _refreshAfterMutation();
        }
        catch (Exception exception) when (exception is Microsoft.Data.Sqlite.SqliteException or IOException or InvalidDataException)
        {
            _notify("Couldn’t clear temporary items. Nothing changed.");
        }
        finally
        {
            _clearInProgress = false;
            StartCollapseTimer();
        }
    }

    private void OnOpenShelfClicked(object sender, RoutedEventArgs e)
    {
        Hide();
        _openShelf();
    }

    private void DisposeNativeResources()
    {
        ReleaseRowVisuals();
        _expandTimer.Stop();
        _collapseTimer.Stop();
        _windowMessageHook.Dispose();
        _inputMessageHook?.Dispose();
        _foregroundWindowHook?.Dispose();
    }

    private void OnBatchContainerContentChanging(ListViewBase sender, ContainerContentChangingEventArgs args)
    {
        if (_realizedRows.Remove(args.ItemContainer, out var previous))
        {
            foreach (var preview in previous.Presentation.PreviewItems) _rowVisuals.Release(preview);
        }
        if (args.InRecycleQueue || !_isExpanded || args.Item is not RailBatchSummary row) return;
        _realizedRows[args.ItemContainer] = row;
        for (var index = 0; index < row.Presentation.PreviewItems.Count; index++)
            _rowVisuals.Realize(row.Presentation.PreviewItems[index], row.Batch.Items[index]);
    }

    private void ReleaseRowVisuals()
    {
        _rowVisuals.Clear();
        _realizedRows.Clear();
        BatchList.ItemsSource = null;
    }
}

