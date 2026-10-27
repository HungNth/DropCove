import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { registerHooks } from "node:module";
import { parse, compileScript, compileTemplate } from "vue/compiler-sfc";
import ts from "typescript";
import { createRenderer, markRaw, nextTick } from "vue";
import { setImmediate as settle } from "node:timers/promises";

const hooks = registerHooks({
  resolve(specifier, context, nextResolve) {
    try { return nextResolve(specifier, context); }
    catch (error) {
      if (error.code === "ERR_MODULE_NOT_FOUND" && specifier.startsWith(".")) return nextResolve(`${specifier}.ts`, context);
      throw error;
    }
  },
  load(url, context, nextLoad) {
    if (/\/(ShellVisual|ActionIcon|BulkPinning|UtilityMenu|ShelfResizeHandles|DropPlaceholder|BatchDragConfirmation)\.vue$/.test(url)) {
      return { format: "module", source: "export default { render() { return null; } }", shortCircuit: true };
    }
    if (!url.endsWith(".vue")) return nextLoad(url, context);
    const { descriptor } = parse(readFileSync(new URL(url), "utf8"), { filename: url });
    const source = descriptor.script || descriptor.scriptSetup
      ? compileScript(descriptor, { id: url, inlineTemplate: true }).content
      : `${compileTemplate({ source: descriptor.template.content, filename: url, id: url }).code}\nexport default { render };`;
    return { format: "module", source: ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2020, module: ts.ModuleKind.ESNext } }).outputText, shortCircuit: true };
  },
});
const { default: DropShelf } = await import("../src/components/shelf/DropShelf.vue");
hooks.deregister();

let focused = [];
function node(type) {
  return markRaw({
    type, props: {}, children: [], parent: null, text: "", clientWidth: 600, clientHeight: 400, scrollTop: 0, open: false,
    setPointerCapture() {}, releasePointerCapture() {}, focus() { focused.push(this); },
    addEventListener() {}, removeEventListener() {},
    querySelector(selector) { return find(this, selector)[0] ?? null; },
    querySelectorAll(selector) { return find(this, selector); },
    contains(other) { for (let cur = other; cur; cur = cur.parent) if (cur === this) return true; return false; },
    close() { this.open = false; }, showModal() { this.open = true; },
    getBoundingClientRect: () => ({ x: 0, y: 0, width: 164, height: 48, top: 0, bottom: 48, left: 0, right: 164 }),
  });
}
function find(root, selector) {
  const out = [];
  const classes = selector.startsWith(".") ? selector.slice(1).split(".") : null;
  const attr = selector.match(/^\[data-batch-id="(.+)"\]/);
  (function walk(n) {
    if (classes && classes.every(c => n.props?.class?.split(/\s+/).includes(c))) out.push(n);
    if (attr && n.props?.["data-batch-id"] === attr[1]) out.push(n);
    n.children?.forEach(walk);
  })(root);
  return out;
}
const renderer = createRenderer({
  createElement: node, createText: () => node("text"), createComment: () => node("comment"),
  setText: (target, text) => { target.text = text; }, setElementText: (target, text) => { target.text = text; },
  patchProp: (target, key, previous, value) => { target.props[key] = value; },
  insert(child, parent, anchor) {
    if (child.parent) child.parent.children.splice(child.parent.children.indexOf(child), 1);
    child.parent = parent;
    const index = anchor ? parent.children.indexOf(anchor) : -1;
    if (index < 0) parent.children.push(child); else parent.children.splice(index, 0, child);
  },
  remove: child => { child.parent.children.splice(child.parent.children.indexOf(child), 1); },
  parentNode: target => target.parent, nextSibling: () => null,
});

