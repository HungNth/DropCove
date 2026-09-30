using System.ComponentModel;
using System.Runtime.InteropServices;

namespace DropCove.Native;

/// <summary>Win32 modifier flags used by global hotkeys.</summary>
[Flags]
public enum HotKeyModifiers : uint
{
    /// <summary>No modifier.</summary>
    None = 0,
    /// <summary>Alt.</summary>
    Alt = 0x0001,
    /// <summary>Control.</summary>
    Control = 0x0002,
    /// <summary>Shift.</summary>
    Shift = 0x0004,
    /// <summary>Windows key.</summary>
    Windows = 0x0008,
    /// <summary>Suppress repeated messages while the key is held.</summary>
    NoRepeat = 0x4000,
}

/// <summary>Represents the outcome of a window-message callback.</summary>
/// <param name="Handled">Whether the message was handled.</param>
/// <param name="Result">The message result.</param>
public readonly record struct WindowMessageResult(bool Handled, nint Result)
{
    /// <summary>Gets an unhandled result.</summary>
    public static WindowMessageResult Unhandled => default;

    /// <summary>Gets a handled zero result.</summary>
    public static WindowMessageResult HandledZero { get; } = new(true, 0);
}

/// <summary>Handles a Win32 window message.</summary>
/// <param name="message">The message identifier.</param>
/// <param name="wParam">The message word parameter.</param>
/// <param name="lParam">The message long parameter.</param>
/// <returns>The callback result.</returns>
public delegate WindowMessageResult WindowMessageHandler(uint message, nuint wParam, nint lParam);

/// <summary>Subclasses one existing window without replacing its original window procedure.</summary>
public sealed class WindowMessageHook : IDisposable
{
    private const nuint SubclassId = 1;
    private readonly nint _windowHandle;
    private readonly WindowMessageHandler _handler;
    private readonly SubclassProc _subclassProc;
    private bool _disposed;

