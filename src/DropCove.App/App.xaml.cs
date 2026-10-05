using DropCove.Native;
using DropCove.Services;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;

namespace DropCove;

/// <summary>Initializes and launches the resident DropCove application.</summary>
public partial class App : Application
{
    private readonly Mutex _singleInstanceMutex;
    private readonly bool _isPrimaryInstance;
    private readonly DispatcherQueue _dispatcherQueue;
    private readonly LaunchProfile _launchProfile;
    private readonly bool _startHidden;
    private EventWaitHandle? _activationEvent;
    private RegisteredWaitHandle? _activationWait;
    private MainWindow? _window;
    private bool _activationPending;

    /// <summary>Creates the application and claims the per-user single-instance key.</summary>
    public App()
    {
        try
        {
            var arguments = Environment.GetCommandLineArgs();
            _launchProfile = LaunchProfile.Parse(arguments);
            _startHidden = arguments.Any(argument =>
                string.Equals(argument, "--autostart", StringComparison.OrdinalIgnoreCase));
        }
        catch (ArgumentException exception)
        {
            WindowInterop.ShowError("Invalid test profile", exception.Message);
            Environment.Exit(1);
            throw;
        }

        InitializeComponent();
        _dispatcherQueue = DispatcherQueue.GetForCurrentThread();
        _singleInstanceMutex = new Mutex(true, _launchProfile.MutexName, out _isPrimaryInstance);

        if (_isPrimaryInstance)
        {
            _activationEvent = new EventWaitHandle(false, EventResetMode.AutoReset, _launchProfile.ActivationEventName);
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
            _window = new MainWindow(_launchProfile);
            _window.Activate();
            await _window.InitializeAsync(_startHidden);

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

    private void SignalPrimaryInstance()
    {
        try
        {
            using var activationEvent = EventWaitHandle.OpenExisting(_launchProfile.ActivationEventName);
            activationEvent.Set();
        }
        catch (WaitHandleCannotBeOpenedException)
        {
            // The primary process is still creating its activation event; its normal launch will show the shelf.
        }
    }
}
