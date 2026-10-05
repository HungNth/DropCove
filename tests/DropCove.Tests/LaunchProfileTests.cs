using DropCove.Services;

namespace DropCove.Tests;

[TestClass]
public sealed class LaunchProfileTests
{
    [TestMethod]
    public void Parse_DefaultArguments_ReturnsLegacyProfile()
    {
        var profile = LaunchProfile.Parse([]);
        var expectedDir = Path.TrimEndingDirectorySeparator(Path.GetFullPath(Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "DropCove")));

        Assert.IsFalse(profile.IsTest);
        Assert.AreEqual(expectedDir, profile.DirectoryPath);
        Assert.AreEqual(Path.Combine(expectedDir, "shelf.db"), profile.DatabasePath);
        Assert.AreEqual(Path.Combine(expectedDir, "settings.json"), profile.SettingsPath);
        Assert.AreEqual(@"Local\DropCove.SingleInstance", profile.MutexName);
        Assert.AreEqual(@"Local\DropCove.Activate", profile.ActivationEventName);
    }

    [TestMethod]
    public void Parse_ValidTestProfile_NormalizesAndDerivesUniqueIdentity()
    {
        var testPath = Path.Combine(Path.GetTempPath(), "DropCove-TestProfile-UnitTests");
        var profile = LaunchProfile.Parse(["--test-profile", testPath]);

        Assert.IsTrue(profile.IsTest);
        Assert.AreEqual(Path.TrimEndingDirectorySeparator(Path.GetFullPath(testPath)), profile.DirectoryPath);
        Assert.AreEqual(Path.Combine(profile.DirectoryPath, "shelf.db"), profile.DatabasePath);
        Assert.AreEqual(Path.Combine(profile.DirectoryPath, "settings.json"), profile.SettingsPath);
        Assert.StartsWith(@"Local\DropCove.SingleInstance.Test.", profile.MutexName);
        Assert.StartsWith(@"Local\DropCove.Activate.Test.", profile.ActivationEventName);

        var lowercasePath = testPath.ToLowerInvariant() + Path.DirectorySeparatorChar;
        var lowercaseProfile = LaunchProfile.Parse(["--test-profile", lowercasePath]);
        Assert.AreEqual(profile.MutexName, lowercaseProfile.MutexName);
        Assert.AreEqual(profile.ActivationEventName, lowercaseProfile.ActivationEventName);
    }

    [TestMethod]
    [DataRow("--test-profile")]
    [DataRow("--test-profile", "relative-path")]
    [DataRow("--test-profile", "--autostart")]
    public void Parse_InvalidArguments_ThrowsArgumentException(params string[] args)
    {
        Assert.Throws<ArgumentException>(() => LaunchProfile.Parse(args));
    }

    [TestMethod]
    public void Parse_InlineTestProfileOption_ThrowsArgumentException()
    {
        var inlineArg = "--test-profile=" + Path.GetTempPath();
        Assert.Throws<ArgumentException>(() => LaunchProfile.Parse([inlineArg]));
    }

    [TestMethod]
    public void Parse_RepeatedTestProfileOption_ThrowsArgumentException()
    {
        var path1 = Path.Combine(Path.GetTempPath(), "profile1");
        var path2 = Path.Combine(Path.GetTempPath(), "profile2");
        Assert.Throws<ArgumentException>(() => LaunchProfile.Parse(["--test-profile", path1, "--test-profile", path2]));
    }

    [TestMethod]
    public void Parse_DefaultDirectoryAsTestProfile_ThrowsArgumentException()
    {
        var defaultDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "DropCove");
        Assert.Throws<ArgumentException>(() => LaunchProfile.Parse(["--test-profile", defaultDir]));
    }
}
