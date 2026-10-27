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

const item = (id, batchId, isPinned) => ({ id, batchId, position: 0, path: `C:\\${id}.txt`, name: `${id}.txt`, isFolder: false, isPinned, availability: "Available", ownership: "External", managedContentKind: null });

async function mount(batches, { failPin = false, managedFiles = 0, failRemove = false } = {}) {
  const callbacks = new Map();
  const nativeEvents = new Map();
  const documentEvents = new Map();
  const commands = [];
  let callbackId = 0;
  let current = batches;
  globalThis.document = { addEventListener: (name, listener) => documentEvents.set(name, listener), removeEventListener: name => documentEvents.delete(name), querySelector: () => null, activeElement: null };
  globalThis.window = {
    __TAURI_INTERNALS__: {
      metadata: { currentWindow: { label: "main" } },
      transformCallback: callback => { callbacks.set(++callbackId, callback); return callbackId; },
      async invoke(command, args) {
        commands.push([command, args]);
        if (command === "plugin:event|listen") { nativeEvents.set(args.event, callbacks.get(args.handler)); return callbackId; }
        if (command === "plugin:event|unlisten") return;
        if (command === "cancel_selection_removal") return;
        if (command === "get_manage_items") return null;
        if (command === "dismiss_drop_shelf") return;
        if (command === "get_shelf") return { batches: current, preferredWidth: 191, height: 180 };
        if (command === "get_shelf_resizing" || command === "get_drop_shelf_clear_temporary_state") return false;
        if (command === "plugin:window|current_monitor") return null;
        if (command === "plugin:window|scale_factor") return 1;
        if (command.endsWith("inner_size") || command.endsWith("outer_size")) return { width: 191, height: 180 };
        if (command.endsWith("outer_position")) return { x: 0, y: 0 };
        if (command === "set_batches_pinned") {
          if (failPin) throw new Error("forced failure");
          current = current.map(batch => args.batchIds.includes(batch.id) ? { ...batch, items: batch.items.map(entry => ({ ...entry, isPinned: args.pinned })) } : batch);
          return { batches: current, preferredWidth: 191, height: 180 };
        }
        if (command === "prepare_batches_removal") {
          const picked = current.filter(batch => args.batchIds.includes(batch.id));
          return { batches: picked.length, items: picked.reduce((n, batch) => n + batch.items.length, 0), managedFiles, token: "opaque-token" };
        }
        if (command === "remove_batches") {
          if (failRemove) throw new Error("forced removal failure");
          current = current.filter(batch => !args.batchIds.includes(batch.id));
          return { batches: current, preferredWidth: 191, height: 180 };
        }
        if (command === "remove_batch") {
          current = current.filter(batch => batch.id !== args.batchId);
          return { batches: current, preferredWidth: 191, height: 180 };
        }
        throw Error(`Unexpected command ${command}`);
      },
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener() {} },
  };
  let shelf;
  const app = renderer.createApp({ setup() { shelf = useShelf(); return () => null; } });
  app.mount({});
  await settle(); await nextTick(); await settle();
  return { shelf, commands, nativeEvents, documentEvents, close() { app.unmount(); delete globalThis.window; delete globalThis.document; } };
}

const fixture = () => [
  { id: "a", createdAt: 0, position: 0, items: [item("a1", "a", false)] },
  { id: "b", createdAt: 0, position: 1, items: [item("b1", "b", false), item("b2", "b", false)] },
  { id: "c", createdAt: 0, position: 2, items: [item("c1", "c", true)] },
];

test("secondary-click on a selected card keeps the whole Batch Selection as the menu target", async () => {
  const { shelf, close } = await mount(fixture());
  try {
    shelf.selectBatch("a", {});
    shelf.selectBatch("b", { ctrlKey: true });
    shelf.openSelectionMenu("b", { x: 5, y: 6 }, false);
    assert.deepEqual(shelf.selection.ids.value, ["a", "b"]);
    assert.deepEqual(shelf.selectionMenu.value.batchIds, ["a", "b"]);
    assert.deepEqual(shelf.selectionMenu.value.anchor, { x: 5, y: 6 });
  } finally { close(); }
});

test("secondary-click on an unselected card replaces selection with only that card", async () => {
  const { shelf, close } = await mount(fixture());
  try {
    shelf.selectBatch("a", {});
    shelf.selectBatch("b", { ctrlKey: true });
    shelf.openSelectionMenu("c", { x: 1, y: 2 }, false);
    assert.deepEqual(shelf.selection.ids.value, ["c"]);
    assert.deepEqual(shelf.selectionMenu.value.batchIds, ["c"]);
  } finally { close(); }
});

