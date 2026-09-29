namespace DropCove.Core;

/// <summary>Defines a configurable global hotkey.</summary>
/// <param name="Control">Whether Control is required.</param>
/// <param name="Alt">Whether Alt is required.</param>
/// <param name="Shift">Whether Shift is required.</param>
/// <param name="Windows">Whether the Windows key is required.</param>
/// <param name="VirtualKey">The Win32 virtual-key code.</param>
public readonly record struct HotKeyDefinition(
    bool Control,
    bool Alt,
    bool Shift,
    bool Windows,
    uint VirtualKey)
{
    /// <summary>Gets the default Ctrl+Shift+Space hotkey.</summary>
    public static HotKeyDefinition Default { get; } = new(true, false, true, false, 0x20);
}

/// <summary>Contains the user-configurable resident utility settings.</summary>
/// <param name="HotKey">The configured global hotkey.</param>
/// <param name="StartWithWindows">Whether DropCove starts when the user signs in.</param>
public sealed record AppSettings(HotKeyDefinition HotKey, bool StartWithWindows)
{
    /// <summary>Gets the default settings for a new user.</summary>
    public static AppSettings Default { get; } = new(HotKeyDefinition.Default, true);
}
