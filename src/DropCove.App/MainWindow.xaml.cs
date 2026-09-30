using DropCove.Core;
using DropCove.Native;
using DropCove.Services;
using Microsoft.UI.Xaml;

namespace DropCove;

/// <summary>Hosts the resident bounded unified Drop Shelf.</summary>
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

    /// <summary>Creates the resident Drop Shelf window and native integrations.</summary>
    public MainWindow()
    {
        InitializeComponent();
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
            (width, height) => WindowInterop.ResizeAnchored(_windowHandle, width, height),
            ShowConfirmDialogAsync);
        if (_manager.RecoveryBackupPath is not null)
        {
            _trayIcon.ShowWarning(
                "Shelf database reset",
                $"The failed database was preserved at {_manager.RecoveryBackupPath}. DropCove started with an empty shelf.");
        }

        _settings = await _settingsStore.LoadAsync();
        StartupRegistration.SetEnabled(_settings.StartWithWindows);
        if (!TryRegisterHotKey(_settings.HotKey))
        {
            _trayIcon.ShowWarning(
                "Hotkey unavailable",
                "DropCove is running. Open Settings from the tray and choose another hotkey.");
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

    private void ShowShelfAt(ShelfRailPlacement? placement)
    {
        if (_manager is null)
        {
            return;
        }

        HideRail();
        _manager.ShowShelf();
        var (width, height) = GetShelfSize(_manager.Batches.Count);
        if (placement is { HasMonitor: true })
        {
            WindowInterop.PositionOnMonitor(_windowHandle, placement.MonitorId, width, height);
        }
        else
        {
            WindowInterop.PositionOnCursorMonitor(_windowHandle, width, height);
        }

        WindowInterop.ShowAndActivate(_windowHandle);
        _page.Focus(FocusState.Programmatic);
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

        HideShelf();
        var placement = GetConfiguredRailPlacement(_settings);
        var state = await _manager.DismissShelfAsync(placement);
        if (state != ShelfDisplayState.EdgeDocked)
        {
            HideRail();
            return;
        }

        _railWindow ??= new EdgeRailWindow(_manager.Batches, OpenShelfFromRail);
        _railWindow.UpdateBatches(_manager.Batches);
        _railWindow.Show(_manager.RailPlacement, _settings.ShowRailOverFullscreen);
    }

    private void OpenShelfFromRail() => ShowShelfAt(_manager?.RailPlacement);

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
        catch
        {
            TryRegisterHotKey(previous.HotKey);
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

    private WindowMessageResult HandleWindowMessage(uint message, nuint wParam, nint lParam)
    {
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

    private static (int Width, int Height) GetShelfSize(int batchCount) => batchCount switch
    {
        <= 1 => (180, 180),
        2 => (180, 236),
        3 => (180, 292),
        _ => (180, 348),
    };
}
