using DropCove.Core;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Windows.Graphics;

namespace DropCove;

/// <summary>Edits the resident utility settings in a normal desktop window.</summary>
public sealed partial class SettingsWindow : Window
{
    private readonly Func<AppSettings, Task<bool>> _applySettingsAsync;

    /// <summary>Creates a settings window.</summary>
    /// <param name="settings">The current settings.</param>
    /// <param name="applySettingsAsync">Applies and saves edited settings.</param>
    public SettingsWindow(AppSettings settings, Func<AppSettings, Task<bool>> applySettingsAsync)
    {
        InitializeComponent();
        _applySettingsAsync = applySettingsAsync;
        AppWindow.Resize(new SizeInt32(380, 420));

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
            !(ControlCheckBox.IsChecked == true ||
              AltCheckBox.IsChecked == true ||
              ShiftCheckBox.IsChecked == true ||
              WindowsCheckBox.IsChecked == true))
        {
            ShowError("Choose a key and at least one modifier.");
            return;
        }

        var settings = new AppSettings(
            new HotKeyDefinition(
                ControlCheckBox.IsChecked == true,
                AltCheckBox.IsChecked == true,
                ShiftCheckBox.IsChecked == true,
                WindowsCheckBox.IsChecked == true,
                key.VirtualKey),
            StartWithWindowsToggle.IsOn);

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

    private sealed record HotKeyChoice(string Name, uint VirtualKey);
}
