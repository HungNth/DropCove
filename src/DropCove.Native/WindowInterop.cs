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

/// <summary>Identifies the edge or corner being resized.</summary>
public enum WindowResizeDirection
{
    /// <summary>No resize direction.</summary>
    None = 0,
    /// <summary>Left edge.</summary>
    Left = 1,
    /// <summary>Right edge.</summary>
    Right = 2,
    /// <summary>Top edge.</summary>
    Top = 3,
    /// <summary>Top-left corner.</summary>
    TopLeft = 4,
    /// <summary>Top-right corner.</summary>
    TopRight = 5,
    /// <summary>Bottom edge.</summary>
    Bottom = 6,
    /// <summary>Bottom-left corner.</summary>
    BottomLeft = 7,
    /// <summary>Bottom-right corner.</summary>
    BottomRight = 8,
}
/// <summary>Represents a window rectangle in screen coordinates.</summary>
/// <param name="Left">Left screen coordinate.</param>
/// <param name="Top">Top screen coordinate.</param>
/// <param name="Right">Right screen coordinate.</param>
/// <param name="Bottom">Bottom screen coordinate.</param>
public readonly record struct WindowBounds(int Left, int Top, int Right, int Bottom)
{
    /// <summary>Gets the width of the bounds.</summary>
    public int Width => Right - Left;
    /// <summary>Gets the height of the bounds.</summary>
    public int Height => Bottom - Top;
}