    /// <summary>Installs a message hook for an existing HWND.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    /// <param name="handler">The message callback.</param>
    public WindowMessageHook(nint windowHandle, WindowMessageHandler handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        _windowHandle = windowHandle;
        _handler = handler;
        _subclassProc = SubclassCallback;

        if (!SetWindowSubclass(windowHandle, _subclassProc, SubclassId, 0))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not subclass the DropCove window.");
        }
    }

    /// <inheritdoc />
    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }

        RemoveWindowSubclass(_windowHandle, _subclassProc, SubclassId);
        _disposed = true;
    }

    private nint SubclassCallback(
        nint windowHandle,
        uint message,
        nuint wParam,
        nint lParam,
        nuint subclassId,
        nuint referenceData)
    {
        var result = _handler(message, wParam, lParam);
        return result.Handled
            ? result.Result
            : DefSubclassProc(windowHandle, message, wParam, lParam);
    }

    private delegate nint SubclassProc(
        nint windowHandle,
        uint message,
        nuint wParam,
        nint lParam,
        nuint subclassId,
        nuint referenceData);

    [DllImport("comctl32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetWindowSubclass(
        nint windowHandle,
        SubclassProc callback,
        nuint subclassId,
        nuint referenceData);

    [DllImport("comctl32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool RemoveWindowSubclass(
        nint windowHandle,
        SubclassProc callback,
        nuint subclassId);

    [DllImport("comctl32.dll")]
    private static extern nint DefSubclassProc(nint windowHandle, uint message, nuint wParam, nint lParam);
}

/// <summary>Registers one replaceable global hotkey against a window.</summary>
public sealed class GlobalHotKey : IDisposable
{
    /// <summary>The Win32 global-hotkey message.</summary>
    public const uint Message = 0x0312;

    private readonly nint _windowHandle;
    private readonly int _id;
    private HotKeyModifiers _modifiers;
    private uint _virtualKey;
    private bool _registered;

    /// <summary>Creates a global-hotkey registration slot.</summary>
    /// <param name="windowHandle">The HWND receiving hotkey messages.</param>
    /// <param name="id">The application-local hotkey identifier.</param>
    public GlobalHotKey(nint windowHandle, int id = 1)
    {
        _windowHandle = windowHandle;
        _id = id;
    }

    /// <summary>Replaces the current registration, restoring it if the new combination conflicts.</summary>
    /// <param name="modifiers">Modifier flags.</param>
    /// <param name="virtualKey">The Win32 virtual-key code.</param>
    /// <returns><see langword="true"/> when the new hotkey was registered.</returns>
    public bool TrySet(HotKeyModifiers modifiers, uint virtualKey)
    {
        var oldModifiers = _modifiers;
        var oldVirtualKey = _virtualKey;
        var hadRegistration = _registered;

        if (hadRegistration)
        {
            UnregisterHotKey(_windowHandle, _id);
            _registered = false;
        }

        var effectiveModifiers = modifiers | HotKeyModifiers.NoRepeat;
        if (RegisterHotKey(_windowHandle, _id, (uint)effectiveModifiers, virtualKey))
        {
            _modifiers = modifiers;
            _virtualKey = virtualKey;
            _registered = true;
            return true;
        }

        if (hadRegistration && RegisterHotKey(
                _windowHandle,
                _id,
                (uint)(oldModifiers | HotKeyModifiers.NoRepeat),
                oldVirtualKey))
        {
            _modifiers = oldModifiers;
            _virtualKey = oldVirtualKey;
            _registered = true;
        }

        return false;
    }

    /// <summary>Returns whether a message belongs to this hotkey.</summary>
    /// <param name="message">The window message.</param>
    /// <param name="wParam">The message identifier parameter.</param>
    /// <returns>Whether this registration owns the message.</returns>
    public bool Matches(uint message, nuint wParam) => message == Message && (int)wParam == _id;

    /// <inheritdoc />
    public void Dispose()
    {
        if (_registered)
        {
            UnregisterHotKey(_windowHandle, _id);
            _registered = false;
        }
    }

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool RegisterHotKey(nint windowHandle, int id, uint modifiers, uint virtualKey);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool UnregisterHotKey(nint windowHandle, int id);
}

/// <summary>Describes one display that can host the Edge Rail.</summary>
/// <param name="Id">The Windows display device name.</param>
/// <param name="Label">The user-visible display label.</param>
public sealed record MonitorOption(string Id, string Label);

/// <summary>Raises callbacks when the foreground window changes.</summary>
public sealed class ForegroundWindowHook : IDisposable
{
    private const uint EventSystemForeground = 0x0003;
    private const uint EventObjectLocationChange = 0x800B;
    private const int ObjidWindow = 0;
    private const uint WineventOutOfContext = 0;
    private readonly WinEventDelegate _foregroundCallback;
    private readonly WinEventDelegate _locationCallback;
    private readonly Action _onForegroundChanged;
    private nint _foregroundHook;
    private nint _locationHook;
    private bool _disposed;

    /// <summary>Installs out-of-context hooks for foreground activation and in-place foreground resize.</summary>
    /// <param name="onForegroundChanged">The callback invoked when the foreground window changes or resizes.</param>
    public ForegroundWindowHook(Action onForegroundChanged)
    {
        ArgumentNullException.ThrowIfNull(onForegroundChanged);
        _onForegroundChanged = onForegroundChanged;
        _foregroundCallback = HandleForegroundChanged;
        _locationCallback = HandleLocationChanged;

        _foregroundHook = SetWinEventHook(
            EventSystemForeground,
            EventSystemForeground,
            0,
            _foregroundCallback,
            0,
            0,
            WineventOutOfContext);
        if (_foregroundHook == 0)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not monitor foreground-window changes.");
        }

        _locationHook = SetWinEventHook(
            EventObjectLocationChange,
            EventObjectLocationChange,
            0,
            _locationCallback,
            0,
            0,
            WineventOutOfContext);
        if (_locationHook == 0)
        {
            UnhookWinEvent(_foregroundHook);
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not monitor foreground-window location changes.");
        }
    }

    /// <inheritdoc />
    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }

        if (_foregroundHook != 0)
        {
            UnhookWinEvent(_foregroundHook);
            _foregroundHook = 0;
        }

        if (_locationHook != 0)
        {
            UnhookWinEvent(_locationHook);
            _locationHook = 0;
        }

        _disposed = true;
    }

    private void HandleForegroundChanged(
        nint hook,
        uint eventType,
        nint windowHandle,
        int objectId,
        int childId,
        uint eventThreadId,
        uint eventTime)
    {
        if (!_disposed)
        {
            _onForegroundChanged();
        }
    }

    private void HandleLocationChanged(
        nint hook,
        uint eventType,
        nint windowHandle,
        int objectId,
        int childId,
        uint eventThreadId,
        uint eventTime)
    {
        if (_disposed || objectId != ObjidWindow || windowHandle == 0)
        {
            return;
        }

        if (windowHandle == GetForegroundWindow())
        {
            _onForegroundChanged();
        }
    }

    [DllImport("user32.dll")]
    private static extern nint GetForegroundWindow();

    private delegate void WinEventDelegate(
        nint hook,
        uint eventType,
        nint windowHandle,
        int objectId,
        int childId,
        uint eventThreadId,
        uint eventTime);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern nint SetWinEventHook(
        uint eventMin,
        uint eventMax,
        nint moduleHandle,
        WinEventDelegate callback,
        uint processId,
        uint threadId,
        uint flags);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool UnhookWinEvent(nint hook);
}

