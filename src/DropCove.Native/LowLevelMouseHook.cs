using System.ComponentModel;
using System.Runtime.InteropServices;

namespace DropCove.Native;

/// <summary>Represents one low-level mouse event needed by shake detection.</summary>
/// <param name="Message">The Win32 mouse message.</param>
/// <param name="X">The screen-space horizontal coordinate.</param>
/// <param name="Y">The screen-space vertical coordinate.</param>
/// <param name="TimestampMilliseconds">The native event timestamp.</param>
/// <param name="LeftButtonDown">Whether the left button was down when the sample was queued.</param>
public readonly record struct LowLevelMouseInput(
    uint Message,
    int X,
    int Y,
    uint TimestampMilliseconds,
    bool LeftButtonDown = false);

/// <summary>Installs a bounded global low-level mouse hook while shake-to-open is enabled.</summary>
public sealed class LowLevelMouseHook : IDisposable
{
    /// <summary>The low-level mouse hook movement message.</summary>
    public const uint MouseMoveMessage = 0x0200;
    /// <summary>The left-button press message.</summary>
    public const uint LeftButtonDownMessage = 0x0201;

    /// <summary>The left-button release message.</summary>
    public const uint LeftButtonUpMessage = 0x0202;

    private const int WhMouseLl = 14;
    private readonly LowLevelMouseHandler _handler;
    private readonly LowLevelMouseProc _callback;
    private nint _hookHandle;
    private bool _leftButtonDown;
    private bool _disposed;
    /// <summary>Installs the hook and forwards movement, left-button press, and left-button release events.</summary>
    /// <param name="handler">The minimal event callback.</param>
    public LowLevelMouseHook(LowLevelMouseHandler handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        _handler = handler;
        _callback = HandleCallback;
        _hookHandle = SetWindowsHookEx(WhMouseLl, _callback, GetModuleHandle(null), 0);
        if (_hookHandle == 0)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not install the shake mouse hook.");
        }
    }

    /// <inheritdoc />
    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }

        if (!UnhookWindowsHookEx(_hookHandle))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not remove the shake mouse hook.");
        }

        _hookHandle = 0;
        _disposed = true;
    }

    private nint HandleCallback(int code, nint wParam, nint lParam)
    {
        if (!_disposed && code >= 0 && (uint)wParam is MouseMoveMessage or LeftButtonDownMessage or LeftButtonUpMessage)
        {
            try
            {
                var message = (uint)wParam;
                if (message == LeftButtonDownMessage)
                {
                    _leftButtonDown = true;
                }
                else if (message == LeftButtonUpMessage)
                {
                    _leftButtonDown = false;
                }

                var nativeInput = Marshal.PtrToStructure<NativeMouseInput>(lParam);
                _handler(new(
                    message,
                    nativeInput.Point.X,
                    nativeInput.Point.Y,
                    nativeInput.TimestampMilliseconds,
                    _leftButtonDown));
            }
            catch
            {
                // Never let an exception escape through the unmanaged hook callback.
            }
        }

        return CallNextHookEx(_hookHandle, code, wParam, lParam);
    }

    /// <summary>Receives the minimal mouse state needed by the application queue.</summary>
    /// <param name="input">The low-level mouse event forwarded to the application queue.</param>
    public delegate void LowLevelMouseHandler(LowLevelMouseInput input);

    [StructLayout(LayoutKind.Sequential)]
    private struct NativeMouseInput
    {
        public NativePoint Point;
        public uint MouseData;
        public uint Flags;
        public uint TimestampMilliseconds;
        public nuint ExtraInfo;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct NativePoint
    {
        public int X;
        public int Y;
    }

    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    private delegate nint LowLevelMouseProc(int code, nint wParam, nint lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern nint SetWindowsHookEx(
        int hookType,
        LowLevelMouseProc callback,
        nint moduleHandle,
        uint threadId);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool UnhookWindowsHookEx(nint hookHandle);

    [DllImport("user32.dll")]
    private static extern nint CallNextHookEx(nint hookHandle, int code, nint wParam, nint lParam);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode)]
    private static extern nint GetModuleHandle(string? moduleName);
}
