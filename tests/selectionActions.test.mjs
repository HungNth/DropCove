import assert from "node:assert/strict";
import test from "node:test";
import { batchSelectionPinAction, itemSelectionPinAction, removalConfirmationCopy } from "../src/composables/selectionActions.ts";

const item = (id, isPinned) => ({ id, isPinned });
const batch = (id, ...items) => ({ id, items });

test("one selected single-item batch is labelled by item and offers the opposite pin state", () => {
  assert.deepEqual(batchSelectionPinAction([batch("a", item("1", false))]), { label: "Pin Item", pinned: true });
  assert.deepEqual(batchSelectionPinAction([batch("a", item("1", true))]), { label: "Unpin Item", pinned: false });
});

test("one selected multi-item batch is one selection unit labelled All Items", () => {
  assert.deepEqual(batchSelectionPinAction([batch("a", item("1", false), item("2", false))]), { label: "Pin All Items", pinned: true });
  assert.deepEqual(batchSelectionPinAction([batch("a", item("1", true), item("2", true))]), { label: "Unpin All Items", pinned: false });
});

test("several selected batches use Selected labels and Mixed or None offer Pin", () => {
  const none = [batch("a", item("1", false)), batch("b", item("2", false), item("3", false))];
  const mixed = [batch("a", item("1", true)), batch("b", item("2", true), item("3", false))];
  const all = [batch("a", item("1", true)), batch("b", item("2", true), item("3", true))];
  assert.deepEqual(batchSelectionPinAction(none), { label: "Pin Selected", pinned: true });
  assert.deepEqual(batchSelectionPinAction(mixed), { label: "Pin Selected", pinned: true });
  assert.deepEqual(batchSelectionPinAction(all), { label: "Unpin Selected", pinned: false });
});

test("removal copy reports exact batch and item scope", () => {
  assert.equal(removalConfirmationCopy({ batches: 3, items: 7, managedFiles: 0 }, "batches").scope, "3 batches · 7 items");
  assert.equal(removalConfirmationCopy({ batches: 2, items: 2, managedFiles: 0 }, "batches").scope, "2 batches · 2 items");
  assert.equal(removalConfirmationCopy({ batches: 0, items: 4, managedFiles: 0 }, "items").scope, "4 items");
});

test("all-External removal says sources are not deleted and promises no file deletion", () => {
  const copy = removalConfirmationCopy({ batches: 2, items: 3, managedFiles: 0 }, "batches");
  assert.match(copy.body, /source files and folders will not be deleted/i);
  assert.doesNotMatch(copy.body, /permanently deleted/i);
});

test("mixed ownership keeps External sources and reports the exact Managed File count", () => {
  const copy = removalConfirmationCopy({ batches: 2, items: 5, managedFiles: 2 }, "batches");
  assert.match(copy.body, /External sources stay/i);
  assert.match(copy.body, /2 DropCove-created files will be permanently deleted/);
  assert.match(removalConfirmationCopy({ batches: 2, items: 5, managedFiles: 1 }, "batches").body, /1 DropCove-created file will be permanently deleted/);
});

test("all-Managed removal reports the permanent deletion count without an External-sources claim", () => {
  const copy = removalConfirmationCopy({ batches: 2, items: 2, managedFiles: 2 }, "batches");
  assert.match(copy.body, /2 DropCove-created files will be permanently deleted/);
  assert.doesNotMatch(copy.body, /External sources stay|will not be deleted/i);
});

test("one selected Manage Items row is labelled by item; several use Selected and Mixed or None offer Pin", () => {
  assert.deepEqual(itemSelectionPinAction([item("1", false)]), { label: "Pin Item", pinned: true });
  assert.deepEqual(itemSelectionPinAction([item("1", true)]), { label: "Unpin Item", pinned: false });
  assert.deepEqual(itemSelectionPinAction([item("1", false), item("2", false)]), { label: "Pin Selected", pinned: true });
  assert.deepEqual(itemSelectionPinAction([item("1", true), item("2", false)]), { label: "Pin Selected", pinned: true });
  assert.deepEqual(itemSelectionPinAction([item("1", true), item("2", true)]), { label: "Unpin Selected", pinned: false });
});
