using System.Text.Json;
using DropCove.Core;

namespace DropCove.Services;

/// <summary>Loads and atomically saves per-user DropCove settings.</summary>
public sealed class SettingsStore
{
    private static readonly JsonSerializerOptions JsonOptions = new() { WriteIndented = true };
    private readonly string _settingsPath;

    /// <summary>Initializes a settings store in the user's local application data directory.</summary>
    public SettingsStore()
    {
        _settingsPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "DropCove",
            "settings.json");
    }

    /// <summary>Loads saved settings or returns defaults when no valid settings file exists.</summary>
    /// <param name="cancellationToken">Cancels file I/O.</param>
    /// <returns>The saved or default settings.</returns>
    public async Task<AppSettings> LoadAsync(CancellationToken cancellationToken = default)
    {
        if (!File.Exists(_settingsPath))
        {
            return AppSettings.Default;
        }

        try
        {
            await using var stream = File.OpenRead(_settingsPath);
            return await JsonSerializer.DeserializeAsync<AppSettings>(stream, JsonOptions, cancellationToken)
                .ConfigureAwait(false) ?? AppSettings.Default;
        }
        catch (IOException)
        {
            return AppSettings.Default;
        }
        catch (JsonException)
        {
            return AppSettings.Default;
        }
    }

    /// <summary>Atomically saves settings for the current user.</summary>
    /// <param name="settings">The settings to save.</param>
    /// <param name="cancellationToken">Cancels file I/O.</param>
    public async Task SaveAsync(AppSettings settings, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(settings);

        var directory = Path.GetDirectoryName(_settingsPath)!;
        Directory.CreateDirectory(directory);
        var temporaryPath = _settingsPath + ".tmp";

        await using (var stream = File.Create(temporaryPath))
        {
            await JsonSerializer.SerializeAsync(stream, settings, JsonOptions, cancellationToken)
                .ConfigureAwait(false);
            await stream.FlushAsync(cancellationToken).ConfigureAwait(false);
        }

        File.Move(temporaryPath, _settingsPath, true);
    }
}
