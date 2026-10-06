using DropCove.Core;

namespace DropCove.Tests;

[TestClass]
public sealed class ShelfSizingPolicyTests
{
    [TestMethod]
    [DataRow(180, 1)]
    [DataRow(347, 1)]
    [DataRow(348, 2)]
    [DataRow(515, 2)]
    [DataRow(516, 3)]
    public void ColumnBoundaries_MatchRenderedCardCapacity(int width, int expected) =>
        Assert.AreEqual(expected, ShelfSizingPolicy.ColumnCount(width));

    [TestMethod]
    [DataRow(180, 0, 180)]
    [DataRow(180, 1, 180)]
    [DataRow(180, 2, 236)]
    [DataRow(180, 3, 292)]
    [DataRow(180, 4, 348)]
    [DataRow(180, 5, 348)]
    [DataRow(348, 2, 180)]
    [DataRow(348, 3, 236)]
    [DataRow(516, 3, 180)]
    [DataRow(516, 4, 236)]
    public void OpeningHeight_UsesResponsiveRowsAndFourRowCap(int width, int batches, int expected) =>
        Assert.AreEqual(expected, ShelfSizingPolicy.AutomaticHeight(width, batches));

    [TestMethod]
    public void AcceptedDrop_GrowsOnlyWithoutManualOverrideAndNeverShrinks()
    {
        Assert.AreEqual(236, ShelfSizingPolicy.HeightAfterAcceptedDrop(180, 180, 2, null));
        Assert.AreEqual(348, ShelfSizingPolicy.HeightAfterAcceptedDrop(180, 348, 1, null));
        Assert.AreEqual(180, ShelfSizingPolicy.HeightAfterAcceptedDrop(180, 180, 5, 180));
        Assert.AreEqual(180, ShelfSizingPolicy.HeightAfterAcceptedDrop(348, 180, 2, null));
    }
}
