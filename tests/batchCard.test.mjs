import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { registerHooks } from "node:module";
import { parse, compileScript, compileTemplate } from "vue/compiler-sfc";
import ts from "typescript";
import { createRenderer, h, markRaw, nextTick, shallowRef } from "vue";
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
    if (url.endsWith("/ShellVisual.vue")) return { format: "module", source: "export default { render() { return null; } }", shortCircuit: true };
    if (!url.endsWith(".vue")) return nextLoad(url, context);
    const { descriptor } = parse(readFileSync(new URL(url), "utf8"), { filename: url });
    const source = descriptor.script || descriptor.scriptSetup
      ? compileScript(descriptor, { id: url, inlineTemplate: true }).content
      : `${compileTemplate({ source: descriptor.template.content, filename: url, id: url }).code}\nexport default { render };`;
    return { format: "module", source: ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2020, module: ts.ModuleKind.ESNext } }).outputText, shortCircuit: true };
  },
});
const { default: ShelfBatchCard } = await import("../src/components/shelf/ShelfBatchCard.vue");
const { default: RailHandle } = await import("../src/components/rail/RailHandle.vue");
hooks.deregister();

let focusCalls = 0;
function node(type) {
  return markRaw({
    type, props: {}, children: [], parent: null, clientWidth: 280, clientHeight: 280, scrollTop: 0,
    setPointerCapture() {}, releasePointerCapture() {}, focus() { focusCalls++; },
    addEventListener() {}, removeEventListener() {}, querySelectorAll: () => [],
    getBoundingClientRect: () => ({ x: 10, y: 20, width: 164, height: 48, top: 20, bottom: 68, left: 10, right: 174 }),
    close() { this.open = false; }, showModal() { this.open = true; },
  });
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

for (const count of [1, 2]) {
  test(`${count}-item card preserves selected drag scope until the threshold and exposes row selection`, async () => {
    const selected = shallowRef(true);
    const actions = [];
    const batch = { id: "batch", items: Array.from({ length: count }, (_, index) => ({ id: `item${index}`, name: `item${index}.txt`, path: `C:\\item${index}.txt`, isPinned: false, isFolder: false, ownership: "External", managedContentKind: null, availability: "Available" })) };
    const app = renderer.createApp({ render: () => h(ShelfBatchCard, { batch, busy: false, selected: selected.value, selectionTabindex: 0, role: "row", onSelect: (...args) => actions.push(["select", ...args]), onDragBatch: id => actions.push(["drag", id]) }) });
    const root = node("root");
    app.mount(root);
    try {
      const article = root.children[0];
      const drag = article.children[0];
      const event = (x, modifiers = {}) => ({ button: 0, buttons: 1, pointerId: 1, clientX: x, clientY: 0, currentTarget: drag, preventDefault() {}, ctrlKey: false, shiftKey: false, ...modifiers });
      assert.equal(article.props.role, "row");
      assert.equal(article.props["aria-selected"], true);
      drag.props.onPointerdown(event(0));
      assert.deepEqual(actions, [], "pointer-down must not collapse an existing multi-selection");
      drag.props.onPointermove(event(5));
      assert.deepEqual(actions, []);
      drag.props.onPointermove(event(6));
      drag.props.onPointerup(event(6));
      assert.deepEqual(actions, [["drag", "batch"]], "both one-item and multi-item cards use batch-set drag");
      actions.length = 0;
      drag.props.onPointerdown(event(0, { ctrlKey: true }));
      drag.props.onPointerup(event(0));
      assert.deepEqual(actions, [["select", "batch", { ctrlKey: true, shiftKey: false }]]);
      selected.value = false;
      await nextTick();
      assert.equal(article.props["aria-selected"], false);
      actions.length = 0;
      const actionCell = article.children[1];
      const button = actionCell.children.find(child => child.type === "button");
      button.props.onClick();
      assert.deepEqual(actions, [], "card actions must not implicitly select or drag the batch");
      // Blur clears pointerFocus after previous pointer events
      drag.props.onBlur({});
      await nextTick();
      assert.ok(!drag.props["data-pointer-focus"]);
      drag.props.onPointerdown(event(0, { shiftKey: true }));
      await nextTick();
      assert.equal(drag.props["data-pointer-focus"], true);
      // Modifier keys like Shift or Control must NOT clear pointerFocus
      drag.props.onKeydown({ key: "Shift" });
      await nextTick();
      assert.equal(drag.props["data-pointer-focus"], true);
      // Non-modifier navigation keys like ArrowDown DO clear pointerFocus
      drag.props.onKeydown({ key: "ArrowDown" });
      await nextTick();
      assert.ok(!drag.props["data-pointer-focus"]);
      assert.equal(focusCalls, 0, "pointer interactions on card-drag must rely on native focus without imperative .focus()");
    } finally { app.unmount(); }
  });
}

test("card selection region secondary-click emits a menu request at the pointer and suppresses the browser menu; action controls do not", () => {
  const requests = [];
  const batch = { id: "batch", items: [{ id: "i", name: "a.txt", path: "C:\\a.txt", isPinned: false, isFolder: false, ownership: "External", managedContentKind: null, availability: "Available" }] };
  const app = renderer.createApp({ render: () => h(ShelfBatchCard, { batch, busy: false, onSelectionMenu: (...args) => requests.push(args) }) });
  const root = node("root");
  app.mount(root);
  try {
    const article = root.children[0];
    const context = (target, modifiers = {}) => ({ button: 2, clientX: 40, clientY: 12, ctrlKey: false, shiftKey: false, target, defaultPrevented: false, preventDefault() { this.defaultPrevented = true; }, ...modifiers });
    const drag = article.children[0];
    const event = context(drag, { ctrlKey: true, shiftKey: true });
    article.props.onContextmenu(event);
    assert.equal(event.defaultPrevented, true, "the WebView browser menu must be suppressed");
    assert.deepEqual(requests, [["batch", { x: 40, y: 12 }, false]], "modifiers are ignored and the request carries only identity and anchor");
    const actionButton = article.children[1].children.find(child => child.type === "button");
    const control = context(actionButton);
    actionButton.parent = article.children[1];
    article.props.onContextmenu({ ...control, target: { closest: selector => selector.includes(".card-actions") ? actionButton : null } });
    assert.equal(requests.length, 1, "secondary-click on an action control must not open Selection Actions");
  } finally { app.unmount(); }
});

test("Context Menu key and Shift+F10 on the focused drag region request the menu at the card; other keys do not", () => {
  const requests = [];
  const batch = { id: "batch", items: [{ id: "i", name: "a.txt", path: "C:\\a.txt", isPinned: false, isFolder: false, ownership: "External", managedContentKind: null, availability: "Available" }] };
  const app = renderer.createApp({ render: () => h(ShelfBatchCard, { batch, busy: false, selectionTabindex: 0, onSelectionMenu: (...args) => requests.push(args) }) });
  const root = node("root");
  app.mount(root);
  try {
    const drag = root.children[0].children[0];
    const key = (name, shiftKey = false) => ({ key: name, shiftKey, defaultPrevented: false, preventDefault() { this.defaultPrevented = true; }, stopPropagation() {} });
    const contextKey = key("ContextMenu");
    drag.props.onKeydown(contextKey);
    assert.equal(contextKey.defaultPrevented, true);
    drag.props.onKeydown(key("F10", true));
    drag.props.onKeydown(key("F10", false));
    drag.props.onKeydown(key("ArrowDown"));
    assert.deepEqual(requests, [["batch", { x: 10, y: 68 }, true], ["batch", { x: 10, y: 68 }, true]]);
  } finally { app.unmount(); }
});

test("Rail batch context actions are type-aware, multi-selection-aware, and never acquire focus", async () => {
  const callbacks = new Map();
  const listeners = new Map();
  let nextCallback = 0;
  const item = (id, batchId, ownership = "External", managedContentKind = null) => ({
    id, batchId, name: `${id}.txt`, path: `C:\\${id}.txt`, isFolder: false, isPinned: false,
    ownership, managedContentKind, availability: "Available",
  });
  const railBatches = [
    { id: "external", items: [item("external-item", "external")] },
    { id: "text", items: [item("text-item", "text", "Managed", "Text")] },
    { id: "image", items: [item("image-item", "image", "Managed", "Image")] },
    { id: "multi", items: [item("multi-1", "multi"), item("multi-2", "multi")] },
  ];
  globalThis.document = { title: "" };
  globalThis.getComputedStyle = () => ({ paddingLeft: "0", paddingRight: "0" });
  globalThis.window = {
    addEventListener() {}, removeEventListener() {},
    __TAURI_INTERNALS__: {
      metadata: { currentWindow: { label: "edge-rail" } },
      transformCallback(callback) { callbacks.set(++nextCallback, callback); return nextCallback; },
      async invoke(command, args) {
        if (command === "plugin:event|listen") { listeners.set(args.event, callbacks.get(args.handler)); return nextCallback; }
        if (command === "get_rail") return { edge: "Right", expanded: true, width: 280, height: 280 };
        if (command === "get_shelf") return { batches: railBatches, preferredWidth: 191, height: 180 };
        if (["plugin:event|unlisten", "set_rail_transient_hold", "invalidate_surface_visuals"].includes(command)) return;
        throw Error(command);
      },
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener() {} },
  };
  const root = node("root");
  const app = renderer.createApp(RailHandle);
  app.mount(root);
  const descendants = parent => [parent, ...parent.children.flatMap(descendants)];
  const card = id => descendants(root).find(element => element.props["data-batch-id"] === id);
  const menu = () => descendants(root).find(element => element.props.role === "menu");
  const labels = () => descendants(menu()).filter(element => element.props.role === "menuitem").map(element => element.text);
  const rightClick = async target => {
    target.props.onContextmenu({ button: 2, clientX: 8, clientY: 8, target: target.children[0], preventDefault() {} });
    await nextTick();
  };
  const select = async (target, ctrlKey = false) => {
    const drag = target.children[0];
    const pointer = { button: 0, buttons: 1, pointerId: 1, clientX: 0, clientY: 0, ctrlKey, shiftKey: false, currentTarget: drag, preventDefault() {} };
    drag.props.onPointerdown(pointer); drag.props.onPointerup(pointer);
    await nextTick();
  };
  try {
    await settle(); await nextTick();
    focusCalls = 0;
    await rightClick(card("external"));
    assert.deepEqual(labels(), ["Show in File Explorer", "Copy Path"]);
    assert.equal(focusCalls, 0, "Rail External menu must not acquire focus");
    await rightClick(card("text"));
    assert.deepEqual(labels(), ["Copy Text"]);
    assert.equal(focusCalls, 0, "Rail Managed Text menu must not acquire focus");
    await rightClick(card("image"));
    assert.equal(menu(), undefined, "Rail Managed Image must not expose an internal path action");
    await rightClick(card("multi"));
    assert.equal(menu(), undefined, "one multi-item Rail batch has no unambiguous Item Action target");

    await select(card("external"));
    await select(card("text"), true);
    await rightClick(card("external"));
    assert.deepEqual(labels(), ["Pin All Selected", "Remove All Selected"]);
    assert.equal(focusCalls, 0, "Rail multi-batch menu must remain no-activate");
    assert.deepEqual(railBatches.filter(batch => card(batch.id).props["data-selected"]).map(batch => batch.id), ["external", "text"]);

    listeners.get("shelf-error")({ payload: "Visible Rail failure" });
    await nextTick();
    assert.ok(descendants(root).some(element => element.text === "Visible Rail failure"));
    assert.ok(descendants(root).every(element => element.props.role !== "alert" && element.props["aria-live"] === undefined));
    await listeners.get("rail-changed")({ payload: { edge: "Right", expanded: false, width: 16, height: 280 } });
    await nextTick();
    assert.ok(descendants(root).every(element => element.props.role !== "alert" && element.props["aria-live"] === undefined));
  } finally {
    app.unmount(); await settle();
    delete globalThis.window; delete globalThis.document; delete globalThis.getComputedStyle;
  }
});