/// <summary>Represents the result of testing cursor position against window resize borders.</summary>
/// <param name="Direction">The resize edge or corner direction, or None if in client area.</param>
/// <param name="HitCode">The Win32 hit-test code (HTLEFT, HTRIGHT, HTCLIENT, etc.).</param>
public readonly record struct WindowHitTestResult(WindowResizeDirection Direction, nint HitCode)
{
    /// <summary>Gets a result indicating the client area.</summary>
    public static WindowHitTestResult Client => new(WindowResizeDirection.None, WindowInterop.HtClient);

    /// <summary>Whether this hit test represents a resize border or corner.</summary>
    public bool IsResizeBorder => Direction != WindowResizeDirection.None;
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
    private readonly nuint _subclassId;
    private readonly nint _windowHandle;
    private readonly WindowMessageHandler _handler;
    private readonly SubclassProc _subclassProc;
    private bool _disposed;

    /// <summary>Installs a message hook for an existing HWND.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    /// <param name="handler">The message callback.</param>
    public WindowMessageHook(nint windowHandle, WindowMessageHandler handler, nuint subclassId = 1)
    {
        ArgumentNullException.ThrowIfNull(handler);
        _windowHandle = windowHandle;
        _handler = handler;
        _subclassId = subclassId;
        _subclassProc = SubclassCallback;

        if (!SetWindowSubclass(windowHandle, _subclassProc, _subclassId, 0))
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

        RemoveWindowSubclass(_windowHandle, _subclassProc, _subclassId);
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
    private readonly WinEventDelegate? _locationCallback;
    private readonly Action _onForegroundChanged;
    private readonly bool _monitorLocationChanges;
    private nint _foregroundHook;
    private nint _locationHook;
    private bool _disposed;
    private uint _locationThreadId;

    /// <summary>Installs an out-of-context hook for foreground activation and optionally foreground resize.</summary>
    /// <param name="onForegroundChanged">The callback invoked when the foreground window changes or resizes.</param>
    /// <param name="monitorLocationChanges">Whether foreground-window location changes also invoke the callback.</param>
    public ForegroundWindowHook(Action onForegroundChanged, bool monitorLocationChanges = true)
    {
        ArgumentNullException.ThrowIfNull(onForegroundChanged);
        _onForegroundChanged = onForegroundChanged;
        _monitorLocationChanges = monitorLocationChanges;
        _foregroundCallback = HandleForegroundChanged;
        _locationCallback = monitorLocationChanges ? HandleLocationChanged : null;

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

        if (!_monitorLocationChanges)
        {
            return;
        }

        try
        {
            UpdateLocationHook();
        }
        catch
        {
            UnhookWinEvent(_foregroundHook);
            throw;
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
        if (_disposed)
        {
            return;
        }

        if (_monitorLocationChanges)
        {
            try
            {
                UpdateLocationHook();
            }
            catch (Win32Exception)
            {
                _locationThreadId = 0;
                return;
            }
        }

        _onForegroundChanged();
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

    private void UpdateLocationHook()
    {
        var threadId = GetWindowThreadProcessId(GetForegroundWindow(), out _);
        if (threadId == _locationThreadId) return;

        if (_locationHook != 0)
        {
            UnhookWinEvent(_locationHook);
            _locationHook = 0;
        }

        _locationThreadId = 0;
        if (threadId == 0) return;

        _locationHook = SetWinEventHook(
            EventObjectLocationChange,
            EventObjectLocationChange,
            0,
            _locationCallback!,
            0,
            threadId,
            WineventOutOfContext);
        if (_locationHook == 0)
        {
            var error = Marshal.GetLastWin32Error();
            // A foreground transition can retire the target thread while this event is dispatched.
            if (GetWindowThreadProcessId(GetForegroundWindow(), out _) != threadId) return;
            throw new Win32Exception(error, "Could not monitor foreground-window location changes.");
        }

        _locationThreadId = threadId;
    }

    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(nint windowHandle, out uint processId);

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
    /// <summary>Win32 non-client mouse-move message.</summary>
    public const uint WmNcMouseMove = 0x00A0;
    /// <summary>Win32 display-configuration change message.</summary>
    public const uint DisplayChangeMessage = 0x007E;

    /// <summary>Win32 per-monitor DPI change message.</summary>
    public const uint DpiChangedMessage = 0x02E0;

    /// <summary>Win32 device-arrival/removal message.</summary>
    public const uint DeviceChangeMessage = 0x0219;


    /// <summary>Win32 mouse-leave message.</summary>
    public const uint MouseLeaveMessage = 0x02A3;
    /// <summary>Win32 non-client calculate size message.</summary>
    public const uint WmNcCalcSize = 0x0083;
    public const uint WmNcHitTest = 0x0084;
    /// <summary>Win32 set-cursor message.</summary>
    public const uint WmSetCursor = 0x0020;
    /// <summary>Win32 enter size/move message.</summary>
    public const uint WmEnterSizeMove = 0x0231;
    /// <summary>Win32 exit size/move message.</summary>
    public const uint WmExitSizeMove = 0x0232;
    /// <summary>Win32 sizing message.</summary>
    public const uint WmSizing = 0x0214;
    /// <summary>Win32 get min/max info message.</summary>
    public const uint WmGetMinMaxInfo = 0x0024;
    /// <summary>Win32 non-client mouse-leave message.</summary>
    public const uint WmNcMouseLeave = 0x02A2;

    /// <summary>Non-client hit test code indicating transparent.</summary>
    public const nint HtTransparent = -1;
    /// <summary>Non-client hit test code for nowhere.</summary>
    public const nint HtNowhere = 0;
    /// <summary>Non-client hit test code for client area.</summary>
    public const nint HtClient = 1;
    /// <summary>Non-client hit test code for caption area.</summary>
    public const nint HtCaptionCode = 2;
    /// <summary>Left border hit test code.</summary>
    public const nint HtLeft = 10;
    /// <summary>Right border hit test code.</summary>
    public const nint HtRight = 11;
    /// <summary>Top border hit test code.</summary>
    public const nint HtTop = 12;
    /// <summary>Top-left corner hit test code.</summary>
    public const nint HtTopLeft = 13;
    /// <summary>Top-right corner hit test code.</summary>
    public const nint HtTopRight = 14;
    /// <summary>Bottom border hit test code.</summary>
    public const nint HtBottom = 15;
    /// <summary>Bottom-left corner hit test code.</summary>
    public const nint HtBottomLeft = 16;
    /// <summary>Bottom-right corner hit test code.</summary>
    public const nint HtBottomRight = 17;

    /// <summary>WMSZ left edge.</summary>
    public const nuint WmszLeft = 1;
    /// <summary>WMSZ right edge.</summary>
    public const nuint WmszRight = 2;
    /// <summary>WMSZ top edge.</summary>
    public const nuint WmszTop = 3;
    /// <summary>WMSZ top-left corner.</summary>
    public const nuint WmszTopLeft = 4;
    /// <summary>WMSZ top-right corner.</summary>
    public const nuint WmszTopRight = 5;
    /// <summary>WMSZ bottom edge.</summary>
    public const nuint WmszBottom = 6;
    /// <summary>WMSZ bottom-left corner.</summary>
    public const nuint WmszBottomLeft = 7;
    /// <summary>WMSZ bottom-right corner.</summary>
    public const nuint WmszBottomRight = 8;

    /// <summary>Default resize border thickness in logical pixels.</summary>
    public const int DefaultResizeBorderThickness = 8;

    /// <summary>Minimum shelf dimension in logical pixels.</summary>
    public const int MinimumShelfDimension = 180;

    private static readonly nint CursorSizeWe = LoadCursorW(0, 32644);
    private static readonly nint CursorSizeNs = LoadCursorW(0, 32645);
    private static readonly nint CursorSizeNwse = LoadCursorW(0, 32642);
    private static readonly nint CursorSizeNesw = LoadCursorW(0, 32643);

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
    private const uint WmMouseActivate = 0x0021;
    private const nint MaNoActivate = 3;
    private const long WsCaption = 0x00C00000L;
    private const long WsThickFrame = 0x00040000L;
    private const long WsMinimizeBox = 0x00020000L;
    private const long WsMaximizeBox = 0x00010000L;
    private const long WsSysMenu = 0x00080000L;
    private const long WsExNoActivate = 0x08000000L;
    private const uint SpiGetClientAreaAnimation = 0x1042;

    private const long WsExToolWindow = 0x00000080L;

    /// <summary>Removes standard caption and resize chrome from the HWND.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    public static void MakeBorderless(nint windowHandle) => MakeBorderless(windowHandle, resizable: false);

    /// <summary>Removes standard caption and chrome from the HWND, optionally retaining sizing borders.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    /// <param name="resizable">Whether to retain sizing borders.</param>
    public static void MakeBorderless(nint windowHandle, bool resizable)
    {
        var style = GetWindowLongPtr(windowHandle, GwlStyle).ToInt64();
        style &= ~(WsCaption | WsMinimizeBox | WsMaximizeBox | WsSysMenu);
        if (resizable)
        {
            style |= WsThickFrame;
        }
        else
        {
            style &= ~WsThickFrame;
        }

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

    /// <summary>Removes standard caption and chrome while retaining resizable borders for the HWND.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    public static void MakeResizableBorderless(nint windowHandle) => MakeBorderless(windowHandle, resizable: true);

    /// <summary>Requests system-rounded corners without overriding DWM policy.</summary>
    /// <param name="windowHandle">The target HWND after its window styles are applied.</param>
    /// <returns><see langword="true" /> if DWM accepts the hint; otherwise, <see langword="false" />.</returns>
    /// <remarks>An accepted hint does not guarantee visible rounding in policy-excluded window states.</remarks>
    public static bool TryRoundWindowCorners(nint windowHandle)
    {
        const int cornerPreferenceAttribute = 33; // DWMWA_WINDOW_CORNER_PREFERENCE
        var preference = 2; // DWMWCP_ROUND
        return DwmSetWindowAttribute(windowHandle, cornerPreferenceAttribute, ref preference, sizeof(int)) >= 0;
    }

    /// <summary>Win32 erase-background message.</summary>
    public const uint EraseBackgroundMessage = 0x0014;

    /// <summary>Lets DWM compose the client area with per-pixel alpha so XAML content can antialias against the desktop.</summary>
    /// <param name="windowHandle">The borderless overlay HWND.</param>
    /// <remarks>Pair with <see cref="ClearTransparentBackground"/> on <see cref="EraseBackgroundMessage"/>; GDI black is transparent under blur-behind.</remarks>
    public static void EnableTransparentSurface(nint windowHandle)
    {
        var margins = default(Margins);
        Marshal.ThrowExceptionForHR(DwmExtendFrameIntoClientArea(windowHandle, ref margins));
        var emptyRegion = CreateRectRgn(-2, -2, -1, -1);
        try
        {
            var blurBehind = new DwmBlurBehind { Flags = DwmBbEnable | DwmBbBlurRegion, Enable = true, BlurRegion = emptyRegion };
            Marshal.ThrowExceptionForHR(DwmEnableBlurBehindWindow(windowHandle, ref blurBehind));
        }
        finally
        {
            DeleteObject(emptyRegion);
        }
    }

    /// <summary>Clears the client area to transparent black for a surface prepared by <see cref="EnableTransparentSurface"/>.</summary>
    /// <param name="windowHandle">The overlay HWND.</param>
    /// <param name="deviceContext">The device context supplied by <see cref="EraseBackgroundMessage"/>.</param>
    /// <returns>A handled result that suppresses the default opaque erase.</returns>
    public static WindowMessageResult ClearTransparentBackground(nint windowHandle, nint deviceContext)
    {
        if (GetClientRect(windowHandle, out var clientRect))
        {
            FillRect(deviceContext, ref clientRect, GetStockObject(BlackBrush));
        }

        return new WindowMessageResult(true, 1);
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
        FindWindowEx(windowHandle, 0, "Microsoft.UI.Content.DesktopChildSiteBridge", null);

    /// <summary>Determines whether the pointer hits a window or one of its native content children.</summary>
    /// <param name="windowHandle">The top-level window HWND.</param>
    /// <returns><see langword="true" /> if the pointer hits the window; otherwise, <see langword="false" />.</returns>
    public static bool IsPointerOverWindow(nint windowHandle) =>
        GetCursorPos(out var cursor) && GetAncestor(WindowFromPoint(cursor), 2) == windowHandle;

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

    /// <summary>Gets the fitted logical shelf width on the cursor monitor without moving a window.</summary>
    /// <param name="logicalWidth">The preferred logical width.</param>
    /// <returns>The physical fitted width expressed in logical pixels.</returns>
    public static double GetOpeningWidthOnCursorMonitor(int logicalWidth)
    {
        if (!GetCursorPos(out var cursor))
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read the cursor position.");
        return GetOpeningWidth(MonitorFromPoint(cursor, MonitorDefaultToNearest), logicalWidth);
    }

    /// <summary>Gets fitted width on a remembered monitor, using the window monitor if that display disconnects.</summary>
    /// <param name="windowHandle">The window supplying the fallback monitor.</param>
    /// <param name="monitorId">The resolved connected monitor device name.</param>
    /// <param name="logicalWidth">The preferred logical width.</param>
    /// <returns>The physical fitted width expressed in logical pixels.</returns>
    public static double GetOpeningWidthOnMonitor(nint windowHandle, string monitorId, int logicalWidth)
    {
        var monitor = FindMonitor(monitorId);
        if (monitor == 0) monitor = MonitorFromWindow(windowHandle, MonitorDefaultToNearest);
        return GetOpeningWidth(monitor, logicalWidth);
    }

    /// <summary>Gets the fitted logical shelf width on the monitor containing a screen point.</summary>
    /// <param name="screenX">The screen horizontal coordinate.</param>
    /// <param name="screenY">The screen vertical coordinate.</param>
    /// <param name="logicalWidth">The preferred logical width.</param>
    /// <returns>The physical fitted width expressed in logical pixels.</returns>
    public static double GetOpeningWidthNearPoint(int screenX, int screenY, int logicalWidth) =>
        GetOpeningWidth(MonitorFromPoint(new Point(screenX, screenY), MonitorDefaultToNearest), logicalWidth);

    private static double GetOpeningWidth(nint monitor, int logicalWidth)
    {
        if (monitor == 0) throw new Win32Exception("Could not resolve the opening monitor.");
        var dpi = GetMonitorDpi(monitor);
        var (width, _) = FitToWorkArea(logicalWidth, MinimumShelfDimension, dpi, GetMonitorInfo(monitor).Work);
        return width * 96.0 / dpi;
    }

    /// <summary>Positions the shelf around the cursor on its current monitor.</summary>
    /// <param name="windowHandle">The shelf window.</param>
    /// <param name="logicalWidth">Width in logical pixels.</param>
    /// <param name="logicalHeight">Height in logical pixels.</param>
    /// <summary>Positions the shelf around a specific screen coordinate on its current monitor.</summary>
    /// <param name="windowHandle">The shelf window.</param>
    /// <param name="screenX">Screen horizontal coordinate.</param>
    /// <param name="screenY">Screen vertical coordinate.</param>
    /// <param name="logicalWidth">Width in logical pixels.</param>
    /// <param name="logicalHeight">Height in logical pixels.</param>
    public static void PositionNearPoint(nint windowHandle, int screenX, int screenY, int logicalWidth, int logicalHeight)
    {
        ValidateLogicalSize(logicalWidth, logicalHeight);
        var target = new Point(screenX, screenY);
        var monitor = MonitorFromPoint(target, MonitorDefaultToNearest);
        if (monitor == 0)
        {
            throw new Win32Exception("Could not resolve the target point monitor.");
        }

        var monitorInfo = GetMonitorInfo(monitor);
        var (width, height) = FitToWorkArea(logicalWidth, logicalHeight, GetMonitorDpi(monitor), monitorInfo.Work);
        var x = Math.Clamp(screenX - width / 2, monitorInfo.Work.Left, monitorInfo.Work.Right - width);
        var y = Math.Clamp(screenY - height / 2, monitorInfo.Work.Top, monitorInfo.Work.Bottom - height);
        SetBounds(windowHandle, x, y, width, height, SwpNoActivate);
    }

    /// <summary>Positions the shelf around the cursor on its current monitor.</summary>
    /// <param name="windowHandle">The shelf window.</param>
    /// <param name="logicalWidth">Width in logical pixels.</param>
    /// <param name="logicalHeight">Height in logical pixels.</param>
    public static void PositionNearCursor(nint windowHandle, int logicalWidth, int logicalHeight)
    {
        ValidateLogicalSize(logicalWidth, logicalHeight);
        if (!GetCursorPos(out var cursor))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read the cursor position.");
        }

        PositionNearPoint(windowHandle, cursor.X, cursor.Y, logicalWidth, logicalHeight);
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
    /// <summary>Resolves a remembered monitor to a connected display or a safe fallback.</summary>
    /// <param name="preferredMonitorId">The remembered display device name.</param>
    /// <param name="fallbackWindowHandle">A window whose current monitor is used when the remembered display is unavailable.</param>
    /// <returns>A connected display device name.</returns>
    public static string ResolveMonitorId(string? preferredMonitorId, nint fallbackWindowHandle)
    {
        if (!string.IsNullOrWhiteSpace(preferredMonitorId) && FindMonitor(preferredMonitorId) != 0)
        {
            return preferredMonitorId;
        }

        if (fallbackWindowHandle != 0)
        {
            try
            {
                return GetMonitorIdForWindow(fallbackWindowHandle);
            }
            catch (Win32Exception)
            {
            }
        }

        var fallback = GetMonitorOptions().FirstOrDefault()?.Id;
        return !string.IsNullOrWhiteSpace(fallback)
            ? fallback
            : throw new InvalidOperationException("No connected display is available.");
    }

    /// <summary>Gets the effective DPI for a native window, defaulting to 96 when Windows cannot provide it.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    /// <returns>The effective horizontal DPI.</returns>
    public static uint GetWindowDpi(nint windowHandle)
    {
        var dpi = GetDpiForWindow(windowHandle);
        return dpi == 0 ? 96u : dpi;
    }

    /// <summary>Scales a logical pixel value using a Windows effective DPI.</summary>
    /// <param name="logicalPixels">The logical pixel value.</param>
    /// <param name="dpi">The effective DPI.</param>
    /// <returns>The rounded physical pixel value.</returns>
    public static int ScaleLogicalPixels(int logicalPixels, uint dpi) => Scale(logicalPixels, dpi);

    /// <summary>Returns whether Windows allows non-essential client-area animations.</summary>
    /// <returns><see langword="true" /> when animations should run; otherwise <see langword="false" />.</returns>
    public static bool AreAnimationsEnabled()
    {
        var enabled = 1;
        return SystemParametersInfo(SpiGetClientAreaAnimation, 0, ref enabled, 0)
            ? enabled != 0
            : true;
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
                    var scalingPercent = (int)Math.Round(GetMonitorDpi(monitor) * 100d / 96d, MidpointRounding.AwayFromZero);
                    monitors.Add(new MonitorOption(
                        id,
                        $"{id} ({info.Monitor.Width} × {info.Monitor.Height}, {scalingPercent}% scaling)"));
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

    /// <summary>Clips only the desktop-facing corners of an Edge Rail window.</summary>
    /// <param name="windowHandle">The rail HWND.</param>
    /// <param name="width">Actual window width in physical pixels.</param>
    /// <param name="height">Actual window height in physical pixels.</param>
    /// <param name="radius">Corner radius in physical pixels.</param>
    /// <param name="dockLeft">Whether the square edge is on the left.</param>
    public static void SetEdgeRailCorners(nint windowHandle, int width, int height, int radius, bool dockLeft)
    {
        var rounded = CreateRoundRectRgn(0, 0, width + 1, height + 1, radius * 2, radius * 2);
        var square = CreateRectRgn(dockLeft ? 0 : width / 2, 0, dockLeft ? (width + 1) / 2 : width, height);
        try
        {
            if (rounded == 0 || square == 0 || CombineRgn(rounded, rounded, square, 2) == 0 || SetWindowRgn(windowHandle, rounded, true) == 0)
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not apply Edge Rail corner geometry.");
            rounded = 0; // Successful SetWindowRgn transfers ownership to Windows.
        }
        finally
        {
            if (rounded != 0) DeleteObject(rounded);
            if (square != 0) DeleteObject(square);
        }
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
    /// <param name="logicalHeight">Height in logical pixels.</param>
    public static void ResizeAnchored(nint windowHandle, int logicalHeight)
    {
        ValidateLogicalSize(MinimumShelfDimension, logicalHeight);
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

        var target = CalculateAnchoredGrowthBounds(
            new WindowBounds(current.Left, current.Top, current.Right, current.Bottom),
            new WindowBounds(monitorInfo.Work.Left, monitorInfo.Work.Top, monitorInfo.Work.Right, monitorInfo.Work.Bottom),
            logicalHeight, dpi);
        SetBounds(windowHandle, target.Left, target.Top, target.Width, target.Height, SwpNoActivate);
    }

    /// <summary>Calculates height-only growth, preserving the top edge unless reachability requires correction.</summary>
    /// <param name="current">The current physical window bounds.</param>
    /// <param name="workArea">The current physical monitor work area.</param>
    /// <param name="logicalHeight">The requested logical height.</param>
    /// <param name="dpi">The monitor DPI.</param>
    /// <returns>The reachable physical target bounds.</returns>
    public static WindowBounds CalculateAnchoredGrowthBounds(
        WindowBounds current, WindowBounds workArea, int logicalHeight, uint dpi)
    {
        var width = Math.Min(current.Width, workArea.Width);
        var height = Math.Min(ScaleLogicalPixels(logicalHeight, dpi), workArea.Height);
        var x = Math.Clamp(current.Left, workArea.Left, workArea.Right - width);
        var y = Math.Clamp(current.Top, workArea.Top, workArea.Bottom - height);
        return new WindowBounds(x, y, x + width, y + height);
    }
    /// <summary>Tests screen coordinates against the window's outer border to detect resize targets.</summary>
    /// <param name="bounds">The window bounds in screen coordinates.</param>
    /// <param name="screenX">Horizontal screen coordinate.</param>
    /// <param name="screenY">Vertical screen coordinate.</param>
    /// <param name="borderThickness">Border thickness in physical pixels.</param>
    /// <returns>The hit-test result including direction and Win32 hit code.</returns>
    public static WindowHitTestResult HitTestBorder(
        WindowBounds bounds,
        int screenX,
        int screenY,
        int borderThickness)
    {
        if (screenX < bounds.Left || screenX >= bounds.Right ||
            screenY < bounds.Top || screenY >= bounds.Bottom)
        {
            return new WindowHitTestResult(WindowResizeDirection.None, HtNowhere);
        }

        var onLeft = screenX < bounds.Left + borderThickness;
        var onRight = screenX >= bounds.Right - borderThickness;
        var onTop = screenY < bounds.Top + borderThickness;
        var onBottom = screenY >= bounds.Bottom - borderThickness;

        if (onTop && onLeft) return new WindowHitTestResult(WindowResizeDirection.TopLeft, HtTopLeft);
        if (onTop && onRight) return new WindowHitTestResult(WindowResizeDirection.TopRight, HtTopRight);
        if (onBottom && onLeft) return new WindowHitTestResult(WindowResizeDirection.BottomLeft, HtBottomLeft);
        if (onBottom && onRight) return new WindowHitTestResult(WindowResizeDirection.BottomRight, HtBottomRight);
        if (onLeft) return new WindowHitTestResult(WindowResizeDirection.Left, HtLeft);
        if (onRight) return new WindowHitTestResult(WindowResizeDirection.Right, HtRight);
        if (onTop) return new WindowHitTestResult(WindowResizeDirection.Top, HtTop);
        if (onBottom) return new WindowHitTestResult(WindowResizeDirection.Bottom, HtBottom);

        return WindowHitTestResult.Client;
    }

    /// <summary>Converts a Win32 hit-test code to a window resize direction.</summary>
    /// <param name="hitCode">The Win32 hit code (e.g. HTLEFT, HTRIGHT, etc.).</param>
    /// <returns>The corresponding <see cref="WindowResizeDirection"/>.</returns>
    public static WindowResizeDirection HitTestCodeToDirection(nint hitCode) => hitCode switch
    {
        HtLeft => WindowResizeDirection.Left,
        HtRight => WindowResizeDirection.Right,
        HtTop => WindowResizeDirection.Top,
        HtTopLeft => WindowResizeDirection.TopLeft,
        HtTopRight => WindowResizeDirection.TopRight,
        HtBottom => WindowResizeDirection.Bottom,
        HtBottomLeft => WindowResizeDirection.BottomLeft,
        HtBottomRight => WindowResizeDirection.BottomRight,
        _ => WindowResizeDirection.None,
    };

    /// <summary>Converts a <see cref="WindowResizeDirection"/> to the corresponding Win32 hit-test code.</summary>
    /// <param name="direction">The resize direction.</param>
    /// <returns>The Win32 hit code.</returns>
    public static nint ToHitTestCode(WindowResizeDirection direction) => direction switch
    {
        WindowResizeDirection.Left => HtLeft,
        WindowResizeDirection.Right => HtRight,
        WindowResizeDirection.Top => HtTop,
        WindowResizeDirection.TopLeft => HtTopLeft,
        WindowResizeDirection.TopRight => HtTopRight,
        WindowResizeDirection.Bottom => HtBottom,
        WindowResizeDirection.BottomLeft => HtBottomLeft,
        WindowResizeDirection.BottomRight => HtBottomRight,
        _ => HtClient,
    };

    /// <summary>Sets the Win32 cursor appropriate for the given resize direction.</summary>
    /// <param name="direction">The resize direction.</param>
    /// <returns>True if a resize cursor was set; false if the direction is None.</returns>
    public static bool SetResizeCursor(WindowResizeDirection direction)
    {
        var cursor = direction switch
        {
            WindowResizeDirection.Left or WindowResizeDirection.Right => CursorSizeWe,
            WindowResizeDirection.Top or WindowResizeDirection.Bottom => CursorSizeNs,
            WindowResizeDirection.TopLeft or WindowResizeDirection.BottomRight => CursorSizeNwse,
            WindowResizeDirection.TopRight or WindowResizeDirection.BottomLeft => CursorSizeNesw,
            _ => 0,
        };

        if (cursor != 0)
        {
            SetCursor(cursor);
            return true;
        }

        return false;
    }

    /// <summary>Gets the current window rectangle in screen coordinates.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    /// <returns>The window bounds.</returns>
    public static WindowBounds GetWindowBounds(nint windowHandle)
    {
        if (!GetWindowRect(windowHandle, out var rect))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read window bounds.");
        }

        return new WindowBounds(rect.Left, rect.Top, rect.Right, rect.Bottom);
    }

    /// <summary>Gets the work area rectangle for the monitor containing the window.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    /// <returns>The monitor work area bounds.</returns>
    public static WindowBounds GetMonitorWorkAreaForWindow(nint windowHandle)
    {
        var monitor = MonitorFromWindow(windowHandle, MonitorDefaultToNearest);
        if (monitor == 0)
        {
            throw new Win32Exception("Could not resolve window monitor.");
        }

        var info = GetMonitorInfo(monitor);
        return new WindowBounds(info.Work.Left, info.Work.Top, info.Work.Right, info.Work.Bottom);
    }

    /// <summary>Constrains candidate resize bounds to minimum dimensions and monitor work area.</summary>
    /// <param name="candidate">The proposed window rectangle.</param>
    /// <param name="direction">The active resize direction (or WMSZ code cast to direction).</param>
    /// <param name="minLogicalWidth">Minimum width in logical pixels.</param>
    /// <param name="minLogicalHeight">Minimum height in logical pixels.</param>
    /// <param name="dpi">Effective monitor DPI.</param>
    /// <param name="workArea">Monitor work area.</param>
    /// <returns>Constrained bounds.</returns>
    public static WindowBounds ConstrainResizeBounds(
        WindowBounds candidate,
        WindowResizeDirection direction,
        int minLogicalWidth,
        int minLogicalHeight,
        uint dpi,
        WindowBounds workArea)
    {
        var workAreaWidth = Math.Max(1, workArea.Width);
        var workAreaHeight = Math.Max(1, workArea.Height);
        var minWidth = Math.Min(Scale(minLogicalWidth, dpi), workAreaWidth);
        var minHeight = Math.Min(Scale(minLogicalHeight, dpi), workAreaHeight);

        var left = candidate.Left;
        var top = candidate.Top;
        var right = candidate.Right;
        var bottom = candidate.Bottom;

        var isLeft = direction is WindowResizeDirection.Left or WindowResizeDirection.TopLeft or WindowResizeDirection.BottomLeft;
        var isTop = direction is WindowResizeDirection.Top or WindowResizeDirection.TopLeft or WindowResizeDirection.TopRight;

        if (right - left < minWidth)
        {
            if (isLeft)
            {
                left = right - minWidth;
            }
            else
            {
                right = left + minWidth;
            }
        }

        if (bottom - top < minHeight)
        {
            if (isTop)
            {
                top = bottom - minHeight;
            }
            else
            {
                bottom = top + minHeight;
            }
        }

        // Clamp to work area
        if (left < workArea.Left)
        {
            left = workArea.Left;
            if (right - left < minWidth && workAreaWidth >= minWidth)
            {
                right = left + minWidth;
            }
        }

        if (right > workArea.Right)
        {
            right = workArea.Right;
            if (right - left < minWidth && workAreaWidth >= minWidth)
            {
                left = right - minWidth;
            }
        }

        if (top < workArea.Top)
        {
            top = workArea.Top;
            if (bottom - top < minHeight && workAreaHeight >= minHeight)
            {
                bottom = top + minHeight;
            }
        }

        if (bottom > workArea.Bottom)
        {
            bottom = workArea.Bottom;
            if (bottom - top < minHeight && workAreaHeight >= minHeight)
            {
                top = bottom - minHeight;
            }
        }

        return new WindowBounds(left, top, right, bottom);
    }

    /// <summary>Handles WM_GETMINMAXINFO to enforce minimum physical size on the window.</summary>
    /// <param name="lParam">Pointer to MINMAXINFO structure.</param>
    /// <param name="minLogicalWidth">Minimum width in logical pixels.</param>
    /// <param name="minLogicalHeight">Minimum height in logical pixels.</param>
    /// <param name="dpi">Window DPI.</param>
    /// <param name="workArea">Monitor work area bounds.</param>
    public static void HandleGetMinMaxInfo(
        nint lParam,
        int minLogicalWidth,
        int minLogicalHeight,
        uint dpi,
        WindowBounds workArea)
    {
        if (lParam == 0)
        {
            return;
        }

        var minWidth = Math.Min(Scale(minLogicalWidth, dpi), Math.Max(1, workArea.Width));
        var minHeight = Math.Min(Scale(minLogicalHeight, dpi), Math.Max(1, workArea.Height));

        var info = Marshal.PtrToStructure<MinMaxInfo>(lParam);
        info.MinTrackSize = new Point(minWidth, minHeight);
        Marshal.StructureToPtr(info, lParam, false);
    }

    /// <summary>Handles WM_SIZING to dynamically clamp the sizing rectangle to minimums and monitor work area.</summary>
    /// <param name="wParam">WMSZ edge code.</param>
    /// <param name="lParam">Pointer to RECT structure.</param>
    /// <param name="minLogicalWidth">Minimum width in logical pixels.</param>
    /// <param name="minLogicalHeight">Minimum height in logical pixels.</param>
    /// <param name="dpi">Window DPI.</param>
    /// <param name="workArea">Monitor work area bounds.</param>
    public static void HandleSizing(
        nuint wParam,
        nint lParam,
        int minLogicalWidth,
        int minLogicalHeight,
        uint dpi,
        WindowBounds workArea)
    {
        if (lParam == 0)
        {
            return;
        }

        var rect = Marshal.PtrToStructure<Rect>(lParam);
        var direction = (WindowResizeDirection)wParam;
        var candidate = new WindowBounds(rect.Left, rect.Top, rect.Right, rect.Bottom);
        var constrained = ConstrainResizeBounds(candidate, direction, minLogicalWidth, minLogicalHeight, dpi, workArea);

        rect.Left = constrained.Left;
        rect.Top = constrained.Top;
        rect.Right = constrained.Right;
        rect.Bottom = constrained.Bottom;
        Marshal.StructureToPtr(rect, lParam, false);
    }

    /// <summary>Unscales physical pixels back to logical pixels for the given DPI.</summary>
    /// <param name="physicalPixels">Physical pixel count.</param>
    /// <param name="dpi">Effective DPI.</param>
    /// <returns>Logical pixels.</returns>
    public static int UnscalePhysicalPixels(int physicalPixels, uint dpi) =>
        (int)Math.Round(physicalPixels * 96d / (dpi == 0 ? 96u : dpi), MidpointRounding.AwayFromZero);

    /// <summary>Places an anchored popup relative to an anchor card within the monitor work area.</summary>
    /// <param name="anchor">The physical screen bounds of the anchor card.</param>
    /// <param name="workArea">The physical screen bounds of the monitor work area.</param>
    /// <param name="requestedLogicalWidth">The requested popup width in logical pixels.</param>
    /// <param name="requestedLogicalHeight">The requested popup height in logical pixels.</param>
    /// <param name="dpi">The effective monitor DPI.</param>
    /// <returns>The calculated physical screen bounds for the popup.</returns>
    public static WindowBounds PlaceAnchoredPopup(
        WindowBounds anchor,
        WindowBounds workArea,
        int requestedLogicalWidth,
        int requestedLogicalHeight,
        uint dpi)
    {
        var effectiveDpi = dpi == 0 ? 96u : dpi;
        var workAreaWidth = Math.Max(0, workArea.Width);
        var workAreaHeight = Math.Max(0, workArea.Height);

        const int minLogicalWidth = 320;
        const int maxLogicalWidth = 480;
        const int minLogicalHeight = 1;
        const int maxLogicalHeight = 480;

        var preferredMargin = Scale(16, effectiveDpi);
        var marginX = Math.Min(preferredMargin, Math.Max(0, (workAreaWidth - 1) / 2));
        var marginY = Math.Min(preferredMargin, Math.Max(0, (workAreaHeight - 1) / 2));
        var availableWidth = workAreaWidth - 2 * marginX;
        var availableHeight = workAreaHeight - 2 * marginY;

        var minPhysicalWidth = Math.Min(Scale(minLogicalWidth, effectiveDpi), availableWidth);
        var maxPhysicalWidth = Math.Min(Scale(maxLogicalWidth, effectiveDpi), availableWidth);
        var targetPhysicalWidth = Math.Clamp(Scale(requestedLogicalWidth, effectiveDpi), minPhysicalWidth, maxPhysicalWidth);

        var minPhysicalHeight = Math.Min(Scale(minLogicalHeight, effectiveDpi), availableHeight);
        var maxPhysicalHeight = Math.Min(Scale(maxLogicalHeight, effectiveDpi), availableHeight);
        var targetPhysicalHeight = Math.Clamp(Scale(requestedLogicalHeight, effectiveDpi), minPhysicalHeight, maxPhysicalHeight);

        var safeLeft = workArea.Left + marginX;
        var safeRight = workArea.Right - marginX;
        var safeTop = workArea.Top + marginY;
        var safeBottom = workArea.Bottom - marginY;

        // Candidate 1: Right of anchor, top-aligned with anchor
        var rightCandidate = new WindowBounds(
            anchor.Right,
            anchor.Top,
            anchor.Right + targetPhysicalWidth,
            anchor.Top + targetPhysicalHeight);

        if (rightCandidate.Left >= safeLeft &&
            rightCandidate.Right <= safeRight &&
            rightCandidate.Top >= safeTop &&
            rightCandidate.Bottom <= safeBottom)
        {
            return rightCandidate;
        }

        // Candidate 2: Left of anchor, top-aligned with anchor
        var leftCandidate = new WindowBounds(
            anchor.Left - targetPhysicalWidth,
            anchor.Top,
            anchor.Left,
            anchor.Top + targetPhysicalHeight);

        if (leftCandidate.Left >= safeLeft &&
            leftCandidate.Right <= safeRight &&
            leftCandidate.Top >= safeTop &&
            leftCandidate.Bottom <= safeBottom)
        {
            return leftCandidate;
        }

        // Candidate 3: Below anchor, left-aligned with anchor
        var belowCandidate = new WindowBounds(
            anchor.Left,
            anchor.Bottom,
            anchor.Left + targetPhysicalWidth,
            anchor.Bottom + targetPhysicalHeight);

        if (belowCandidate.Left >= safeLeft &&
            belowCandidate.Right <= safeRight &&
            belowCandidate.Top >= safeTop &&
            belowCandidate.Bottom <= safeBottom)
        {
            return belowCandidate;
        }

        // Candidate 4: Above anchor, left-aligned with anchor
        var aboveCandidate = new WindowBounds(
            anchor.Left,
            anchor.Top - targetPhysicalHeight,
            anchor.Left + targetPhysicalWidth,
            anchor.Top);

        if (aboveCandidate.Left >= safeLeft &&
            aboveCandidate.Right <= safeRight &&
            aboveCandidate.Top >= safeTop &&
            aboveCandidate.Bottom <= safeBottom)
        {
            return aboveCandidate;
        }

        // Fallback: Clamp the right candidate within the work area respecting margins where possible.
        var clampedMinX = workArea.Left + marginX;
        var clampedMaxX = workArea.Right - marginX - targetPhysicalWidth;
        var clampedLeft = Math.Clamp(rightCandidate.Left, clampedMinX, clampedMaxX);

        var clampedMinY = workArea.Top + marginY;
        var clampedMaxY = workArea.Bottom - marginY - targetPhysicalHeight;
        var clampedTop = Math.Clamp(rightCandidate.Top, clampedMinY, clampedMaxY);

        return new WindowBounds(
            clampedLeft,
            clampedTop,
            clampedLeft + targetPhysicalWidth,
            clampedTop + targetPhysicalHeight);
    }

    /// <summary>Sets the window position and size explicitly, clamped to work area.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    /// <param name="logicalWidth">Width in logical pixels.</param>
    /// <param name="logicalHeight">Height in logical pixels.</param>
    public static void ResizeAndClamp(nint windowHandle, int logicalWidth, int logicalHeight)
    {
        ValidateLogicalSize(logicalWidth, logicalHeight);
        if (!GetWindowRect(windowHandle, out var current))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read window bounds.");
        }

        var monitor = MonitorFromWindow(windowHandle, MonitorDefaultToNearest);
        if (monitor == 0)
        {
            throw new Win32Exception("Could not resolve window monitor.");
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

    /// <summary>Reasserts the window's topmost z-order without activating or moving it.</summary>
    /// <param name="windowHandle">The target HWND.</param>
    public static void ReassertTopmost(nint windowHandle) =>
        SetWindowPos(windowHandle, HwndTopmost, 0, 0, 0, 0, SwpNoMove | SwpNoSize | SwpNoActivate);

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

        public Point(int x, int y)
        {
            X = x;
            Y = y;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct Rect
    {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
        public int Width => Right - Left;
        public int Height => Bottom - Top;

        public Rect(int left, int top, int right, int bottom)
        {
            Left = left;
            Top = top;
            Right = right;
            Bottom = bottom;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct MinMaxInfo
    {
        public Point Reserved;
        public Point MaxSize;
        public Point MaxPosition;
        public Point MinTrackSize;
        public Point MaxTrackSize;
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


    [DllImport("dwmapi.dll")]
    private static extern int DwmSetWindowAttribute(nint windowHandle, int attribute, ref int value, int size);

    private const uint DwmBbEnable = 0x1;
    private const uint DwmBbBlurRegion = 0x2;
    private const int BlackBrush = 4;

    [StructLayout(LayoutKind.Sequential)]
    private struct Margins
    {
        public int Left;
        public int Right;
        public int Top;
        public int Bottom;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct DwmBlurBehind
    {
        public uint Flags;
        [MarshalAs(UnmanagedType.Bool)]
        public bool Enable;
        public nint BlurRegion;
        [MarshalAs(UnmanagedType.Bool)]
        public bool TransitionOnMaximized;
    }

    [DllImport("dwmapi.dll")]
    private static extern int DwmExtendFrameIntoClientArea(nint windowHandle, ref Margins margins);

    [DllImport("dwmapi.dll")]
    private static extern int DwmEnableBlurBehindWindow(nint windowHandle, ref DwmBlurBehind blurBehind);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetClientRect(nint windowHandle, out Rect rect);

    [DllImport("user32.dll")]
    private static extern int FillRect(nint deviceContext, ref Rect rect, nint brush);

    [DllImport("gdi32.dll")]
    private static extern nint GetStockObject(int index);

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
    private static extern bool GetCursorPos(out Point point);

    [DllImport("user32.dll")]
    private static extern nint WindowFromPoint(Point point);

    [DllImport("user32.dll")]
    private static extern nint GetAncestor(nint windowHandle, uint flags);

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

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SystemParametersInfo(
        uint action,
        uint parameter,
        ref int value,
        uint updateFlags);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool EnableWindow(nint windowHandle, [MarshalAs(UnmanagedType.Bool)] bool enable);

    [DllImport("user32.dll", CharSet = CharSet.Unicode, EntryPoint = "MessageBoxW")]
    private static extern int MessageBox(nint windowHandle, string text, string caption, uint type);

    [DllImport("user32.dll")]
    private static extern nint LoadCursorW(nint hInstance, int lpCursorName);

    [DllImport("user32.dll")]
    private static extern nint SetCursor(nint hCursor);
    [DllImport("gdi32.dll")]
    private static extern nint CreateRoundRectRgn(int left, int top, int right, int bottom, int ellipseWidth, int ellipseHeight);

    [DllImport("gdi32.dll")]
    private static extern nint CreateRectRgn(int left, int top, int right, int bottom);

    [DllImport("gdi32.dll")]
    private static extern int CombineRgn(nint destination, nint source1, nint source2, int mode);

    [DllImport("gdi32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DeleteObject(nint value);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern int SetWindowRgn(nint windowHandle, nint region, [MarshalAs(UnmanagedType.Bool)] bool redraw);
}
