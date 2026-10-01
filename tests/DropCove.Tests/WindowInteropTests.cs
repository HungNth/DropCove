using DropCove.Native;

namespace DropCove.Tests;

[TestClass]
public sealed class WindowInteropTests
{
    [TestMethod]
    [DataRow(420, 96u, 420)]
    [DataRow(420, 144u, 630)]
    [DataRow(700, 192u, 1400)]
    [DataRow(1, 144u, 2)]
    public void ScaleLogicalPixels_UsesEffectiveMonitorDpi(int logicalPixels, uint dpi, int expectedPhysicalPixels)
    {
        var scaled = WindowInterop.ScaleLogicalPixels(logicalPixels, dpi);

        Assert.AreEqual(expectedPhysicalPixels, scaled);
    }
}
