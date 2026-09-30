using DropCove.Core;

namespace DropCove.Tests;

[TestClass]
public sealed class ShakeNaturalGestureTests
{
    [TestMethod]
    public void Observe_NaturalHumanShake_WithDiagonalMovement_ShouldTrigger()
    {
        var detector = new ShakeDetector(ShakeSensitivity.Normal);
        
        // A human moving mouse left and right naturally has small Y drift:
        // (0,0) -> (25, 5) -> (-25, 4) -> (25, 6) -> (-25, 3)
        // Each movement > 12px horizontal, with small Y variation
        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 0)));
        Assert.IsFalse(detector.Observe(new ShakeSample(25, 5, 50)));
        Assert.IsFalse(detector.Observe(new ShakeSample(-25, 9, 100)));
        // Reversal 2: Left -> Right -> triggers at Normal sensitivity (2 reversals)
        Assert.IsTrue(detector.Observe(new ShakeSample(25, 15, 150)));
    }
}
