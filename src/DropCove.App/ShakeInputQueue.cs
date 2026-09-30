using DropCove.Native;
using Microsoft.UI.Dispatching;

namespace DropCove;

/// <summary>Coalesces low-level mouse input into one bounded UI-thread work queue.</summary>
internal sealed class ShakeInputQueue
{
    private const int Capacity = 64;
    private readonly object _gate = new();
    private readonly DispatcherQueue _dispatcherQueue;
    private readonly DispatcherQueueHandler _drainHandler;
    private readonly Action<LowLevelMouseInput> _consume;
    private readonly LowLevelMouseInput[] _buffer = new LowLevelMouseInput[Capacity];
    private int _head;
    private int _count;
    private bool _drainQueued;
    private bool _leftButtonDown;

    /// <summary>Creates a bounded queue for low-level hook events.</summary>
    /// <param name="dispatcherQueue">The UI dispatcher that owns gesture analysis.</param>
    /// <param name="consume">The UI-thread gesture analysis callback.</param>
    public ShakeInputQueue(DispatcherQueue dispatcherQueue, Action<LowLevelMouseInput> consume)
    {
        ArgumentNullException.ThrowIfNull(dispatcherQueue);
        ArgumentNullException.ThrowIfNull(consume);
        _dispatcherQueue = dispatcherQueue;
        _consume = consume;
        _drainHandler = Drain;
    }

    /// <summary>Queues one hook sample without doing gesture analysis.</summary>
    public void Enqueue(LowLevelMouseInput input)
    {
        var shouldSchedule = false;
        lock (_gate)
        {
            if (input.Message == LowLevelMouseHook.LeftButtonDownMessage)
            {
                _leftButtonDown = true;
                _head = 0;
                _count = 0;
            }
            else if (input.Message == LowLevelMouseHook.LeftButtonUpMessage)
            {
                _leftButtonDown = false;
            }
            else if (input.Message == LowLevelMouseHook.MouseMoveMessage)
            {
                input = input with { LeftButtonDown = _leftButtonDown };
            }

            if (_count == Capacity)
            {
                _head = (_head + 1) % Capacity;
                _count--;
            }

            var tail = (_head + _count) % Capacity;
            _buffer[tail] = input;
            _count++;
            if (!_drainQueued)
            {
                _drainQueued = true;
                shouldSchedule = true;
            }
        }

        if (shouldSchedule && !_dispatcherQueue.TryEnqueue(_drainHandler))
        {
            lock (_gate)
            {
                _drainQueued = false;
            }
        }
    }

    /// <summary>Clears queued samples and resets the hook-side button state.</summary>
    public void Clear()
    {
        lock (_gate)
        {
            _head = 0;
            _count = 0;
            _leftButtonDown = false;
        }
    }

    private void Drain()
    {
        while (true)
        {
            LowLevelMouseInput input;
            lock (_gate)
            {
                if (_count == 0)
                {
                    _drainQueued = false;
                    return;
                }

                input = _buffer[_head];
                _head = (_head + 1) % Capacity;
                _count--;
            }

            _consume(input);
        }
    }
}
