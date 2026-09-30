using DropCove.Core;
using DropCove.Services;

namespace DropCove.Tests;

[TestClass]
public sealed class SettingsStoreTests
{
    private static readonly System.Collections.Concurrent.ConcurrentBag<string> TemporaryDirectories = [];

    [ClassCleanup]
    public static void CleanupTemporarySettings()
    {
        while (TemporaryDirectories.TryTake(out var directory))
        {
            if (Directory.Exists(directory))
            {
                Directory.Delete(directory, recursive: true);
            }
        }
    }

    [TestMethod]
    public async Task SaveAndLoad_RoundTripsEdgeRailSettings()
    {
        var directory = CreateTemporaryDirectory();
        var store = new SettingsStore(Path.Combine(directory, "settings.json"));
        var settings = new AppSettings(
            HotKeyDefinition.Default,
            false,
            ShelfRailEdge.Left,
            @"\\.\DISPLAY2",
            true);

        await store.SaveAsync(settings);
        var restored = await store.LoadAsync();

        Assert.AreEqual(settings, restored);
    }

    [TestMethod]
    public async Task Load_LegacySettingsUsesRightRailDefaults()
    {
        var directory = CreateTemporaryDirectory();
        var path = Path.Combine(directory, "settings.json");
        await File.WriteAllTextAsync(
            path,
            """
            {
              "HotKey": {
                "Control": true,
                "Alt": false,
                "Shift": true,
                "Windows": false,
                "VirtualKey": 32
              },
              "StartWithWindows": true
            }
            """);

        var restored = await new SettingsStore(path).LoadAsync();

        Assert.AreEqual(ShelfRailEdge.Right, restored.RailEdge);
        Assert.AreEqual(string.Empty, restored.RailMonitorId);
        Assert.IsFalse(restored.ShowRailOverFullscreen);
    }

    [TestMethod]
    public async Task Load_ExplicitFullscreenSetting_IsPreserved()
    {
        var directory = CreateTemporaryDirectory();
        var path = Path.Combine(directory, "settings.json");
        await File.WriteAllTextAsync(
            path,
            """
            {
              "HotKey": {
                "Control": true,
                "Alt": false,
                "Shift": true,
                "Windows": false,
                "VirtualKey": 32
              },
              "StartWithWindows": true,
              "RailEdge": "Left",
              "RailMonitorId": "\\\\.\\DISPLAY1",
              "ShowRailOverFullscreen": true
            }
            """);

        var restored = await new SettingsStore(path).LoadAsync();

        Assert.AreEqual(ShelfRailEdge.Left, restored.RailEdge);
        Assert.AreEqual(@"\\.\DISPLAY1", restored.RailMonitorId);
        Assert.IsTrue(restored.ShowRailOverFullscreen);
    }

    private static string CreateTemporaryDirectory()
    {
        var directory = Path.Combine(Path.GetTempPath(), $"DropCove-SettingsTests-{Guid.NewGuid():N}");
        Directory.CreateDirectory(directory);
        TemporaryDirectories.Add(directory);
        return directory;
    }
}
