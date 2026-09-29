namespace DropCove.Core;

/// <summary>Describes one filesystem item offered by an incoming drop.</summary>
/// <param name="Path">The filesystem path supplied by Windows.</param>
/// <param name="Name">The user-visible item name.</param>
/// <param name="IsFolder">Whether the item is a folder.</param>
public sealed record IncomingShelfItem(string Path, string Name, bool IsFolder);

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

/// <summary>Reports the outcome of one incoming drop.</summary>
/// <param name="Batch">The created batch, or <see langword="null"/> when no item was accepted.</param>
/// <param name="AcceptedCount">The number of accepted unique paths.</param>
/// <param name="SkippedUnsupportedCount">The number of unsupported entries.</param>
/// <param name="DuplicateCount">The number of duplicate paths removed within this drop.</param>
public sealed record DropAcceptance(
    ShelfBatch? Batch,
    int AcceptedCount,
    int SkippedUnsupportedCount,
    int DuplicateCount);
