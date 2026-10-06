using DropCove.Core;

namespace DropCove.Tests;

[TestClass]
public sealed class DropShelfPersistenceTests
{
    private static readonly System.Collections.Concurrent.ConcurrentBag<string> TemporaryDirectories = [];

    [ClassCleanup]
    public static void CleanupTemporaryDatabases()
    {
        foreach (var directory in TemporaryDirectories)
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [TestMethod]
    public async Task WidthOnlyResize_RestoresWidthWithoutManualHeight()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        await manager.AcceptDropAsync([new(@"C:\Work\Width.txt", "Width.txt", false)]);

        await manager.SetSizingStateAsync(new ShelfSizingState(350, null));

        var restored = await OpenAsync(databasePath);
        Assert.AreEqual(350, restored.SizingState.PreferredWidth);
        Assert.IsNull(restored.SizingState.ManualHeightOverride);
    }

    [TestMethod]
    public async Task PendingWidth_FirstAcceptancePersistsNoManualHeight()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        await manager.SetSizingStateAsync(new ShelfSizingState(350, null));
        var before = await OpenAsync(databasePath);
        Assert.AreEqual(ShelfSizingState.Default, before.SizingState);

        await manager.AcceptDropAsync([new(@"C:\Work\First.txt", "First.txt", false)]);

