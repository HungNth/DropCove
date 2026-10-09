namespace DropCove.Core;

/// <summary>Describes one filesystem item offered by an incoming drop.</summary>
/// <param name="Path">The filesystem path supplied by Windows.</param>
/// <param name="Name">The user-visible item name.</param>
/// <param name="IsFolder">Whether the item is a folder.</param>
public sealed record IncomingShelfItem(string Path, string Name, bool IsFolder);

/// <summary>Describes the visible Drop Shelf presentation.</summary>
public enum ShelfDisplayState
{
    /// <summary>No DropCove surface is visible.</summary>
    Hidden,
    /// <summary>The single resizable Drop Shelf presentation is visible.</summary>
    Visible,
    /// <summary>The non-empty shelf is visible as the collapsed Edge Rail.</summary>
    EdgeDocked,
}


/// <summary>Represents durable user sizing intent independently from actual window bounds.</summary>
/// <param name="PreferredWidth">The preferred logical width.</param>
/// <param name="ManualHeightOverride">The explicit logical height, or null for automatic height.</param>
public readonly record struct ShelfSizingState(int PreferredWidth, int? ManualHeightOverride)
{
    /// <summary>Gets the compact width with automatic height enabled.</summary>
    public static ShelfSizingState Default => new(180, null);

    /// <summary>Gets whether all selected dimensions satisfy the logical minimum.</summary>
    public bool IsValid => PreferredWidth >= 180 && (ManualHeightOverride is null or >= 180);

    /// <summary>Resolves a completed native resize without locking automatic height on side-edge changes.</summary>
    /// <param name="width">The final logical width.</param>
    /// <param name="startHeight">The resize-start logical height.</param>
    /// <param name="endHeight">The final logical height.</param>
    /// <param name="verticalEdge">Whether the selected edge or corner permits vertical resizing.</param>
    /// <returns>The updated user sizing intent.</returns>
    public ShelfSizingState AfterUserResize(int width, int startHeight, int endHeight, bool verticalEdge) =>
        new(width, verticalEdge && startHeight != endHeight ? endHeight : ManualHeightOverride);
}

/// <summary>Identifies the screen edge used by the collapsed Edge Rail.</summary>
public enum ShelfRailEdge
{
    /// <summary>The left edge of the selected monitor.</summary>
    Left,
    /// <summary>The right edge of the selected monitor.</summary>
    Right,
}

/// <summary>Describes the remembered Edge Rail monitor and edge.</summary>
/// <param name="MonitorId">The Windows display device name, such as <c>\\.\DISPLAY1</c>.</param>
/// <param name="Edge">The selected monitor edge.</param>
public sealed record ShelfRailPlacement(string MonitorId, ShelfRailEdge Edge)
{
    /// <summary>Gets the default placement before the first non-empty dismissal.</summary>
    public static ShelfRailPlacement Default { get; } = new(string.Empty, ShelfRailEdge.Right);

    /// <summary>Gets whether a monitor has been selected.</summary>
    public bool HasMonitor => !string.IsNullOrWhiteSpace(MonitorId);
}

/// <summary>Represents one file or folder reference inside a shelf batch.</summary>
/// <param name="Id">The occurrence identity inside DropCove.</param>
/// <param name="Path">The referenced filesystem path.</param>
/// <param name="Name">The user-visible item name.</param>
/// <param name="IsFolder">Whether the item is a folder.</param>
/// <param name="IsPinned">Whether accepted drag-out operations retain the reference.</param>
/// <param name="Availability">The path's latest event-boundary classification.</param>
public sealed record ShelfItem(
    Guid Id,
    string Path,
    string Name,
    bool IsFolder,
    bool IsPinned = false,
    ItemAvailability Availability = ItemAvailability.Available);

/// <summary>Represents the ordered items accepted from one drop operation.</summary>
/// <param name="Id">The batch identity.</param>
/// <param name="CreatedAt">When the drop was accepted.</param>
/// <param name="Items">The accepted items in source order.</param>
public sealed record ShelfBatch(Guid Id, DateTimeOffset CreatedAt, IReadOnlyList<ShelfItem> Items);


/// <summary>Classifies the filesystem availability of a Shelf Item path.</summary>
public enum ItemAvailability
{
    /// <summary>The file or folder exists and is accessible.</summary>
    Available,
    /// <summary>The file or folder is confirmed absent.</summary>
    Missing,
    /// <summary>The path cannot currently be accessed because of volume, network, cloud, or permission state.</summary>
    Unavailable,
}

/// <summary>Result of preparing a batch for drag-out.</summary>
/// <param name="AvailableItems">Items that are confirmed available.</param>
/// <param name="UnavailableItems">Items that are inaccessible but retained.</param>
/// <param name="MissingItemsRemovedCount">The count of confirmed-Missing items pruned from the shelf.</param>
public sealed record BatchDragPreparation(
    IReadOnlyList<ShelfItem> AvailableItems,
    IReadOnlyList<ShelfItem> UnavailableItems,
    int MissingItemsRemovedCount);
/// <summary>Describes how one native drag-out operation completed.</summary>
public enum DragOutOutcome
{
    /// <summary>The user canceled the drag.</summary>
    Canceled,
    /// <summary>The destination rejected the offered data.</summary>
    Rejected,
    /// <summary>The operation failed before a destination accepted it.</summary>
    Failed,
    /// <summary>The destination accepted the item with the Copy effect.</summary>
    AcceptedCopy,
}

/// <summary>Distinguishes new content, a duplicate batch, and unsupported incoming data.</summary>
public enum DropDisposition
{
    /// <summary>A new Shelf Batch was created.</summary>
    Added,
    /// <summary>The incoming path set already exists in a retained Shelf Batch.</summary>
    Duplicate,
    /// <summary>No supported path remained after incoming filtering.</summary>
    Unsupported,
}

/// <summary>Reports the outcome of one incoming drop.</summary>
/// <param name="Disposition">Whether the drop created a batch, was duplicate, or was unsupported.</param>
/// <param name="Batch">The created batch, or <see langword="null"/> when no batch was added.</param>
/// <param name="AcceptedCount">The number of unique paths added to DropCove.</param>
/// <param name="SkippedUnsupportedCount">The number of unsupported entries.</param>
/// <param name="DuplicatePathCount">The number of repeated paths removed within this drop.</param>
public sealed record DropAcceptance(
    DropDisposition Disposition,
    ShelfBatch? Batch,
    int AcceptedCount,
    int SkippedUnsupportedCount,
    int DuplicatePathCount);
