using DropCove.Core;
using DropCove.Native;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Automation.Peers;
using Microsoft.UI.Xaml.Input;
using Windows.System;
using Windows.Graphics;

namespace DropCove;

/// <summary>Edits the resident utility settings in a normal desktop window.</summary>
public sealed partial class SettingsWindow : Window
{
    private readonly Func<AppSettings, Task<bool>> _applySettingsAsync;
    private IReadOnlyList<MonitorOption> _availableMonitorOptions;
    private readonly IReadOnlyList<RailEdgeChoice> _edgeChoices;
    private readonly IReadOnlyList<ShakeSensitivityChoice> _shakeSensitivityChoices;
    private HotKeyDefinition _draftHotKey;
    private bool _isListening;
    private VirtualKey _activationKey;
    private VirtualKey _completedKey;
    private int? _captureMessageTime;
    private const string RecorderHelp = "Click or press Enter or Space to record a shortcut.";
    private const string InvalidShortcutHelp = "Use at least one modifier and a supported key.";

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
        _availableMonitorOptions = monitorOptions;
        _edgeChoices = Enum.GetValues<ShelfRailEdge>()
            .Select(edge => new RailEdgeChoice(edge, edge.ToString()))
            .ToArray();
        _shakeSensitivityChoices = Enum.GetValues<ShakeSensitivity>()
            .Select(sensitivity => new ShakeSensitivityChoice(sensitivity, sensitivity.ToString()))
            .ToArray();

        var windowHandle = WinRT.Interop.WindowNative.GetWindowHandle(this);
        var dpi = WindowInterop.GetWindowDpi(windowHandle);
        AppWindow.Resize(new SizeInt32(
            WindowInterop.ScaleLogicalPixels(420, dpi),
            WindowInterop.ScaleLogicalPixels(700, dpi)));

        RailEdgeComboBox.ItemsSource = _edgeChoices;
        RailEdgeComboBox.SelectedItem = _edgeChoices.First(choice => choice.Edge == settings.RailEdge);
        var monitorChoices = CreateMonitorChoices(settings.RailMonitorId);
        RailMonitorComboBox.ItemsSource = monitorChoices;
        RailMonitorComboBox.SelectedItem = monitorChoices.First(choice =>
            string.Equals(choice.Id, settings.RailMonitorId, StringComparison.OrdinalIgnoreCase));
        ShowRailOverFullscreenToggle.IsOn = settings.ShowRailOverFullscreen;
        ShakeEnabledToggle.IsOn = settings.ShakeEnabled;
        ShakeSensitivityComboBox.ItemsSource = _shakeSensitivityChoices;
        ShakeSensitivityComboBox.SelectedItem = _shakeSensitivityChoices.First(choice => choice.Sensitivity == settings.ShakeSensitivity);

