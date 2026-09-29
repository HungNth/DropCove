using DropCove.Native;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;

namespace DropCove;

/// <summary>Initializes and launches the resident DropCove application.</summary>
public partial class App : Application
{
    private const string MutexName = @"Local\DropCove.SingleInstance";
    private const string ActivationEventName = @"Local\DropCove.Activate";
    private readonly Mutex _singleInstanceMutex;
    private readonly bool _isPrimaryInstance;
    private readonly DispatcherQueue _dispatcherQueue;
    private EventWaitHandle? _activationEvent;
    private RegisteredWaitHandle? _activationWait;
    private MainWindow? _window;
    private bool _activationPending;

    /// <summary>Creates the application and claims the per-user single-instance key.</summary>
    public App()
    {
        InitializeComponent();
        _dispatcherQueue = DispatcherQueue.GetForCurrentThread();
        _singleInstanceMutex = new Mutex(true, MutexName, out _isPrimaryInstance);

        if (_isPrimaryInstance)
        {
            _activationEvent = new EventWaitHandle(false, EventResetMode.AutoReset, ActivationEventName);
            _activationWait = ThreadPool.RegisterWaitForSingleObject(
                _activationEvent,
                (_, _) => _dispatcherQueue.TryEnqueue(ActivatePrimaryWindow),
                null,
                Timeout.Infinite,
                executeOnlyOnce: false);
        }
    }

    /// <inheritdoc />
    protected override async void OnLaunched(LaunchActivatedEventArgs args)
    {
        if (!_isPrimaryInstance)
        {
            SignalPrimaryInstance();
            Environment.Exit(0);
            return;
        }

        try
        {
            _window = new MainWindow();
            _window.Activate();
            var startHidden = Environment.GetCommandLineArgs()
                .Any(argument => string.Equals(argument, "--autostart", StringComparison.OrdinalIgnoreCase));
            await _window.InitializeAsync(startHidden);

            if (_activationPending)
            {
                _activationPending = false;
                _window.ShowShelf();
            }
        }
        catch (Exception exception)
        {
            WindowInterop.ShowError("DropCove could not start", exception.Message);
            Current.Exit();
        }
    }

    private void ActivatePrimaryWindow()
    {
        if (_window is null)
        {
            _activationPending = true;
            return;
        }

        _window.ShowShelf();
    }

    private static void SignalPrimaryInstance()
    {
        try
        {
            using var activationEvent = EventWaitHandle.OpenExisting(ActivationEventName);
            activationEvent.Set();
        }
        catch (WaitHandleCannotBeOpenedException)
        {
            // The primary process is still creating its activation event; its normal launch will show the shelf.
        }
    }
}
