import assert from "node:assert/strict";
import test from "node:test";
import { effectScope, shallowRef } from "vue";
import { usePreparedDrag } from "../src/composables/usePreparedDrag.ts";

test("item dispatch rejection releases preparation and retains the selected occurrences", async () => {
  const batch = { id: "active", items: [{ id: "first" }, { id: "second" }] };
  const batches = shallowRef([batch]);
  const selected = shallowRef(["first", "second"]);
  const busy = shallowRef(false);
  const status = shallowRef("");
  let prepared = null;
  globalThis.window = { __TAURI_INTERNALS__: { async invoke(command, args) {
    if (command === "prepare_item_drag") {
      prepared = { token: "owned-item-token", ids: [...args.itemIds] };
      return { token: prepared.token, available: 2, unavailable: 0, snapshot: { batches: [batch] } };
    }
    if (command === "start_item_drag") throw new Error("IPC rejected before native dispatch");
    if (command === "cancel_item_drag") {
      assert.equal(args.token, prepared.token);
      prepared = null;
      return { batches: [batch] };
    }
    throw new Error(`Unexpected platform command: ${command}`);
  } } };
  const scope = effectScope();
  const drag = scope.run(() => usePreparedDrag("item", batches, selected, busy, status, async () => {}, () => { selected.value = []; }));
  try {
    await drag.start(["first", "second"]);
    assert.equal(prepared, null, "a rejected IPC must not strand an armed native token");
    assert.deepEqual(selected.value, ["first", "second"]);
    assert.match(status.value, /IPC rejected before native dispatch/);
    assert.equal(busy.value, false);
  } finally {
    drag.dispose();
    scope.stop();
    delete globalThis.window;
  }
});
