namespace DropCove.Core;

/// <summary>Controls the movement distance and reversal count required by shake detection.</summary>
public enum ShakeSensitivity
{
    /// <summary>Requires larger movement and more reversals.</summary>
    Low,
    /// <summary>Uses the balanced default movement threshold.</summary>
    Normal,
    /// <summary>Triggers with smaller movement and fewer reversals.</summary>
    High,
}

/// <summary>Represents one low-level mouse movement sample.</summary>
/// <param name="X">The screen-space horizontal coordinate.</param>
/// <param name="Y">The screen-space vertical coordinate.</param>
/// <param name="TimestampMilliseconds">The wrapping native event timestamp in milliseconds.</param>
public readonly record struct ShakeSample(int X, int Y, uint TimestampMilliseconds);

/// <summary>Detects bounded quick direction reversals without polling cursor state.</summary>
public sealed class ShakeDetector
{
    private const uint DetectionWindowMilliseconds = 600;
    private const uint CooldownMilliseconds = 500;
    private readonly int _minimumMovement;
    private readonly int _requiredReversals;
    private ShakeDirection _lastDirection;
    private int _anchorX;
    private int _anchorY;
    private uint _lastSampleTimestamp;
    private int _reversals;
    private uint _gestureStartTimestamp;
    private uint _lastTriggerTimestamp;
    private bool _hasTriggered;
    private bool _hasSample;
    /// <param name="sensitivity">The movement sensitivity profile.</param>
    public ShakeDetector(ShakeSensitivity sensitivity)
    {
        (_minimumMovement, _requiredReversals) = sensitivity switch
        {
            ShakeSensitivity.Low => (24, 4),
            ShakeSensitivity.Normal => (10, 2),
            ShakeSensitivity.High => (8, 2),
            _ => throw new ArgumentOutOfRangeException(nameof(sensitivity), sensitivity, "Unknown shake sensitivity."),
        };
    }

    /// <summary>Consumes one event-driven movement sample.</summary>
    /// <param name="sample">The sample supplied by the low-level mouse hook.</param>
    /// <returns><see langword="true"/> when the sample completes a shake gesture.</returns>
    public bool Observe(ShakeSample sample)
    {
        if (_hasTriggered && unchecked(sample.TimestampMilliseconds - _lastTriggerTimestamp) < CooldownMilliseconds)
        {
            Remember(sample);
            return false;
        }

        if (!_hasSample)
        {
            Remember(sample);
            return false;
        }
        var elapsed = unchecked(sample.TimestampMilliseconds - _lastSampleTimestamp);
        _lastSampleTimestamp = sample.TimestampMilliseconds;
        if (elapsed > DetectionWindowMilliseconds)
        {
            Reset();
            Remember(sample);
            return false;
        }

        var deltaX = sample.X - _anchorX;
        var deltaY = sample.Y - _anchorY;
        var distance = Math.Max(Math.Abs(deltaX), Math.Abs(deltaY));
        if (distance < _minimumMovement)
        {
            return false;
        }

        var direction = GetDirection(deltaX, deltaY);
        _anchorX = sample.X;
        _anchorY = sample.Y;

        if (_reversals == 0 && _lastDirection == ShakeDirection.None)
        {
            _gestureStartTimestamp = sample.TimestampMilliseconds;
        }
        else if (unchecked(sample.TimestampMilliseconds - _gestureStartTimestamp) > DetectionWindowMilliseconds)
        {
            Reset();
            Remember(sample);
            return false;
        }
        if (_lastDirection != ShakeDirection.None)
        {
            if (direction == Opposite(_lastDirection))
            {
                _reversals++;
            }
            else if (direction != _lastDirection)
            {
                _reversals = 0;
            }
        }

        _lastDirection = direction;
        if (_reversals < _requiredReversals)
        {
            return false;
        }
        _lastTriggerTimestamp = sample.TimestampMilliseconds;
        _hasTriggered = true;
        Reset();
        return true;
    }

    /// <summary>Clears the current partial gesture.</summary>
    public void Reset()
    {
        _lastDirection = ShakeDirection.None;
        _anchorX = 0;
        _anchorY = 0;
        _lastSampleTimestamp = 0;
        _gestureStartTimestamp = 0;
        _reversals = 0;
        _hasSample = false;
    }

    private void Remember(ShakeSample sample)
    {
        _anchorX = sample.X;
        _anchorY = sample.Y;
        _lastSampleTimestamp = sample.TimestampMilliseconds;
        _hasSample = true;
    }
    private ShakeDirection GetDirection(int deltaX, int deltaY)
    {
        var distance = Math.Max(Math.Abs(deltaX), Math.Abs(deltaY));
        if (distance < _minimumMovement)
        {
            return ShakeDirection.None;
        }

        return Math.Abs(deltaX) >= Math.Abs(deltaY)
            ? deltaX > 0 ? ShakeDirection.Right : ShakeDirection.Left
            : deltaY > 0 ? ShakeDirection.Down : ShakeDirection.Up;
    }

    private static ShakeDirection Opposite(ShakeDirection direction) => direction switch
    {
        ShakeDirection.Left => ShakeDirection.Right,
        ShakeDirection.Right => ShakeDirection.Left,
        ShakeDirection.Up => ShakeDirection.Down,
        ShakeDirection.Down => ShakeDirection.Up,
        _ => ShakeDirection.None,
    };

    private enum ShakeDirection
    {
        None,
        Left,
        Right,
        Up,
        Down,
    }
}
