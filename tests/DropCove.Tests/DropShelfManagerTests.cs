using DropCove.Core;

namespace DropCove.Tests;

[TestClass]
public sealed class DropShelfManagerTests
{
    [TestMethod]
    public void AcceptDrop_PreservesOrderAndDeduplicatesOnlyWithinTheDrop()
    {
        var manager = new DropShelfManager();
        var first = manager.AcceptDrop(
        [
            new(@"C:\Work\A.png", "A.png", false),
            new(@"C:\Work\B.png", "B.png", false),
            new(@"c:\work\A.png", "A duplicate.png", false),
        ]);
        var second = manager.AcceptDrop([new(@"C:\Work\A.png", "A.png", false)]);

        Assert.AreEqual(2, first.AcceptedCount);
        Assert.AreEqual(1, first.DuplicateCount);
        CollectionAssert.AreEqual(
            new[] { @"C:\Work\A.png", @"C:\Work\B.png" },
            first.Batch!.Items.Select(item => item.Path).ToArray());
        Assert.AreSame(second.Batch, manager.Batches[0]);
        Assert.AreSame(first.Batch, manager.Batches[1]);
    }

    [TestMethod]
    public void AcceptDrop_MixedPayloadCountsUnsupportedItemsAndCreatesOneBatch()
    {
        var manager = new DropShelfManager();

        var result = manager.AcceptDrop(
        [
            new(@"C:\Work\Report.docx", "Report.docx", false),
            new(string.Empty, "Stream-only item", false),
            new(@"\\server\share\Folder", "Folder", true),
        ], skippedUnsupportedCount: 1);

        Assert.AreEqual(2, result.AcceptedCount);
        Assert.AreEqual(2, result.SkippedUnsupportedCount);
        Assert.HasCount(1, manager.Batches);
    }

    [TestMethod]
    public void AcceptDrop_UnsupportedOnlyPayloadCreatesNoBatch()
    {
        var manager = new DropShelfManager();

        var result = manager.AcceptDrop([new(string.Empty, "Virtual item", false)]);

        Assert.IsNull(result.Batch);
        Assert.AreEqual(1, result.SkippedUnsupportedCount);
        Assert.IsEmpty(manager.Batches);
    }
    [TestMethod]
    public void SetPinned_UpdatesTheVisibleLifecycleState()
    {
        var manager = CreateManagerWithOneItem(out var item);

        Assert.IsTrue(manager.SetPinned(item.Id, true));
        Assert.IsTrue(manager.Batches[0].Items[0].IsPinned);
        Assert.IsTrue(manager.SetPinned(item.Id, false));
        Assert.IsFalse(manager.Batches[0].Items[0].IsPinned);
    }

    [TestMethod]
    public void CompleteItemDrag_AcceptedCopyConsumesTemporaryButRetainsPinned()
    {
        var temporaryManager = CreateManagerWithOneItem(out var temporaryItem);
        var pinnedManager = CreateManagerWithOneItem(out var pinnedItem);
        pinnedManager.SetPinned(pinnedItem.Id, true);

        Assert.IsTrue(temporaryManager.CompleteItemDrag(temporaryItem.Id, DragOutOutcome.AcceptedCopy));
        Assert.IsEmpty(temporaryManager.Batches);
        Assert.IsFalse(pinnedManager.CompleteItemDrag(pinnedItem.Id, DragOutOutcome.AcceptedCopy));
        Assert.IsTrue(pinnedManager.Batches[0].Items[0].IsPinned);
    }

    [TestMethod]
    [DataRow(DragOutOutcome.Canceled)]
    [DataRow(DragOutOutcome.Rejected)]
    [DataRow(DragOutOutcome.Failed)]
    public void CompleteItemDrag_UnsuccessfulOutcomeRetainsReference(DragOutOutcome outcome)
    {
        var manager = CreateManagerWithOneItem(out var item);

        Assert.IsFalse(manager.CompleteItemDrag(item.Id, outcome));
        Assert.AreEqual(item.Id, manager.Batches[0].Items[0].Id);
    }

