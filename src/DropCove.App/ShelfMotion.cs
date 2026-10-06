using System.Numerics;

using DropCove.Native;
using Microsoft.UI.Composition;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Hosting;

namespace DropCove;

/// <summary>Applies short post-state-change composition transitions to visible shelf surfaces.</summary>
internal static class ShelfMotion
{
    private static readonly TimeSpan TransitionDuration = TimeSpan.FromMilliseconds(120);

    /// <summary>Animates the shelf content when the window first becomes visible.</summary>
    /// <param name="element">The visible shelf content.</param>
    public static void PlayEntrance(UIElement element) => Play(element, 0.94f, 0.78f);

    /// <summary>Animates a completed shelf state mutation or presentation change.</summary>
    /// <param name="element">The updated shelf surface.</param>
    public static void PlayStateChange(UIElement element) => Play(element, 0.98f, 0.86f);
    /// <summary>Animates a newly realized Shelf Batch after it has entered the shared state.</summary>
    /// <param name="element">The realized batch element.</param>
    public static void PlayInsertion(UIElement element) => Play(element, 0.96f, 0.7f);

    /// <summary>Animates one realized element out before its already-committed removal is rebound.</summary>
    /// <param name="element">The realized element being removed.</param>
    /// <returns>A task that completes when the short exit transition finishes.</returns>
    public static Task PlayExitAsync(UIElement element)
    {
        ArgumentNullException.ThrowIfNull(element);

        var visual = ElementCompositionPreview.GetElementVisual(element);
        Reset(visual);
        if (!WindowInterop.AreAnimationsEnabled())
        {
            return Task.CompletedTask;
        }

        var compositor = visual.Compositor;
        var opacity = compositor.CreateScalarKeyFrameAnimation();
        opacity.Duration = TransitionDuration;
        opacity.InsertKeyFrame(0f, 1f);
        opacity.InsertKeyFrame(1f, 0f);

        var scale = compositor.CreateVector3KeyFrameAnimation();
        scale.Duration = TransitionDuration;
        scale.InsertKeyFrame(0f, Vector3.One);
        scale.InsertKeyFrame(1f, new Vector3(0.98f));

        var completion = new TaskCompletionSource<bool>(TaskCreationOptions.RunContinuationsAsynchronously);
        var batch = compositor.CreateScopedBatch(CompositionBatchTypes.Animation);
        batch.Completed += (_, _) =>
        {
            visual.StopAnimation(nameof(Visual.Opacity));
            visual.StopAnimation(nameof(Visual.Scale));
            visual.Opacity = 0f;
            visual.Scale = new Vector3(0.98f);
            opacity.Dispose();
            scale.Dispose();
            batch.Dispose();
            completion.TrySetResult(true);
        };
        visual.StartAnimation(nameof(Visual.Opacity), opacity);
        visual.StartAnimation(nameof(Visual.Scale), scale);
        batch.End();
        return completion.Task;
    }
    /// <summary>Restores an element after an exit animation before it can be recycled.</summary>
    /// <param name="element">The element whose composition state must return to its default.</param>
    public static void Reset(UIElement element)
    {
        ArgumentNullException.ThrowIfNull(element);
        Reset(ElementCompositionPreview.GetElementVisual(element));
    }



    /// <summary>Animates the Edge Rail after its expanded state changes.</summary>
    /// <param name="element">The rail root.</param>
    /// <param name="expanded">Whether the rail is now expanded.</param>
    public static void PlayRailTransition(UIElement element, bool expanded) =>
        Play(element, expanded ? 0.96f : 1.04f, 0.9f);

    private static void Play(UIElement element, float startingScale, float startingOpacity)
    {
        ArgumentNullException.ThrowIfNull(element);

        var visual = ElementCompositionPreview.GetElementVisual(element);
        Reset(visual);

        if (!WindowInterop.AreAnimationsEnabled())
        {
            return;
        }

        var compositor = visual.Compositor;
        var easing = compositor.CreateCubicBezierEasingFunction(
            new Vector2(0.2f, 0.0f),
            new Vector2(0.0f, 1.0f));

        var opacity = compositor.CreateScalarKeyFrameAnimation();
        opacity.Duration = TransitionDuration;
        opacity.InsertKeyFrame(0f, startingOpacity);
        opacity.InsertKeyFrame(1f, 1f, easing);

        var scale = compositor.CreateVector3KeyFrameAnimation();
        scale.Duration = TransitionDuration;
        scale.InsertKeyFrame(0f, new Vector3(startingScale));
        scale.InsertKeyFrame(1f, Vector3.One, easing);

        var batch = compositor.CreateScopedBatch(CompositionBatchTypes.Animation);
        batch.Completed += (_, _) =>
        {
            visual.StopAnimation(nameof(Visual.Opacity));
            visual.StopAnimation(nameof(Visual.Scale));
            visual.Opacity = 1f;
            visual.Scale = Vector3.One;
            opacity.Dispose();
            scale.Dispose();
            easing.Dispose();
            batch.Dispose();
        };
        visual.StartAnimation(nameof(Visual.Opacity), opacity);
        visual.StartAnimation(nameof(Visual.Scale), scale);
        batch.End();
    }

    private static void Reset(Visual visual)
    {
        visual.StopAnimation(nameof(Visual.Opacity));
        visual.StopAnimation(nameof(Visual.Scale));
        visual.Opacity = 1f;
        visual.Scale = Vector3.One;
    }
}
