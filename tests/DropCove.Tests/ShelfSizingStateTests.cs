using DropCove.Core;

namespace DropCove.Tests;

[TestClass]
public sealed class ShelfSizingStateTests
{
    [TestMethod]
    [DataRow(null)]
    [DataRow(292)]
    public void WidthOnlyResize_PreservesOptionalManualHeight(int? manualHeight)
    {
        var state = new ShelfSizingState(180, manualHeight);
        Assert.AreEqual(new ShelfSizingState(350, manualHeight), state.AfterUserResize(350, 236, 236, false));
    }

    [TestMethod]
    public void VerticalResize_ChangesOverrideOnlyWhenHeightChanges()
    {
        var state = new ShelfSizingState(350, null);
        Assert.AreEqual(new ShelfSizingState(350, null), state.AfterUserResize(350, 236, 236, true));
        Assert.AreEqual(new ShelfSizingState(350, 180), state.AfterUserResize(350, 236, 180, true));
        Assert.AreEqual(new ShelfSizingState(420, 292), state.AfterUserResize(420, 236, 292, true));
    }
}
