using DropCove.Core;

namespace DropCove.Tests;

[TestClass]
public sealed class ShakeGestureWindowTests
{
    [TestMethod]
    public void Observe_ShakePrecededByIdleOrSmallMovements_ShouldTrigger()
    {
        var detector = new ShakeDetector(ShakeSensitivity.Normal);

        // A user clicks and drags a file:
        // First 500ms: cursor hovers or moves slowly (delta < 12px)
        Assert.IsFalse(detector.Observe(new ShakeSample(100, 100, 1000)));
        Assert.IsFalse(detector.Observe(new ShakeSample(102, 100, 1100)));
        Assert.IsFalse(detector.Observe(new ShakeSample(104, 100, 1200)));
        Assert.IsFalse(detector.Observe(new ShakeSample(105, 100, 1300)));
        Assert.IsFalse(detector.Observe(new ShakeSample(106, 100, 1400)));

        // Then starts quick shake reversals:
        // Reversal 1: Right -> Left (delta -30px)
        Assert.IsFalse(detector.Observe(new ShakeSample(136, 100, 1450)));
        Assert.IsFalse(detector.Observe(new ShakeSample(106, 100, 1500)));
        // Reversal 2: Left -> Right (delta +30px) -> triggers at Normal sensitivity (2 reversals)
        Assert.IsTrue(detector.Observe(new ShakeSample(136, 100, 1550)));
    }
}
