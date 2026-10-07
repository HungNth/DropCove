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
    [DataRow(1, 136)]
    [DataRow(2, 204)]
    [DataRow(3, 272)]
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
        // 1 batch (136) growing to 2 batches (204)
        Assert.AreEqual(204, EdgeRailSizingPolicy.HeightAfterMutation(136, 2));

        // 2 batches (204) growing to 5 batches (408)
        Assert.AreEqual(408, EdgeRailSizingPolicy.HeightAfterMutation(204, 5));

        // 8 batches (612) growing to 9 batches (640)
        Assert.AreEqual(640, EdgeRailSizingPolicy.HeightAfterMutation(612, 9));
    }

    [TestMethod]
    public void HeightAfterMutation_PreservesCurrentHeightWhenTargetShrinksOrRemainsSame()
    {
        // 3 batches removed down to 2 while expanded at 272: hold at 272
        Assert.AreEqual(272, EdgeRailSizingPolicy.HeightAfterMutation(272, 2));

        // 8 batches removed down to 1 while expanded at 612: hold at 612
        Assert.AreEqual(612, EdgeRailSizingPolicy.HeightAfterMutation(612, 1));

        // Equal target: remains 204
        Assert.AreEqual(204, EdgeRailSizingPolicy.HeightAfterMutation(204, 2));

        // Removal down to 0 while expanded: preserves current height (manager handles hide transition)
        Assert.AreEqual(272, EdgeRailSizingPolicy.HeightAfterMutation(272, 0));
    }

    [TestMethod]
    public void TargetExpandedHeight_RecomputesFreshSmallerTargetOnNextExpansion()
    {
        // Demonstrates the lifecycle:
        // Expanded with 3 batches: height is 272.
        var currentHeight = EdgeRailSizingPolicy.TargetExpandedHeight(3);
        Assert.AreEqual(272, currentHeight);

        // One batch removed while expanded: HeightAfterMutation preserves 272 (no live shrink).
        var heldHeight = EdgeRailSizingPolicy.HeightAfterMutation(currentHeight, 2);
        Assert.AreEqual(272, heldHeight);

        // Rail collapses and re-expands with the remaining 2 batches: target height recomputes to 204.
        var recomputedHeight = EdgeRailSizingPolicy.TargetExpandedHeight(2);
        Assert.AreEqual(204, recomputedHeight);
    }
}
