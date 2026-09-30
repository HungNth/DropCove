using DropCove.Core;

namespace DropCove.Tests;

[TestClass]
public sealed class ShakeDetectorTests
{
    [TestMethod]
    public void Observe_NormalSensitivity_TriggersAfterTwoQuickReversals()
    {
        var detector = new ShakeDetector(ShakeSensitivity.Normal);

        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 0)));
        Assert.IsFalse(detector.Observe(new ShakeSample(20, 0, 50)));
        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 100)));
        Assert.IsTrue(detector.Observe(new ShakeSample(20, 0, 150)));
    }

    [TestMethod]
    public void Observe_IgnoresSmallOrSlowReversals()
    {
        var detector = new ShakeDetector(ShakeSensitivity.Normal);

        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 0)));
        Assert.IsFalse(detector.Observe(new ShakeSample(4, 0, 50)));
        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 100)));
        Assert.IsFalse(detector.Observe(new ShakeSample(20, 0, 600)));
        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 1_100)));
        Assert.IsFalse(detector.Observe(new ShakeSample(20, 0, 1_600)));
    }

    [TestMethod]
    public void Observe_HighSensitivity_UsesShorterMovementThreshold()
    {
        var detector = new ShakeDetector(ShakeSensitivity.High);

        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 0)));
        Assert.IsFalse(detector.Observe(new ShakeSample(10, 0, 50)));
        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 100)));
        Assert.IsTrue(detector.Observe(new ShakeSample(10, 0, 150)));
    }

    [TestMethod]
    public void Observe_RejectsReversalsSpreadAcrossTheGestureWindow()
    {
        var detector = new ShakeDetector(ShakeSensitivity.Normal);

        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 0)));
        Assert.IsFalse(detector.Observe(new ShakeSample(20, 0, 350)));
        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 700)));
        Assert.IsFalse(detector.Observe(new ShakeSample(20, 0, 1_050)));
        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 1_400)));
    }

    [TestMethod]
    public void Observe_AllowsQuickGestureAcrossNativeTimestampWrap()
    {
        var detector = new ShakeDetector(ShakeSensitivity.High);
        var start = uint.MaxValue - 100;

        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, start)));
        Assert.IsFalse(detector.Observe(new ShakeSample(10, 0, start + 50)));
        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 50)));
        Assert.IsTrue(detector.Observe(new ShakeSample(10, 0, 100)));
    }

    [TestMethod]
    public void Observe_AllowsDistinctMovementSamplesWithSameTimestamp()
    {
        var detector = new ShakeDetector(ShakeSensitivity.High);

        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 0)));
        Assert.IsFalse(detector.Observe(new ShakeSample(10, 0, 50)));
        Assert.IsFalse(detector.Observe(new ShakeSample(0, 0, 50)));
        Assert.IsTrue(detector.Observe(new ShakeSample(10, 0, 100)));
    }

    [TestMethod]
    public void Observe_RealisticContinuousMouseStream_TriggersAtComfortableStroke()
    {
        var detector = new ShakeDetector(ShakeSensitivity.Normal);

        // Realistic 125Hz mouse reports moving smoothly right 18px, left 18px, right 18px
        // Small incremental deltas of 1-3px per report
        uint t = 100;
        int x = 100;
        int y = 100;
        Assert.IsFalse(detector.Observe(new ShakeSample(x, y, t)));

        // Stroke 1: Move right to 118 (+18px in 6 steps of 3px)
        for (var i = 0; i < 6; i++)
        {
            x += 3;
            t += 10;
            Assert.IsFalse(detector.Observe(new ShakeSample(x, y, t)));
        }

        // Stroke 2: Reversal 1 - Move left to 100 (-18px in 6 steps of 3px)
        for (var i = 0; i < 6; i++)
        {
            x -= 3;
            t += 10;
            Assert.IsFalse(detector.Observe(new ShakeSample(x, y, t)));
        }

        // Stroke 3: Reversal 2 - Move right to 118 (+18px in 6 steps of 3px)
        var triggered = false;
        for (var i = 0; i < 6; i++)
        {
            x += 3;
            t += 10;
            if (detector.Observe(new ShakeSample(x, y, t)))
            {
                triggered = true;
                break;
            }
        }

        Assert.IsTrue(triggered, "Continuous realistic stream must accumulate anchor displacement and trigger on 2 reversals.");
    }

    [TestMethod]
    public void Observe_DenseSubThresholdSamples_AccumulatesStrokeAndTriggers()
    {
        var detector = new ShakeDetector(ShakeSensitivity.Normal);

        // High-cadence stream: 1px movement per sample every 2ms (500Hz gaming mouse / high DPI)
        // None of the individual samples reach the 10px minimum movement threshold on their own.
        uint t = 1000;
        int x = 500;
        int y = 500;
        Assert.IsFalse(detector.Observe(new ShakeSample(x, y, t)));

        // Swing 1: Move right by 15px (15 samples of 1px)
        for (var i = 0; i < 15; i++)
        {
            x++;
            t += 2;
            Assert.IsFalse(detector.Observe(new ShakeSample(x, y, t)));
        }

        // Swing 2 (Reversal 1): Move left by 15px (15 samples of 1px)
        for (var i = 0; i < 15; i++)
        {
            x--;
            t += 2;
            Assert.IsFalse(detector.Observe(new ShakeSample(x, y, t)));
        }

        // Swing 3 (Reversal 2): Move right by 15px
        var triggered = false;
        for (var i = 0; i < 15; i++)
        {
            x++;
            t += 2;
            if (detector.Observe(new ShakeSample(x, y, t)))
            {
                triggered = true;
                break;
            }
        }

        Assert.IsTrue(triggered, "Dense sub-threshold samples must accumulate into swings and trigger on second reversal.");
    }
}
