namespace DropCove.Core;

/// <summary>Resolves Edge Rail dimensions and adaptive expanded height.</summary>
public static class EdgeRailSizingPolicy
{
    /// <summary>The resting Rail Handle width in logical pixels.</summary>
    public const int HandleWidth = 16;

    /// <summary>The resting Rail Handle height in logical pixels.</summary>
    public const int HandleHeight = 96;

    /// <summary>The fixed expanded Edge Rail width in logical pixels.</summary>
    public const int ExpandedWidth = 280;

    /// <summary>The maximum expanded Edge Rail height in logical pixels.</summary>
    public const int MaxHeight = 640;

    /// <summary>The fixed height of a Shelf Batch summary row in logical pixels.</summary>
    public const int RowHeight = 64;

    /// <summary>The gap between adjacent Shelf Batch summary rows in logical pixels.</summary>
    public const int Gap = 4;

    /// <summary>Calculates the expanded target height for a given Shelf Batch count.</summary>
    /// <param name="batchCount">The number of Shelf Batches.</param>
    /// <returns>
    /// A positive logical height between 136 and 640 logical pixels when <paramref name="batchCount"/> is positive;
    /// otherwise, 0.
    /// </returns>
    public static int TargetExpandedHeight(int batchCount)
    {
        if (batchCount <= 0)
        {
            return 0;
        }

        // Tiers: min(640, 68 * n + 68). For n >= 9, result is capped at MaxHeight (640).
        // Check for overflow before multiplication if batchCount is huge.
        if (batchCount >= 9)
        {
            return MaxHeight;
        }

        return 68 * batchCount + 68;
    }

    /// <summary>Resolves grow-only height after an accepted mutation while expanded.</summary>
    /// <param name="currentHeight">The current actual logical height of the expanded rail.</param>
    /// <param name="batchCount">The new Shelf Batch count.</param>
    /// <returns>The taller of the current height or the newly derived target height.</returns>
    public static int HeightAfterMutation(int currentHeight, int batchCount) =>
        Math.Max(currentHeight, TargetExpandedHeight(batchCount));
}
