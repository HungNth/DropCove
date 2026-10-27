import assert from "node:assert/strict";
import test from "node:test";
import { createRenderer, nextTick } from "vue";
import { registerHooks } from "node:module";
import { setImmediate as settle } from "node:timers/promises";

const hooks = registerHooks({
  resolve(specifier, context, nextResolve) {
    try { return nextResolve(specifier, context); }
    catch (error) {
      if (error.code === "ERR_MODULE_NOT_FOUND" && specifier.startsWith(".")) return nextResolve(`${specifier}.ts`, context);
      throw error;
    }
  },
});
const { useManageItems } = await import("../src/composables/useManageItems.ts");
hooks.deregister();

const renderer = createRenderer({
  createElement: () => ({}), createText: () => ({}), createComment: () => ({}),
  setText() {}, setElementText() {}, patchProp() {}, insert() {}, remove() {},
  parentNode: () => null, nextSibling: () => null,
});

const item = (id, isPinned) => ({ id, batchId: "batch", position: 0, path: `C:\\${id}.txt`, name: `${id}.txt`, isFolder: false, isPinned, availability: "Available", ownership: "External", managedContentKind: null });

async function mount({ failPin = false } = {}) {
  const callbacks = new Map();
  const nativeEvents = new Map();
  const commands = [];
  let callbackId = 0;
  let batch = { id: "batch", createdAt: 0, position: 0, items: [item("i1", false), item("i2", false), item("i3", true)] };
  globalThis.document = { addEventListener() {}, removeEventListener() {}, querySelector: () => null, activeElement: null };
  globalThis.window = {
    __TAURI_INTERNALS__: {
      metadata: { currentWindow: { label: "manage_items" } },
      transformCallback: callback => { callbacks.set(++callbackId, callback); return callbackId; },
      async invoke(command, args) {
        commands.push([command, args]);
        if (command === "plugin:event|listen") { nativeEvents.set(args.event, callbacks.get(args.handler)); return callbackId; }
        if (command === "plugin:event|unlisten") return;
        if (command === "cancel_selection_removal") return;
        if (command === "get_manage_items") return batch;
        if (command === "cancel_item_drag") return { batches: [batch] };
        if (command === "set_items_pinned") {
          if (failPin) throw new Error("forced failure");
          batch = { ...batch, items: batch.items.map(entry => args.itemIds.includes(entry.id) ? { ...entry, isPinned: args.pinned } : entry) };
          return { batches: [batch], preferredWidth: 191, height: 180 };
        }
        if (command === "prepare_items_removal") {
          return { batches: 1, items: args.itemIds.length, managedFiles: 0, token: "item-tok" };
        }
        if (command === "remove_items") {
          batch = { ...batch, items: batch.items.filter(entry => !args.itemIds.includes(entry.id)) };
          return { batches: [batch], preferredWidth: 191, height: 180 };
        }
        if (command === "remove_item") {
          batch = { ...batch, items: batch.items.filter(entry => entry.id !== args.itemId) };
          return { batches: [batch], preferredWidth: 191, height: 180 };
        }
        throw Error(`Unexpected command ${command}`);
      },
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener() {} },
  };
  let manage;
  const app = renderer.createApp({ setup() { manage = useManageItems(); return () => null; } });
  app.mount({});
  await settle(); await nextTick(); await settle();
  return { manage, commands, nativeEvents, close() { app.unmount(); delete globalThis.window; delete globalThis.document; } };
}

test("secondary-click on a selected row keeps the whole Item Selection as the menu target", async () => {
  const { manage, close } = await mount();
  try {
    manage.selectItem("i1", {});
    manage.selectItem("i2", { ctrlKey: true });
    manage.openSelectionMenu("i2", { x: 4, y: 5 }, false);
    assert.deepEqual(manage.selection.ids.value, ["i1", "i2"]);
    assert.deepEqual(manage.selectionMenu.value.itemIds, ["i1", "i2"]);
    assert.deepEqual(manage.selectionMenu.value.anchor, { x: 4, y: 5 });
  } finally { close(); }
});

