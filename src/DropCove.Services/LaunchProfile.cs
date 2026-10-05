using System.Security.Cryptography;
using System.Text;

namespace DropCove.Services;

/// <summary>Resolves persistent storage and resident instance identity for an application launch.</summary>
public sealed class LaunchProfile
{
    private LaunchProfile(string directoryPath, bool isTest)
    {
        DirectoryPath = directoryPath;
        IsTest = isTest;
        DatabasePath = Path.Combine(directoryPath, "shelf.db");
        SettingsPath = Path.Combine(directoryPath, "settings.json");
        var suffix = isTest
            ? ".Test." + Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(directoryPath.ToUpperInvariant())))
            : string.Empty;
        MutexName = @"Local\DropCove.SingleInstance" + suffix;
        ActivationEventName = @"Local\DropCove.Activate" + suffix;
    }

    /// <summary>Gets the directory containing the profile's database and settings.</summary>
    public string DirectoryPath { get; }
    /// <summary>Gets a value indicating whether the launch uses an explicit test-only profile.</summary>
    public bool IsTest { get; }
    /// <summary>Gets the shelf database path.</summary>
    public string DatabasePath { get; }
    /// <summary>Gets the settings file path.</summary>
    public string SettingsPath { get; }
    /// <summary>Gets the named mutex used to claim this profile's resident instance.</summary>
    public string MutexName { get; }
    /// <summary>Gets the named event used to activate this profile's resident instance.</summary>
    public string ActivationEventName { get; }

    /// <summary>Resolves the default profile or an explicit <c>--test-profile</c> directory without writing files.</summary>
    /// <param name="arguments">The process arguments, including unrelated existing launch options.</param>
    /// <returns>The storage paths and instance identity for this launch.</returns>
    /// <exception cref="ArgumentException">The test-profile option is repeated, lacks an absolute directory, or selects the normal profile.</exception>
    public static LaunchProfile Parse(IReadOnlyList<string> arguments)
    {
        var defaultDirectory = Path.TrimEndingDirectorySeparator(Path.GetFullPath(Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "DropCove")));
        string? testDirectory = null;
        for (var index = 0; index < arguments.Count; index++)
        {
            if (arguments[index].StartsWith("--test-profile=", StringComparison.OrdinalIgnoreCase))
            {
                throw new ArgumentException("Use --test-profile followed by a separate absolute directory argument.", nameof(arguments));
            }

            if (!string.Equals(arguments[index], "--test-profile", StringComparison.OrdinalIgnoreCase)) continue;
            if (testDirectory is not null || ++index == arguments.Count ||
                string.IsNullOrWhiteSpace(arguments[index]) || !Path.IsPathFullyQualified(arguments[index]))
            {
                throw new ArgumentException("Use --test-profile exactly once with an absolute directory path.", nameof(arguments));
            }

            testDirectory = Path.TrimEndingDirectorySeparator(Path.GetFullPath(arguments[index]));
        }

        if (testDirectory is null) return new LaunchProfile(defaultDirectory, isTest: false);
        if (string.Equals(testDirectory, defaultDirectory, StringComparison.OrdinalIgnoreCase))
        {
            throw new ArgumentException("A test profile must not use the normal DropCove profile directory.", nameof(arguments));
        }

        return new LaunchProfile(testDirectory, isTest: true);
    }
}