    [TestMethod]
    public void RemoveItem_RemovesTheEmptyBatch()
    {
        var manager = CreateManagerWithOneItem(out var item);

        Assert.IsTrue(manager.RemoveItem(item.Id));
        Assert.IsEmpty(manager.Batches);
    }

    [TestMethod]
    public void CompleteBatchDrag_ConsumesTemporaryAndRetainsPinned()
    {
        var manager = new DropShelfManager();
        var acceptance = manager.AcceptDrop(
        [
            new(@"C:\Work\Temp.txt", "Temp.txt", false),
            new(@"C:\Work\Pinned.txt", "Pinned.txt", false),
        ]);
        var batch = acceptance.Batch!;
        manager.SetPinned(batch.Items[1].Id, true);

        Assert.IsTrue(manager.CompleteBatchDrag(batch.Id, DragOutOutcome.AcceptedCopy));
        Assert.HasCount(1, manager.Batches);
        Assert.HasCount(1, manager.Batches[0].Items);
        Assert.AreEqual(@"C:\Work\Pinned.txt", manager.Batches[0].Items[0].Path);
    }

    [TestMethod]
    [DataRow(DragOutOutcome.Canceled)]
    [DataRow(DragOutOutcome.Rejected)]
    [DataRow(DragOutOutcome.Failed)]
    public void CompleteBatchDrag_UnsuccessfulOutcomeRetainsAllReferences(DragOutOutcome outcome)
    {
        var manager = new DropShelfManager();
        var acceptance = manager.AcceptDrop(
        [
            new(@"C:\Work\One.txt", "One.txt", false),
            new(@"C:\Work\Two.txt", "Two.txt", false),
        ]);
        var batch = acceptance.Batch!;

        Assert.IsFalse(manager.CompleteBatchDrag(batch.Id, outcome));
        Assert.HasCount(1, manager.Batches);
        Assert.HasCount(2, manager.Batches[0].Items);
    }

    [TestMethod]
    public void RemoveBatch_RemovesBatchCompletely()
    {
        var manager = new DropShelfManager();
        var acceptance = manager.AcceptDrop([new(@"C:\Work\One.txt", "One.txt", false)]);
        var batch = acceptance.Batch!;

        Assert.IsTrue(manager.RemoveBatch(batch.Id));
        Assert.IsEmpty(manager.Batches);
    }

    [TestMethod]
    public void ClearTemporaryItems_RemovesOnlyTemporaryItems()
    {
        var manager = new DropShelfManager();
        var b1 = manager.AcceptDrop([new(@"C:\Work\TempOnly.txt", "TempOnly.txt", false)]).Batch!;
        var b2 = manager.AcceptDrop(
        [
            new(@"C:\Work\Temp2.txt", "Temp2.txt", false),
            new(@"C:\Work\Pinned2.txt", "Pinned2.txt", false),
        ]).Batch!;
        manager.SetPinned(b2.Items[1].Id, true);

        var removed = manager.ClearTemporaryItems();
        Assert.AreEqual(2, removed);
        Assert.HasCount(1, manager.Batches);
        Assert.HasCount(1, manager.Batches[0].Items);
        Assert.AreEqual(@"C:\Work\Pinned2.txt", manager.Batches[0].Items[0].Path);
    }

    [TestMethod]
    public void PrepareBatchForDrag_RemovesMissingItemsPrunesEmptyBatchAndRetainsUnavailable()
    {
        var manager = new DropShelfManager();
        manager.AcceptDrop([
            new(@"C:\Work\Present.txt", "Present.txt", false),
            new(@"C:\Work\Missing.txt", "Missing.txt", false),
            new(@"Z:\Network\Unavailable.txt", "Unavailable.txt", false),
        ]);

        var prep = manager.PrepareBatchForDrag(
            manager.Batches[0].Id,
            path => path.Contains("Missing")
                ? ItemAvailability.Missing
                : path.Contains("Unavailable")
                    ? ItemAvailability.Unavailable
                    : ItemAvailability.Available);

        Assert.AreEqual(1, prep.MissingItemsRemovedCount);
        Assert.HasCount(1, prep.AvailableItems);
        Assert.AreEqual(@"C:\Work\Present.txt", prep.AvailableItems[0].Path);
        Assert.HasCount(1, prep.UnavailableItems);
        Assert.AreEqual(@"Z:\Network\Unavailable.txt", prep.UnavailableItems[0].Path);
        Assert.HasCount(2, manager.Batches[0].Items);
    }

