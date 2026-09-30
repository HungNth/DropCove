using DropCove.Core;
using DropCove.Native;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;

namespace DropCove;

/// <summary>Hosts the bounded collapsed or expanded Edge Rail overlay.</summary>
public sealed partial class EdgeRailWindow : Window
{
    private const int CollapsedWidth = 64;
    private const int CollapsedHeight = 112;
    private const int ExpandedWidth = 320;
    private const int ExpandedHeight = 640;
    private readonly nint _windowHandle;
    private nint _inputWindowHandle;
    private readonly Action _openShelf;
    private readonly DispatcherQueueTimer _expandTimer;
    private readonly DispatcherQueueTimer _collapseTimer;
    private readonly WindowMessageHook _windowMessageHook;
    private WindowMessageHook? _inputMessageHook;
    private readonly ForegroundWindowHook _foregroundWindowHook;
    private readonly DispatcherQueue _dispatcherQueue;
    private ShelfRailPlacement? _placement;
    private bool _isExpanded;
    private bool _pointerInside;
    private bool _expandPending;
    private bool _flyoutOpen;
    private bool _requestedVisible;
    private bool _showOverFullscreen;

    /// <summary>Creates an Edge Rail for the current Shelf Batches.</summary>
    /// <param name="batches">The batches represented by the rail.</param>
    /// <param name="openShelf">Opens and activates the unified Drop Shelf.</param>
    public EdgeRailWindow(IReadOnlyList<ShelfBatch> batches, Action openShelf)
    {
        ArgumentNullException.ThrowIfNull(batches);
        ArgumentNullException.ThrowIfNull(openShelf);

        InitializeComponent();
        _windowHandle = WinRT.Interop.WindowNative.GetWindowHandle(this);
        WindowInterop.MakeBorderless(_windowHandle);
        WindowInterop.MakeNoActivate(_windowHandle);
        _windowMessageHook = new WindowMessageHook(_windowHandle, HandleWindowMessage);
        _dispatcherQueue = DispatcherQueue.GetForCurrentThread();
        _expandTimer = _dispatcherQueue.CreateTimer();
        _expandTimer.Interval = TimeSpan.FromMilliseconds(200);
        _expandTimer.Tick += OnExpandTimerTick;
        _collapseTimer = _dispatcherQueue.CreateTimer();
        _collapseTimer.Interval = TimeSpan.FromMilliseconds(300);
        _collapseTimer.Tick += OnCollapseTimerTick;
        _foregroundWindowHook = new ForegroundWindowHook(() => _dispatcherQueue.TryEnqueue(ApplyVisibility));
        _openShelf = openShelf;
        Closed += (_, _) => DisposeNativeResources();
        UpdateBatches(batches);
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
        BatchList.ItemsSource = batches.Select(CreateBatchSummary).ToArray();
    }

    /// <summary>Positions and shows the rail without activating DropCove.</summary>
    /// <param name="placement">The selected monitor and edge.</param>
    /// <param name="showOverFullscreen">Whether fullscreen foreground windows may remain covered.</param>
    public void Show(ShelfRailPlacement placement, bool showOverFullscreen)
    {
        ArgumentNullException.ThrowIfNull(placement);
        _placement = placement;
        _showOverFullscreen = showOverFullscreen;
        _requestedVisible = true;
        _isExpanded = false;
        _pointerInside = false;
        _flyoutOpen = false;
        _expandPending = false;
        _expandTimer.Stop();
        _collapseTimer.Stop();
        ApplyVisibility();
    }

    /// <summary>Hides the rail window without destroying it.</summary>
    public void Hide()
    {
        _requestedVisible = false;
        _pointerInside = false;
        _expandPending = false;
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
        if (!_isExpanded && !_expandPending)
        {
            _expandPending = true;
            _expandTimer.Start();
        }
    }

    private void OnRailPointerExited()
    {
        _pointerInside = false;
        _expandPending = false;
        _expandTimer.Stop();
        StartCollapseTimer();
    }

    private void OnRailPointerMoved(object sender, PointerRoutedEventArgs e) => OnRailPointerEntered();

    private void OnRailPointerExited(object sender, PointerRoutedEventArgs e) => OnRailPointerExited();

    private void OnExpandTimerTick(DispatcherQueueTimer sender, object args)
    {
        sender.Stop();
        _expandPending = false;
        if (_pointerInside && !_isExpanded)
        {
            SetExpanded(true);
        }
    }

    private void OnCollapseTimerTick(DispatcherQueueTimer sender, object args)
    {
        sender.Stop();
        if (!_pointerInside && !_flyoutOpen && _isExpanded)
        {
            SetExpanded(false);
        }
    }

    private void StartCollapseTimer()
    {
        if (!_pointerInside && !_flyoutOpen && _isExpanded)
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
        UpdateScrollMode();
        ApplyVisibility();
    }

    private void UpdateScrollMode() =>
        BatchScroller.VerticalScrollBarVisibility = _isExpanded
            ? ScrollBarVisibility.Auto
            : ScrollBarVisibility.Hidden;

    private void ApplyVisibility()
    {
        if (!_requestedVisible || _placement is null)
        {
            WindowInterop.Hide(_windowHandle);
            return;
        }

        if (!_showOverFullscreen && WindowInterop.IsForegroundWindowFullscreen(_placement.MonitorId))
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
            _placement.MonitorId,
            _placement.Edge == ShelfRailEdge.Left,
            _isExpanded ? ExpandedWidth : CollapsedWidth,
            _isExpanded ? ExpandedHeight : CollapsedHeight);
        WindowInterop.ShowNoActivate(_windowHandle);
        EnsureInputWindowHook();
        _dispatcherQueue.TryEnqueue(EnsureInputWindowHook);
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
        _foregroundWindowHook.Dispose();
    }

    private static RailBatchSummary CreateBatchSummary(ShelfBatch batch)
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
            title,
            subtitle,
            first.IsFolder ? "\uE8B7" : "\uE8A5",
            pinnedGlyph,
            $"Edge Rail batch {title}");
    }
}

internal sealed record RailBatchSummary(
    string Title,
    string Subtitle,
    string Glyph,
    string PinnedGlyph,
    string AutomationName);