/// <summary>Provides the minimal Win32 window operations required by the Drop Shelf.</summary>
public static class WindowInterop
{
    /// <summary>Win32 mouse-move message.</summary>
    public const uint MouseMoveMessage = 0x0200;

    /// <summary>Win32 mouse-leave message.</summary>
    public const uint MouseLeaveMessage = 0x02A3;

    private const int GwlStyle = -16;
    private const int GwlExStyle = -20;
    private const nint HwndTopmost = -1;
    private const uint MonitorDefaultToNearest = 2;
    private const uint SwpNoActivate = 0x0010;
    private const uint SwpNoMove = 0x0002;
    private const uint SwpNoSize = 0x0001;
    private const uint SwpFrameChanged = 0x0020;
    private const uint SwpShowWindow = 0x0040;
    private const int SwHide = 0;
    private const int SwShow = 5;
    private const int SwShowNoActivate = 4;
    private const uint WmNcLButtonDown = 0x00A1;
    private const uint WmMouseActivate = 0x0021;
    private const nint MaNoActivate = 3;
    private const nuint HtCaption = 2;
    private const long WsCaption = 0x00C00000L;
    private const long WsThickFrame = 0x00040000L;
    private const long WsMinimizeBox = 0x00020000L;
    private const long WsMaximizeBox = 0x00010000L;
    private const long WsSysMenu = 0x00080000L;
    private const long WsExNoActivate = 0x08000000L;
    private const long WsExToolWindow = 0x00000080L;

    /// <summary>Removes standard caption and resize chrome from the HWND.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    public static void MakeBorderless(nint windowHandle)
    {
        var style = GetWindowLongPtr(windowHandle, GwlStyle).ToInt64();
        style &= ~(WsCaption | WsThickFrame | WsMinimizeBox | WsMaximizeBox | WsSysMenu);
        SetWindowLongPtr(windowHandle, GwlStyle, new nint(style));
        SetWindowPos(
            windowHandle,
            0,
            0,
            0,
            0,
            0,
            SwpNoMove | SwpNoSize | SwpNoActivate | SwpFrameChanged);
    }

