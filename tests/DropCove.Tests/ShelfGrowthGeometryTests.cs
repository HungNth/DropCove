using DropCove.Native;

namespace DropCove.Tests;

[TestClass]
public sealed class ShelfGrowthGeometryTests
{
    [TestMethod]
    public void Growth_KeepsWidthAndTopWhenTargetFits()
    {
        var current = new WindowBounds(1100, 250, 1280, 430);
        var work = new WindowBounds(0, 0, 1920, 1032);
        Assert.AreEqual(new WindowBounds(1100, 250, 1280, 486),
            WindowInterop.CalculateAnchoredGrowthBounds(current, work, 236, 96));
    }

    [TestMethod]
    public void Growth_ShiftsUpOnlyEnoughToStayReachable()
    {
        var current = new WindowBounds(1100, 796, 1280, 1032);
        var work = new WindowBounds(0, 0, 1920, 1032);
        Assert.AreEqual(new WindowBounds(1100, 740, 1280, 1032),
            WindowInterop.CalculateAnchoredGrowthBounds(current, work, 292, 96));
    }

    [TestMethod]
    public void Growth_ScalesHeightWithoutRoundingExistingPhysicalWidth()
    {
        var current = new WindowBounds(100, 100, 371, 370);
        var work = new WindowBounds(0, 0, 1920, 1032);
        Assert.AreEqual(new WindowBounds(100, 100, 371, 622),
            WindowInterop.CalculateAnchoredGrowthBounds(current, work, 348, 144));
    }

    [TestMethod]
    public void Growth_ClampsToSmallerThanMinimumWorkArea()
    {
        var current = new WindowBounds(20, 20, 200, 200);
        var work = new WindowBounds(0, 0, 120, 140);
        Assert.AreEqual(work, WindowInterop.CalculateAnchoredGrowthBounds(current, work, 348, 96));
    }
}
