using DropCove.Core;
using DropCove.Native;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Windows.Graphics;

namespace DropCove;

/// <summary>Edits the resident utility settings in a normal desktop window.</summary>
public sealed partial class SettingsWindow : Window
{
    private readonly Func<AppSettings, Task<bool>> _applySettingsAsync;
    private readonly IReadOnlyList<MonitorOption> _monitorOptions;
    private readonly IReadOnlyList<RailEdgeChoice> _edgeChoices;
    private readonly IReadOnlyList<ShakeSensitivityChoice> _shakeSensitivityChoices;

    /// <summary>Creates a settings window.</summary>
    /// <param name="settings">The current settings.</param>
    /// <param name="monitorOptions">The currently connected displays.</param>
    /// <param name="applySettingsAsync">Applies and saves edited settings.</param>
    public SettingsWindow(
        AppSettings settings,
        IReadOnlyList<MonitorOption> monitorOptions,
        Func<AppSettings, Task<bool>> applySettingsAsync)
    {
        ArgumentNullException.ThrowIfNull(settings);
        ArgumentNullException.ThrowIfNull(monitorOptions);
        ArgumentNullException.ThrowIfNull(applySettingsAsync);

        InitializeComponent();
        _applySettingsAsync = applySettingsAsync;
        _monitorOptions = monitorOptions;
        _edgeChoices = Enum.GetValues<ShelfRailEdge>()
            .Select(edge => new RailEdgeChoice(edge, edge.ToString()))
            .ToArray();
        _shakeSensitivityChoices = Enum.GetValues<ShakeSensitivity>()
            .Select(sensitivity => new ShakeSensitivityChoice(sensitivity, sensitivity.ToString()))
            .ToArray();
        AppWindow.Resize(new SizeInt32(420, 700));

        RailEdgeComboBox.ItemsSource = _edgeChoices;
        RailEdgeComboBox.SelectedItem = _edgeChoices.First(choice => choice.Edge == settings.RailEdge);
        RailMonitorComboBox.ItemsSource = _monitorOptions;
        RailMonitorComboBox.SelectedItem = _monitorOptions.FirstOrDefault(
            option => string.Equals(option.Id, settings.RailMonitorId, StringComparison.OrdinalIgnoreCase))
            ?? _monitorOptions.FirstOrDefault();
        ShowRailOverFullscreenToggle.IsOn = settings.ShowRailOverFullscreen;
        ShakeEnabledToggle.IsOn = settings.ShakeEnabled;
        ShakeSensitivityComboBox.ItemsSource = _shakeSensitivityChoices;
        ShakeSensitivityComboBox.SelectedItem = _shakeSensitivityChoices.First(choice => choice.Sensitivity == settings.ShakeSensitivity);

        ControlCheckBox.IsChecked = settings.HotKey.Control;
        AltCheckBox.IsChecked = settings.HotKey.Alt;
        ShiftCheckBox.IsChecked = settings.HotKey.Shift;
        WindowsCheckBox.IsChecked = settings.HotKey.Windows;
        StartWithWindowsToggle.IsOn = settings.StartWithWindows;

        KeyComboBox.ItemsSource = KeyChoices;
        KeyComboBox.SelectedItem = KeyChoices.FirstOrDefault(choice => choice.VirtualKey == settings.HotKey.VirtualKey)
            ?? KeyChoices[0];
    }

    private static IReadOnlyList<HotKeyChoice> KeyChoices { get; } = CreateKeyChoices();

    private async void OnSaveClicked(object sender, RoutedEventArgs e)
    {
        if (KeyComboBox.SelectedItem is not HotKeyChoice key ||
            RailEdgeComboBox.SelectedItem is not RailEdgeChoice edge ||
            RailMonitorComboBox.SelectedItem is not MonitorOption monitor ||
            ShakeSensitivityComboBox.SelectedItem is not ShakeSensitivityChoice sensitivity ||
            !(ControlCheckBox.IsChecked == true ||
              AltCheckBox.IsChecked == true ||
              ShiftCheckBox.IsChecked == true ||
              WindowsCheckBox.IsChecked == true))
        {
            ShowError("Choose a rail edge, target monitor, shake sensitivity, key, and at least one modifier.");
            return;
        }

        var settings = new AppSettings(
            new HotKeyDefinition(
                ControlCheckBox.IsChecked == true,
                AltCheckBox.IsChecked == true,
                ShiftCheckBox.IsChecked == true,
                WindowsCheckBox.IsChecked == true,
                key.VirtualKey),
            StartWithWindowsToggle.IsOn,
            edge.Edge,
            monitor.Id,
            ShowRailOverFullscreenToggle.IsOn,
            ShakeEnabledToggle.IsOn,
            sensitivity.Sensitivity);

        try
        {
            if (!await _applySettingsAsync(settings))
            {
                ShowError("That hotkey is already registered by another application.");
                return;
            }

            Close();
        }
        catch (Exception exception)
        {
            ShowError($"Settings could not be saved: {exception.Message}");
        }
    }

    private void OnCancelClicked(object sender, RoutedEventArgs e) => Close();

    private void ShowError(string message)
    {
        ErrorInfoBar.Message = message;
        ErrorInfoBar.IsOpen = true;
    }

    private static IReadOnlyList<HotKeyChoice> CreateKeyChoices()
    {
        var choices = new List<HotKeyChoice> { new("Space", 0x20) };
        for (var key = 'A'; key <= 'Z'; key++)
        {
            choices.Add(new HotKeyChoice(key.ToString(), key));
        }

        for (uint functionKey = 1; functionKey <= 12; functionKey++)
        {
            choices.Add(new HotKeyChoice($"F{functionKey}", 0x6F + functionKey));
        }

        return choices;
    }

    private sealed record RailEdgeChoice(ShelfRailEdge Edge, string Name);
    private sealed record HotKeyChoice(string Name, uint VirtualKey);
    private sealed record ShakeSensitivityChoice(ShakeSensitivity Sensitivity, string Name);
}