    /// <summary>Marks an overlay window as a tool window that never activates from pointer input.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    public static void MakeNoActivate(nint windowHandle)
    {
        var extendedStyle = GetWindowLongPtr(windowHandle, GwlExStyle).ToInt64();
        extendedStyle |= WsExNoActivate | WsExToolWindow;
        SetWindowLongPtr(windowHandle, GwlExStyle, new nint(extendedStyle));
        SetWindowPos(
            windowHandle,
            0,
            0,
            0,
            0,
            0,
            SwpNoMove | SwpNoSize | SwpNoActivate | SwpFrameChanged);
    }

    /// <summary>Returns the native result that prevents mouse activation for an overlay window.</summary>
    /// <param name="message">The window message.</param>
    /// <returns>A handled no-activate result for <c>WM_MOUSEACTIVATE</c>; otherwise an unhandled result.</returns>
    public static WindowMessageResult PreventMouseActivation(uint message) =>
        message == WmMouseActivate ? new WindowMessageResult(true, MaNoActivate) : WindowMessageResult.Unhandled;

    /// <summary>Gets the first native content child of a WinUI window.</summary>
    /// <param name="windowHandle">The top-level WinUI window HWND.</param>
    /// <returns>The content child HWND, or zero when none exists.</returns>
    public static nint GetFirstChildWindow(nint windowHandle) =>
        FindWindowEx(windowHandle, 0, null, null);

    /// <summary>Requests a <c>WM_MOUSELEAVE</c> notification for an overlay window.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    public static void TrackMouseLeave(nint windowHandle)
    {
        var tracking = new TrackMouseEventParameters
        {
            Size = (uint)Marshal.SizeOf<TrackMouseEventParameters>(),
            Flags = 0x00000002,
            WindowHandle = windowHandle,
        };
        TrackMouseEvent(ref tracking);
    }

    /// <summary>Begins a normal system window move from a custom drag strip.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    public static void BeginMove(nint windowHandle)
    {
        ReleaseCapture();
        SendMessage(windowHandle, WmNcLButtonDown, HtCaption, 0);
    }

    /// <summary>Positions the window in the center of the cursor monitor and makes it topmost.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    /// <param name="logicalWidth">Width in logical pixels.</param>
    /// <param name="logicalHeight">Height in logical pixels.</param>
    public static void PositionOnCursorMonitor(nint windowHandle, int logicalWidth, int logicalHeight)
    {
        ValidateLogicalSize(logicalWidth, logicalHeight);
        if (!GetCursorPos(out var cursor))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read the cursor position.");
        }

        var monitor = MonitorFromPoint(cursor, MonitorDefaultToNearest);
        if (monitor == 0)
        {
            throw new Win32Exception("Could not resolve the cursor monitor.");
        }

