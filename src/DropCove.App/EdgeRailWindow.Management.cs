using DropCove.Native;
using DropCove.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Windows.ApplicationModel.DataTransfer;
using Windows.System;

namespace DropCove;

public sealed partial class EdgeRailWindow
{
    private readonly ShelfItemProjectionList _managementItems = new();
    private readonly Dictionary<UIElement, ShelfItemViewModel> _managementRealizations = [];
    private RailBatchSummary? _managementRow;
    private Button? _managementAnchor;
    private Grid? _managementContent;
    private ItemsRepeater? _managementList;
    private ToggleButton? _managementBulkPin;
    private TextBlock? _managementSummary;
    private bool _managementKeyboardOpened;

    private void OnManageItemsClicked(object sender, RoutedEventArgs e)
    {
        if (sender is Button { Tag: RailBatchSummary row } button) OpenManagementFlyout(button, row, keyboard: false);
    }

    private void OnManageItemsKeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (e.Key is not (VirtualKey.Enter or VirtualKey.Space)) return;
        if (sender is Button { Tag: RailBatchSummary row } button)
        {
            e.Handled = true;
            OpenManagementFlyout(button, row, keyboard: true);
        }
    }

    private void OpenManagementFlyout(Button anchor, RailBatchSummary row, bool keyboard)
    {
        if (ReferenceEquals(_managementRow, row))
        {
            CloseManagementFlyout(keyboard);
            return;
        }
        if (!row.Presentation.IsMultiItem || !row.ActionsEnabled) return;
        CloseManagementFlyout(false);
        _managementRow = row;
        row.SetManagementOpen(true);
        _managementAnchor = anchor;
        _managementKeyboardOpened = keyboard;
        _managementSummary = new TextBlock { FontSize = 12, VerticalAlignment = VerticalAlignment.Center, TextTrimming = TextTrimming.CharacterEllipsis };
        _managementBulkPin = new ToggleButton
        {
            Width = 24,
            Height = 24,
            IsThreeState = true,
            Tag = row,
            Style = (Style)RailSurface.Resources["RailActionToggleButtonStyle"],
            Content = row.Presentation,
            ContentTemplate = (DataTemplate)Application.Current.Resources["BulkPinIconTemplate"],
        };
        _managementBulkPin.Click += OnRailPinClicked;
        var header = new Grid { ColumnSpacing = 8 };
        header.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        header.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        header.Children.Add(_managementSummary);
        Grid.SetColumn(_managementBulkPin, 1);
        header.Children.Add(_managementBulkPin);
        _managementList = new ItemsRepeater
        {
            Layout = new StackLayout { Spacing = 4 },
            ItemTemplate = (DataTemplate)RailSurface.Resources["RailManagementItemTemplate"],
            VerticalCacheLength = 0,
            HorizontalCacheLength = 0,
            ItemsSource = _managementItems,
        };
        _managementList.ElementPrepared += OnManagementItemPrepared;
        _managementList.ElementClearing += OnManagementItemClearing;
        var scroll = new ScrollViewer
        {
            Content = _managementList,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
            HorizontalScrollMode = ScrollMode.Disabled,
            VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
            MaxHeight = 432,
            IsTabStop = false,
        };
        _managementContent = new Grid { RowSpacing = 4, MinWidth = 304, MaxWidth = 464, MaxHeight = 464 };
        _managementContent.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        _managementContent.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
        _managementContent.Children.Add(header);
        Grid.SetRow(scroll, 1);
        _managementContent.Children.Add(scroll);
        _managementContent.KeyDown += OnManagementKeyDown;
        _openFlyout = new Flyout
        {
            Content = _managementContent,
            FlyoutPresenterStyle = (Style)RailSurface.Resources["RailManagementPresenterStyle"],
            ShouldConstrainToRootBounds = false,
        };
        _openFlyout.Opened += OnManagementFlyoutOpened;
        _openFlyout.Closing += OnManagementFlyoutClosing;
        _openFlyout.Closed += OnManagementFlyoutClosed;
        RefreshManagementItems();
        SetFlyoutOpen(true);
        _openFlyout.ShowAt(anchor, new FlyoutShowOptions
        {
            Placement = _placement?.Edge == ShelfRailEdge.Left ? FlyoutPlacementMode.Right : FlyoutPlacementMode.Left,
            ShowMode = keyboard ? FlyoutShowMode.Standard : FlyoutShowMode.Transient,
        });
    }

    private void OnManagementFlyoutOpened(object? sender, object e)
    {
        if (_managementKeyboardOpened) _managementBulkPin?.Focus(FocusState.Keyboard);
    }

    private void OnManagementFlyoutClosing(FlyoutBase sender, FlyoutBaseClosingEventArgs args)
    {
        if (_managementRow is { } row && (_pendingMutations.Contains(row.Batch.Id) || _sourceDragInProgress)) args.Cancel = true;
    }

    private void OnManagementFlyoutClosed(object? sender, object e)
    {
        if (ReferenceEquals(sender, _openFlyout)) ReleaseManagementContent(restoreFocus: false);
    }

    private void CloseManagementFlyout(bool restoreFocus)
    {
        if (_openFlyout is null) return;
        var flyout = _openFlyout;
        flyout.Opened -= OnManagementFlyoutOpened;
        flyout.Closing -= OnManagementFlyoutClosing;
        flyout.Closed -= OnManagementFlyoutClosed;
        flyout.Hide();
        ReleaseManagementContent(restoreFocus);
    }

    private void ReleaseManagementContent(bool restoreFocus)
    {
        var anchor = _managementAnchor;
        var row = _managementRow;
        if (_managementList is not null)
        {
            _managementList.ElementPrepared -= OnManagementItemPrepared;
            _managementList.ElementClearing -= OnManagementItemClearing;
            _managementList.ItemsSource = null;
        }
        _itemVisuals.Clear();
        _managementRealizations.Clear();
        _managementItems.ClearPresentation();
        if (_managementBulkPin is not null)
        {
            _managementBulkPin.Click -= OnRailPinClicked;
            _managementBulkPin.Tag = null;
            _managementBulkPin.Content = null;
        }
        if (_managementContent is not null) _managementContent.KeyDown -= OnManagementKeyDown;
        if (_openFlyout is not null)
        {
            _openFlyout.Opened -= OnManagementFlyoutOpened;
            _openFlyout.Closing -= OnManagementFlyoutClosing;
            _openFlyout.Closed -= OnManagementFlyoutClosed;
            _openFlyout.Content = null;
        }
        _openFlyout = null;
        row?.SetManagementOpen(false);
        _managementRow = null;
        _managementList = null;
        _managementBulkPin = null;
        _managementSummary = null;
        _managementContent = null;
        _managementAnchor = null;
        _managementKeyboardOpened = false;
        SetFlyoutOpen(false);
        if (WindowInterop.IsPointerOverWindow(_windowHandle)) OnRailPointerEntered();
        if (restoreFocus && anchor is { IsLoaded: true })
        {
            if (anchor.Visibility == Visibility.Visible) anchor.Focus(FocusState.Keyboard);
            else if (anchor.Parent is DependencyObject parent && FocusManager.FindFirstFocusableElement(parent) is Control fallback)
                fallback.Focus(FocusState.Keyboard);
        }
    }

    private void RefreshManagementItems()
    {
        if (_managementRow is not { } row) return;
        if (row.Batch.Items.Count == 0)
        {
            CloseManagementFlyout(_managementKeyboardOpened);
            return;
        }
        _managementItems.Update(row.Batch.Items);
        _managementSummary!.Text = row.Presentation.BulkPinSummary;
        _managementBulkPin!.IsChecked = row.Presentation.BulkPinState;
        Microsoft.UI.Xaml.Automation.AutomationProperties.SetName(_managementBulkPin, row.Presentation.BulkPinAutomationName);
        ToolTipService.SetToolTip(_managementBulkPin, row.Presentation.BulkPinAutomationName);
        SetManagementBusy(row, !row.ActionsEnabled);
    }

    private void SetManagementBusy(RailBatchSummary row, bool busy)
    {
        if (!ReferenceEquals(row, _managementRow)) return;
        if (_managementBulkPin is not null) _managementBulkPin.IsEnabled = !busy;
        foreach (var item in _managementItems.MaterializedItems) item.SetBusy(busy);
    }

    private void OnManagementItemPrepared(ItemsRepeater sender, ItemsRepeaterElementPreparedEventArgs args)
    {
        if (args.Element is not FrameworkElement { Tag: ShelfItemViewModel item }) return;
        item.SetBusy(_managementRow?.ActionsEnabled == false);
        _managementRealizations[args.Element] = item;
        _itemVisuals.Realize(item, item.Item);
    }

    private void OnManagementItemClearing(ItemsRepeater sender, ItemsRepeaterElementClearingEventArgs args)
    {
        if (!_managementRealizations.Remove(args.Element, out var item)) return;
        _itemVisuals.Release(item);
        foreach (var realized in _managementRealizations.Values)
            if (ReferenceEquals(realized, item)) return;
        _managementItems.Release(item);
    }

    private async void OnManagementItemPinClicked(object sender, RoutedEventArgs e)
    {
        if (sender is not ToggleButton { Tag: ShelfItemViewModel item } toggle || _managementRow is not { } row) return;
        toggle.IsChecked = item.IsPinned;
        var keyboardFocused = toggle.FocusState == FocusState.Keyboard;
        if (!BeginMutation(row)) return;
        try
        {
            if (await _manager.SetPinnedAsync(item.Item.Id, !item.IsPinned)) await _refreshAfterMutation();
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

    private async void OnManagementItemRemoveClicked(object sender, RoutedEventArgs e)
    {
        if (sender is not Button { Tag: ShelfItemViewModel item } button || _managementRow is not { } row) return;
        var keyboardFocused = button.FocusState == FocusState.Keyboard;
        var rowIndex = _managementItems.IndexOf(item);
        if (!BeginMutation(row)) return;
        try
        {
            if (await _manager.RemoveItemAsync(item.Item.Id))
            {
                await _refreshAfterMutation();
                if (keyboardFocused && _openFlyout is not null)
                {
                    var targetIndex = Math.Clamp(rowIndex >= 0 ? rowIndex : 0, 0, Math.Max(0, _managementItems.Count - 1));
                    _dispatcherQueue.TryEnqueue(Microsoft.UI.Dispatching.DispatcherQueuePriority.Low, () =>
                    {
                        if (_openFlyout is null || _managementList is null || _managementItems.Count == 0) return;
                        var targetRow = _managementList.GetOrCreateElement(targetIndex);
                        targetRow.UpdateLayout();
                        var next = FocusManager.FindLastFocusableElement(targetRow) ?? FocusManager.FindFirstFocusableElement(targetRow);
                        if (next is Control control) control.Focus(FocusState.Keyboard);
                    });
                }
            }
            else _notify("Couldn’t remove item. Nothing changed.");
        }
        catch (Exception exception) when (exception is Microsoft.Data.Sqlite.SqliteException or IOException or InvalidDataException)
        {
            _notify("Couldn’t remove item. Nothing changed.");
        }
        finally
        {
            EndMutation(row);
            // Do not attempt to refocus a removed button; target focus is set asynchronously above.
        }
    }

    private void OnManagementKeyDown(object sender, KeyRoutedEventArgs args)
    {
        if (args.Key == VirtualKey.Escape)
        {
            args.Handled = true;
            if (_managementRow is { } row && !_pendingMutations.Contains(row.Batch.Id) && !_sourceDragInProgress)
                CloseManagementFlyout(restoreFocus: true);
        }
        else if (args.Key == VirtualKey.Tab)
        {
            var shift = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.Shift);
            MoveManagementFocus((shift & Windows.UI.Core.CoreVirtualKeyStates.Down) != 0);
            args.Handled = true;
        }
    }

    private void MoveManagementFocus(bool backward)
    {
        if (_managementContent is null || _managementList is null || _managementBulkPin is null || _managementItems.Count == 0) return;
        var focused = FocusManager.GetFocusedElement(_managementContent.XamlRoot) as DependencyObject;
        var element = focused;
        while (element is not null && !ReferenceEquals(VisualTreeHelper.GetParent(element), _managementList))
            element = VisualTreeHelper.GetParent(element);
        var rowIndex = element is UIElement rowElement ? _managementList.GetElementIndex(rowElement) : -1;
        var actionCount = 1 + _managementItems.Count * 2;
        var current = ReferenceEquals(focused, _managementBulkPin) ? 0 :
            rowIndex >= 0 ? 1 + rowIndex * 2 + (focused is ToggleButton ? 0 : 1) :
            backward ? 0 : actionCount - 1;
        var next = (current + (backward ? -1 : 1) + actionCount) % actionCount;
        if (next == 0)
        {
            _managementBulkPin.Focus(FocusState.Keyboard);
            return;
        }
        var rowAction = next - 1;
        var targetRow = _managementList.GetOrCreateElement(rowAction / 2);
        targetRow.UpdateLayout();
        var target = rowAction % 2 == 0 ? FocusManager.FindFirstFocusableElement(targetRow) : FocusManager.FindLastFocusableElement(targetRow);
        if (target is Control control)
        {
            control.StartBringIntoView();
            control.Focus(FocusState.Keyboard);
        }
    }

    private async void OnRailItemDragStarting(UIElement sender, DragStartingEventArgs args)
    {
        if (sender is not FrameworkElement { Tag: ShelfItemViewModel item })
        {
            args.Cancel = true;
            return;
        }
        _sourceDragInProgress = true;
        _collapseTimer.Stop();
        try
        {
            var error = await _dragDropService.PrepareItemDragAsync(item.Item, args);
            if (error is not null) _notify($"Could not drag {item.Name}: {error}");
            if (args.Cancel) await _refreshAfterMutation();
        }
        finally
        {
            if (args.Cancel)
            {
                _sourceDragInProgress = false;
                StartCollapseTimer();
            }
        }
    }

    private async void OnRailItemDropCompleted(UIElement sender, DropCompletedEventArgs args)
    {
        if (sender is not FrameworkElement { Tag: ShelfItemViewModel item }) return;
        try
        {
            await _dragDropService.CompleteItemDragAsync(item.Item.Id, args.DropResult);
            await _refreshAfterMutation();
        }
        finally
        {
            _sourceDragInProgress = false;
            StartCollapseTimer();
        }
    }
}
