namespace DropCove.Core;

/// <summary>Owns the in-memory Shelf Batch state and display transitions used by the application seam.</summary>
public sealed class DropShelfManager
{
    private readonly List<ShelfBatch> _batches;
    private readonly ShelfDatabase? _database;
    private readonly SemaphoreSlim _mutationGate = new(1, 1);
    private readonly Dictionary<Guid, HashSet<Guid>> _preparedBatchItems = [];
    private ShelfDisplayState _displayState = ShelfDisplayState.Hidden;
    private ShelfRailPlacement _railPlacement = ShelfRailPlacement.Default;
    private ShelfSizingState _sizingState = ShelfSizingState.Default;
    /// <summary>Creates an in-memory manager for transient callers.</summary>
    public DropShelfManager()
    {
        _batches = [];
    }

    private DropShelfManager(
        ShelfDatabase database,
        IReadOnlyList<ShelfBatch> batches,
        ShelfRailPlacement railPlacement,
        ShelfSizingState sizingState,
        string? recoveryBackupPath)
    {
        _database = database;
        _batches = [.. batches];
        _railPlacement = railPlacement;
        _sizingState = sizingState;
        RecoveryBackupPath = recoveryBackupPath;
    }

    /// <summary>Gets Shelf Batches in newest-first order.</summary>
    public IReadOnlyList<ShelfBatch> Batches => _batches;

    /// <summary>Gets the preserved failed database path when startup recovered from database failure.</summary>
    public string? RecoveryBackupPath { get; }

    /// <summary>Gets the current shelf presentation state.</summary>
    public ShelfDisplayState DisplayState => _displayState;

    /// <summary>Gets the remembered Edge Rail placement.</summary>
    public ShelfRailPlacement RailPlacement => _railPlacement;

    /// <summary>Gets durable width and optional explicit height.</summary>
    public ShelfSizingState SizingState => _sizingState;

    /// <summary>Records user-selected sizing intent, persisting it only while content exists.</summary>
    /// <param name="state">The preferred width and optional manual height.</param>
    /// <param name="cancellationToken">Cancels persistence.</param>
    public async Task SetSizingStateAsync(
        ShelfSizingState state,
        CancellationToken cancellationToken = default)
    {
        if (!state.IsValid)
        {
            throw new ArgumentOutOfRangeException(nameof(state), state, "Selected dimensions must be at least 180 logical pixels.");
        }

        using var mutation = await LockMutationsAsync(cancellationToken);
        if (_batches.Count > 0 && _database is not null)
        {
            await _database.SaveSizingStateAsync(state, cancellationToken).ConfigureAwait(false);
        }

        _sizingState = state;
    }

    /// <summary>Marks the resizable Drop Shelf as visible.</summary>
    public void ShowShelf() => _displayState = ShelfDisplayState.Visible;

    /// <summary>Marks the shelf as hidden without changing held Shelf Batches.</summary>
    public void HideShelf() => _displayState = ShelfDisplayState.Hidden;

    /// <summary>Dismisses the shelf into Hidden or EdgeDocked based on held content.</summary>
    /// <param name="placement">The monitor and edge to remember for a non-empty shelf.</param>
    /// <returns>The resulting display state.</returns>
    public ShelfDisplayState DismissShelf(ShelfRailPlacement placement)
    {
        ArgumentNullException.ThrowIfNull(placement);
        if (_batches.Count == 0)
        {
            _displayState = ShelfDisplayState.Hidden;
            return _displayState;
        }

        _railPlacement = placement;
        _displayState = ShelfDisplayState.EdgeDocked;
        return _displayState;
    }

    /// <summary>Dismisses the shelf and persists the remembered Edge Rail placement.</summary>
    /// <param name="placement">The monitor and edge to remember for a non-empty shelf.</param>
    /// <param name="cancellationToken">Cancels persistence.</param>
    /// <returns>The resulting display state.</returns>
    public async Task<ShelfDisplayState> DismissShelfAsync(
        ShelfRailPlacement placement,
        CancellationToken cancellationToken = default)
    {
        using var mutation = await LockMutationsAsync(cancellationToken);
        var state = DismissShelf(placement);
        if (state == ShelfDisplayState.EdgeDocked && _database is not null)
        {
            await _database.SaveRailPlacementAsync(_railPlacement, cancellationToken);
        }

        return state;
    }