    [TestMethod]
    public void PrepareBatchForDrag_RemovesEmptyBatchWhenAllMissing()
    {
        var manager = new DropShelfManager();
        manager.AcceptDrop([new(@"C:\Work\Missing.txt", "Missing.txt", false)]);

        var prep = manager.PrepareBatchForDrag(manager.Batches[0].Id, _ => ItemAvailability.Missing);

        Assert.AreEqual(1, prep.MissingItemsRemovedCount);
        Assert.IsEmpty(prep.AvailableItems);
        Assert.IsEmpty(manager.Batches);
    }

    [TestMethod]
    public void PrepareThenAcceptedBatchDrag_ConsumesRemainingTemporaryItems()
    {
        var manager = new DropShelfManager();
        var batch = manager.AcceptDrop([
            new(@"C:\Work\Present.txt", "Present.txt", false),
            new(@"C:\Work\Missing.txt", "Missing.txt", false),
        ]).Batch!;

        manager.PrepareBatchForDrag(
            batch.Id,
            path => path.Contains("Missing") ? ItemAvailability.Missing : ItemAvailability.Available);
        manager.CompleteBatchDrag(batch.Id, DragOutOutcome.AcceptedCopy);

        Assert.IsEmpty(manager.Batches);
    }

    [TestMethod]
    public void PrepareThenAcceptedSingleItemBatchDrag_ConsumesTemporaryReference()
    {
        var manager = new DropShelfManager();
        var batch = manager.AcceptDrop([
            new(@"C:\Work\Only.txt", "Only.txt", false),
        ]).Batch!;

        var preparation = manager.PrepareBatchForDrag(
            batch.Id,
            _ => ItemAvailability.Available);

        Assert.HasCount(1, preparation.AvailableItems);
        Assert.IsTrue(manager.CompleteBatchDrag(batch.Id, DragOutOutcome.AcceptedCopy));
        Assert.IsEmpty(manager.Batches);
    }

    [TestMethod]
    public void PrepareThenCanceledBatchDrag_RetainsAvailableItemsWithoutRestoringMissing()
    {
        var manager = new DropShelfManager();
        var batch = manager.AcceptDrop([
            new(@"C:\Work\Present.txt", "Present.txt", false),
            new(@"C:\Work\Missing.txt", "Missing.txt", false),
        ]).Batch!;

        manager.PrepareBatchForDrag(
            batch.Id,
            path => path.Contains("Missing") ? ItemAvailability.Missing : ItemAvailability.Available);
        manager.CompleteBatchDrag(batch.Id, DragOutOutcome.Canceled);

        Assert.HasCount(1, manager.Batches);
        Assert.HasCount(1, manager.Batches[0].Items);
        Assert.AreEqual(@"C:\Work\Present.txt", manager.Batches[0].Items[0].Path);
    }

    [TestMethod]
    public void DismissShelf_UsesHiddenForEmptyAndEdgeDockedForNonEmpty()
    {
        var manager = new DropShelfManager();
        var placement = new ShelfRailPlacement("DISPLAY2", ShelfRailEdge.Left);

        manager.ShowShelf();
        Assert.AreEqual(ShelfDisplayState.Hidden, manager.DismissShelf(placement));

        manager.AcceptDrop([new(@"C:\Work\Item.txt", "Item.txt", false)]);
        manager.ShowShelf();

        Assert.AreEqual(ShelfDisplayState.EdgeDocked, manager.DismissShelf(placement));
        Assert.AreEqual(placement, manager.RailPlacement);
    }

