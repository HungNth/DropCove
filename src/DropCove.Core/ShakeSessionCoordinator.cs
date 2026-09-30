namespace DropCove.Core;

/// <summary>Tracks the lifecycle of a shake-summoned session to prevent premature dismissal.</summary>
public sealed class ShakeSessionCoordinator
{
    private bool _isButtonDown;
    private bool _isSummoned;
    private bool _isAwaitingDropOutcome;

    /// <summary>Whether a shake-summoned session is currently active.</summary>
    public bool IsSummoned => _isSummoned;

    /// <summary>Whether the cursor is currently over a drop target awaiting outcome.</summary>
    public bool IsAwaitingDropOutcome => _isAwaitingDropOutcome;

    /// <summary>Whether the left mouse button is currently held down.</summary>
    public bool IsButtonDown => _isButtonDown;

    /// <summary>Records that a shake gesture successfully summoned the shelf.</summary>
    /// <returns><see langword="true"/> if the summon is valid; <see langword="false"/> if the mouse was released before summon.</returns>
    public bool TrySummon()
    {
        if (!_isButtonDown)
        {
            return false;
        }

        _isSummoned = true;
        _isAwaitingDropOutcome = false;
        return true;
    }

    /// <summary>Records that drag entered the shelf window.</summary>
    public void OnDragEnter()
    {
        if (_isSummoned)
        {
            _isAwaitingDropOutcome = true;
        }
    }

    /// <summary>Records that drag left the shelf window.</summary>
    /// <returns>Always false: leaving the window while button is held must NEVER restore the shelf.</returns>
    public bool OnDragLeave()
    {
        _isAwaitingDropOutcome = false;
        // Never restore on DragLeave while mouse is held down!
        return false;
    }

    /// <summary>Records a mouse button state change.</summary>
    /// <param name="isDown">Whether the left mouse button is pressed.</param>
    /// <returns><see langword="true"/> if the session should be restored/dismissed.</returns>
    public bool OnMouseButtonChanged(bool isDown)
    {
        _isButtonDown = isDown;
        if (!isDown && _isSummoned && !_isAwaitingDropOutcome)
        {
            _isSummoned = false;
            return true;
        }

        return false;
    }

    /// <summary>Records drop completion.</summary>
    /// <param name="accepted">Whether the drop was accepted.</param>
    /// <returns><see langword="true"/> if the session should be restored/dismissed.</returns>
    public bool OnDropCompleted(bool accepted)
    {
        _isAwaitingDropOutcome = false;
        if (accepted)
        {
            // Drop accepted: session ends normally, shelf stays open
            _isSummoned = false;
            return false;
        }

        // Drop rejected: restore prior state
        _isSummoned = false;
        return true;
    }

    /// <summary>Forces reset of the session.</summary>
    public void Reset()
    {
        _isButtonDown = false;
        _isSummoned = false;
        _isAwaitingDropOutcome = false;
    }
}
