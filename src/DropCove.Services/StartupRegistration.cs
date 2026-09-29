using System.Runtime.Versioning;
using Microsoft.Win32;

namespace DropCove.Services;

/// <summary>Maintains the current user's DropCove sign-in startup registration.</summary>
[SupportedOSPlatform("windows")]
public static class StartupRegistration
{
    private const string RunKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string ValueName = "DropCove";

    /// <summary>Enables or disables startup for the current executable.</summary>
    /// <param name="enabled">Whether DropCove should start at sign-in.</param>
    public static void SetEnabled(bool enabled)
    {
        using var runKey = Registry.CurrentUser.OpenSubKey(RunKeyPath, writable: true)
            ?? Registry.CurrentUser.CreateSubKey(RunKeyPath, writable: true);

        if (!enabled)
        {
            runKey.DeleteValue(ValueName, throwOnMissingValue: false);
            return;
        }

        var executablePath = Environment.ProcessPath
            ?? throw new InvalidOperationException("The DropCove executable path is unavailable.");
        runKey.SetValue(ValueName, $"\"{executablePath}\" --autostart", RegistryValueKind.String);
    }
}