    [TestMethod]
    public async Task RailPlacementCanBeUpdatedWhileShelfIsDocked()
    {
        var manager = CreateManagerWithOneItem(out _);
        var initial = new ShelfRailPlacement("DISPLAY1", ShelfRailEdge.Right);
        var updated = new ShelfRailPlacement("DISPLAY2", ShelfRailEdge.Left);

        manager.ShowShelf();
        manager.DismissShelf(initial);
        await manager.SetRailPlacementAsync(updated);

        Assert.AreEqual(ShelfDisplayState.EdgeDocked, manager.DisplayState);
        Assert.AreEqual(updated, manager.RailPlacement);
    }

    [TestMethod]
    public void RemovingLastBatch_HidesAnEdgeDockedShelf()
    {
        var manager = CreateManagerWithOneItem(out var item);
        manager.ShowShelf();
        manager.DismissShelf(new ShelfRailPlacement("DISPLAY1", ShelfRailEdge.Right));

        Assert.IsTrue(manager.RemoveItem(item.Id));
        Assert.AreEqual(ShelfDisplayState.Hidden, manager.DisplayState);
    }

    [TestMethod]
    public void ShowingShelf_LeavesEdgePlacementAvailableForTheNextDismissal()
    {
        var manager = CreateManagerWithOneItem(out _);
        var placement = new ShelfRailPlacement("DISPLAY1", ShelfRailEdge.Right);

        manager.ShowShelf();
        manager.DismissShelf(placement);
        manager.ShowShelf();

        Assert.AreEqual(ShelfDisplayState.UnifiedShelf, manager.DisplayState);
        Assert.AreEqual(placement, manager.RailPlacement);
    }
    [TestMethod]
    public void AcceptedDragFromRail_RemovesBatchBeforeShelfIsShownAgain()
    {
        var manager = CreateManagerWithOneItem(out var item);
        var placement = new ShelfRailPlacement("DISPLAY1", ShelfRailEdge.Right);
        manager.DismissShelf(placement);
        Assert.AreEqual(ShelfDisplayState.EdgeDocked, manager.DisplayState);

        // Simulate dragging and completing from the rail
        var preparation = manager.PrepareBatchForDrag(manager.Batches[0].Id, _ => ItemAvailability.Available);
        Assert.HasCount(1, preparation.AvailableItems);
        Assert.IsTrue(manager.CompleteBatchDrag(manager.Batches[0].Id, DragOutOutcome.AcceptedCopy));
        Assert.IsEmpty(manager.Batches);

        // Now user re-opens the shelf
        manager.ShowShelf();
        Assert.IsEmpty(manager.Batches);
    }

    [TestMethod]
    public void EdgeDockedShelf_AcceptsSuccessiveDrops_MaintainsBatchOrdering()
    {
        var manager = CreateManagerWithOneItem(out _);
        var placement = new ShelfRailPlacement("DISPLAY1", ShelfRailEdge.Right);
        manager.DismissShelf(placement);
        Assert.AreEqual(ShelfDisplayState.EdgeDocked, manager.DisplayState);

        var firstDrop = manager.AcceptDrop([
            new(@"C:\Work\RailFirst.txt", "RailFirst.txt", false),
        ]);
        var secondDrop = manager.AcceptDrop([
            new(@"C:\Work\RailSecond.txt", "RailSecond.txt", false),
        ]);

        Assert.HasCount(3, manager.Batches);
        Assert.AreSame(secondDrop.Batch, manager.Batches[0]);
        Assert.AreSame(firstDrop.Batch, manager.Batches[1]);
        Assert.AreEqual(ShelfDisplayState.EdgeDocked, manager.DisplayState);
    }


    private static DropShelfManager CreateManagerWithOneItem(out ShelfItem item)
    {
        var manager = new DropShelfManager();
        var acceptance = manager.AcceptDrop([new(@"C:\Work\Item.txt", "Item.txt", false)]);
        item = acceptance.Batch!.Items[0];
        return manager;
    }

}
