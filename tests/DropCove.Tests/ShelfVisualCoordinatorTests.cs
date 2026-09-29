using DropCove.Core;

namespace DropCove.Tests;

[TestClass]
public sealed class ShelfVisualCoordinatorTests
{
    [TestMethod]
    public async Task NonImageItem_UsesNativeIconWithoutRequestingThumbnail()
    {
        var provider = new FakeVisualProvider
        {
            NativeIcon = "native-icon",
            Thumbnail = "thumbnail",
        };
        var coordinator = new ShelfVisualCoordinator<string>(provider, item => item.Name.EndsWith(".png", StringComparison.OrdinalIgnoreCase));

        var result = await coordinator.LoadAsync(CreateItem("notes.txt"));

        Assert.AreEqual("native-icon", result.Display);
        Assert.IsFalse(result.UsedThumbnail);
        Assert.AreEqual(1, provider.NativeIconCalls);
        Assert.AreEqual(0, provider.ThumbnailCalls);
    }

    [TestMethod]
    public async Task ImageItem_UsesThumbnailWhenShellReturnsOne()
    {
        var provider = new FakeVisualProvider
        {
            NativeIcon = "native-icon",
            Thumbnail = "thumbnail",
        };
        var coordinator = new ShelfVisualCoordinator<string>(provider, item => item.Name.EndsWith(".png", StringComparison.OrdinalIgnoreCase));

        var result = await coordinator.LoadAsync(CreateItem("photo.png"));

        Assert.AreEqual("thumbnail", result.Display);
        Assert.IsTrue(result.UsedThumbnail);
        Assert.AreEqual(1, provider.NativeIconCalls);
        Assert.AreEqual(1, provider.ThumbnailCalls);
    }

    [TestMethod]
    public async Task AvailableThumbnailDoesNotWaitForSlowNativeIcon()
    {
        var provider = new FakeVisualProvider
        {
            NativeIcon = "native-icon",
            Thumbnail = "thumbnail",
            WaitForNativeCompletion = true,
        };
        var coordinator = new ShelfVisualCoordinator<string>(provider, _ => true);
        var request = coordinator.LoadAsync(CreateItem("photo.png"));

        await provider.NativeStarted.Task;
        await provider.ThumbnailStarted.Task;
        var completed = await Task.WhenAny(request, Task.Delay(TimeSpan.FromSeconds(1)));

        Assert.AreSame(request, completed);
        var result = await request;
        Assert.AreEqual("thumbnail", result.Display);
        Assert.IsTrue(result.UsedThumbnail);

        provider.NativeRelease.TrySetResult(true);
        await provider.NativeFinished.Task;
    }

    [TestMethod]
    public async Task ThumbnailUnavailable_RetainsNativeIconAndDoesNotCacheFailure()
    {
        var provider = new FakeVisualProvider
        {
            NativeIcon = "native-icon",
            Thumbnail = null,
        };
        var coordinator = new ShelfVisualCoordinator<string>(provider, item => true);
        var item = CreateItem("photo.png");

        var first = await coordinator.LoadAsync(item);
        var second = await coordinator.LoadAsync(item);

        Assert.AreEqual("native-icon", first.Display);
        Assert.IsFalse(first.UsedThumbnail);
        Assert.AreEqual("native-icon", second.Display);
        Assert.IsFalse(second.UsedThumbnail);
        Assert.AreEqual(2, provider.ThumbnailCalls);
    }

    [TestMethod]
    public async Task Cancellation_ReachesVisibleRequestProvider()
    {
        var provider = new FakeVisualProvider
        {
            NativeIcon = "native-icon",
            WaitForThumbnailCancellation = true,
        };
        var coordinator = new ShelfVisualCoordinator<string>(provider, item => true);
        using var cancellation = new CancellationTokenSource();
        var request = coordinator.LoadAsync(CreateItem("photo.png"), cancellation.Token);

        await provider.ThumbnailStarted.Task;
        cancellation.Cancel();

        var canceled = false;
        try
        {
            await request;
        }
        catch (OperationCanceledException)
        {
            canceled = true;
        }

        Assert.IsTrue(canceled);
        Assert.IsTrue(provider.ThumbnailToken.IsCancellationRequested);
    }

    private static ShelfItem CreateItem(string name) =>
        new(Guid.NewGuid(), $"C:\\Work\\{name}", name, false);

    private sealed class FakeVisualProvider : IShelfVisualProvider<string>
    {
        public string? NativeIcon { get; init; }
        public string? Thumbnail { get; init; }
        public bool WaitForNativeCompletion { get; init; }
        public bool WaitForThumbnailCancellation { get; init; }
        public int NativeIconCalls { get; private set; }
        public int ThumbnailCalls { get; private set; }
        public CancellationToken ThumbnailToken { get; private set; }
        public TaskCompletionSource<bool> NativeStarted { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        public TaskCompletionSource<bool> NativeRelease { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        public TaskCompletionSource<bool> NativeFinished { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        public TaskCompletionSource<bool> ThumbnailStarted { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        public async Task<string?> LoadNativeIconAsync(ShelfItem item, CancellationToken cancellationToken)
        {
            NativeIconCalls++;
            if (WaitForNativeCompletion)
            {
                NativeStarted.TrySetResult(true);
                await NativeRelease.Task.WaitAsync(cancellationToken);
                NativeFinished.TrySetResult(true);
            }

            return NativeIcon;
        }

        public async Task<string?> LoadThumbnailAsync(ShelfItem item, CancellationToken cancellationToken)
        {
            ThumbnailCalls++;
            ThumbnailToken = cancellationToken;
            ThumbnailStarted.TrySetResult(true);
            if (WaitForThumbnailCancellation)
            {
                await Task.Delay(Timeout.InfiniteTimeSpan, cancellationToken);
            }

            return Thumbnail;
        }
    }
}