        _draftHotKey = settings.HotKey;
        StartWithWindowsToggle.IsOn = settings.StartWithWindows;
        RenderHotKey();
        Activated += (_, args) =>
        {
            if (args.WindowActivationState == WindowActivationState.Deactivated)
            {
                CancelRecording();
            }
            else
            {
                RenderHotKey();
            }
        };
    }

    internal void UpdateMonitorOptions(IReadOnlyList<MonitorOption> monitorOptions)
    {
        ArgumentNullException.ThrowIfNull(monitorOptions);
        var selectedId = (RailMonitorComboBox.SelectedItem as MonitorOption)?.Id ?? string.Empty;
        _availableMonitorOptions = monitorOptions;
        var monitorChoices = CreateMonitorChoices(selectedId);
        RailMonitorComboBox.ItemsSource = monitorChoices;
        RailMonitorComboBox.SelectedItem = monitorChoices.First(choice =>
            string.Equals(choice.Id, selectedId, StringComparison.OrdinalIgnoreCase));
    }

    private IReadOnlyList<MonitorOption> CreateMonitorChoices(string? selectedMonitorId)
    {
        var choices = new List<MonitorOption>
        {
            new(string.Empty, "Use remembered monitor"),
        };
        choices.AddRange(_availableMonitorOptions);
        if (!string.IsNullOrWhiteSpace(selectedMonitorId) &&
            choices.All(choice => !string.Equals(choice.Id, selectedMonitorId, StringComparison.OrdinalIgnoreCase)))
        {
            choices.Add(new MonitorOption(selectedMonitorId, $"{selectedMonitorId} (currently unavailable)"));
        }

        return choices;
    }

    internal bool HandleRegisteredHotKey(nint keyData)
    {
        if (_isListening)
        {
            var modifiers = (HotKeyModifiers)((long)keyData & 0xFFFF);
            CompleteRecording(new HotKeyDefinition(
                modifiers.HasFlag(HotKeyModifiers.Control),
                modifiers.HasFlag(HotKeyModifiers.Alt),
                modifiers.HasFlag(HotKeyModifiers.Shift),
                modifiers.HasFlag(HotKeyModifiers.Windows),
                (uint)(((long)keyData >> 16) & 0xFFFF)));
            _captureMessageTime = null;
            return true;
        }

        // WM_HOTKEY and WinUI key input from the same press can arrive in either order.
        var suppress = _captureMessageTime == WindowInterop.GetKeyboardMessageTime();
        _captureMessageTime = null;
        return suppress;
    }

    private void OnHotKeyRecorderClicked(object sender, RoutedEventArgs e)
    {
        HotKeyRecorder.Focus(FocusState.Programmatic);
        BeginRecording();
    }

    private void BeginRecording()
    {
        if (_isListening) return;
        _isListening = true;
        _completedKey = VirtualKey.None;
        SaveButton.IsEnabled = false;
        ErrorInfoBar.IsOpen = false;
        RenderHotKey();
    }

    private void CancelRecording()
    {
        _activationKey = VirtualKey.None;
        _completedKey = VirtualKey.None;
        if (!_isListening) return;
        _isListening = false;
        SaveButton.IsEnabled = true;
        RenderHotKey();
    }

    private void OnHotKeyLostFocus(object sender, RoutedEventArgs e) => CancelRecording();
    private void OnHotKeyGotFocus(object sender, RoutedEventArgs e) => HotKeyEditor.StartBringIntoView();

    private void OnHotKeyPreviewKeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (!_isListening)
        {
            if (e.Key == _completedKey || _activationKey != VirtualKey.None)
            {
                e.Handled = true;
            }
            else if (e.Key is VirtualKey.Enter or VirtualKey.Space)
            {
                e.Handled = true;
                if (!e.KeyStatus.WasKeyDown) _activationKey = e.Key;
            }
            return;
        }

        var modifiers = WindowInterop.GetCurrentHotKeyModifiers();
        if (e.Key == VirtualKey.Tab && modifiers == HotKeyModifiers.None) return;
        e.Handled = true;
        if (e.Key == VirtualKey.Escape)
        {
            CancelRecording();
            return;
        }

        if (IsModifier(e.Key))
        {
            RenderHotKey();
            return;
        }

        CompleteRecording(new HotKeyDefinition(
            modifiers.HasFlag(HotKeyModifiers.Control),
            modifiers.HasFlag(HotKeyModifiers.Alt),
            modifiers.HasFlag(HotKeyModifiers.Shift),
            modifiers.HasFlag(HotKeyModifiers.Windows),
            (uint)e.Key));
    }

    private void OnHotKeyPreviewKeyUp(object sender, KeyRoutedEventArgs e)
    {
        if (_activationKey == e.Key)
        {
            _activationKey = VirtualKey.None;
            e.Handled = true;
            BeginRecording();
        }
        else if (_completedKey == e.Key)
        {
            _completedKey = VirtualKey.None;
            e.Handled = true;
        }
        else if (_isListening)
        {
            e.Handled = true;
            if (IsModifier(e.Key)) RenderHotKey();
        }
    }

    private void CompleteRecording(HotKeyDefinition hotKey)
    {
        var keyLabel = hotKey.Control || hotKey.Alt || hotKey.Shift || hotKey.Windows
            ? WindowInterop.GetHotKeyKeyLabel(hotKey.VirtualKey)
            : null;
        if (keyLabel is null)
        {
            RenderHotKey();
            HotKeyStatusText.Text = InvalidShortcutHelp;
            AnnounceRecorder();
            HotKeyEditor.StartBringIntoView();
            return;
        }

        _captureMessageTime = WindowInterop.GetKeyboardMessageTime();
        _completedKey = (VirtualKey)hotKey.VirtualKey;
        _draftHotKey = hotKey;
        _isListening = false;
        SaveButton.IsEnabled = true;
        RenderHotKey(keyLabel);
    }

    private void RenderHotKey(string? capturedKeyLabel = null)
    {
        var modifiers = _isListening
            ? WindowInterop.GetCurrentHotKeyModifiers()
            : (_draftHotKey.Control ? HotKeyModifiers.Control : 0) |
              (_draftHotKey.Alt ? HotKeyModifiers.Alt : 0) |
              (_draftHotKey.Shift ? HotKeyModifiers.Shift : 0) |
              (_draftHotKey.Windows ? HotKeyModifiers.Windows : 0);
        var label = (_isListening ? "" : capturedKeyLabel ?? WindowInterop.GetHotKeyKeyLabel(_draftHotKey.VirtualKey)) ?? "Unsupported shortcut";
        var prefix = (modifiers.HasFlag(HotKeyModifiers.Control) ? "Ctrl + " : "") +
                     (modifiers.HasFlag(HotKeyModifiers.Alt) ? "Alt + " : "") +
                     (modifiers.HasFlag(HotKeyModifiers.Shift) ? "Shift + " : "") +
                     (modifiers.HasFlag(HotKeyModifiers.Windows) ? "Win + " : "");
        HotKeyValueText.Text = _isListening ? prefix + "…" : prefix + label;
        HotKeyStatusText.Text = _isListening ? "Press shortcut… Escape cancels." : RecorderHelp;
        AutomationProperties.SetName(HotKeyRecorder, "Global hotkey, " + HotKeyValueText.Text);
        AutomationProperties.SetItemStatus(HotKeyRecorder, _isListening ? "Listening" : "Shortcut selected");
        AutomationProperties.SetHelpText(HotKeyRecorder, _isListening ? "Press a shortcut with at least one modifier. Escape cancels recording." : RecorderHelp);
        if (HotKeyRecorder.FocusState != FocusState.Unfocused) HotKeyEditor.StartBringIntoView();
        AnnounceRecorder();
    }

    private void AnnounceRecorder() => FrameworkElementAutomationPeer.FromElement(HotKeyStatusText)
        ?.RaiseAutomationEvent(AutomationEvents.LiveRegionChanged);

    private static bool IsModifier(VirtualKey key) => key is
        VirtualKey.Control or VirtualKey.LeftControl or VirtualKey.RightControl or
        VirtualKey.Menu or VirtualKey.LeftMenu or VirtualKey.RightMenu or
        VirtualKey.Shift or VirtualKey.LeftShift or VirtualKey.RightShift or
        VirtualKey.LeftWindows or VirtualKey.RightWindows;


    private async void OnSaveClicked(object sender, RoutedEventArgs e)
    {
        if (_isListening) return;
        if (RailEdgeComboBox.SelectedItem is not RailEdgeChoice edge ||
            RailMonitorComboBox.SelectedItem is not MonitorOption monitor ||
            ShakeSensitivityComboBox.SelectedItem is not ShakeSensitivityChoice sensitivity ||
            !(_draftHotKey.Control || _draftHotKey.Alt || _draftHotKey.Shift || _draftHotKey.Windows) ||
            WindowInterop.GetHotKeyKeyLabel(_draftHotKey.VirtualKey) is null)
        {
            ShowError("Choose a rail edge, target monitor, shake sensitivity, and a valid hotkey.");
            return;
        }

        var settings = new AppSettings(
            _draftHotKey,
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
                ShowError("DropCove couldn’t register this hotkey. It may be unavailable or reserved by Windows. Choose another.");
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
        ErrorInfoBar.UpdateLayout();
        ErrorInfoBar.StartBringIntoView();
    }


    private sealed record RailEdgeChoice(ShelfRailEdge Edge, string Name);
    private sealed record ShakeSensitivityChoice(ShakeSensitivity Sensitivity, string Name);
}
