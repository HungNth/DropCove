import assert from "node:assert/strict";
import test from "node:test";
import { setImmediate as settle } from "node:timers/promises";
import { createRenderer, nextTick } from "vue";
import { useShelf } from "../src/composables/useShelf.ts";

const renderer = createRenderer({
  createElement: () => ({}), createText: () => ({}), createComment: () => ({}),
  setText() {}, setElementText() {}, patchProp() {}, insert() {}, remove() {},
  parentNode: () => null, nextSibling: () => null,
});

test("Escape cancels availability confirmation, closes Manage Items, clears selection, then dismisses", async () => {
  const events = new Map();
  const callbacks = new Map();
  const commands = [];
  let callbackId = 0;
  let activeManagement = null;
  let delayedManagement = null;
  const batch = { id: "batch", createdAt: 0, position: 0, items: [{ id: "item", batchId: "batch", position: 0, path: "C:\\a.txt", name: "a.txt", isFolder: false, isPinned: false, availability: "Available" }] };
  const snapshot = { batches: [batch], preferredWidth: 191, height: 180 };
  globalThis.document = { addEventListener: (name, listener) => events.set(name, listener), removeEventListener: name => events.delete(name), querySelector: () => null };
  globalThis.window = {
    __TAURI_INTERNALS__: {
      metadata: { currentWindow: { label: "main" } },
      transformCallback: callback => { callbacks.set(++callbackId, callback); return callbackId; },
      async invoke(command) {
        commands.push(command);
        if (command === "plugin:event|listen") return callbackId;
        if (command === "plugin:event|unlisten") return;
        if (command === "get_shelf") return snapshot;
        if (command === "get_shelf_resizing" || command === "get_drop_shelf_clear_temporary_state") return false;
        if (command === "plugin:window|current_monitor") return null;
        if (command === "plugin:window|scale_factor") return 1;
        if (command.endsWith("inner_size") || command.endsWith("outer_size")) return { width: 191, height: 180 };
        if (command.endsWith("outer_position")) return { x: 0, y: 0 };
        if (command === "prepare_batch_drag") return { token: "prepared", available: 1, unavailable: 1, snapshot };
        if (command === "cancel_batch_drag") return snapshot;
        if (command === "get_manage_items") return delayedManagement ?? activeManagement;
        if (command === "close_manage_items") { activeManagement = null; return; }
        if (command === "dismiss_drop_shelf") return;
        throw Error(`Unexpected command ${command}`);
      },
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener() {} },
  };
  let shelf;
  const app = renderer.createApp({ setup() { shelf = useShelf(); return () => null; } });
  app.mount({});
  try {
    await settle(); await nextTick(); await settle();
    assert.equal(shelf.batches.value[0].id, "batch", shelf.status.value);
    shelf.selectBatch("batch", {});
    await shelf.startBatchDrag("batch");
    assert.equal(shelf.pendingConfirmation.value.token, "prepared");
    const escape = { key: "Escape", defaultPrevented: false, repeat: false };
    await events.get("keydown")(escape);
    assert.equal(shelf.pendingConfirmation.value, null);
    assert.deepEqual(shelf.selection.ids.value, ["batch"]);
    assert.ok(commands.includes("cancel_batch_drag"));
    await shelf.startBatchDrag("batch");
    shelf.confirmBatchDrag();
    await events.get("keydown")(escape);
    assert.deepEqual(shelf.selection.ids.value, ["batch"], "Escape cancels an armed token before clearing selection");
    assert.equal(commands.filter(command => command === "cancel_batch_drag").length, 2);
    activeManagement = batch;
    await events.get("keydown")(escape);
    assert.equal(activeManagement, null);
    assert.deepEqual(shelf.selection.ids.value, ["batch"]);
    await events.get("keydown")(escape);
    assert.deepEqual(shelf.selection.ids.value, []);
    assert.equal(commands.filter(command => command === "dismiss_drop_shelf").length, 0);
    await events.get("keydown")(escape);
    assert.equal(commands.filter(command => command === "dismiss_drop_shelf").length, 1);
    let releaseManagement;
    delayedManagement = new Promise(resolve => { releaseManagement = resolve; });
    shelf.selectBatch("batch", {});
    const oldEscape = events.get("keydown")(escape);
    shelf.batches.value = [...shelf.batches.value, { ...batch, id: "new-batch" }];
    shelf.selectBatch("new-batch", {});
    releaseManagement(null);
    await oldEscape;
    assert.deepEqual(shelf.selection.ids.value, ["new-batch"], "an earlier asynchronous Escape must not clear a later selection");
  } finally { app.unmount(); delete globalThis.window; delete globalThis.document; }
});
