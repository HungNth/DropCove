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
    private int _expandedHeight;
    private readonly nint _windowHandle;
    private nint _inputWindowHandle;
    private readonly Action _openShelf;
    private readonly Action<string> _notify;
    private readonly Func<Task> _refreshAfterMutation;
    private readonly Func<DataPackageView, Task<DropAcceptance>> _acceptStorageDrop;
    private readonly DragDropService _dragDropService;
    private readonly DropShelfManager _manager;
    private readonly Func<string, string, Task<bool>> _confirm;
    private readonly DispatcherQueueTimer _expandTimer;
    private readonly DispatcherQueueTimer _collapseTimer;
    private readonly WindowMessageHook _windowMessageHook;
    private WindowMessageHook? _inputMessageHook;
    private ForegroundWindowHook? _foregroundWindowHook;
    private readonly DispatcherQueue _dispatcherQueue;
    private ShelfRailPlacement? _placement;
    private Flyout? _openFlyout;
    private bool _isExpanded;
    private bool _pointerInside;
    private bool _dragInside;
    private bool _expandPending;
    private bool _flyoutOpen;
    private bool _clearInProgress;
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
    public EdgeRailWindow(
        DropShelfManager manager,
        Func<string, string, Task<bool>> confirm,
        Func<DataPackageView, Task<DropAcceptance>> acceptStorageDrop,
        Func<Task> refreshAfterMutation,
        Action openShelf,
        Action<string> notify)
    {
        ArgumentNullException.ThrowIfNull(manager);
        ArgumentNullException.ThrowIfNull(confirm);
        ArgumentNullException.ThrowIfNull(acceptStorageDrop);
        ArgumentNullException.ThrowIfNull(refreshAfterMutation);
        ArgumentNullException.ThrowIfNull(openShelf);
        ArgumentNullException.ThrowIfNull(notify);

        InitializeComponent();
        _windowHandle = WinRT.Interop.WindowNative.GetWindowHandle(this);
        WindowInterop.MakeBorderless(_windowHandle);
        var initialDpi = WindowInterop.GetWindowDpi(_windowHandle);
        AppWindow.Resize(new Windows.Graphics.SizeInt32(
            WindowInterop.ScaleLogicalPixels(CollapsedWidth, initialDpi),
            WindowInterop.ScaleLogicalPixels(CollapsedHeight, initialDpi)));
        WindowInterop.MakeNoActivate(_windowHandle);
        _windowMessageHook = new WindowMessageHook(_windowHandle, HandleWindowMessage);
        _dispatcherQueue = DispatcherQueue.GetForCurrentThread();
        _expandTimer = _dispatcherQueue.CreateTimer();
        _expandTimer.Interval = TimeSpan.FromMilliseconds(200);
        _expandTimer.Tick += OnExpandTimerTick;
        _collapseTimer = _dispatcherQueue.CreateTimer();
        _collapseTimer.Interval = TimeSpan.FromMilliseconds(300);
        _collapseTimer.Tick += OnCollapseTimerTick;
        _dragDropService = new DragDropService(manager, confirm);
        _manager = manager;
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

    private IReadOnlyList<RailBatchSummary>? _lastBatches;

    /// <summary>Rebuilds the compact batch summaries.</summary>
    /// <param name="batches">The current Shelf Batches.</param>
    public void UpdateBatches(IReadOnlyList<ShelfBatch> batches)
    {
        ArgumentNullException.ThrowIfNull(batches);
        if (_lastBatches is null || _lastBatches.Count != batches.Count || !AreBatchesEqual(_lastBatches, batches))
        {
            var summaries = new RailBatchSummary[batches.Count];
            for (var index = 0; index < summaries.Length; index++)
                summaries[index] = CreateBatchSummary(batches[index], index < summaries.Length - 1);
            _lastBatches = summaries;
            BatchList.ItemsSource = summaries;
        }
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

    private static bool AreBatchesEqual(IReadOnlyList<RailBatchSummary> a, IReadOnlyList<ShelfBatch> b)
    {
        for (var i = 0; i < a.Count; i++)
        {
            if (!ReferenceEquals(a[i].Batch, b[i])) return false;
        }
        return true;
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
        RailRoot.CornerRadius = placement.Edge == ShelfRailEdge.Left ? new CornerRadius(0, 16, 16, 0) : new CornerRadius(16, 0, 0, 16);
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
        _openFlyout?.Hide();
        _openFlyout = null;
        _flyoutOpen = false;
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
        _foregroundWindowHook = null;
        _pointerInside = false;
        _dragInside = false;
        _expandPending = false;
        _openFlyout?.Hide();
        _openFlyout = null;
        _flyoutOpen = false;
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

    private void OnRailPointerEntered()
    {
        _pointerInside = true;
        _collapseTimer.Stop();
        BeginExpandDelay();
    }

    private void OnRailPointerExited()
    {
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
        if (!_pointerInside && !_dragInside && !_flyoutOpen && !_clearInProgress && _isExpanded)
        {
            SetExpanded(false);
        }
    }

    private void StartCollapseTimer()
    {
        if (!_pointerInside && !_dragInside && !_flyoutOpen && !_clearInProgress && _isExpanded)
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
            var geometry = (Width: size.Width, Height: size.Height, Radius: WindowInterop.ScaleLogicalPixels(16, WindowInterop.GetWindowDpi(_windowHandle)), DockLeft: _placement.Edge == ShelfRailEdge.Left);
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
        _inputMessageHook = new WindowMessageHook(inputWindowHandle, HandleWindowMessage);
    }

    private WindowMessageResult HandleWindowMessage(uint message, nuint wParam, nint lParam)
    {
        if (message is WindowInterop.DisplayChangeMessage or WindowInterop.DpiChangedMessage or WindowInterop.DeviceChangeMessage)
        {
            _dispatcherQueue.TryEnqueue(ApplyVisibility);
        }

        if (message == WindowInterop.MouseMoveMessage)
        {
            WindowInterop.TrackMouseLeave(_inputWindowHandle == 0 ? _windowHandle : _inputWindowHandle);
            OnRailPointerEntered();
        }
        else if (message == WindowInterop.MouseLeaveMessage)
        {
            OnRailPointerExited();
        }

        return WindowInterop.PreventMouseActivation(message);
    }

    private void OnBatchFlyoutButtonPointerPressed(object sender, PointerRoutedEventArgs e)
    {
        if (sender is Button { Flyout: Flyout flyout })
        {
            flyout.Placement = _placement?.Edge == ShelfRailEdge.Left
                ? FlyoutPlacementMode.Right
                : FlyoutPlacementMode.Left;
        }
    }

    private void OnBatchFlyoutOpened(object sender, object e)
    {
        if (sender is Flyout flyout)
        {
            _openFlyout = flyout;
        }

        SetFlyoutOpen(true);
    }

    private void OnBatchFlyoutClosed(object sender, object e)
    {
        if (sender is Flyout flyout && ReferenceEquals(_openFlyout, flyout))
        {
            _openFlyout = null;
        }

        SetFlyoutOpen(false);
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
            if (outcome.Batch is null)
            {
                _notify("No supported filesystem paths were found.");
                return;
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

        var (error, missingRemoved, unavailableRetained) =
            await _dragDropService.PrepareBatchDragAsync(summary.Batch, e);
        if (missingRemoved > 0 && e.Cancel)
        {
            await _refreshAfterMutation();
        }

        if (error is not null)
        {
            _notify(error);
        }
        else if (unavailableRetained > 0)
        {
            _notify($"Dragging available items ({unavailableRetained} unavailable item(s) retained).");
        }
    }

    private async void OnRailBatchDropCompleted(UIElement sender, DropCompletedEventArgs e)
    {
        if (sender is not FrameworkElement { DataContext: RailBatchSummary summary })
        {
            return;
        }

        _openFlyout?.Hide();
        await _dragDropService.CompleteBatchDragAsync(summary.Batch.Id, e.DropResult);
        await _refreshAfterMutation();
    }

    private async void OnRailItemDragStarting(UIElement sender, DragStartingEventArgs e)
    {
        if (sender is not FrameworkElement { DataContext: RailItemSummary summary })
        {
            e.Cancel = true;
            return;
        }
        var error = await _dragDropService.PrepareItemDragAsync(summary.Item, e);
        if (error is not null)
        {
            _notify($"Could not drag {summary.Name}: {error}");
        }
    }

    private async void OnRailItemDropCompleted(UIElement sender, DropCompletedEventArgs e)
    {
        if (sender is not FrameworkElement { DataContext: RailItemSummary summary })
        {
            return;
        }

        _openFlyout?.Hide();
        await _dragDropService.CompleteItemDragAsync(summary.Item.Id, e.DropResult);
        await _refreshAfterMutation();
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
        _expandTimer.Stop();
        _collapseTimer.Stop();
        _windowMessageHook.Dispose();
        _inputMessageHook?.Dispose();
        _foregroundWindowHook?.Dispose();
    }

    private static RailBatchSummary CreateBatchSummary(ShelfBatch batch, bool hasFollowingBatch)
    {
        var first = batch.Items[0];
        var title = batch.Items.Count == 1
            ? first.Name
            : $"{batch.Items.Count} items";
        var subtitle = batch.Items.Count == 1
            ? first.IsFolder ? "Folder" : "File"
            : first.Name;
        var pinnedGlyph = batch.Items.Any(item => item.IsPinned) ? "\uE718" : string.Empty;
        return new RailBatchSummary(
            batch,
            title,
            subtitle,
            first.IsFolder ? "\uE8B7" : "\uE8A5",
            pinnedGlyph,
            $"Edge Rail batch {title}",
            hasFollowingBatch);
    }
}

internal sealed record RailBatchSummary(
    ShelfBatch Batch,
    string Title,
    string Subtitle,
    string Glyph,
    string PinnedGlyph,
    string AutomationName,
    bool HasFollowingBatch)
{
    private IReadOnlyList<RailItemSummary>? _items;

    public IReadOnlyList<RailItemSummary> Items => _items ??= Batch.Items.Select(CreateItemSummary).ToArray();
    public Thickness RowMargin => new(0, 0, 0, HasFollowingBatch ? EdgeRailSizingPolicy.Gap : 0);
    public Visibility FlyoutVisibility => Batch.Items.Count > 1 ? Visibility.Visible : Visibility.Collapsed;
    public string FlyoutAutomationName => $"Show items in {Title}";

    private static RailItemSummary CreateItemSummary(ShelfItem item) => new(
        item,
        item.Name,
        item.IsFolder ? "Folder" : "File",
        item.IsFolder ? "\uE8B7" : "\uE8A5",
        item.IsPinned ? "\uE718" : string.Empty,
        $"Drag {item.Name}");
    }

internal sealed record RailItemSummary(
    ShelfItem Item,
    string Name,
    string Type,
    string Glyph,
    string PinnedGlyph,
    string DragAutomationName);