        var restored = await OpenAsync(databasePath);
        Assert.AreEqual(new ShelfSizingState(350, null), restored.SizingState);
    }

    [TestMethod]
    [DataRow(true)]
    [DataRow(false)]
    public async Task LegacySizing_CutoverPreservesOnlyRetainedContent(bool retainContent)
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        if (retainContent)
        {
            await manager.AcceptDropAsync([new(@"C:\Work\Legacy.txt", "Legacy.txt", false)]);
        }
        using (var connection = new Microsoft.Data.Sqlite.SqliteConnection($"Data Source={databasePath};Pooling=False"))
        {
            connection.Open();
            using var command = connection.CreateCommand();
            command.CommandText = "DROP TABLE shelf_settings; CREATE TABLE shelf_settings (id INTEGER PRIMARY KEY, preferred_width INTEGER NOT NULL, preferred_height INTEGER NOT NULL); INSERT INTO shelf_settings VALUES (1, 350, 180);";
            command.ExecuteNonQuery();
        }

        var restored = await OpenAsync(databasePath);
        var expected = retainContent ? new ShelfSizingState(350, 180) : ShelfSizingState.Default;
        Assert.AreEqual(expected, restored.SizingState);
        var reopened = await OpenAsync(databasePath);
        Assert.AreEqual(expected, reopened.SizingState);
        Assert.IsNull(reopened.RecoveryBackupPath);
    }

    [TestMethod]
    public async Task Restart_RestoresBatchOrderAndShelfItemMetadata()
    {
        var databasePath = CreateDatabasePath();
        var manager = await DropShelfManager.OpenAsync(databasePath, _ => ItemAvailability.Available);
        var older = await manager.AcceptDropAsync(
        [
            new(@"C:\Work\First.txt", "First.txt", false),
            new(@"C:\Work\Folder", "Folder", true),
        ]);
        var newer = await manager.AcceptDropAsync([new(@"C:\Work\Latest.txt", "Latest.txt", false)]);
        await manager.SetPinnedAsync(older.Batch!.Items[1].Id, true);

        var restored = await DropShelfManager.OpenAsync(databasePath, _ => ItemAvailability.Available);

        Assert.HasCount(2, restored.Batches);
        Assert.AreEqual(newer.Batch!.Id, restored.Batches[0].Id);
        Assert.AreEqual(older.Batch.Id, restored.Batches[1].Id);
        Assert.AreEqual(older.Batch.CreatedAt, restored.Batches[1].CreatedAt);
        CollectionAssert.AreEqual(
            older.Batch.Items.Select(item => item.Id).ToArray(),
            restored.Batches[1].Items.Select(item => item.Id).ToArray());
        Assert.IsTrue(restored.Batches[1].Items[1].IsFolder);
        Assert.IsTrue(restored.Batches[1].Items[1].IsPinned);
    }

    [TestMethod]
    public async Task CompletedMutations_AreCommittedImmediately()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        var managed = (await manager.AcceptDropAsync(
        [
            new(@"C:\Work\Remove.txt", "Remove.txt", false),
            new(@"C:\Work\Keep.txt", "Keep.txt", false),
        ])).Batch!;
        var temporaryBatch = (await manager.AcceptDropAsync(
            [new(@"C:\Work\Temporary.txt", "Temporary.txt", false)])).Batch!;

        await manager.SetPinnedAsync(managed.Items[1].Id, true);
        manager = await OpenAsync(databasePath);
        Assert.IsTrue(FindItem(manager, managed.Items[1].Id).IsPinned);

        await manager.SetPinnedAsync(managed.Items[1].Id, false);
        await manager.RemoveItemAsync(managed.Items[0].Id);
        manager = await OpenAsync(databasePath);
        Assert.IsFalse(FindItem(manager, managed.Items[1].Id).IsPinned);
        Assert.IsFalse(ContainsItem(manager, managed.Items[0].Id));

        await manager.SetPinnedAsync(managed.Items[1].Id, true);
        Assert.AreEqual(1, await manager.ClearTemporaryItemsAsync());
        manager = await OpenAsync(databasePath);
        Assert.IsTrue(ContainsItem(manager, managed.Items[1].Id));
        Assert.IsFalse(manager.Batches.Any(batch => batch.Id == temporaryBatch.Id));

        Assert.IsTrue(await manager.RemoveBatchAsync(managed.Id));
        manager = await OpenAsync(databasePath);
        Assert.IsEmpty(manager.Batches);
    }

    [TestMethod]
    public async Task MissingCleanupAndSuccessfulDragOut_AreDurable()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        var itemBatch = (await manager.AcceptDropAsync(
        [
            new(@"C:\Work\Missing.txt", "Missing.txt", false),
            new(@"C:\Work\Drag.txt", "Drag.txt", false),
        ])).Batch!;

        var preparation = await manager.PrepareBatchForDragAsync(
            itemBatch.Id,
            path => path.EndsWith("Missing.txt", StringComparison.Ordinal)
                ? ItemAvailability.Missing
                : ItemAvailability.Available);
        Assert.AreEqual(1, preparation.MissingItemsRemovedCount);
        Assert.IsTrue(await manager.CompleteItemDragAsync(itemBatch.Items[1].Id, DragOutOutcome.AcceptedCopy));

        var batchDrag = (await manager.AcceptDropAsync(
        [
            new(@"C:\Work\BatchTemp.txt", "BatchTemp.txt", false),
            new(@"C:\Work\BatchPinned.txt", "BatchPinned.txt", false),
        ])).Batch!;
        await manager.SetPinnedAsync(batchDrag.Items[1].Id, true);
        await manager.PrepareBatchForDragAsync(batchDrag.Id, _ => ItemAvailability.Available);
        Assert.IsTrue(await manager.CompleteBatchDragAsync(batchDrag.Id, DragOutOutcome.AcceptedCopy));

        manager = await OpenAsync(databasePath);
        Assert.IsFalse(ContainsItem(manager, itemBatch.Items[0].Id));
        Assert.IsFalse(ContainsItem(manager, itemBatch.Items[1].Id));
        Assert.IsFalse(ContainsItem(manager, batchDrag.Items[0].Id));
        Assert.IsTrue(FindItem(manager, batchDrag.Items[1].Id).IsPinned);
    }

    [TestMethod]
    public async Task CrashBeforeSuccessfulDragCommit_RestoresTemporaryReference()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        var item = (await manager.AcceptDropAsync(
            [new(@"C:\Work\Pending.txt", "Pending.txt", false)])).Batch!.Items[0];

        var restored = await OpenAsync(databasePath);
        Assert.AreEqual(item.Id, restored.Batches[0].Items[0].Id);
        Assert.IsFalse(restored.Batches[0].Items[0].IsPinned);


    }
    [TestMethod]
    public async Task Restart_ReclassifiesRestoredPaths()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        await manager.AcceptDropAsync(
        [
            new(@"C:\Work\Available.txt", "Available.txt", false),
            new(@"C:\Work\Missing.txt", "Missing.txt", false),
            new(@"Z:\Offline\Unavailable.txt", "Unavailable.txt", false),
        ]);

        var restored = await DropShelfManager.OpenAsync(databasePath, path => path switch
        {
            var value when value.EndsWith("Missing.txt", StringComparison.Ordinal) => ItemAvailability.Missing,
            var value when value.EndsWith("Unavailable.txt", StringComparison.Ordinal) => ItemAvailability.Unavailable,
            _ => ItemAvailability.Available,
        });

        CollectionAssert.AreEqual(
            new[] { ItemAvailability.Available, ItemAvailability.Missing, ItemAvailability.Unavailable },
            restored.Batches[0].Items.Select(item => item.Availability).ToArray());
    }

    [TestMethod]
    public async Task CorruptDatabase_IsPreservedAndReplacedWithAnEmptyDatabase()
    {
        var databasePath = CreateDatabasePath();
        var corruptBytes = "not a sqlite database"u8.ToArray();
        await File.WriteAllBytesAsync(databasePath, corruptBytes);

        var manager = await OpenAsync(databasePath);

        Assert.IsEmpty(manager.Batches);
        Assert.IsNotNull(manager.RecoveryBackupPath);
        Assert.IsTrue(File.Exists(manager.RecoveryBackupPath));
        CollectionAssert.AreEqual(corruptBytes, await File.ReadAllBytesAsync(manager.RecoveryBackupPath));
        Assert.IsTrue(File.Exists(databasePath));

        var reopened = await OpenAsync(databasePath);
        Assert.IsEmpty(reopened.Batches);
        Assert.IsNull(reopened.RecoveryBackupPath);
    }

    [TestMethod]
    [DataRow(DragOutOutcome.Canceled)]
    [DataRow(DragOutOutcome.Rejected)]
    [DataRow(DragOutOutcome.Failed)]
    public async Task UnsuccessfulPreparedBatchDrag_RetainsReferencesAfterRestart(DragOutOutcome outcome)
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        var batch = (await manager.AcceptDropAsync(
            [new(@"C:\Work\Retained.txt", "Retained.txt", false)])).Batch!;
        await manager.PrepareBatchForDragAsync(batch.Id, _ => ItemAvailability.Available);

        Assert.IsFalse(await manager.CompleteBatchDragAsync(batch.Id, outcome));
        var restored = await OpenAsync(databasePath);

        Assert.AreEqual(batch.Items[0].Id, restored.Batches[0].Items[0].Id);
    }

    [TestMethod]
    public async Task AcceptedPartialBatchDrag_RetainsUnavailableReferencesAfterRestart()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        var batch = (await manager.AcceptDropAsync(
        [
            new(@"C:\Work\Available.txt", "Available.txt", false),
            new(@"Z:\Offline\Unavailable.txt", "Unavailable.txt", false),
        ])).Batch!;
        await manager.PrepareBatchForDragAsync(
            batch.Id,
            path => path.StartsWith(@"Z:\", StringComparison.Ordinal)
                ? ItemAvailability.Unavailable
                : ItemAvailability.Available);

        Assert.IsTrue(await manager.CompleteBatchDragAsync(batch.Id, DragOutOutcome.AcceptedCopy));
        var restored = await OpenAsync(databasePath);

        Assert.HasCount(1, restored.Batches);
        Assert.HasCount(1, restored.Batches[0].Items);
        Assert.AreEqual(batch.Items[1].Id, restored.Batches[0].Items[0].Id);
    }

    [TestMethod]
    public async Task Restart_RestoresRailPlacement()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        await manager.AcceptDropAsync([new(@"C:\Work\Docked.txt", "Docked.txt", false)]);
        var placement = new ShelfRailPlacement("DISPLAY2", ShelfRailEdge.Left);
        manager.ShowShelf();
        Assert.AreEqual(ShelfDisplayState.EdgeDocked, await manager.DismissShelfAsync(placement));
        var restored = await OpenAsync(databasePath);

        Assert.AreEqual(placement, restored.RailPlacement);
        Assert.AreEqual(ShelfDisplayState.Hidden, restored.DisplayState);
    }

    [TestMethod]
    public async Task Restart_RestoresUpdatedRailPlacement()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        await manager.AcceptDropAsync([new(@"C:\Work\Docked.txt", "Docked.txt", false)]);
        manager.ShowShelf();
        await manager.DismissShelfAsync(new ShelfRailPlacement("DISPLAY1", ShelfRailEdge.Right));

        var updated = new ShelfRailPlacement("DISPLAY2", ShelfRailEdge.Left);
        await manager.SetRailPlacementAsync(updated);

        var restored = await OpenAsync(databasePath);
        Assert.AreEqual(updated, restored.RailPlacement);
    }

    private static Task<DropShelfManager> OpenAsync(string databasePath) =>
        DropShelfManager.OpenAsync(databasePath, _ => ItemAvailability.Available);

    private static bool ContainsItem(DropShelfManager manager, Guid itemId) =>
        manager.Batches.SelectMany(batch => batch.Items).Any(item => item.Id == itemId);

    private static ShelfItem FindItem(DropShelfManager manager, Guid itemId) =>
        manager.Batches.SelectMany(batch => batch.Items).Single(item => item.Id == itemId);

    private static string CreateDatabasePath()
    {
        var directory = Path.Combine(Path.GetTempPath(), "DropCove.Tests", Guid.NewGuid().ToString("N"));
        TemporaryDirectories.Add(directory);
        Directory.CreateDirectory(directory);
        return Path.Combine(directory, "shelf.db");
    }

    [TestMethod]
    public async Task ShakeSummonDrop_PersistsBatchAndRetainsItemSemantics()
    {
        var databasePath = CreateDatabasePath();
        var manager = await DropShelfManager.OpenAsync(databasePath, _ => ItemAvailability.Available);

        // Simulate initial state: shelf is Hidden
        Assert.AreEqual(ShelfDisplayState.Hidden, manager.DisplayState);

        // Simulate shake summon: shelf transitions to the Visible presentation
        manager.ShowShelf();
        Assert.AreEqual(ShelfDisplayState.Visible, manager.DisplayState);

        // Accept drop after shake: contains a regular file and a folder
        var dropResult = await manager.AcceptDropAsync([
            new(@"C:\Files\Document.pdf", "Document.pdf", false),
            new(@"C:\Files\ProjectFolder", "ProjectFolder", true),
        ]);

        Assert.IsNotNull(dropResult.Batch);
        Assert.AreEqual(2, dropResult.AcceptedCount);
        Assert.HasCount(1, manager.Batches);

        // By default items are Temporary (IsPinned == false)
        var item1 = dropResult.Batch.Items[0];
        var item2 = dropResult.Batch.Items[1];
        Assert.IsFalse(item1.IsPinned);
        Assert.IsFalse(item2.IsPinned);
        Assert.IsTrue(item2.IsFolder);

        // User pins item 1
        await manager.SetPinnedAsync(item1.Id, true);
        Assert.IsTrue(FindItem(manager, item1.Id).IsPinned);

        // Re-open from database to verify persistence across restarts
        var restored = await DropShelfManager.OpenAsync(databasePath, _ => ItemAvailability.Available);
        Assert.HasCount(1, restored.Batches);
        Assert.AreEqual(dropResult.Batch.Id, restored.Batches[0].Id);
        Assert.HasCount(2, restored.Batches[0].Items);
        var restoredItem1 = restored.Batches[0].Items[0];
        var restoredItem2 = restored.Batches[0].Items[1];
        Assert.IsTrue(restoredItem1.IsPinned);
        Assert.IsFalse(restoredItem2.IsPinned);
        Assert.IsTrue(restoredItem2.IsFolder);
    }

    [TestMethod]
    public async Task SizingState_PersistedWhileContentExists_RestoresAcrossRestart()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        await manager.AcceptDropAsync([new(@"C:\Work\Item.txt", "Item.txt", false)]);

        var customSize = new ShelfSizingState(350, 420);
        await manager.SetSizingStateAsync(customSize);
        Assert.AreEqual(customSize, manager.SizingState);

        var restored = await OpenAsync(databasePath);
        Assert.AreEqual(customSize, restored.SizingState);
    }

    [TestMethod]
    public async Task SizingState_ResizingEmptyShelf_PersistsWhenFirstItemAccepted()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);

        var customSize = new ShelfSizingState(240, 320);
        await manager.SetSizingStateAsync(customSize);
        Assert.AreEqual(customSize, manager.SizingState);

        // Empty shelf restart does not persist preferred size
        var restoredBeforeItem = await OpenAsync(databasePath);
        Assert.AreEqual(ShelfSizingState.Default, restoredBeforeItem.SizingState);

        // Now accept first item on the manager that had preferred size set
        await manager.AcceptDropAsync([new(@"C:\Work\First.txt", "First.txt", false)]);

        // After accepting first item, customSize should be durable across restart
        var restoredAfterItem = await OpenAsync(databasePath);
        Assert.AreEqual(customSize, restoredAfterItem.SizingState);
    }

    [TestMethod]
    public async Task FinalItemRemoved_DeletesSizingStateAndTransitionsToHidden()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        var drop = await manager.AcceptDropAsync([new(@"C:\Work\Single.txt", "Single.txt", false)]);
        var itemId = drop.Batch!.Items[0].Id;

        manager.ShowShelf();
        var customSize = new ShelfSizingState(300, 300);
        await manager.SetSizingStateAsync(customSize);

        // Remove the final item
        Assert.IsTrue(await manager.RemoveItemAsync(itemId));
        Assert.AreEqual(ShelfDisplayState.Hidden, manager.DisplayState);
        Assert.AreEqual(ShelfSizingState.Default, manager.SizingState);

        // Restart confirms deletion in database
        var restored = await OpenAsync(databasePath);
        Assert.AreEqual(ShelfSizingState.Default, restored.SizingState);
    }

    [TestMethod]
    public async Task FinalItemRemovedFromEdgeDocked_DeletesSizingState_PreservesRailPlacement()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        var drop = await manager.AcceptDropAsync([new(@"C:\Work\Single.txt", "Single.txt", false)]);
        var itemId = drop.Batch!.Items[0].Id;

        var placement = new ShelfRailPlacement("DISPLAY2", ShelfRailEdge.Left);
        manager.ShowShelf();
        await manager.SetSizingStateAsync(new ShelfSizingState(250, 250));
        Assert.AreEqual(ShelfDisplayState.EdgeDocked, await manager.DismissShelfAsync(placement));

        // Remove final item from EdgeDocked
        Assert.IsTrue(await manager.RemoveItemAsync(itemId));
        Assert.AreEqual(ShelfDisplayState.Hidden, manager.DisplayState);
        Assert.AreEqual(ShelfSizingState.Default, manager.SizingState);

        // Restart confirms rail placement preserved, preferred size reset
        var restored = await OpenAsync(databasePath);
        Assert.AreEqual(placement, restored.RailPlacement);
        Assert.AreEqual(ShelfSizingState.Default, restored.SizingState);
    }

    [TestMethod]
    public async Task ClearTemporaryItems_RemovingFinalItem_DeletesSizingState()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        await manager.AcceptDropAsync([new(@"C:\Work\Temp.txt", "Temp.txt", false)]);
        manager.ShowShelf();
        await manager.SetSizingStateAsync(new ShelfSizingState(220, 220));

        var removed = await manager.ClearTemporaryItemsAsync();
        Assert.AreEqual(1, removed);
        Assert.AreEqual(ShelfDisplayState.Hidden, manager.DisplayState);
        Assert.AreEqual(ShelfSizingState.Default, manager.SizingState);

        var restored = await OpenAsync(databasePath);
        Assert.AreEqual(ShelfSizingState.Default, restored.SizingState);
    }

    [TestMethod]
    public async Task StaleSizingStateWithoutItems_CleansUpEmptyBatchesOnStartup()
    {
        var databasePath = CreateDatabasePath();
        // Seed database with empty batches but a setting record directly
        var manager = await OpenAsync(databasePath);
        await manager.AcceptDropAsync([new(@"C:\Work\Temp.txt", "Temp.txt", false)]);
        await manager.SetSizingStateAsync(new ShelfSizingState(300, 300));

        // Manually delete items from sqlite to simulate stale state where items are gone
        using (var conn = new Microsoft.Data.Sqlite.SqliteConnection($"Data Source={databasePath};Pooling=False"))
        {
            conn.Open();
            using var cmd = conn.CreateCommand();
            cmd.CommandText = "DELETE FROM shelf_items;";
            cmd.ExecuteNonQuery();
        }

        var restored = await OpenAsync(databasePath);
        Assert.AreEqual(ShelfSizingState.Default, restored.SizingState);
        Assert.IsEmpty(restored.Batches);

        // Verify settings row was deleted from sqlite
        using (var conn = new Microsoft.Data.Sqlite.SqliteConnection($"Data Source={databasePath};Pooling=False"))
        {
            conn.Open();
            using var cmd = conn.CreateCommand();
            cmd.CommandText = "SELECT COUNT(*) FROM shelf_settings WHERE id = 1";
            Assert.AreEqual(0L, (long)cmd.ExecuteScalar()!);
        }
    }
    [TestMethod]
    public async Task FinalRemoval_WhenSizingStateDeletionFails_RetainsPersistedItemAndSize()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        var batch = (await manager.AcceptDropAsync([new(@"C:\Work\Keep.txt", "Keep.txt", false)])).Batch!;
        await manager.SetSizingStateAsync(new ShelfSizingState(350, 420));
        using (var connection = new Microsoft.Data.Sqlite.SqliteConnection($"Data Source={databasePath};Pooling=False"))
        {
            connection.Open();
            using var command = connection.CreateCommand();
            command.CommandText = "CREATE TRIGGER reject_size_delete BEFORE DELETE ON shelf_settings BEGIN SELECT RAISE(ABORT, 'size deletion failed'); END";
            command.ExecuteNonQuery();
        }

        await Assert.ThrowsAsync<Microsoft.Data.Sqlite.SqliteException>(() => manager.RemoveItemAsync(batch.Items[0].Id));

        var restored = await OpenAsync(databasePath);
        Assert.AreEqual(batch.Items[0].Id, restored.Batches.Single().Items.Single().Id);
        Assert.AreEqual(new ShelfSizingState(350, 420), restored.SizingState);
    }
    [TestMethod]
    public async Task FirstAcceptance_WhenSizingStateSaveFails_DoesNotPersistRejectedBatch()
    {
        var databasePath = CreateDatabasePath();
        var manager = await OpenAsync(databasePath);
        await manager.SetSizingStateAsync(new ShelfSizingState(350, 420));
        using (var connection = new Microsoft.Data.Sqlite.SqliteConnection($"Data Source={databasePath};Pooling=False"))
        {
            connection.Open();
            using var command = connection.CreateCommand();
            command.CommandText = "CREATE TRIGGER reject_size_insert BEFORE INSERT ON shelf_settings BEGIN SELECT RAISE(ABORT, 'size save failed'); END";
            command.ExecuteNonQuery();
        }

        await Assert.ThrowsAsync<Microsoft.Data.Sqlite.SqliteException>(() => manager.AcceptDropAsync([new(@"C:\Work\Rejected.txt", "Rejected.txt", false)]));

        var restored = await OpenAsync(databasePath);
        Assert.IsEmpty(restored.Batches);
        Assert.IsEmpty(manager.Batches);
        Assert.AreEqual(new ShelfSizingState(350, 420), manager.SizingState);
    }
}
