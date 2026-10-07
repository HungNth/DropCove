using DropCove.Core;

namespace DropCove.Tests;

[TestClass]
public sealed class EdgeRailSizingPolicyTests
{
    [TestMethod]
    [DataRow(0, 0)]
    [DataRow(-1, 0)]
    [DataRow(-100, 0)]
    [DataRow(int.MinValue, 0)]
    public void TargetExpandedHeight_NonPositiveBatchCount_ReturnsZero(int batchCount, int expected) =>
        Assert.AreEqual(expected, EdgeRailSizingPolicy.TargetExpandedHeight(batchCount));

    [TestMethod]
    [DataRow(1, 280)]
    [DataRow(2, 280)]
    [DataRow(3, 280)]
    [DataRow(4, 340)]
    [DataRow(5, 408)]
    [DataRow(6, 476)]
    [DataRow(7, 544)]
    [DataRow(8, 612)]
    public void TargetExpandedHeight_OneThroughEightBatches_MatchesDeterministicTiers(int batchCount, int expected) =>
        Assert.AreEqual(expected, EdgeRailSizingPolicy.TargetExpandedHeight(batchCount));

    [TestMethod]
    [DataRow(9, 640)]
    [DataRow(10, 640)]
    [DataRow(100, 640)]
    [DataRow(1000, 640)]
    [DataRow(1_000_000, 640)]
    [DataRow(int.MaxValue, 640)]
    public void TargetExpandedHeight_NineOrMoreBatches_CapsAtMaxHeightWithoutOverflow(int batchCount, int expected) =>
        Assert.AreEqual(expected, EdgeRailSizingPolicy.TargetExpandedHeight(batchCount));

    [TestMethod]
    public void HeightAfterMutation_GrowsWhenNewTargetExceedsCurrentHeight()
    {
        // 1 batch (280) growing to 4 batches (340)
        Assert.AreEqual(340, EdgeRailSizingPolicy.HeightAfterMutation(280, 4));

        // 2 batches (280) growing to 5 batches (408)
        Assert.AreEqual(408, EdgeRailSizingPolicy.HeightAfterMutation(280, 5));


        // 8 batches (612) growing to 9 batches (640)
        Assert.AreEqual(640, EdgeRailSizingPolicy.HeightAfterMutation(612, 9));
    }

    [TestMethod]
    public void HeightAfterMutation_PreservesCurrentHeightWhenTargetShrinksOrRemainsSame()
    {
        // 4 batches removed down to 2 while expanded at 340: hold at 340
        Assert.AreEqual(340, EdgeRailSizingPolicy.HeightAfterMutation(340, 2));

        // 8 batches removed down to 1 while expanded at 612: hold at 612
        Assert.AreEqual(612, EdgeRailSizingPolicy.HeightAfterMutation(612, 1));

        // Equal target: remains 280
        Assert.AreEqual(280, EdgeRailSizingPolicy.HeightAfterMutation(280, 2));
        Assert.AreEqual(280, EdgeRailSizingPolicy.HeightAfterMutation(280, 1));

        // Removal down to 0 while expanded: preserves current height (manager handles hide transition)
        Assert.AreEqual(272, EdgeRailSizingPolicy.HeightAfterMutation(272, 0));
    }

    [TestMethod]
    public void TargetExpandedHeight_RecomputesFreshSmallerTargetOnNextExpansion()
    {
        // Demonstrates the lifecycle:
        // Expanded with 5 batches: height is 408.
        var currentHeight = EdgeRailSizingPolicy.TargetExpandedHeight(5);
        Assert.AreEqual(408, currentHeight);

        // One batch removed while expanded: HeightAfterMutation preserves 408 (no live shrink).
        var heldHeight = EdgeRailSizingPolicy.HeightAfterMutation(currentHeight, 4);
        Assert.AreEqual(408, heldHeight);

        // Rail collapses and re-expands with the remaining 4 batches: target height recomputes to 340.
        var recomputedHeight = EdgeRailSizingPolicy.TargetExpandedHeight(4);
        Assert.AreEqual(340, recomputedHeight);
    }
}
