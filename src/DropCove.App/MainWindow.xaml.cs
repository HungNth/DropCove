using System.ComponentModel;
using DropCove.Core;
using DropCove.Native;
using DropCove.Services;
using Microsoft.UI.Xaml;
using Microsoft.UI.Dispatching;

namespace DropCove;

/// <summary>Hosts the resident Compact and Expanded Drop Shelf presentations.</summary>
public sealed partial class MainWindow : Window
{
    private const uint WindowCloseMessage = 0x0010;
    private const uint InstallerExitMessage = 0x8002;
    private DropShelfManager? _manager;
    private readonly SettingsStore _settingsStore = new();
    private readonly nint _windowHandle;
    private readonly WindowMessageHook _messageHook;
    private readonly GlobalHotKey _globalHotKey;
    private readonly TrayIcon _trayIcon;
    private readonly MainPage _page;
    private AppSettings _settings = AppSettings.Default;
    private SettingsWindow? _settingsWindow;
    private EdgeRailWindow? _railWindow;
    private bool _disposed;
    private bool _surfaceReflowQueued;

    private readonly DispatcherQueue _dispatcherQueue;
    private readonly ShakeInputQueue _shakeInputQueue;
    private readonly ShakeSessionCoordinator _shakeSession = new();
    private ShakeDetector? _shakeDetector;
    private LowLevelMouseHook? _shakeMouseHook;
    private ShakeRestoreState? _shakeRestore;
    private long _shakeSessionVersion;
    private bool _shakeRestoreInFlight;
    private readonly SemaphoreSlim _shakeTransitionGate = new(1, 1);
    private bool _shakeButtonDown;
    private bool _shakeTriggeredForButton;
    private int _shakeSummonX;
    private int _shakeSummonY;
    private Task _shakeOperationTask = Task.CompletedTask;
    /// <summary>Creates the resident Drop Shelf window and native integrations.</summary>
    public MainWindow()
    {
        InitializeComponent();
        _dispatcherQueue = DispatcherQueue.GetForCurrentThread();
        _shakeInputQueue = new(_dispatcherQueue, ProcessShakeInput);
        _windowHandle = WinRT.Interop.WindowNative.GetWindowHandle(this);
        WindowInterop.MakeBorderless(_windowHandle);

        var iconPath = Path.Combine(AppContext.BaseDirectory, "Assets", "AppIcon.ico");
        if (File.Exists(iconPath))
        {
            AppWindow.SetIcon(iconPath);
        }

        RootFrame.Navigate(typeof(MainPage));
        _page = (MainPage)RootFrame.Content;
        _messageHook = new WindowMessageHook(_windowHandle, HandleWindowMessage);
        _globalHotKey = new GlobalHotKey(_windowHandle);
        _trayIcon = new TrayIcon(
            _windowHandle,
            iconPath,
            ShowShelf,
            ShowSettings,
            ExitApplication);

        Closed += (_, _) => DisposeResidentResources();
    }

    /// <summary>Loads resident settings and either shows the shelf or keeps startup hidden.</summary>
    /// <param name="startHidden">Whether this launch came from the sign-in startup registration.</param>
    public async Task InitializeAsync(bool startHidden)
    {
        HideAllSurfaces();
        var databasePath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "DropCove",
            "shelf.db");
        _manager = await DropShelfManager.OpenAsync(databasePath, DragDropService.CheckAvailability);
        await _page.InitializeAsync(
            _manager,
            DismissShelf,
            ShowSettings,
            () => WindowInterop.BeginMove(_windowHandle),
            ResizeShelfForPresentation,
            ShowConfirmDialogAsync,
            OnShakeDragStarted,
            OnShakeDragCanceled,
            OnShakeDropCompletedAsync);
        if (_manager.RecoveryBackupPath is not null)
        {
            _trayIcon.ShowWarning(
                "Shelf database reset",
                $"The failed database was preserved at {_manager.RecoveryBackupPath}. DropCove started with an empty shelf.");
        }

