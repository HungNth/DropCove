using Microsoft.UI.Windowing;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Windows.Graphics;

namespace DropCove;

/// <summary>Displays a modern WinUI confirmation dialog matching DropCove styling, positioned over DropCove.</summary>
public sealed partial class ConfirmWindow : Window
{
    private readonly nint _ownerHandle;
    private readonly TaskCompletionSource<bool> _completionSource = new();
    private readonly nint _dialogHandle;
    private readonly DispatcherQueue _dispatcherQueue;
    private readonly Native.WindowMessageHook _windowMessageHook;


    public ConfirmWindow(nint ownerHandle, string title, string message)
    {
        InitializeComponent();
        _ownerHandle = ownerHandle;
        Title = title;
        var parts = message.Split(["\n\n"], StringSplitOptions.None);
        MessageTextBlock.Text = parts[0];
        SubMessageTextBlock.Text = parts.Length > 1 ? parts[1] : string.Empty;
        SubMessageTextBlock.Visibility = parts.Length > 1 ? Visibility.Visible : Visibility.Collapsed;
        _dialogHandle = WinRT.Interop.WindowNative.GetWindowHandle(this);
        _dispatcherQueue = DispatcherQueue.GetForCurrentThread();
        _windowMessageHook = new Native.WindowMessageHook(_dialogHandle, HandleWindowMessage);
        ResizeForCurrentDpi();


        if (AppWindow.Presenter is OverlappedPresenter presenter)
        {
            presenter.IsResizable = false;
            presenter.IsMaximizable = false;
            presenter.IsMinimizable = false;
            presenter.IsAlwaysOnTop = true;
        }

        Closed += (_, _) =>
        {
            _windowMessageHook.Dispose();
            if (_ownerHandle != 0)
            {
                Native.WindowInterop.SetWindowEnabled(_ownerHandle, true);
            }
            _completionSource.TrySetResult(false);
        };

    }

    /// <summary>Shows the confirmation window modally and waits for user choice.</summary>
    /// <returns>True if user clicked Yes; false if No or window closed.</returns>
    public async Task<bool> ShowDialogAsync()
    {
        if (_ownerHandle != 0)
        {
            Native.WindowInterop.SetWindowEnabled(_ownerHandle, false);
        }
        Activate();
        return await _completionSource.Task;
    }
    private void ResizeForCurrentDpi()
    {
        var dpi = Native.WindowInterop.GetWindowDpi(_dialogHandle);
        var width = Native.WindowInterop.ScaleLogicalPixels(360, dpi);
        var height = Native.WindowInterop.ScaleLogicalPixels(190, dpi);
        if (_ownerHandle != 0)
        {
            Native.WindowInterop.CenterOverWindow(_dialogHandle, _ownerHandle, width, height);
        }
        else
        {
            AppWindow.Resize(new SizeInt32(width, height));
        }
    }

    private Native.WindowMessageResult HandleWindowMessage(uint message, nuint wParam, nint lParam)
    {
        if (message is Native.WindowInterop.DisplayChangeMessage or Native.WindowInterop.DpiChangedMessage)
        {
            _dispatcherQueue.TryEnqueue(ResizeForCurrentDpi);
        }

        return Native.WindowMessageResult.Unhandled;
    }


    private void OnYesClicked(object sender, RoutedEventArgs e)
    {
        _completionSource.TrySetResult(true);
        Close();
    }

    private void OnNoClicked(object sender, RoutedEventArgs e)
    {
        _completionSource.TrySetResult(false);
        Close();
    }
}
