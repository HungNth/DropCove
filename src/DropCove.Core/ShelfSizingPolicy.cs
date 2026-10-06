namespace DropCove.Core;

/// <summary>Resolves responsive card capacity and transient automatic shelf height.</summary>
public static class ShelfSizingPolicy
{
    /// <summary>Gets columns fitting 164-pixel cards, four-pixel gaps and 16-pixel outer padding.</summary>
    /// <param name="width">The actual logical window width.</param>
    /// <returns>At least one column.</returns>
    public static int ColumnCount(double width) => Math.Max(1, (int)Math.Floor((Math.Max(0, width - 16) + 4) / 168));

    /// <summary>Gets the bounded height tier for the current responsive rows.</summary>
    /// <param name="width">The actual logical window width.</param>
    /// <param name="batchCount">The number of Shelf Batches.</param>
    /// <returns>A logical height of 180, 236, 292 or 348.</returns>
    public static int AutomaticHeight(double width, int batchCount)
    {
        var columns = ColumnCount(width);
        var rows = batchCount / columns + (batchCount % columns == 0 ? 0 : 1);
        return 180 + 56 * Math.Clamp(rows - 1, 0, 3);
    }

    /// <summary>Resolves grow-only height after an accepted visible drop.</summary>
    /// <param name="width">The current actual logical width.</param>
    /// <param name="height">The current actual logical height.</param>
    /// <param name="batchCount">The accepted Shelf Batch count.</param>
    /// <param name="manualHeight">The optional explicit height override.</param>
    /// <returns>The current height or a taller automatic tier.</returns>
    public static int HeightAfterAcceptedDrop(double width, int height, int batchCount, int? manualHeight) =>
        manualHeight.HasValue ? height : Math.Max(height, AutomaticHeight(width, batchCount));
}