    /// <summary>Updates and persists the remembered Edge Rail placement.</summary>
    /// <param name="placement">The monitor and edge to remember.</param>
    /// <param name="cancellationToken">Cancels persistence.</param>
    public async Task SetRailPlacementAsync(
        ShelfRailPlacement placement,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(placement);
        using var mutation = await LockMutationsAsync(cancellationToken);
        if (_database is not null)
        {
            await _database.SaveRailPlacementAsync(placement, cancellationToken);
        }

        _railPlacement = placement;
    }


    /// <summary>Opens a persistent manager and restores its shelf state.</summary>
    /// <param name="databasePath">The per-user SQLite database path.</param>
    /// <param name="classifyAvailability">Classifies restored paths without changing persisted metadata.</param>
    /// <param name="cancellationToken">Cancels database startup work.</param>
    /// <returns>The restored manager.</returns>
    public static async Task<DropShelfManager> OpenAsync(
        string databasePath,
        Func<string, ItemAvailability> classifyAvailability,
        CancellationToken cancellationToken = default)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(databasePath);
        ArgumentNullException.ThrowIfNull(classifyAvailability);
        var database = new ShelfDatabase(databasePath);
        var opened = await database.OpenAndLoadAsync(cancellationToken).ConfigureAwait(false);
        var batches = opened.Batches.Select(batch => batch with
        {
            Items = batch.Items.Select(item => item with
            {
                Availability = ClassifyRestoredPath(item.Path, classifyAvailability),
            }).ToArray(),
        }).ToArray();
        return new DropShelfManager(database, batches, opened.RailPlacement, opened.SizingState, opened.RecoveryBackupPath);
    }

    /// <summary>Accepts and persists one incoming drop.</summary>
    public async Task<DropAcceptance> AcceptDropAsync(
        IEnumerable<IncomingShelfItem> items,
        int skippedUnsupportedCount = 0,
        CancellationToken cancellationToken = default)
    {
        using var mutation = await LockMutationsAsync(cancellationToken);
        var outcome = AcceptDrop(items, skippedUnsupportedCount);
        if (outcome.Batch is not null && _database is not null)
        {
            try
            {
                await _database.AddBatchAsync(outcome.Batch, _batches.Count == 1 ? _sizingState : null, cancellationToken);
            }
            catch
            {
                _batches.Remove(outcome.Batch);
                throw;
            }
        }

        return outcome;
    }

    /// <summary>Updates and persists one Shelf Item's pinned lifecycle state.</summary>
    public async Task<bool> SetPinnedAsync(
        Guid itemId,
        bool isPinned,
        CancellationToken cancellationToken = default)
    {
        using var mutation = await LockMutationsAsync(cancellationToken);
        if (!TryFindItem(itemId, out var batchIndex, out var itemIndex))
        {
            return false;
        }

        var previous = _batches[batchIndex].Items[itemIndex].IsPinned;
        if (previous == isPinned)
        {
            return true;
        }

        if (_database is not null)
        {
            await _database.SetPinnedAsync(itemId, isPinned, cancellationToken);
        }

        return SetPinned(itemId, isPinned);
    }

    /// <summary>Removes and persists one Shelf Item reference.</summary>
    public async Task<bool> RemoveItemAsync(Guid itemId, CancellationToken cancellationToken = default)
    {
        using var mutation = await LockMutationsAsync(cancellationToken);
        if (!TryFindItem(itemId, out _, out _))
        {
            return false;
        }

        if (_database is not null)
        {
            await _database.RemoveItemsAsync([itemId], cancellationToken);
        }
        return RemoveItem(itemId);
    }

    /// <summary>Applies and persists a completed one-item drag-out result.</summary>
    public async Task<bool> CompleteItemDragAsync(
        Guid itemId,
        DragOutOutcome outcome,
        CancellationToken cancellationToken = default)
    {
        using var mutation = await LockMutationsAsync(cancellationToken);
        if (outcome != DragOutOutcome.AcceptedCopy ||
            !TryFindItem(itemId, out var batchIndex, out var itemIndex) ||
            _batches[batchIndex].Items[itemIndex].IsPinned)
        {
            return false;
        }

        if (_database is not null)
        {
            await _database.RemoveItemsAsync([itemId], cancellationToken);
        }
        return RemoveItem(itemId);
    }

    /// <summary>Removes and persists an entire Shelf Batch.</summary>
    public async Task<bool> RemoveBatchAsync(Guid batchId, CancellationToken cancellationToken = default)
    {
        using var mutation = await LockMutationsAsync(cancellationToken);
        if (!_batches.Any(batch => batch.Id == batchId))
        {
            return false;
        }

        if (_database is not null)
        {
            await _database.RemoveBatchAsync(batchId, cancellationToken);
        }
        return RemoveBatch(batchId);
    }

    /// <summary>Removes and persists all Temporary Items.</summary>
    public async Task<int> ClearTemporaryItemsAsync(CancellationToken cancellationToken = default)
    {
        using var mutation = await LockMutationsAsync(cancellationToken);
        var removedCount = _batches.Sum(batch => batch.Items.Count(item => !item.IsPinned));
        if (removedCount == 0)
        {
            return 0;
        }

        if (_database is not null)
        {
            await _database.ClearTemporaryItemsAsync(cancellationToken);
        }
        ClearTemporaryItems();
        return removedCount;
    }

    /// <summary>Classifies a batch and transactionally persists confirmed-Missing cleanup.</summary>
    public async Task<BatchDragPreparation> PrepareBatchForDragAsync(
        Guid batchId,
        Func<string, ItemAvailability> checkAvailability,
        CancellationToken cancellationToken = default)
    {
        using var mutation = await LockMutationsAsync(cancellationToken);
        ArgumentNullException.ThrowIfNull(checkAvailability);
        var batch = _batches.FirstOrDefault(candidate => candidate.Id == batchId);
        if (batch is null)
        {
            return new BatchDragPreparation([], [], 0);
        }

        var availability = batch.Items.ToDictionary(
            item => item.Path,
            item => checkAvailability(item.Path),
            StringComparer.OrdinalIgnoreCase);
        var missingIds = batch.Items
            .Where(item => availability[item.Path] == ItemAvailability.Missing)
            .Select(item => item.Id)
            .ToArray();
        if (_database is not null && missingIds.Length > 0)
        {
            await _database.RemoveItemsAsync(missingIds, cancellationToken);
        }
        var preparation = PrepareBatchForDrag(batchId, path => availability[path]);
        if (preparation.AvailableItems.Count == 0)
        {
            _preparedBatchItems.Remove(batchId);
        }
        else
        {
            _preparedBatchItems[batchId] = preparation.AvailableItems.Select(item => item.Id).ToHashSet();
        }

        return preparation;
    }

    /// <summary>Applies and persists a completed whole-batch drag-out result.</summary>
    public async Task<bool> CompleteBatchDragAsync(
        Guid batchId,
        DragOutOutcome outcome,
        CancellationToken cancellationToken = default)
    {
        using var mutation = await LockMutationsAsync(cancellationToken);
        if (!_preparedBatchItems.Remove(batchId, out var participatingItemIds) ||
            outcome != DragOutOutcome.AcceptedCopy)
        {
            return false;
        }

        var batch = _batches.FirstOrDefault(candidate => candidate.Id == batchId);
        if (batch is null)
        {
            return false;
        }

        var temporaryIds = batch.Items
            .Where(item => participatingItemIds.Contains(item.Id) && !item.IsPinned)
            .Select(item => item.Id)
            .ToHashSet();
        if (temporaryIds.Count == 0)
        {
            return false;
        }

        if (_database is not null)
        {
            await _database.RemoveItemsAsync(temporaryIds, cancellationToken);
        }
        return RemoveItems(temporaryIds);
    }

    /// <summary>Accepts one incoming drop and creates at most one Shelf Batch.</summary>
    /// <param name="items">The filesystem items offered by the drop in source order.</param>
    /// <param name="skippedUnsupportedCount">Unsupported entries already identified by the platform adapter.</param>
    /// <returns>The accepted, skipped, and deduplicated outcome.</returns>
    public DropAcceptance AcceptDrop(
        IEnumerable<IncomingShelfItem> items,
        int skippedUnsupportedCount = 0)
    {
        ArgumentNullException.ThrowIfNull(items);
        ArgumentOutOfRangeException.ThrowIfNegative(skippedUnsupportedCount);

        var accepted = new List<ShelfItem>();
        var seenPaths = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        var duplicates = 0;
        var skipped = skippedUnsupportedCount;

        foreach (var item in items)
        {
            if (!IsUsablePath(item.Path))
            {
                skipped++;
                continue;
            }

            if (!seenPaths.Add(item.Path))
            {
                duplicates++;
                continue;
            }

            accepted.Add(new ShelfItem(
                Guid.NewGuid(),
                item.Path,
                string.IsNullOrWhiteSpace(item.Name) ? GetFallbackName(item.Path) : item.Name,
                item.IsFolder));
        }

        if (accepted.Count == 0)
        {
            return new DropAcceptance(null, 0, skipped, duplicates);
        }

        var batch = new ShelfBatch(Guid.NewGuid(), DateTimeOffset.UtcNow, accepted);
        _batches.Insert(0, batch);
        return new DropAcceptance(batch, accepted.Count, skipped, duplicates);
    }

    /// <summary>Updates one Shelf Item's pinned lifecycle state.</summary>
    /// <param name="itemId">The Shelf Item identity.</param>
    /// <param name="isPinned">Whether accepted drag-outs should retain the reference.</param>
    /// <returns>Whether the item exists.</returns>
    public bool SetPinned(Guid itemId, bool isPinned)
    {
        if (!TryFindItem(itemId, out var batchIndex, out var itemIndex))
        {
            return false;
        }

        var batch = _batches[batchIndex];
        if (batch.Items[itemIndex].IsPinned == isPinned)
        {
            return true;
        }

        var items = batch.Items.ToArray();
        items[itemIndex] = items[itemIndex] with { IsPinned = isPinned };
        _batches[batchIndex] = batch with { Items = items };
        return true;
    }

    /// <summary>Removes one Shelf Item reference and removes its batch when empty.</summary>
    /// <param name="itemId">The Shelf Item identity.</param>
    /// <returns>Whether the item was removed.</returns>
    public bool RemoveItem(Guid itemId)
    {
        if (!TryFindItem(itemId, out var batchIndex, out var itemIndex))
        {
            return false;
        }

        var batch = _batches[batchIndex];
        if (batch.Items.Count == 1)
        {
            _batches.RemoveAt(batchIndex);
        }
        else
        {
            var items = batch.Items.Where((_, index) => index != itemIndex).ToArray();
            _batches[batchIndex] = batch with { Items = items };
        }

        ResetOnEmpty();
        return true;
    }

    private bool RemoveItems(IReadOnlySet<Guid> itemIds)
    {
        var changed = false;
        for (var index = _batches.Count - 1; index >= 0; index--)
        {
            var batch = _batches[index];
            if (!batch.Items.Any(item => itemIds.Contains(item.Id)))
            {
                continue;
            }

            changed = true;
            var retained = batch.Items.Where(item => !itemIds.Contains(item.Id)).ToArray();
            if (retained.Length == 0)
            {
                _batches.RemoveAt(index);
            }
            else
            {
                _batches[index] = batch with { Items = retained };
            }
        }

        ResetOnEmpty();
        return changed;
    }

    /// <summary>Applies a completed one-item drag-out result.</summary>
    /// <param name="itemId">The participating Shelf Item identity.</param>
    /// <param name="outcome">The native drag result.</param>
    /// <returns>Whether shelf state changed.</returns>
    public bool CompleteItemDrag(Guid itemId, DragOutOutcome outcome)
    {
        if (outcome != DragOutOutcome.AcceptedCopy ||
            !TryFindItem(itemId, out var batchIndex, out var itemIndex))
        {
            return false;
        }

        return !_batches[batchIndex].Items[itemIndex].IsPinned && RemoveItem(itemId);
    }

    /// <summary>Removes an entire Shelf Batch.</summary>
    /// <param name="batchId">The batch identity.</param>
    /// <returns>Whether the batch was removed.</returns>
    public bool RemoveBatch(Guid batchId)
    {
        var index = _batches.FindIndex(batch => batch.Id == batchId);
        if (index < 0)
        {
            return false;
        }

        _batches.RemoveAt(index);
        ResetOnEmpty();
        return true;
    }

    /// <summary>Removes all temporary items across all batches, pruning batches that become empty.</summary>
    /// <returns>The number of items removed.</returns>
    public int ClearTemporaryItems()
    {
        var removedCount = 0;
        for (var i = _batches.Count - 1; i >= 0; i--)
        {
            var batch = _batches[i];
            var pinned = batch.Items.Where(item => item.IsPinned).ToArray();
            removedCount += batch.Items.Count - pinned.Length;
            if (pinned.Length == 0)
            {
                _batches.RemoveAt(i);
            }
            else if (pinned.Length != batch.Items.Count)
            {
                _batches[i] = batch with { Items = pinned };
            }
        }

        ResetOnEmpty();
        return removedCount;
    }

    private void ResetOnEmpty()
    {
        if (_batches.Count == 0)
        {
            if (_displayState is ShelfDisplayState.Visible or ShelfDisplayState.EdgeDocked)
            {
                _displayState = ShelfDisplayState.Hidden;
            }
            _sizingState = ShelfSizingState.Default;
        }
    }
    /// <summary>Cleans up confirmed missing filesystem paths in a batch before drag-out begins, classifying availability.</summary>
    /// <param name="batchId">The batch identity.</param>
    /// <param name="checkAvailability">A func classifying path availability.</param>
    /// <returns>The drag preparation outcome.</returns>
    public BatchDragPreparation PrepareBatchForDrag(Guid batchId, Func<string, ItemAvailability> checkAvailability)
    {
        ArgumentNullException.ThrowIfNull(checkAvailability);
        var index = _batches.FindIndex(batch => batch.Id == batchId);
        if (index < 0)
        {
            return new BatchDragPreparation([], [], 0);
        }

        var batch = _batches[index];
        var retained = new List<ShelfItem>(batch.Items.Count);
        var available = new List<ShelfItem>(batch.Items.Count);
        var unavailable = new List<ShelfItem>();
        var missingCount = 0;

        foreach (var item in batch.Items)
        {
            var status = checkAvailability(item.Path);
            var classified = item with { Availability = status };
            switch (status)
            {
                case ItemAvailability.Missing:
                    missingCount++;
                    break;
                case ItemAvailability.Unavailable:
                    retained.Add(classified);
                    unavailable.Add(classified);
                    break;
                default:
                    retained.Add(classified);
                    available.Add(classified);
                    break;
            }
        }

        if (missingCount > 0)
        {
            if (retained.Count == 0)
            {
                _batches.RemoveAt(index);
            }
            else
            {
                _batches[index] = batch with { Items = retained };
            }
        }

        ResetOnEmpty();
        return new BatchDragPreparation(available, unavailable, missingCount);
    }

    /// <summary>Applies a completed whole-batch drag-out result.</summary>
    /// <param name="batchId">The participating Shelf Batch identity.</param>
    /// <param name="outcome">The native drag result.</param>
    /// <returns>Whether shelf state changed.</returns>
    public bool CompleteBatchDrag(Guid batchId, DragOutOutcome outcome)
    {
        if (outcome != DragOutOutcome.AcceptedCopy)
        {
            return false;
        }

        var index = _batches.FindIndex(batch => batch.Id == batchId);
        if (index < 0)
        {
            return false;
        }

        var batch = _batches[index];
        var pinned = batch.Items.Where(item => item.IsPinned).ToArray();
        if (pinned.Length == 0)
        {
            _batches.RemoveAt(index);
            ResetOnEmpty();
            return true;
        }

        if (pinned.Length != batch.Items.Count)
        {
            _batches[index] = batch with { Items = pinned };
            ResetOnEmpty();
            return true;
        }

        return false;
    }

    private bool TryFindItem(Guid itemId, out int batchIndex, out int itemIndex)
    {
        for (batchIndex = 0; batchIndex < _batches.Count; batchIndex++)
        {
            var items = _batches[batchIndex].Items;
            for (itemIndex = 0; itemIndex < items.Count; itemIndex++)
            {
                if (items[itemIndex].Id == itemId)
                {
                    return true;
                }
            }
        }

        batchIndex = -1;

        itemIndex = -1;
        return false;
    }

    private async ValueTask<MutationLease> LockMutationsAsync(CancellationToken cancellationToken)
    {
        await _mutationGate.WaitAsync(cancellationToken).ConfigureAwait(false);
        return new MutationLease(_mutationGate);
    }
    private static ItemAvailability ClassifyRestoredPath(
        string path,
        Func<string, ItemAvailability> classifyAvailability)
    {
        try
        {
            return classifyAvailability(path);
        }
        catch
        {
            return ItemAvailability.Unavailable;
        }
    }

    private static bool IsUsablePath(string path) =>
        !string.IsNullOrWhiteSpace(path) && Path.IsPathFullyQualified(path);

    private static string GetFallbackName(string path)
    {
        var trimmed = Path.TrimEndingDirectorySeparator(path);
        var name = Path.GetFileName(trimmed);
        return string.IsNullOrEmpty(name) ? path : name;
    }
    private readonly struct MutationLease(SemaphoreSlim gate) : IDisposable
    {
        public void Dispose() => gate.Release();
    }

}