test("secondary-click on an unselected row replaces selection with only that row", async () => {
  const { manage, close } = await mount();
  try {
    manage.selectItem("i1", {});
    manage.selectItem("i2", { ctrlKey: true });
    manage.openSelectionMenu("i3", { x: 0, y: 0 }, false);
    assert.deepEqual(manage.selection.ids.value, ["i3"]);
    assert.deepEqual(manage.selectionMenu.value.itemIds, ["i3"]);
  } finally { close(); }
});

test("pinSelection sends the active batch and exact item identities, retains selection, and closes the menu", async () => {
  const { manage, commands, close } = await mount();
  try {
    manage.selectItem("i1", {});
    manage.selectItem("i2", { ctrlKey: true });
    manage.openSelectionMenu("i1", { x: 0, y: 0 }, false);
    assert.equal(await manage.pinSelection(true), true);
    assert.deepEqual(commands.find(([name]) => name === "set_items_pinned")[1], { batchId: "batch", itemIds: ["i1", "i2"], pinned: true });
    assert.deepEqual(manage.selection.ids.value, ["i1", "i2"]);
    assert.equal(manage.selectionMenu.value, null);
    assert.ok(manage.batch.value.items.every(entry => entry.isPinned));
  } finally { close(); }
});

test("a failed pinSelection resolves false, reports the error, and keeps selection", async () => {
  const { manage, close } = await mount({ failPin: true });
  try {
    manage.selectItem("i1", {});
    manage.openSelectionMenu("i1", { x: 0, y: 0 }, false);
    assert.equal(await manage.pinSelection(true), false);
    assert.deepEqual(manage.selection.ids.value, ["i1"]);
    assert.match(manage.status.value, /forced failure/);
    assert.equal(manage.selectionMenu.value, null);
  } finally { close(); }
});

test("a popup refresh that removes the target closes the menu", async () => {
  const { manage, nativeEvents, close } = await mount();
  try {
    manage.openSelectionMenu("i1", { x: 0, y: 0 }, false);
    assert.ok(manage.selectionMenu.value);
    nativeEvents.get("manage-items-opened")({ payload: null });
    await settle(); await nextTick();
    assert.equal(manage.selectionMenu.value, null, "a replaced or reopened popup must discard the menu");
  } finally { close(); }
});

test("one selected Manage Items row removes immediately without a confirmation", async () => {
  const { manage, commands, close } = await mount();
  try {
    manage.selectItem("i1", {});
    manage.openSelectionMenu("i1", { x: 0, y: 0 }, false);
    await manage.removeSelection();
    assert.deepEqual(commands.find(([name]) => name === "remove_item")[1], { itemId: "i1" });
    assert.equal(manage.removalConfirmation.value, null);
  } finally { close(); }
});

test("several selected rows request an item removal preview and confirm removes them", async () => {
  const { manage, commands, close } = await mount();
  try {
    manage.selectItem("i1", {});
    manage.selectItem("i2", { ctrlKey: true });
    manage.openSelectionMenu("i1", { x: 0, y: 0 }, false);
    await manage.removeSelection();
    assert.deepEqual(commands.find(([name]) => name === "prepare_items_removal")[1], { batchId: "batch", itemIds: ["i1", "i2"] });
    assert.equal(manage.removalConfirmation.value.preview.token, "item-tok");
    await manage.confirmRemoval();
    assert.deepEqual(commands.find(([name]) => name === "remove_items")[1], { batchId: "batch", itemIds: ["i1", "i2"], token: "item-tok" });
    assert.equal(manage.removalConfirmation.value, null);
    assert.deepEqual(manage.batch.value.items.map(entry => entry.id), ["i3"]);
  } finally { close(); }
});
