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
}
