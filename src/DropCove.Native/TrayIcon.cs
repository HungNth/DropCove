using System.ComponentModel;
using System.Runtime.InteropServices;

namespace DropCove.Native;

/// <summary>Hosts the DropCove notification-area icon and its native context menu.</summary>
public sealed class TrayIcon : IDisposable
{
    /// <summary>The application-defined message used for tray callbacks.</summary>
    public const uint CallbackMessage = 0x8001;

    private const uint NimAdd = 0x00000000;
    private const uint NimModify = 0x00000001;
    private const uint NimDelete = 0x00000002;
    private const uint NifMessage = 0x00000001;
    private const uint NifIcon = 0x00000002;
    private const uint NifTip = 0x00000004;
    private const uint NifInfo = 0x00000010;
    private const uint NiifWarning = 0x00000002;
    private const uint WmLButtonUp = 0x0202;
    private const uint WmLButtonDoubleClick = 0x0203;
    private const uint WmRButtonUp = 0x0205;
    private const uint MfString = 0x00000000;
    private const uint MfSeparator = 0x00000800;
    private const uint TpmRightButton = 0x0002;
    private const uint TpmReturnCommand = 0x0100;
    private const uint ImageIcon = 1;
    private const uint LrLoadFromFile = 0x0010;
    private const uint LrDefaultSize = 0x0040;
    private const int SettingsCommand = 1;
    private const int ExitCommand = 2;

    private readonly nint _windowHandle;
    private readonly Action _showShelf;
    private readonly Action _showSettings;
    private readonly Action _exit;
    private readonly bool _ownsIcon;
    private NotifyIconData _data;
    private bool _disposed;

    /// <summary>Creates and displays the DropCove tray icon.</summary>
    /// <param name="windowHandle">The HWND receiving callback messages.</param>
    /// <param name="iconPath">The application icon path.</param>
    /// <param name="showShelf">Shows the Drop Shelf.</param>
    /// <param name="showSettings">Shows Settings.</param>
    /// <param name="exit">Exits DropCove.</param>
    public TrayIcon(
        nint windowHandle,
        string iconPath,
        Action showShelf,
        Action showSettings,
        Action exit)
    {
        _windowHandle = windowHandle;
        _showShelf = showShelf;
        _showSettings = showSettings;
        _exit = exit;

        var icon = LoadImage(0, iconPath, ImageIcon, 0, 0, LrLoadFromFile | LrDefaultSize);
        _ownsIcon = icon != 0;
        if (icon == 0)
        {
            icon = LoadIcon(0, new nint(32512));
        }

        _data = CreateData(icon);
        if (!ShellNotifyIcon(NimAdd, ref _data))
        {
            if (_ownsIcon)
            {
                DestroyIcon(icon);
            }

            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not create the DropCove tray icon.");
        }
    }

    /// <summary>Processes a tray callback delivered to the owner window.</summary>
    /// <param name="lParam">The callback message parameter.</param>
    public void HandleCallback(nint lParam)
    {
        switch ((uint)lParam)
        {
            case WmLButtonUp:
            case WmLButtonDoubleClick:
                _showShelf();
                break;
            case WmRButtonUp:
                ShowContextMenu();
                break;
        }
    }

    /// <summary>Shows a tray balloon describing a recoverable resident-app problem.</summary>
    /// <param name="title">The balloon title.</param>
    /// <param name="message">The balloon message.</param>
    public void ShowWarning(string title, string message)
    {
        var data = _data;
        data.Flags = NifInfo;
        data.InfoTitle = title;
        data.Info = message;
        data.InfoFlags = NiifWarning;
        ShellNotifyIcon(NimModify, ref data);
    }

    /// <inheritdoc />
    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }

        ShellNotifyIcon(NimDelete, ref _data);
        if (_ownsIcon && _data.Icon != 0)
        {
            DestroyIcon(_data.Icon);
        }

        _disposed = true;
    }

    private NotifyIconData CreateData(nint icon) => new()
    {
        Size = (uint)Marshal.SizeOf<NotifyIconData>(),
        WindowHandle = _windowHandle,
        Id = 1,
        Flags = NifMessage | NifIcon | NifTip,
        CallbackMessage = CallbackMessage,
        Icon = icon,
        Tip = "DropCove",
        Info = string.Empty,
        InfoTitle = string.Empty,
    };

    private void ShowContextMenu()
    {
        var menu = CreatePopupMenu();
        if (menu == 0)
        {
            return;
        }

        try
        {
            AppendMenu(menu, MfString, SettingsCommand, "Settings");
            AppendMenu(menu, MfSeparator, 0, null);
            AppendMenu(menu, MfString, ExitCommand, "Exit DropCove");
            GetCursorPos(out var point);
            SetForegroundWindow(_windowHandle);
            var command = TrackPopupMenu(
                menu,
                TpmRightButton | TpmReturnCommand,
                point.X,
                point.Y,
                0,
                _windowHandle,
                0);

            if (command == SettingsCommand)
            {
                _showSettings();
            }
            else if (command == ExitCommand)
            {
                _exit();
            }
        }
        finally
        {
            DestroyMenu(menu);
        }
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct NotifyIconData
    {
        public uint Size;
        public nint WindowHandle;
        public uint Id;
        public uint Flags;
        public uint CallbackMessage;
        public nint Icon;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)]
        public string Tip;
        public uint State;
        public uint StateMask;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 256)]
        public string Info;
        public uint TimeoutOrVersion;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 64)]
        public string InfoTitle;
        public uint InfoFlags;
        public Guid GuidItem;
        public nint BalloonIcon;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct Point
    {
        public int X;
        public int Y;
    }

    [DllImport("shell32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool Shell_NotifyIcon(uint message, ref NotifyIconData data);

    private static bool ShellNotifyIcon(uint message, ref NotifyIconData data) =>
        Shell_NotifyIcon(message, ref data);

    [DllImport("user32.dll", CharSet = CharSet.Unicode, EntryPoint = "LoadImageW")]
    private static extern nint LoadImage(
        nint instance,
        string name,
        uint type,
        int width,
        int height,
        uint load);

    [DllImport("user32.dll", EntryPoint = "LoadIconW")]
    private static extern nint LoadIcon(nint instance, nint iconName);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DestroyIcon(nint icon);

    [DllImport("user32.dll")]
    private static extern nint CreatePopupMenu();

    [DllImport("user32.dll", CharSet = CharSet.Unicode, EntryPoint = "AppendMenuW")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool AppendMenu(nint menu, uint flags, nuint itemId, string? text);

    [DllImport("user32.dll")]
    private static extern uint TrackPopupMenu(
        nint menu,
        uint flags,
        int x,
        int y,
        int reserved,
        nint windowHandle,
        nint rectangle);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DestroyMenu(nint menu);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetCursorPos(out Point point);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetForegroundWindow(nint windowHandle);
}
