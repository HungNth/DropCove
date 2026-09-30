using DropCove.Core;

namespace DropCove.Tests;

[TestClass]
public sealed class ShakeNaturalSpeedTests
{
    [TestMethod]
    public void Observe_ComfortableNaturalShake_TriggersAtNormalSensitivity()
    {
        // Realistic human shake during a drag operation:
        // Speed: ~500ms total duration, ~120-150ms per half-cycle reversal.
        // Displacement: ~15-20px per swing (comfortable wrist flick, not whole-arm thrashing).
        // A natural shake: Center -> Right (+18px) -> Left (-20px) -> Right (+18px)
        // With 600ms detection window and 2 reversals (or 3 reversals within 600ms):
        var detector = new ShakeDetector(ShakeSensitivity.Normal);

        uint t = 1000;
        Assert.IsFalse(detector.Observe(new ShakeSample(100, 100, t)));

        // Move right (+18px) over 120ms
        Assert.IsFalse(detector.Observe(new ShakeSample(118, 101, t += 120)));

        // Reverse left (-20px) over 130ms -> Reversal 1
        Assert.IsFalse(detector.Observe(new ShakeSample(98, 99, t += 130)));

        // Reverse right (+20px) over 130ms -> Reversal 2
        var triggeredOnReversal2 = detector.Observe(new ShakeSample(118, 100, t += 130));

        // In macOS Dropover / Yoink, 2 quick reversals (Left-Right-Left or Right-Left-Right)
        // within a comfortable ~600ms window triggers smoothly without arm fatigue.
        Assert.IsTrue(triggeredOnReversal2, "Natural comfortable shake should trigger on 2 reversals within 600ms at Normal sensitivity");
    }
}
