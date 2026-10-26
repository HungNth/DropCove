import assert from "node:assert/strict";
import test from "node:test";
import { effectScope, shallowRef } from "vue";
import { useSelection } from "../src/composables/useSelection.ts";

function fixture(count = 60) {
  const scope = effectScope();
  const batches = shallowRef(Array.from({ length: count }, (_, index) => ({ id: String(index), items: [{ id: `i${index}` }] })));
  const selection = scope.run(() => useSelection(batches));
  return { batches, selection, close: () => scope.stop() };
}

test("Ctrl toggle-off remains the Shift anchor across virtualized ranges", () => {
  const f = fixture();
  try {
    f.selection.select("2", {});
    f.selection.select("2", { ctrlKey: true });
    f.selection.select("55", { shiftKey: true });
    assert.deepEqual(f.selection.ids.value, Array.from({ length: 54 }, (_, index) => String(index + 2)));
    assert.equal(f.selection.itemCount.value, 54);
    f.selection.select("4", { ctrlKey: true, shiftKey: true });
    assert.deepEqual(f.selection.ids.value, ["2", "3", "4"]);
  } finally { f.close(); }
});

test("drag scope preserves a selected set but isolates an unselected card", () => {
  const f = fixture(4);
  try {
    f.selection.select("0", {});
    f.selection.select("2", { ctrlKey: true });
    assert.deepEqual(f.selection.dragIds("2"), ["0", "2"]);
    assert.deepEqual(f.selection.dragIds("1"), ["1"]);
    assert.deepEqual(f.selection.ids.value, ["0", "2"]);
  } finally { f.close(); }
});

test("mutations retain surviving identities and clear a removed anchor", () => {
  const f = fixture(4);
  try {
    f.selection.select("0", {});
    f.selection.select("2", { ctrlKey: true });
    f.batches.value = f.batches.value.filter(batch => batch.id !== "2");
    assert.deepEqual(f.selection.ids.value, ["0"]);
    f.selection.select("3", { shiftKey: true });
    assert.deepEqual(f.selection.ids.value, ["3"]);
    f.batches.value = [{ id: "new", items: [{ id: "new-item" }] }, ...f.batches.value];
    assert.deepEqual(f.selection.ids.value, ["3"]);
  } finally { f.close(); }
});

test("Ctrl+A preserves a valid anchor and does not create an absent one", () => {
  const f = fixture(5);
  try {
    f.selection.select("2", {});
    f.selection.selectAll();
    f.selection.select("4", { shiftKey: true });
    assert.deepEqual(f.selection.ids.value, ["2", "3", "4"]);
    f.selection.clear();
    f.selection.selectAll();
    f.selection.select("4", { shiftKey: true });
    assert.deepEqual(f.selection.ids.value, ["4"]);
  } finally { f.close(); }
});
