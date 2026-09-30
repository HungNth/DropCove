using DropCove.Core;
using DropCove.Native;

namespace DropCove.Tests;

[TestClass]
public sealed class ShakeInputSequenceSeamTests
{
    [TestMethod]
    public void LowLevelMouseInput_PreservesLeftButtonDownStateAcrossMovement()
    {
        // Verifies the end-to-end contract between mouse messages and shake detection:
        // LeftButtonDown sets button state true, subsequent moves retain button state,
        // and detector observes the shake sequence while button is pressed.
        var detector = new ShakeDetector(ShakeSensitivity.Normal);
        var buttonDown = false;
        var triggered = false;

        void ProcessInput(LowLevelMouseInput input)
        {
            if (input.Message == LowLevelMouseHook.LeftButtonDownMessage)
            {
                buttonDown = true;
                detector.Reset();
                return;
            }

            if (input.Message == LowLevelMouseHook.LeftButtonUpMessage)
            {
                buttonDown = false;
                return;
            }

            if (input.Message == LowLevelMouseHook.MouseMoveMessage)
            {
                buttonDown = input.LeftButtonDown;
                if (buttonDown && detector.Observe(new ShakeSample(input.X, input.Y, input.TimestampMilliseconds)))
                {
                    triggered = true;
                }
            }
        }

        // 1. Mouse down
        ProcessInput(new LowLevelMouseInput(LowLevelMouseHook.LeftButtonDownMessage, 100, 100, 0, LeftButtonDown: true));
        Assert.IsTrue(buttonDown);

        // 2. Mouse moves while dragging (hook sets LeftButtonDown = true on moves)
        // First move establishes baseline position
        ProcessInput(new LowLevelMouseInput(LowLevelMouseHook.MouseMoveMessage, 100, 100, 10, LeftButtonDown: true));

        // Stroke 1: Right (+20px)
        ProcessInput(new LowLevelMouseInput(LowLevelMouseHook.MouseMoveMessage, 120, 100, 50, LeftButtonDown: true));
        Assert.IsTrue(buttonDown);
        Assert.IsFalse(triggered);

        // Reversal 1: Left (-20px)
        ProcessInput(new LowLevelMouseInput(LowLevelMouseHook.MouseMoveMessage, 100, 100, 100, LeftButtonDown: true));
        Assert.IsTrue(buttonDown);
        Assert.IsFalse(triggered);

        // Reversal 2: Right (+20px) -> Shake triggers!
        ProcessInput(new LowLevelMouseInput(LowLevelMouseHook.MouseMoveMessage, 120, 100, 150, LeftButtonDown: true));
        Assert.IsTrue(buttonDown);
        Assert.IsTrue(triggered);

        // 3. Mouse up
        ProcessInput(new LowLevelMouseInput(LowLevelMouseHook.LeftButtonUpMessage, 120, 100, 200, LeftButtonDown: false));
        Assert.IsFalse(buttonDown);
    }

    [TestMethod]
    public void ShakeSessionCoordinator_DragLeaveDoesNotRestore_OnlyMouseUpOrRejectedDropRestores()
    {
        var coordinator = new ShakeSessionCoordinator();
        coordinator.OnMouseButtonChanged(isDown: true);

        // 1. Shake triggers and summons shelf
        Assert.IsTrue(coordinator.TrySummon());
        Assert.IsTrue(coordinator.IsSummoned);
        Assert.IsFalse(coordinator.IsAwaitingDropOutcome);

        // 2. Cursor drags into shelf
        coordinator.OnDragEnter();
        Assert.IsTrue(coordinator.IsAwaitingDropOutcome);
        Assert.IsTrue(coordinator.IsSummoned);

        // 3. User moves cursor off shelf or over window border -> DragLeave
        // Bug was: immediately restored and dismissed shelf!
        // Fix contract: OnDragLeave must return false and NOT restore
        var shouldRestoreOnLeave = coordinator.OnDragLeave();
        Assert.IsFalse(shouldRestoreOnLeave, "Must not restore on DragLeave while dragging");
        Assert.IsTrue(coordinator.IsSummoned, "Session must stay active");
        Assert.IsFalse(coordinator.IsAwaitingDropOutcome);

        // 4. User moves cursor back onto shelf -> DragEnter
        coordinator.OnDragEnter();
        Assert.IsTrue(coordinator.IsAwaitingDropOutcome);
        Assert.IsTrue(coordinator.IsSummoned);

        // 5. User drops file successfully
        var shouldRestoreOnAccept = coordinator.OnDropCompleted(accepted: true);
        Assert.IsFalse(shouldRestoreOnAccept, "Accepted drop keeps shelf open");
        Assert.IsFalse(coordinator.IsSummoned);
    }

    [TestMethod]
    public void ShakeSessionCoordinator_MouseReleasedBeforeQueuedSummon_AbortsSummon()
    {
        var coordinator = new ShakeSessionCoordinator();

        // Mouse goes down and triggers gesture detection
        coordinator.OnMouseButtonChanged(isDown: true);

        // But user quickly releases the mouse before the queued UI-thread summon executes
        coordinator.OnMouseButtonChanged(isDown: false);

        // When summon task finally runs, it must be aborted
        Assert.IsFalse(coordinator.TrySummon(), "Must not summon if mouse was released before summon execution");
        Assert.IsFalse(coordinator.IsSummoned);
    }

    [TestMethod]
    public void ShakeSessionCoordinator_MouseUpOutsideShelf_RestoresPriorState()
    {
        var coordinator = new ShakeSessionCoordinator();
        coordinator.OnMouseButtonChanged(isDown: true);
        Assert.IsTrue(coordinator.TrySummon());

        // User shook to summon, but then released mouse outside the shelf (cancelled gesture)
        coordinator.OnDragEnter();
        coordinator.OnDragLeave();

        var shouldRestoreOnRelease = coordinator.OnMouseButtonChanged(isDown: false);
        Assert.IsTrue(shouldRestoreOnRelease, "Releasing mouse when no drop is accepted must restore prior state");
        Assert.IsFalse(coordinator.IsSummoned);
    }

    [TestMethod]
    public void ShakeSessionCoordinator_RejectedDrop_RestoresPriorState()
    {
        var coordinator = new ShakeSessionCoordinator();
        coordinator.OnMouseButtonChanged(isDown: true);
        Assert.IsTrue(coordinator.TrySummon());
        coordinator.OnDragEnter();

        // Dropped unsupported item
        var shouldRestore = coordinator.OnDropCompleted(accepted: false);
        Assert.IsTrue(shouldRestore, "Rejected drop must restore prior state");
        Assert.IsFalse(coordinator.IsSummoned);
    }
}
