namespace DropCove.Tests;

[TestClass]
public sealed class TestRunnerSmokeTests
{
    [TestMethod]
    public void TestAssemblyLoads()
    {
        Assert.IsFalse(string.IsNullOrWhiteSpace(typeof(TestRunnerSmokeTests).Assembly.Location));
    }
}