test("pinSelection sends exact batch identities, retains selection, and closes the menu", async () => {
  const { shelf, commands, close } = await mount(fixture());
  try {
    shelf.selectBatch("a", {});
    shelf.selectBatch("b", { ctrlKey: true });
    shelf.openSelectionMenu("a", { x: 0, y: 0 }, false);
    await shelf.pinSelection(true);
    assert.deepEqual(commands.find(([name]) => name === "set_batches_pinned")[1], { batchIds: ["a", "b"], pinned: true });
    assert.deepEqual(shelf.selection.ids.value, ["a", "b"]);
    assert.equal(shelf.selectionMenu.value, null);
    assert.ok(shelf.batches.value[0].items.every(entry => entry.isPinned));
  } finally { close(); }
});

test("a failed pinSelection keeps selection, reports the error, and closes the menu", async () => {
  const { shelf, close } = await mount(fixture(), { failPin: true });
  try {
    shelf.selectBatch("a", {});
    shelf.openSelectionMenu("a", { x: 0, y: 0 }, false);
    await shelf.pinSelection(true);
    assert.deepEqual(shelf.selection.ids.value, ["a"]);
    assert.match(shelf.status.value, /forced failure/);
    assert.equal(shelf.selectionMenu.value, null);
    assert.ok(!shelf.batches.value[0].items[0].isPinned);
  } finally { close(); }
});

test("hiding the shelf discards an open Selection Actions Menu", async () => {
  const { shelf, nativeEvents, close } = await mount(fixture());
  try {
    shelf.openSelectionMenu("a", { x: 0, y: 0 }, false);
    assert.ok(shelf.selectionMenu.value);
    nativeEvents.get("shelf-hidden")({ payload: null });
    assert.equal(shelf.selectionMenu.value, null, "native hiding must discard the menu, not merely hide its pixels");
  } finally { close(); }
});

test("Escape closes an open Selection Actions Menu first and keeps selection and the surface", async () => {
  const { shelf, commands, documentEvents, close } = await mount(fixture());
  try {
    shelf.openSelectionMenu("a", { x: 0, y: 0 }, false);
    await documentEvents.get("keydown")({ key: "Escape", defaultPrevented: false, repeat: false });
    assert.equal(shelf.selectionMenu.value, null);
    assert.deepEqual(shelf.selection.ids.value, ["a"], "Escape on the menu must retain selection");
    assert.ok(!commands.some(([name]) => name === "dismiss_drop_shelf"), "the first Escape must not dismiss the shelf");
  } finally { close(); }
});

test("a single selected batch removes immediately without a confirmation", async () => {
  const { shelf, commands, close } = await mount(fixture());
  try {
    shelf.selectBatch("b", {});
    shelf.openSelectionMenu("b", { x: 0, y: 0 }, false);
    await shelf.removeSelection();
    assert.deepEqual(commands.find(([name]) => name === "remove_batch")[1], { batchId: "b" });
    assert.equal(shelf.removalConfirmation.value, null, "singleton Remove stays immediate");
    assert.ok(!commands.some(([name]) => name === "prepare_batches_removal"));
  } finally { close(); }
});

test("several selected batches open a confirmation with authoritative counts and remove nothing yet", async () => {
  const { shelf, commands, close } = await mount(fixture(), { managedFiles: 1 });
  try {
    shelf.selectBatch("a", {});
    shelf.selectBatch("b", { ctrlKey: true });
    shelf.openSelectionMenu("a", { x: 0, y: 0 }, false);
    await shelf.removeSelection();
    assert.deepEqual(commands.find(([name]) => name === "prepare_batches_removal")[1], { batchIds: ["a", "b"] });
    assert.equal(shelf.selectionMenu.value, null, "the menu closes before confirmation appears");
    assert.deepEqual(shelf.removalConfirmation.value.preview, { batches: 2, items: 3, managedFiles: 1, token: "opaque-token" });
    assert.ok(!commands.some(([name]) => name === "remove_batches"), "nothing is removed until confirmed");
    assert.equal(shelf.batches.value.length, 3);
  } finally { close(); }
});

test("confirming sends only identities and the opaque token, then prunes selection", async () => {
  const { shelf, commands, close } = await mount(fixture());
  try {
    shelf.selectBatch("a", {});
    shelf.selectBatch("b", { ctrlKey: true });
    shelf.openSelectionMenu("a", { x: 0, y: 0 }, false);
    await shelf.removeSelection();
    await shelf.confirmRemoval();
    assert.deepEqual(commands.find(([name]) => name === "remove_batches")[1], { batchIds: ["a", "b"], token: "opaque-token" });
    assert.equal(shelf.removalConfirmation.value, null);
    assert.deepEqual(shelf.batches.value.map(batch => batch.id), ["c"]);
    assert.deepEqual(shelf.selection.ids.value, []);
  } finally { close(); }
});