        _settings = await _settingsStore.LoadAsync();
        _shakeDetector = _settings.ShakeEnabled
            ? new ShakeDetector(_settings.ShakeSensitivity)
            : null;
        StartupRegistration.SetEnabled(_settings.StartWithWindows);
        if (!TryRegisterHotKey(_settings.HotKey))
        {
            _trayIcon.ShowWarning(
                "Hotkey unavailable",
                "DropCove is running. Open Settings from the tray and choose another hotkey.");
        }

        if (!TryConfigureShakeHook(_settings.ShakeEnabled))
        {
            _trayIcon.ShowWarning(
                "Shake unavailable",
                "DropCove is running, but the global shake hook could not be installed.");
        }

        if (startHidden)
        {
            HideAllSurfaces();
        }
        else
        {
            ShowShelf();
        }
    }

    /// <summary>Shows and activates the shelf on the monitor containing the cursor.</summary>
    public void ShowShelf() => ShowShelfAt(null);
    private void ResizeShelfForPresentation(ShelfDisplayState presentation)
    {
        if (_manager is null || presentation is not (ShelfDisplayState.Compact or ShelfDisplayState.Expanded))
        {
            return;
        }

        var (width, height) = GetShelfSize(presentation, _manager.Batches.Count);
        WindowInterop.ResizeAnchored(_windowHandle, width, height);
    }

    private async void ShowShelfAt(ShelfRailPlacement? placement)
    {
        if (_manager is null)
        {
            return;
        }

        HideRail();
        _manager.ShowShelf(ShelfDisplayState.Compact);
        await _page.RefreshAsync();
        var (width, height) = GetShelfSize(_manager.DisplayState, _manager.Batches.Count);
        if (placement is { HasMonitor: true })
        {
            var monitorId = WindowInterop.ResolveMonitorId(placement.MonitorId, _windowHandle);
            WindowInterop.PositionOnMonitor(_windowHandle, monitorId, width, height);
        }
        else
        {
            WindowInterop.PositionOnCursorMonitor(_windowHandle, width, height);
        }

        WindowInterop.ShowAndActivate(_windowHandle);
        _page.Focus(FocusState.Programmatic);
        _page.PlayShelfAppearance();
    }
    private void ProcessShakeInput(LowLevelMouseInput input)
    {
        if (_shakeMouseHook is null || _shakeDetector is null)
        {
            return;
        }
        if (input.Message == LowLevelMouseHook.LeftButtonDownMessage)
        {
            _shakeButtonDown = true;
            _shakeTriggeredForButton = false;
            _shakeDetector?.Reset();
            _shakeSession.OnMouseButtonChanged(true);
            return;
        }

        if (input.Message == LowLevelMouseHook.LeftButtonUpMessage)
        {
            _shakeButtonDown = false;
            _shakeTriggeredForButton = false;
            if (_shakeSession.OnMouseButtonChanged(false) && _shakeRestore is not null)
            {
                QueueShakeOperation(RestoreShakeStateAsync, "Shake restore failed");
            }

            return;
        }

        if (input.Message != LowLevelMouseHook.MouseMoveMessage)
        {
            return;
        }

        var wasButtonDown = _shakeButtonDown;
        _shakeButtonDown = input.LeftButtonDown;
        if (!wasButtonDown && _shakeButtonDown)
        {
            _shakeTriggeredForButton = false;
            _shakeDetector?.Reset();
            _shakeSession.OnMouseButtonChanged(true);
        }
        if (wasButtonDown && !_shakeButtonDown)
        {
            if (_shakeSession.OnMouseButtonChanged(false) && _shakeRestore is not null)
            {
                QueueShakeOperation(RestoreShakeStateAsync, "Shake restore failed");
            }
        }
        if (_shakeButtonDown &&
            !_shakeTriggeredForButton &&
            _shakeRestore is null &&
            _shakeDetector?.Observe(new ShakeSample(input.X, input.Y, input.TimestampMilliseconds)) == true)
        {
            _shakeTriggeredForButton = true;
            var targetX = input.X;
            var targetY = input.Y;
            QueueShakeOperation(() => SummonShelfByShakeCoreAsync(targetX, targetY), "Shake summon failed");
        }
    }

    private void QueueShakeOperation(Func<Task> operation, string failureTitle) =>
        _shakeOperationTask = RunQueuedShakeOperationAsync(_shakeOperationTask, operation, failureTitle);

    private async Task RunQueuedShakeOperationAsync(Task previous, Func<Task> operation, string failureTitle)
    {
        try
        {
            await previous;
            await operation();
        }
        catch (Exception exception)
        {
            Exception? restoreException = null;
            try
            {
                await RestoreShakeStateAsync();
            }
            catch (Exception recoveryException)
            {
                restoreException = recoveryException;
            }

            if (restoreException is not null || _shakeRestoreInFlight)
            {
                _shakeRestore = null;
                _shakeRestoreInFlight = false;
                _shakeSession.Reset();
                _manager?.HideShelf();
                HideAllSurfaces();
            }

            var message = restoreException is null
                ? exception.Message
                : $"{exception.Message} Restore failed: {restoreException.Message}";
            _trayIcon.ShowWarning(failureTitle, message);
        }
    }

    private async Task SummonShelfByShakeCoreAsync(int targetX, int targetY)
    {
        if (_manager is null ||
            _shakeRestore is not null ||
            _shakeRestoreInFlight ||
            _manager.DisplayState is ShelfDisplayState.Compact or ShelfDisplayState.Expanded)
        {
            return;
        }

        // Guard against race condition: if the user released the mouse before the queued summon task executes
        if (!_shakeButtonDown || !_shakeSession.TrySummon())
        {
            return;
        }

        _shakeSummonX = targetX;
        _shakeSummonY = targetY;
        var sessionVersion = ++_shakeSessionVersion;
        _shakeRestore = new ShakeRestoreState(_manager.DisplayState, _manager.RailPlacement);
        _shakeDetector?.Reset();
        HideAllSurfaces();
        await ShowShelfNearPointCoreAsync(targetX, targetY, sessionVersion);
    }

    private Task ShowShelfNearCursorCoreAsync(long? sessionVersion, bool refresh = true) =>
        ShowShelfNearPointCoreAsync(_shakeSummonX, _shakeSummonY, sessionVersion, refresh);
    private async Task ShowShelfNearPointCoreAsync(int targetX, int targetY, long? sessionVersion, bool refresh = true)
    {
        if (_manager is null ||
            sessionVersion is not null && (sessionVersion != _shakeSessionVersion || _shakeRestore is null))
        {
            return;
        }

        HideRail();
        _manager.ShowShelf(ShelfDisplayState.Compact);
        if (refresh)
        {
            await _page.RefreshAsync();
        }
        if (sessionVersion is not null && (sessionVersion != _shakeSessionVersion || _shakeRestore is null))
        {
            return;
        }

        var (width, height) = GetShelfSize(_manager.DisplayState, _manager.Batches.Count);
        WindowInterop.PositionNearPoint(_windowHandle, targetX, targetY, width, height);
        WindowInterop.ShowAndActivate(_windowHandle);
        _page.Focus(FocusState.Programmatic);
        if (refresh)
        {
            _page.PlayShelfAppearance();
        }
    }

    private void OnShakeDragStarted()
    {
        _shakeSession.OnDragEnter();
    }

    private void OnShakeDragCanceled()
    {
        // Cursor left the shelf window bounds. Do NOT restore here while left mouse button is held down.
        // ShakeSessionCoordinator will trigger restore only when the button is released.
        _shakeSession.OnDragLeave();
    }
    private async Task OnShakeDropCompletedAsync(bool accepted)
    {
        var restoreAfterRelease = false;
        await _shakeTransitionGate.WaitAsync();
        try
        {
            if (_shakeRestore is null && !_shakeRestoreInFlight)
            {
                return;
            }
            if (accepted)
            {
                _shakeSessionVersion++;
                _shakeRestore = null;
                _shakeRestoreInFlight = false;
                _shakeDetector?.Reset();
                _shakeSession.OnDropCompleted(true);
                await ShowShelfNearCursorCoreAsync(null, refresh: false);
            }
            else if (_shakeRestore is not null)
            {
                _shakeSession.OnDropCompleted(false);
                restoreAfterRelease = true;
            }
        }
        finally
        {
            _shakeTransitionGate.Release();
        }

        if (restoreAfterRelease)
        {
            await RestoreShakeStateAsync();
        }
    }

    private async Task RestoreShakeStateAsync()
    {
        if (_shakeRestore is not { } restore)
        {
            return;
        }

        var sessionVersion = ++_shakeSessionVersion;
        _shakeRestore = null;
        _shakeRestoreInFlight = true;
        _shakeSession.Reset();
        _shakeDetector?.Reset();
        HideAllSurfaces();
        await RestoreShakeStateCoreAsync(restore, sessionVersion);
        if (sessionVersion == _shakeSessionVersion)
        {
            _shakeRestoreInFlight = false;
        }
    }

    private async Task RestoreShakeStateCoreAsync(ShakeRestoreState restore, long sessionVersion)
    {
        await _shakeTransitionGate.WaitAsync();
        try
        {
            if (sessionVersion != _shakeSessionVersion || _manager is null)
            {
                return;
            }

            if (restore.DisplayState == ShelfDisplayState.EdgeDocked)
            {
                await DismissShelfAtPlacement(restore.RailPlacement, sessionVersion);
            }
            else
            {
                _manager.HideShelf();
            }
        }
        finally
        {
            _shakeTransitionGate.Release();
        }
    }

    private async Task ToggleShelfAsync()
    {
        if (_railWindow is not null && WindowInterop.IsVisible(_railWindow.WindowHandle))
        {
            ShowShelf();
        }
        else if (WindowInterop.IsVisible(_windowHandle))
        {
            await DismissShelf();
        }
        else
        {
            ShowShelf();
        }
    }

    private async Task DismissShelf()
    {
        if (_manager is null)
        {
            return;
        }

        await DismissShelfAtPlacement(GetConfiguredRailPlacement(_settings));
    }

    private async Task DismissShelfAtPlacement(ShelfRailPlacement placement, long? shakeSessionVersion = null)
    {
        if (_manager is null || shakeSessionVersion is not null && shakeSessionVersion != _shakeSessionVersion)
        {
            return;
        }

        HideShelf();
        var state = await _manager.DismissShelfAsync(placement);
        if (shakeSessionVersion is not null && shakeSessionVersion != _shakeSessionVersion)
        {
            return;
        }

        if (state != ShelfDisplayState.EdgeDocked)
        {
            HideRail();
            return;
        }

        _railWindow ??= new EdgeRailWindow(
            _manager,
            ShowConfirmDialogAsync,
            _page.AcceptStorageDropAsync,
            RefreshRailAfterMutationAsync,
            OpenShelfFromRail,
            message => _trayIcon.ShowWarning("Edge Rail drag", message));
        _railWindow.UpdateBatches(_manager.Batches);
        _railWindow.Show(_manager.RailPlacement, _settings.ShowRailOverFullscreen);
    }

    private void OpenShelfFromRail() => ShowShelfAt(_manager?.RailPlacement);

    private async Task RefreshRailAfterMutationAsync()
    {
        if (_manager is null || _railWindow is null)
        {
            return;
        }

        await _page.RefreshAsync();
        if (_manager.Batches.Count == 0)
        {
            _railWindow.Hide();
        }
        else
        {
            _railWindow.UpdateBatches(_manager.Batches);
        }
    }

    private void HideAllSurfaces()
    {
        HideShelf();
        HideRail();
    }

    private void HideShelf() => WindowInterop.Hide(_windowHandle);

    private void HideRail() => _railWindow?.Hide();

    private void ShowSettings()
    {
        if (_settingsWindow is not null)
        {
            _settingsWindow.Activate();
            return;
        }

        _settingsWindow = new SettingsWindow(
            _settings,
            WindowInterop.GetMonitorOptions(),
            ApplySettingsAsync);
        _settingsWindow.Closed += (_, _) => _settingsWindow = null;
        _settingsWindow.Activate();
    }

    private async Task<bool> ShowConfirmDialogAsync(string title, string message)
    {
        var confirmWindow = new ConfirmWindow(_windowHandle, title, message);
        return await confirmWindow.ShowDialogAsync();
    }

    private async Task<bool> ApplySettingsAsync(AppSettings settings)
    {
        var previous = _settings;
        if (!TryRegisterHotKey(settings.HotKey))
        {
            _trayIcon.ShowWarning(
                "Hotkey conflict",
                "That combination is already in use. Choose another hotkey in DropCove Settings.");
            return false;
        }

        try
        {
            if (!TryConfigureShakeHook(settings.ShakeEnabled))
            {
                throw new InvalidOperationException("The global shake hook could not be installed.");
            }

            _shakeDetector = settings.ShakeEnabled
                ? new ShakeDetector(settings.ShakeSensitivity)
                : null;
            StartupRegistration.SetEnabled(settings.StartWithWindows);
            await _settingsStore.SaveAsync(settings);
            _settings = settings;

            if (_manager is not null && _manager.Batches.Count > 0)
            {
                var placement = GetConfiguredRailPlacement(settings);
                await _manager.SetRailPlacementAsync(placement);
                if (_railWindow?.IsRequestedVisible == true)
                {
                    _railWindow.UpdateBatches(_manager.Batches);
                    _railWindow.Show(placement, settings.ShowRailOverFullscreen);
                }
            }

            return true;
        }
        catch (Exception exception)
        {
            TryRegisterHotKey(previous.HotKey);
            var restoredShake = TryConfigureShakeHook(previous.ShakeEnabled);
            if (!restoredShake)
            {
                var retainedHook = _shakeMouseHook is not null;
                var retainedSettings = previous with
                {
                    ShakeEnabled = retainedHook,
                    ShakeSensitivity = retainedHook ? settings.ShakeSensitivity : previous.ShakeSensitivity,
                };
                _shakeDetector = retainedHook
                    ? new ShakeDetector(retainedSettings.ShakeSensitivity)
                    : null;
                StartupRegistration.SetEnabled(previous.StartWithWindows);
                _settings = retainedSettings;
                try
                {
                    await _settingsStore.SaveAsync(retainedSettings);
                }
                catch (Exception persistenceException)
                {
                    _trayIcon.ShowWarning("Shake rollback", persistenceException.Message);
                }

                _trayIcon.ShowWarning(
                    retainedHook ? "Shake remains enabled" : "Shake disabled",
                    $"Settings could not be rolled back safely: {exception.Message}");
                throw;
            }

            _shakeDetector = previous.ShakeEnabled
                ? new ShakeDetector(previous.ShakeSensitivity)
                : null;
            StartupRegistration.SetEnabled(previous.StartWithWindows);
            await _settingsStore.SaveAsync(previous);
            _settings = previous;
            throw;
        }
    }

    private ShelfRailPlacement GetConfiguredRailPlacement(AppSettings settings)
    {
        if (_manager is null)
        {
            throw new InvalidOperationException("The shelf manager has not been initialized.");
        }

        var monitorId = !string.IsNullOrWhiteSpace(settings.RailMonitorId)
            ? settings.RailMonitorId
            : _manager.RailPlacement.HasMonitor
                ? _manager.RailPlacement.MonitorId
                : WindowInterop.GetMonitorIdForWindow(_windowHandle);
        return new ShelfRailPlacement(monitorId, settings.RailEdge);
    }

    private bool TryRegisterHotKey(HotKeyDefinition hotKey) =>
        _globalHotKey.TrySet(ToNativeModifiers(hotKey), hotKey.VirtualKey);

    private bool TryConfigureShakeHook(bool enabled)
    {
        if (!enabled)
        {
            try
            {
                _shakeMouseHook?.Dispose();
                _shakeMouseHook = null;
            }
            catch (Win32Exception)
            {
                return false;
            }

            _shakeInputQueue.Clear();
            _shakeButtonDown = false;
            _shakeTriggeredForButton = false;
            _shakeDetector = null;
            if (_shakeRestore is not null)
            {
                QueueShakeOperation(RestoreShakeStateAsync, "Shake restore failed");
            }

            return true;
        }

        if (_shakeMouseHook is not null)
        {
            return true;
        }

        try
        {
            _shakeMouseHook = new LowLevelMouseHook(_shakeInputQueue.Enqueue);
            return true;
        }
        catch (Win32Exception)
        {
            _shakeMouseHook = null;
            return false;
        }
    }

    private WindowMessageResult HandleWindowMessage(uint message, nuint wParam, nint lParam)
    {
        if (message is WindowInterop.DisplayChangeMessage or WindowInterop.DpiChangedMessage or WindowInterop.DeviceChangeMessage)
        {
            QueueSurfaceReflow();
        }

        if (_globalHotKey.Matches(message, wParam))
        {
            DispatcherQueue.TryEnqueue(async () => await ToggleShelfAsync());
            return WindowMessageResult.HandledZero;
        }

        if (message == TrayIcon.CallbackMessage)
        {
            _trayIcon.HandleCallback(lParam);
            return WindowMessageResult.HandledZero;
        }

        if (message == WindowCloseMessage)
        {
            DispatcherQueue.TryEnqueue(async () => await DismissShelf());
            return WindowMessageResult.HandledZero;
        }

        if (message == InstallerExitMessage)
        {
            DispatcherQueue.TryEnqueue(ExitApplication);
            return WindowMessageResult.HandledZero;
        }

        return WindowMessageResult.Unhandled;
    }

    private void QueueSurfaceReflow()
    {
        if (_surfaceReflowQueued)
        {
            return;
        }

        _surfaceReflowQueued = true;
        if (!DispatcherQueue.TryEnqueue(() =>
            {
                _surfaceReflowQueued = false;
                ReflowSurfaces();
            }))
        {
            _surfaceReflowQueued = false;
        }
    }

    private void ReflowSurfaces()
    {
        if (_manager is null)
        {
            return;
        }

        _settingsWindow?.UpdateMonitorOptions(WindowInterop.GetMonitorOptions());
        if (WindowInterop.IsVisible(_windowHandle) &&
            _manager.DisplayState is ShelfDisplayState.Compact or ShelfDisplayState.Expanded)
        {
            ResizeShelfForPresentation(_manager.DisplayState);
        }

        _railWindow?.Reposition();
    }

    private void ExitApplication()
    {
        _settingsWindow?.Close();
        DisposeResidentResources();
        Application.Current.Exit();
    }

    private void DisposeResidentResources()
    {
        if (_disposed)
        {
            return;
        }

        _trayIcon.Dispose();
        _railWindow?.Close();
        _railWindow = null;
        try
        {
            _shakeMouseHook?.Dispose();
            _shakeMouseHook = null;
        }
        catch (Win32Exception exception)
        {
            _trayIcon.ShowWarning("Shake shutdown", exception.Message);
        }
        _globalHotKey.Dispose();
        _messageHook.Dispose();
        _disposed = true;
    }

    private static HotKeyModifiers ToNativeModifiers(HotKeyDefinition hotKey)
    {
        var modifiers = HotKeyModifiers.None;
        if (hotKey.Control) modifiers |= HotKeyModifiers.Control;
        if (hotKey.Alt) modifiers |= HotKeyModifiers.Alt;
        if (hotKey.Shift) modifiers |= HotKeyModifiers.Shift;
        if (hotKey.Windows) modifiers |= HotKeyModifiers.Windows;
        return modifiers;
    }

    private readonly record struct ShakeRestoreState(
        ShelfDisplayState DisplayState,
        ShelfRailPlacement RailPlacement);

    private static (int Width, int Height) GetShelfSize(ShelfDisplayState presentation, int batchCount)
    {
        if (presentation == ShelfDisplayState.Expanded)
        {
            return (720, 640);
        }

        var visibleBatchCount = Math.Clamp(batchCount, 0, 3);
        var height = visibleBatchCount switch
        {
            <= 1 => 180,
            2 => 236,
            _ => 292,
        };
        return (180, height);
    }
}
