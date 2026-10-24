import assert from "node:assert/strict";
import test from "node:test";
import { effectScope, nextTick, shallowRef } from "vue";
import { useVirtualCollection } from "../src/composables/useVirtualCollection.ts";

test("off-screen batch focus survives responsive columns and virtual row churn", async () => {
  const scope = effectScope();
  const items = shallowRef(Array.from({ length: 60 }, (_, index) => ({ id: String(index) })));
  const columns = shallowRef(2);
  let focused = null;
  const element = {
    clientWidth: 400, clientHeight: 100, scrollTop: 0,
    addEventListener() {}, removeEventListener() {},
    querySelectorAll: () => [],
    querySelector(selector) {
      const index = Number(selector.match(/\d+/)[0]);
      return { querySelector: () => ({ focus: () => { focused = items.value[index].id; } }) };
    },
  };
  globalThis.getComputedStyle = () => ({ paddingLeft: "0", paddingRight: "0" });
  try {
    const collection = scope.run(() => useVirtualCollection(items, shallowRef(element), columns, 48, 4));
    await collection.focus(55, -2);
    await nextTick();
    assert.equal(focused, "55");
    assert.ok(collection.entries.value.some(entry => entry.item.id === "55"));
    assert.ok(element.scrollTop > 0, "keyboard navigation scrolls the target into view");
    columns.value = 3;
    await nextTick();
    assert.ok(collection.entries.value.some(entry => entry.item.id === "55"), "responsive reflow must keep the focused batch mounted");
    items.value = items.value.filter(item => Number(item.id) < 3 || Number(item.id) > 8);
    await nextTick(); await nextTick();
    assert.ok(collection.entries.value.some(entry => entry.item.id === "55"), "removing earlier batches must retain focus by identity rather than old row index");
    await collection.focus(0, -2);
    assert.equal(focused, "0");
  } finally { scope.stop(); delete globalThis.getComputedStyle; }
});