const item = (id, batchId, isPinned) => ({ id, batchId, position: 0, path: `C:\\${id}.txt`, name: `${id}.txt`, isFolder: false, isPinned, availability: "Available", ownership: "External", managedContentKind: null });
const fixture = () => [
  { id: "a", createdAt: 0, position: 0, items: [item("a1", "a", false)] },
  { id: "b", createdAt: 0, position: 1, items: [item("b1", "b", false), item("b2", "b", false)] },
  { id: "c", createdAt: 0, position: 2, items: [{ ...item("c1", "c", false), ownership: "Managed", managedContentKind: "Text" }] },
  { id: "d", createdAt: 0, position: 3, items: [{ ...item("d1", "d", false), ownership: "Managed", managedContentKind: "Image" }] },
];

async function mount() {
  const callbacks = new Map();
  const commands = [];
  let callbackId = 0;
  let current = fixture();
  globalThis.document = { addEventListener() {}, removeEventListener() {}, querySelector: () => null, activeElement: null };
  globalThis.CSS = { escape: value => value };
  globalThis.getComputedStyle = () => ({ gridTemplateColumns: "1fr", paddingLeft: "0", paddingRight: "0", paddingTop: "0", paddingBottom: "0" });
  globalThis.ResizeObserver = class { observe() {} unobserve() {} disconnect() {} };
  globalThis.window = {
    innerWidth: 600, innerHeight: 400, addEventListener() {}, removeEventListener() {},
    setTimeout: (cb, ms = 0) => setTimeout(cb, ms), clearTimeout: id => clearTimeout(id),
    __TAURI_INTERNALS__: {
      metadata: { currentWindow: { label: "main" } },
      transformCallback: callback => { callbacks.set(++callbackId, callback); return callbackId; },
      async invoke(command, args) {
        commands.push([command, args]);
        if (command === "plugin:event|listen") return callbackId;
        if (command === "plugin:event|unlisten") return;
        if (command === "cancel_selection_removal") return;
        if (command === "get_shelf") return { batches: current, preferredWidth: 191, height: 180 };
        if (command === "get_shelf_resizing" || command === "get_drop_shelf_clear_temporary_state") return false;
        if (command === "get_manage_items") return null;
        if (command === "close_manage_items") return;
        if (command === "perform_item_action") return;
        if (command === "plugin:window|current_monitor") return null;
        if (command === "plugin:window|scale_factor") return 1;
        if (command.endsWith("inner_size") || command.endsWith("outer_size")) return { width: 600, height: 400 };
        if (command.endsWith("outer_position")) return { x: 0, y: 0 };
        if (command === "prepare_batches_removal") {
          const picked = current.filter(batch => args.batchIds.includes(batch.id));
          return { batches: picked.length, items: picked.reduce((n, batch) => n + batch.items.length, 0), managedFiles: 0, token: "tok" };
        }
        if (command === "remove_batches") { current = current.filter(batch => !args.batchIds.includes(batch.id)); return { batches: current, preferredWidth: 191, height: 180 }; }
        if (command === "remove_batch") { current = current.filter(batch => batch.id !== args.batchId); return { batches: current, preferredWidth: 191, height: 180 }; }
        if (command === "set_batches_pinned") {
          current = current.map(batch => args.batchIds.includes(batch.id) ? { ...batch, items: batch.items.map(entry => ({ ...entry, isPinned: args.pinned })) } : batch);
          return { batches: current, preferredWidth: 191, height: 180 };
        }
        throw Error(`Unexpected command ${command}`);
      },
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener() {} },
  };
  const root = node("root");
  const app = renderer.createApp(DropShelf);
  app.mount(root);
  await settle(); await nextTick(); await settle(); await nextTick();
  return { root, commands, close() { app.unmount(); delete globalThis.window; delete globalThis.document; } };
}

const cards = root => find(root, ".card");
const menu = root => find(root, ".selection-actions-menu")[0] ?? null;
const menuItems = root => (menu(root)?.children ?? []).filter(child => child.props?.role === "menuitem");
const labels = root => menuItems(root).map(button => button.text);
const contextmenu = (card, x = 5, y = 6) => card.props.onContextmenu({ button: 2, clientX: x, clientY: y, target: card, preventDefault() {}, stopPropagation() {} });
const menuKey = (key, shiftKey = false) => ({
  key,
  shiftKey,
  defaultPrevented: false,
  preventDefault() { this.defaultPrevented = true; },
  stopPropagation() {},
});