        var monitorInfo = GetMonitorInfo(monitor);
        var (width, height) = FitToWorkArea(logicalWidth, logicalHeight, GetMonitorDpi(monitor), monitorInfo.Work);
        var x = monitorInfo.Work.Left + (monitorInfo.Work.Width - width) / 2;
        var y = monitorInfo.Work.Top + (monitorInfo.Work.Height - height) / 2;
        SetBounds(windowHandle, x, y, width, height, SwpNoActivate);
    }

    /// <summary>Gets the Windows display device name for a window's monitor.</summary>
    /// <param name="windowHandle">The target window.</param>
    /// <returns>The display device name, such as <c>\\.\DISPLAY1</c>.</returns>
    public static string GetMonitorIdForWindow(nint windowHandle)
    {
        var monitor = MonitorFromWindow(windowHandle, MonitorDefaultToNearest);
        if (monitor == 0)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not resolve the window monitor.");
        }

        return GetMonitorInfo(monitor).DeviceName
            ?? throw new InvalidOperationException("The monitor did not provide a display device name.");
    }

    /// <summary>Returns the currently connected displays in Windows enumeration order.</summary>
    /// <returns>The display options available to Edge Rail settings.</returns>
    public static IReadOnlyList<MonitorOption> GetMonitorOptions()
    {
        var monitors = new List<MonitorOption>();
        EnumDisplayMonitors(
            0,
            0,
            (monitor, _, _, _) =>
            {
                var info = GetMonitorInfo(monitor);
                var id = info.DeviceName ?? string.Empty;
                if (!string.IsNullOrWhiteSpace(id))
                {
                    monitors.Add(new MonitorOption(id, $"{id} ({info.Monitor.Width} × {info.Monitor.Height})"));
                }

                return true;
            },
            0);
        return monitors;
    }

    /// <summary>Determines whether the foreground window occupies the bounds of the selected rail monitor.</summary>
    /// <param name="monitorId">The selected Windows display device name.</param>
    /// <returns><see langword="true"/> for a non-shell window covering the selected monitor; otherwise <see langword="false"/>.</returns>
    public static bool IsForegroundWindowFullscreen(string monitorId)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(monitorId);
        var foregroundWindow = GetForegroundWindow();
        if (foregroundWindow == 0 || IsShellWindow(foregroundWindow) || !GetWindowRect(foregroundWindow, out var windowRect))
        {
            return false;
        }

        var monitor = MonitorFromWindow(foregroundWindow, MonitorDefaultToNearest);
        if (monitor == 0)
        {
            return false;
        }

        var monitorInfo = GetMonitorInfo(monitor);
        if (!string.Equals(monitorInfo.DeviceName, monitorId, StringComparison.OrdinalIgnoreCase))
        {
            return false;
        }

        var monitorBounds = monitorInfo.Monitor;
        return windowRect.Left <= monitorBounds.Left &&
               windowRect.Top <= monitorBounds.Top &&
               windowRect.Right >= monitorBounds.Right &&
               windowRect.Bottom >= monitorBounds.Bottom;
    }

    /// <summary>Positions the unified shelf at the center of a remembered monitor.</summary>
    /// <param name="windowHandle">The shelf window.</param>
    /// <param name="monitorId">The remembered display device name.</param>
    /// <param name="logicalWidth">The shelf width in logical pixels.</param>
    /// <param name="logicalHeight">The shelf height in logical pixels.</param>
    public static void PositionOnMonitor(
        nint windowHandle,
        string monitorId,
        int logicalWidth,
        int logicalHeight)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(monitorId);
        ValidateLogicalSize(logicalWidth, logicalHeight);

        var monitor = FindMonitor(monitorId);
        if (monitor == 0)
        {
            monitor = MonitorFromWindow(windowHandle, MonitorDefaultToNearest);
        }

        if (monitor == 0)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not resolve the shelf monitor.");
        }

        var monitorInfo = GetMonitorInfo(monitor);
        var (width, height) = FitToWorkArea(logicalWidth, logicalHeight, GetMonitorDpi(monitor), monitorInfo.Work);
        var x = monitorInfo.Work.Left + (monitorInfo.Work.Width - width) / 2;
        var y = monitorInfo.Work.Top + (monitorInfo.Work.Height - height) / 2;
        SetBounds(windowHandle, x, y, width, height, SwpNoActivate);
    }


    /// <summary>Positions a bounded, topmost Edge Rail over a monitor work area without activating it.</summary>
    /// <param name="windowHandle">The rail window.</param>
    /// <param name="monitorId">The remembered display device name.</param>
    /// <param name="dockLeft">Whether to dock to the left edge; otherwise the right edge is used.</param>
    /// <param name="logicalWidth">The rail width in logical pixels.</param>
    /// <param name="logicalHeight">The rail height in logical pixels.</param>
    public static void PositionEdgeRail(
        nint windowHandle,
        string monitorId,
        bool dockLeft,
        int logicalWidth,
        int logicalHeight)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(monitorId);
        ValidateLogicalSize(logicalWidth, logicalHeight);

        var monitor = FindMonitor(monitorId);
        if (monitor == 0)
        {
            monitor = MonitorFromWindow(windowHandle, MonitorDefaultToNearest);
        }

        if (monitor == 0)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not resolve the Edge Rail monitor.");
        }

        var monitorInfo = GetMonitorInfo(monitor);
        var (width, height) = FitToWorkArea(logicalWidth, logicalHeight, GetMonitorDpi(monitor), monitorInfo.Work);
        var x = dockLeft ? monitorInfo.Work.Left : monitorInfo.Work.Right - width;
        var y = monitorInfo.Work.Top + (monitorInfo.Work.Height - height) / 2;
        SetBounds(windowHandle, x, y, width, height, SwpNoActivate | SwpShowWindow);
    }

    /// <summary>Shows a topmost window without activating it.</summary>
    /// <param name="windowHandle">The target window.</param>
    public static void ShowNoActivate(nint windowHandle)
    {
        ShowWindow(windowHandle, SwShowNoActivate);
        SetWindowPos(windowHandle, HwndTopmost, 0, 0, 0, 0, SwpNoMove | SwpNoSize | SwpNoActivate | SwpShowWindow);
    }

    /// <summary>Resizes while retaining the top edge and clamping the window to its monitor work area.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    /// <param name="logicalWidth">Width in logical pixels.</param>
    /// <param name="logicalHeight">Height in logical pixels.</param>
    public static void ResizeAnchored(nint windowHandle, int logicalWidth, int logicalHeight)
    {
        ValidateLogicalSize(logicalWidth, logicalHeight);
        if (!GetWindowRect(windowHandle, out var current))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read the DropCove window bounds.");
        }

        var monitor = MonitorFromWindow(windowHandle, MonitorDefaultToNearest);
        if (monitor == 0)
        {
            throw new Win32Exception("Could not resolve the DropCove monitor.");
        }

        var monitorInfo = GetMonitorInfo(monitor);
        var dpi = GetDpiForWindow(windowHandle);
        if (dpi == 0)
        {
            dpi = GetMonitorDpi(monitor);
        }

        var (width, height) = FitToWorkArea(logicalWidth, logicalHeight, dpi, monitorInfo.Work);
        var maxX = monitorInfo.Work.Right - width;
        var maxY = monitorInfo.Work.Bottom - height;
        var x = Math.Clamp(current.Left, monitorInfo.Work.Left, maxX);
        var y = Math.Clamp(current.Top, monitorInfo.Work.Top, maxY);
        SetBounds(windowHandle, x, y, width, height, SwpNoActivate);
    }

    /// <summary>Shows, activates, and keeps the window topmost.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    public static void ShowAndActivate(nint windowHandle)
    {
        ShowWindow(windowHandle, SwShow);
        SetWindowPos(windowHandle, HwndTopmost, 0, 0, 0, 0, SwpNoMove | SwpNoSize | SwpShowWindow);
        SetForegroundWindow(windowHandle);
    }

    /// <summary>Hides a window without destroying it.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    public static void Hide(nint windowHandle) => ShowWindow(windowHandle, SwHide);

    /// <summary>Returns whether a window is visible.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    /// <returns>Whether the window is visible.</returns>
    public static bool IsVisible(nint windowHandle) => IsWindowVisible(windowHandle);

    /// <summary>Shows a modal native error dialog.</summary>
    /// <param name="title">The dialog title.</param>
    /// <param name="message">The error message.</param>
    public static void ShowError(string title, string message) =>
        MessageBox(0, message, title, 0x00000010);

    /// <summary>Shows a modal native confirmation dialog attached to an owner window.</summary>
    /// <param name="windowHandle">The owner window HWND.</param>
    /// <param name="title">The dialog title.</param>
    /// <param name="message">The confirmation question.</param>
    /// <returns>True if the user chose Yes; otherwise, false.</returns>
    public static bool Confirm(nint windowHandle, string title, string message) =>
        MessageBox(windowHandle, message, title, 0x00000024 /* MB_YESNO | MB_ICONQUESTION */) == 6 /* IDYES */;

    /// <summary>Positions a dialog centered over an owner window, clamped to the monitor work area.</summary>
    /// <param name="dialogHandle">The dialog HWND.</param>
    /// <param name="ownerHandle">The owner window HWND.</param>
    /// <param name="dialogWidth">Width in physical pixels.</param>
    /// <param name="dialogHeight">Height in physical pixels.</param>
    public static void CenterOverWindow(nint dialogHandle, nint ownerHandle, int dialogWidth, int dialogHeight)
    {
        if (!GetWindowRect(ownerHandle, out var ownerRect))
        {
            PositionOnCursorMonitor(dialogHandle, dialogWidth, dialogHeight);
            return;
        }

        var monitor = MonitorFromWindow(ownerHandle, MonitorDefaultToNearest);
        var monitorInfo = GetMonitorInfo(monitor);
        var targetX = (ownerRect.Left + ownerRect.Right) / 2 - dialogWidth / 2;
        var targetY = (ownerRect.Top + ownerRect.Bottom) / 2 - dialogHeight / 2;

        var x = Math.Clamp(targetX, monitorInfo.Work.Left, Math.Max(monitorInfo.Work.Left, monitorInfo.Work.Right - dialogWidth));
        var y = Math.Clamp(targetY, monitorInfo.Work.Top, Math.Max(monitorInfo.Work.Top, monitorInfo.Work.Bottom - dialogHeight));

        SetBounds(dialogHandle, x, y, dialogWidth, dialogHeight, SwpShowWindow);
        SetWindowPos(dialogHandle, HwndTopmost, 0, 0, 0, 0, SwpNoMove | SwpNoSize | SwpShowWindow);
    }

    /// <summary>Enables or disables input to a window.</summary>
    /// <param name="windowHandle">The window HWND.</param>
    /// <param name="enable">Whether input is enabled.</param>
    public static void SetWindowEnabled(nint windowHandle, bool enable) =>
        EnableWindow(windowHandle, enable);

    /// <summary>Sets the owner window for a popup/dialog.</summary>
    /// <param name="dialogHandle">The dialog HWND.</param>
    /// <param name="ownerHandle">The owner window HWND.</param>
    public static void SetOwnerWindow(nint dialogHandle, nint ownerHandle)
    {
        SetWindowLongPtr(dialogHandle, -8 /* GWLP_HWNDPARENT */, ownerHandle);
    }

    private static bool IsShellWindow(nint windowHandle)
    {
        var className = GetWindowClassName(windowHandle);
        return className is "Progman" or "WorkerW" or "Shell_TrayWnd";
    }

    private static string GetWindowClassName(nint windowHandle)
    {
        var buffer = new System.Text.StringBuilder(64);
        var length = GetClassName(windowHandle, buffer, buffer.Capacity);
        return length == 0 ? string.Empty : buffer.ToString();
    }

    private static void ValidateLogicalSize(int logicalWidth, int logicalHeight)
    {
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(logicalWidth);
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(logicalHeight);
    }

    private static (int Width, int Height) FitToWorkArea(
        int logicalWidth,
        int logicalHeight,
        uint dpi,
        Rect workArea)
    {
        if (workArea.Width <= 0 || workArea.Height <= 0)
        {
            throw new InvalidOperationException("The monitor work area is empty.");
        }

        return (
            Math.Clamp(Scale(logicalWidth, dpi), 1, workArea.Width),
            Math.Clamp(Scale(logicalHeight, dpi), 1, workArea.Height));
    }

    private static void SetBounds(nint windowHandle, int x, int y, int width, int height, uint flags)
    {
        if (!SetWindowPos(windowHandle, HwndTopmost, x, y, width, height, flags))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not position the DropCove window.");
        }
    }

    private static nint FindMonitor(string monitorId)
    {
        nint found = 0;
        EnumDisplayMonitors(
            0,
            0,
            (monitor, _, _, _) =>
            {
                if (string.Equals(GetMonitorInfo(monitor).DeviceName ?? string.Empty, monitorId, StringComparison.OrdinalIgnoreCase))
                {
                    found = monitor;
                    return false;
                }

                return true;
            },
            0);
        return found;
    }

    private static MonitorInfo GetMonitorInfo(nint monitor)
    {
        var info = new MonitorInfo { Size = (uint)Marshal.SizeOf<MonitorInfo>() };
        if (!GetMonitorInfoW(monitor, ref info))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read monitor work area.");
        }

        return info;
    }

    private static uint GetMonitorDpi(nint monitor) =>
        GetDpiForMonitor(monitor, 0, out var dpiX, out _) == 0 ? dpiX : 96;

    private static int Scale(int logicalPixels, uint dpi) =>
        (int)Math.Round(logicalPixels * dpi / 96d, MidpointRounding.AwayFromZero);

    [StructLayout(LayoutKind.Sequential)]
    private readonly struct Point
    {
        public readonly int X;
        public readonly int Y;
    }

    [StructLayout(LayoutKind.Sequential)]
    private readonly struct Rect
    {
        public readonly int Left;
        public readonly int Top;
        public readonly int Right;
        public readonly int Bottom;
        public int Width => Right - Left;
        public int Height => Bottom - Top;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct MonitorInfo
    {
        public uint Size;
        public Rect Monitor;
        public Rect Work;
        public uint Flags;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
        public string? DeviceName;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct TrackMouseEventParameters
    {
        public uint Size;
        public uint Flags;
        public nint WindowHandle;
        public uint HoverTime;
    }


    [DllImport("user32.dll", EntryPoint = "GetWindowLongPtrW")]
    private static extern nint GetWindowLongPtr(nint windowHandle, int index);

    [DllImport("user32.dll", EntryPoint = "SetWindowLongPtrW", SetLastError = true)]
    private static extern nint SetWindowLongPtr(nint windowHandle, int index, nint newValue);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetWindowPos(
        nint windowHandle,
        nint insertAfter,
        int x,
        int y,
        int width,
        int height,
        uint flags);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern nint FindWindowEx(
        nint parentWindowHandle,
        nint childAfter,
        string? className,
        string? windowName);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool TrackMouseEvent(ref TrackMouseEventParameters tracking);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool ReleaseCapture();

    [DllImport("user32.dll")]
    private static extern nint SendMessage(nint windowHandle, uint message, nuint wParam, nint lParam);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetCursorPos(out Point point);

    [DllImport("user32.dll")]
    private static extern nint MonitorFromPoint(Point point, uint flags);

    [DllImport("user32.dll")]
    private static extern nint MonitorFromWindow(nint windowHandle, uint flags);

    private delegate bool MonitorEnumProc(nint monitor, nint deviceContext, nint clipRectangle, nint data);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool EnumDisplayMonitors(
        nint deviceContext,
        nint clipRectangle,
        MonitorEnumProc callback,
        nint data);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetMonitorInfoW(nint monitor, ref MonitorInfo monitorInfo);

    [DllImport("shcore.dll")]
    private static extern int GetDpiForMonitor(nint monitor, int dpiType, out uint dpiX, out uint dpiY);

    [DllImport("user32.dll")]
    private static extern uint GetDpiForWindow(nint windowHandle);

    [DllImport("user32.dll")]
    private static extern nint GetForegroundWindow();

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern int GetClassName(nint windowHandle, System.Text.StringBuilder className, int maxCount);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetWindowRect(nint windowHandle, out Rect rect);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool ShowWindow(nint windowHandle, int command);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetForegroundWindow(nint windowHandle);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool IsWindowVisible(nint windowHandle);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool EnableWindow(nint windowHandle, [MarshalAs(UnmanagedType.Bool)] bool enable);
    [DllImport("user32.dll", CharSet = CharSet.Unicode, EntryPoint = "MessageBoxW")]
    private static extern int MessageBox(nint windowHandle, string text, string caption, uint type);
}