test("cancelling keeps selection and every batch; a failed confirm keeps them too and reports the error", async () => {
  const cancelled = await mount(fixture());
  try {
    cancelled.shelf.selectBatch("a", {});
    cancelled.shelf.selectBatch("b", { ctrlKey: true });
    cancelled.shelf.openSelectionMenu("a", { x: 0, y: 0 }, false);
    await cancelled.shelf.removeSelection();
    await cancelled.shelf.cancelRemoval();
    assert.equal(cancelled.shelf.removalConfirmation.value, null);
    assert.deepEqual(cancelled.shelf.selection.ids.value, ["a", "b"]);
    assert.equal(cancelled.shelf.batches.value.length, 3);
  } finally { cancelled.close(); }
  const failed = await mount(fixture(), { failRemove: true });
  try {
    failed.shelf.selectBatch("a", {});
    failed.shelf.selectBatch("b", { ctrlKey: true });
    failed.shelf.openSelectionMenu("a", { x: 0, y: 0 }, false);
    await failed.shelf.removeSelection();
    await failed.shelf.confirmRemoval();
    assert.equal(failed.shelf.removalConfirmation.value, null);
    assert.deepEqual(failed.shelf.selection.ids.value, ["a", "b"]);
    assert.equal(failed.shelf.batches.value.length, 3);
    assert.match(failed.shelf.status.value, /forced removal failure/);
  } finally { failed.close(); }
});

test("while a removal confirmation is open, selection changes, new menus, and Pin are blocked", async () => {
  const { shelf, commands, close } = await mount(fixture());
  try {
    shelf.selectBatch("a", {});
    shelf.selectBatch("b", { ctrlKey: true });
    shelf.openSelectionMenu("a", { x: 0, y: 0 }, false);
    await shelf.removeSelection();
    shelf.selectBatch("c", {});
    assert.deepEqual(shelf.selection.ids.value, ["a", "b"], "selection must not change under an open confirmation");
    shelf.openSelectionMenu("c", { x: 0, y: 0 }, false);
    assert.equal(shelf.selectionMenu.value, null);
    await shelf.pinSelection(true);
    assert.ok(!commands.some(([name]) => name === "set_batches_pinned"));
  } finally { close(); }
});

test("Escape cancels an open removal confirmation first and retains selection", async () => {
  const { shelf, commands, documentEvents, close } = await mount(fixture());
  try {
    shelf.selectBatch("a", {});
    shelf.selectBatch("b", { ctrlKey: true });
    shelf.openSelectionMenu("a", { x: 0, y: 0 }, false);
    await shelf.removeSelection();
    await documentEvents.get("keydown")({ key: "Escape", defaultPrevented: false, repeat: false });
    assert.equal(shelf.removalConfirmation.value, null);
    assert.deepEqual(shelf.selection.ids.value, ["a", "b"]);
    assert.ok(!commands.some(([name]) => name === "dismiss_drop_shelf"));
  } finally { close(); }
});

test("an open removal confirmation blocks every competing state-owner mutation", async () => {
  const { shelf, commands, close } = await mount(fixture());
  try {
    shelf.selectBatch("a", {});
    shelf.selectBatch("b", { ctrlKey: true });
    shelf.openSelectionMenu("a", { x: 0, y: 0 }, false);
    await shelf.removeSelection();
    assert.ok(shelf.removalConfirmation.value);
    const before = commands.length;
    await shelf.removeBatch("c");
    await shelf.setItemPinned("c1", false);
    await shelf.setBatchPinned("c", false);
    await shelf.startItemDrag("c1");
    await shelf.dismissDropShelf();
    await shelf.startResize("East");
    await shelf.openManageItems("c", { x: 0, y: 0, width: 1, height: 1 });
    await shelf.requestClearTemporary();
    await shelf.startMove({ button: 0 });
    assert.deepEqual(commands.slice(before), [], "no IPC command may start while the confirmation owns the surface");
    assert.ok(shelf.removalConfirmation.value, "the confirmation itself is unaffected");
  } finally { close(); }
});

test("an open removal confirmation also blocks starting a batch drag", async () => {
  const { shelf, commands, close } = await mount(fixture());
  try {
    shelf.selectBatch("a", {});
    shelf.selectBatch("b", { ctrlKey: true });
    shelf.openSelectionMenu("a", { x: 0, y: 0 }, false);
    await shelf.removeSelection();
    const before = commands.length;
    await shelf.startBatchDrag("c");
    assert.deepEqual(commands.slice(before), [], "a drag must not be prepared under an open confirmation");
  } finally { close(); }
});