test("mounted Drop Shelf: one-item batch secondary-click uses type-aware Item Actions; ambiguous batches have no menu", async () => {
  const { root, commands, close } = await mount();
  try {
    const [external, multi, text, image] = cards(root);
    assert.ok(external && multi && text && image, "all card classifications render");
    contextmenu(external);
    await nextTick();
    assert.deepEqual(labels(root), ["Show in File Explorer", "Copy Path"]);
    focused = [];
    menuItems(root)[1].props.onClick({ stopPropagation() {} });
    await settle(); await nextTick();
    assert.deepEqual(commands.find(([name]) => name === "perform_item_action"), ["perform_item_action", { itemId: "a1", action: "CopyPath" }]);
    assert.equal(menu(root), null, "an activated Item Action closes the menu");
    assert.equal(focused.at(-1)?.props?.["data-batch-id"], "a", "Copy Path restores focus to the invoking card");
    focused = [];
    const contextKey = menuKey("ContextMenu");
    external.children[0].props.onKeydown(contextKey);
    await nextTick(); await settle(); await nextTick();
    assert.equal(contextKey.defaultPrevented, true);
    assert.deepEqual(labels(root), ["Show in File Explorer", "Copy Path"], "keyboard invocation resolves the same singleton Item Actions");
    assert.ok(focused.at(-1) === menuItems(root)[0], "keyboard invocation focuses the first action");
    focused = [];
    menu(root).props.onKeydown({ key: "Escape", preventDefault() {}, stopPropagation() {} });
    await nextTick();
    assert.equal(menu(root), null);
    assert.equal(focused.at(-1)?.props?.["data-batch-id"], "a", "Escape restores focus to the invoking card");
    contextmenu(text);
    await nextTick();
    assert.deepEqual(labels(root), ["Copy Text"]);
    contextmenu(multi);
    await nextTick();
    assert.equal(menu(root), null, "a multi-item batch has no unambiguous Item Action target");
    contextmenu(image);
    await nextTick();
    assert.equal(menu(root), null, "Managed Image paths remain private");
  } finally { close(); }
});

test("mounted Drop Shelf: keyboard-opened Remove All Selected uses the full Batch Selection and confirms before removal", async () => {
  const { root, commands, close } = await mount();
  try {
    const [first, second] = cards(root);
    first.props.onSelect?.call?.(null);
    // build a two-batch selection through the real pointer path
    for (const [card, ctrlKey] of [[first, false], [second, true]]) {
      const drag = card.children[0];
      const ev = { button: 0, buttons: 1, pointerId: 1, clientX: 0, clientY: 0, currentTarget: drag, preventDefault() {}, ctrlKey, shiftKey: false };
      drag.props.onPointerdown(ev);
      drag.props.onPointerup(ev);
    }
    await nextTick();
    const shiftF10 = menuKey("F10", true);
    second.children[0].props.onKeydown(shiftF10);
    await nextTick();
    assert.equal(shiftF10.defaultPrevented, true);
    assert.deepEqual(labels(root), ["Pin All Selected", "Remove All Selected"]);
    const remove = menuItems(root)[1];
    focused = [];
    remove.props.onClick({ stopPropagation() {} });
    await settle(); await nextTick(); await settle(); await nextTick();
    const dialog = find(root, ".selection-removal-confirmation")[0];
    assert.ok(dialog, "the confirmation dialog must render");
    assert.equal(dialog.open, true, "the dialog must be shown modally");
    assert.match(JSON.stringify(dialog, (k, v) => (k === "parent" ? undefined : v)), /2 batches · 3 items/);
    assert.ok(!commands.some(([name]) => name === "remove_batches"), "nothing is removed before confirmation");
    const cancel = dialog.children[1].children[0];
    assert.equal(cancel.text, "Cancel");
    assert.ok(focused.includes(cancel), "Cancel must be focused initially");
  } finally { close(); }
});
