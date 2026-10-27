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
    if (url.endsWith("/ShellVisual.vue") || url.endsWith("/ActionIcon.vue") || url.endsWith("/BulkPinning.vue")) {
      return { format: "module", source: "export default { render() { return null; } }", shortCircuit: true };
    }
    if (!url.endsWith(".vue")) return nextLoad(url, context);
    const { descriptor } = parse(readFileSync(new URL(url), "utf8"), { filename: url });
    const source = descriptor.script || descriptor.scriptSetup
      ? compileScript(descriptor, { id: url, inlineTemplate: true }).content
      : `${compileTemplate({ source: descriptor.template.content, filename: url, id: url }).code}\nexport default { render };`;
    return {
      format: "module",
      source: ts.transpileModule(source, {
        compilerOptions: { target: ts.ScriptTarget.ES2020, module: ts.ModuleKind.ESNext },
      }).outputText,
      shortCircuit: true,
    };
  },
});
const { default: ManageItems } = await import("../src/components/shelf/ManageItems.vue");
hooks.deregister();
let focusedNode = null;

function matchesSelector(cur, sel) {
  if (!cur || !cur.props) return false;
  if (sel.includes(",")) return sel.split(",").some(part => matchesSelector(cur, part.trim()));
  if (sel === "button:not(:disabled)") {
    return cur.type === "button" && !cur.props.disabled;
  }
  if (sel.includes("[") && sel.endsWith("]")) {
    const attrMatch = sel.match(/^([^[]*)\[([\w-]+)(?:=(?:"([^"]*)"|'([^']*)'|([\w-]+)))?\]$/);
    if (attrMatch) {
      const [, prefix, attr, v1, v2, v3] = attrMatch;
      const val = v1 ?? v2 ?? v3;
      if (prefix && !matchesSelector(cur, prefix)) return false;
      const actual = cur.dataset?.[attr.slice(5).replace(/-([a-z])/g, (_, l) => l.toUpperCase())] ?? cur.props[attr];
      if (val === undefined) return actual !== undefined;
      return String(actual) === String(val);
    }
  }
  if (sel.startsWith(".")) {
    const cls = sel.slice(1);
    if (cls.includes(".")) {
      const parts = cls.split(".");
      return parts.every(p => cur.props.class?.split(/\s+/).includes(p));
    }
    return Boolean(cur.props.class?.split(/\s+/).includes(cls));
  }
  return cur.type === sel;
}

function node(type) {
  const n = markRaw({
    type,
    props: {},
    children: [],
    parent: null,
    clientWidth: 240,
    clientHeight: 300,
    scrollTop: 0,
    dataset: {},
    getBoundingClientRect() {
      return { x: 0, y: 0, width: 240, height: 54, top: 0, left: 0, right: 240, bottom: 54 };
    },
    setPointerCapture() {},
    releasePointerCapture() {},
    focus() {
      focusedNode = this;
      if (globalThis.document) globalThis.document.activeElement = this;
    },
    addEventListener() {},
    removeEventListener() {},
    contains(child) {
      let cur = child;
      while (cur) {
        if (cur === this) return true;
        cur = cur.parent;
      }
      return false;
    },
    matches(selector) {
      return matchesSelector(this, selector);
    },
    closest(selector) {
      let cur = this;
      while (cur) {
        if (matchesSelector(cur, selector)) return cur;
        cur = cur.parent;
      }
      return null;
    },
    querySelector(selector) {
      if (selector.includes(",")) return selector.split(",").map(part => this.querySelector(part.trim())).find(Boolean) ?? null;
      const parts = selector.trim().split(/\s+/);
      let current = [this];
      for (const part of parts) {
        const next = [];
        for (const el of current) {
          function findFirst(n) {
            if (!n) return null;
            if (n !== el && matchesSelector(n, part)) return n;
            for (const child of n.children || []) {
              const found = findFirst(child);
              if (found) return found;
            }
            return null;
          }
          const f = findFirst(el);
          if (f) {
            next.push(f);
            break;
          }
        }
        if (!next.length) return null;
        current = next;
      }
      return current[0] ?? null;
    },
    querySelectorAll(selector) {
      const parts = selector.trim().split(/\s+/);
      let current = [this];
      for (const part of parts) {
        const next = [];
        for (const el of current) {
          function collect(n) {
            if (!n) return;
            if (n !== el && matchesSelector(n, part)) next.push(n);
            for (const child of n.children || []) collect(child);
          }
          collect(el);
        }
        current = next;
      }
      return current;
    },
    close() { this.open = false; },
    showModal() { this.open = true; },
  });
  return n;
}

const renderer = createRenderer({
  createElement: node,
  createText: () => node("text"),
  createComment: () => node("comment"),
  setText: (target, text) => { target.text = text; },
  setElementText: (target, text) => { target.text = text; },
  patchProp: (target, key, previous, value) => {
    target.props[key] = value;
    if (key.startsWith("data-")) {
      const camel = key.slice(5).replace(/-([a-z])/g, (_, l) => l.toUpperCase());
      target.dataset[camel] = String(value);
    }
  },
  insert(child, parent, anchor) {
    if (child.parent) child.parent.children.splice(child.parent.children.indexOf(child), 1);
    child.parent = parent;
    const index = anchor ? parent.children.indexOf(anchor) : -1;
    if (index < 0) parent.children.push(child);
    else parent.children.splice(index, 0, child);
  },
  remove: child => {
    if (!child.parent) return;
    const idx = child.parent.children.indexOf(child);
    if (idx >= 0) child.parent.children.splice(idx, 1);
  },
  parentNode: target => target.parent,
  nextSibling: () => null,
});

function simulateDragGesture(target, dx = 10, dy = 10) {
  const downEvent = {
    button: 0,
    buttons: 1,
    pointerId: 1,
    clientX: 0,
    clientY: 0,
    currentTarget: target,
    preventDefault() {},
    ctrlKey: false,
    shiftKey: false,
  };
  target.props.onPointerdown?.(downEvent);
  const moveEvent = {
    button: 0,
    buttons: 1,
    pointerId: 1,
    clientX: dx,
    clientY: dy,
    currentTarget: target,
    preventDefault() {},
    ctrlKey: false,
    shiftKey: false,
  };
  target.props.onPointermove?.(moveEvent);
  const upEvent = {
    button: 0,
    buttons: 0,
    pointerId: 1,
    clientX: dx,
    clientY: dy,
    currentTarget: target,
    preventDefault() {},
    ctrlKey: false,
    shiftKey: false,
  };
  target.props.onPointerup?.(upEvent);
}

for (const itemCount of [3, 65]) test(`ManageItems ${itemCount} items preserves selection, right-click boundaries, and Escape ordering`, async () => {
  const events = new Map();
  const callbacks = new Map();
  const commands = [];
  let callbackId = 0;
  let preparedToken = null;
  let activeBatch = {
    id: "batch1",
    createdAt: 0,
    position: 0,
    items: [
      { id: "item1", batchId: "batch1", position: 0, path: "C:\\1.txt", name: "1.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null },
      { id: "item2", batchId: "batch1", position: 1, path: "C:\\private\\2.txt", name: "2.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "Managed", managedContentKind: "Text" },
      { id: "item3", batchId: "batch1", position: 2, path: "C:\\private\\3.png", name: "3.png", isFolder: false, isPinned: false, availability: "Available", ownership: "Managed", managedContentKind: "Image" },
    ],
  };
  for (let index = 3; index < itemCount; index++) {
    activeBatch.items.push({ ...activeBatch.items[0], id: `item${index + 1}`, position: index, path: `C:\\${index + 1}.txt`, name: `${index + 1}.txt` });
  }
  globalThis.CSS = { escape: value => value };

  globalThis.document = {
    addEventListener: (name, listener) => events.set(name, listener),
    removeEventListener: name => events.delete(name),
    querySelector: () => null,
    activeElement: null,
  };
  globalThis.getComputedStyle = () => ({
    paddingLeft: "0",
    paddingRight: "0",
    paddingTop: "0",
    paddingBottom: "0",
    gridTemplateColumns: "1fr",
  });
  globalThis.window = {
    addEventListener() {},
    removeEventListener() {},
    setTimeout: (cb, ms = 0) => setTimeout(cb, ms),
    clearTimeout: (id) => clearTimeout(id),
    __TAURI_INTERNALS__: {
      metadata: { currentWindow: { label: "manage_items" } },
      transformCallback: callback => { callbacks.set(++callbackId, callback); return callbackId; },
      async invoke(command, args) {
        commands.push([command, args]);
        if (command === "plugin:event|listen") return callbackId;
        if (command === "plugin:event|unlisten") return;
        if (command === "get_manage_items") return activeBatch;
        if (command === "get_manage_items_origin") return "main";
        if (command === "get_manage_items_keyboard") return true;
        if (command === "close_manage_items") return;
        if (command === "perform_item_action") return;
        if (command === "prepare_item_drag") {
          preparedToken = "item_token_1";
          return { token: preparedToken, available: 1, unavailable: 1, snapshot: { batches: [activeBatch] } };
        }
        if (command === "cancel_item_drag") {
          preparedToken = null;
          return { batches: [activeBatch] };
        }
        throw Error(`Unexpected invoke command: ${command}`);
      },
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener() {} },
  };

  const root = node("root");
  const app = renderer.createApp(ManageItems);
  app.mount(root);

  try {
    await settle(); await nextTick(); await settle();
    const main = root.children[0];
    assert.equal(main.type, "main");

    // Grid container
    function findGrid(n) {
      if (n.props?.role === "grid") return n;
      for (const c of n.children || []) {
        const found = findGrid(c);
        if (found) return found;
      }
      return null;
    }
    const grid = findGrid(main);
    assert.ok(grid, "item list must expose role='grid'");
    assert.equal(grid.props["aria-multiselectable"], "true");

    // Initial keyboard open focused first item drag region.
    assert.ok(focusedNode, "keyboard open must focus the first item drag region");
    assert.equal(focusedNode.props["aria-label"], "Drag C:\\1.txt to copy");

    const rowFor = id => grid.children.find(child => child.props?.["data-item-id"] === id);
    const menu = () => main.querySelector(".selection-actions-menu");
    const menuLabels = () => menu().children.filter(child => child.props?.role === "menuitem").map(child => child.text);
    const rightClick = async id => {
      const row = rowFor(id);
      const context = new Event("contextmenu", { cancelable: true, bubbles: true });
      Object.defineProperty(context, "button", { value: 2 });
      Object.defineProperty(context, "clientX", { value: 7 });
      Object.defineProperty(context, "clientY", { value: 9 });
      Object.defineProperty(context, "target", { value: row.children[0] });
      row.props.onContextmenu(context);
      await nextTick(); await settle(); await nextTick();
      assert.equal(context.defaultPrevented, true);
    };

    assert.equal(main.querySelectorAll(".more").length, 0, "Manage Items rows must not render the redundant More trigger");
    await rightClick("item1");
    assert.deepEqual(menuLabels(), ["Show in File Explorer", "Copy Path"]);
    focusedNode = null;
    menu().children.filter(child => child.props?.role === "menuitem")[1].props.onClick({ stopPropagation() {} });
    await settle(); await nextTick(); await settle(); await nextTick();
    assert.deepEqual(commands.find(([name]) => name === "perform_item_action"), ["perform_item_action", { itemId: "item1", action: "CopyPath" }]);
    assert.equal(focusedNode?.props?.["aria-label"], "Drag C:\\1.txt to copy", "Copy Path restores focus to the invoking row");
    focusedNode = null;
    const contextKey = { key: "ContextMenu", shiftKey: false, defaultPrevented: false, currentTarget: rowFor("item1").children[0], preventDefault() { this.defaultPrevented = true; }, stopPropagation() {} };
    rowFor("item1").children[0].props.onKeydown(contextKey);
    await nextTick(); await settle(); await nextTick();
    assert.equal(contextKey.defaultPrevented, true);
    assert.deepEqual(menuLabels(), ["Show in File Explorer", "Copy Path"]);
    assert.ok(focusedNode === menu().children.find(child => child.props?.role === "menuitem"), "keyboard invocation focuses the first action");
    focusedNode = null;
    menu().props.onKeydown({ key: "Escape", preventDefault() {}, stopPropagation() {} });
    await nextTick(); await settle(); await nextTick();
    assert.equal(menu(), null);
    assert.equal(focusedNode?.props?.["aria-label"], "Drag C:\\1.txt to copy", "Escape restores focus to the invoking row");
    await rightClick("item2");
    assert.deepEqual(menuLabels(), ["Copy Text"]);
    await rightClick("item3");
    assert.equal(menu(), null, "Managed Image rows expose no internal-path menu");
    grid.props.onPointerup({ target: { closest: () => null } });
    await nextTick();
    rowFor("item1").children[0].focus();

    // Up / Down / Space selection
    const gridKeydown = grid.props.onKeydown;
    assert.ok(gridKeydown);

    // Space on item 1 selects it
    await gridKeydown({ key: " ", target: focusedNode, preventDefault() {}, ctrlKey: false, shiftKey: false });
    await nextTick();

    // Check header summary displays "1 selected · 0/3 pinned"
    const summarySpan = main.children[0].children[0].children[1];
    assert.equal(summarySpan.text, `1 selected · 0/${itemCount} pinned`);

    // ArrowDown moves focus to item 2
    await gridKeydown({ key: "ArrowDown", target: focusedNode, preventDefault() {} });
    await nextTick();

    // Ctrl+Space on item 2 toggles item 2 into selection
    await gridKeydown({ key: " ", target: focusedNode, preventDefault() {}, ctrlKey: true, shiftKey: false });
    await nextTick();
    assert.equal(summarySpan.text, `2 selected · 0/${itemCount} pinned`);

    const selectedIds = () => grid.children.filter(c => c.props?.["aria-selected"] === true).map(c => c.props["data-item-id"]);
    const beforeCommands = commands.length;
    // 0.0.28 supersedes the 0.0.27 suppression-only contract: a secondary-click on a selected row keeps the
    // selection and anchor, still suppresses the WebView menu, and opens the Selection Actions Menu.
    const row = grid.children.find(c => c.props?.["data-item-id"] === "item1");
    const drag = row.children[0];
    const pointer = { button: 2, buttons: 2, pointerId: 2, clientX: 0, clientY: 0, currentTarget: drag, ctrlKey: false, shiftKey: false };
    drag.props.onPointerdown(pointer);
    drag.props.onPointermove({ ...pointer, clientX: 20 });
    const context = new Event("contextmenu", { cancelable: true, bubbles: true });
    Object.defineProperty(context, "button", { value: 2 });
    row.props.onContextmenu?.(context);
    drag.props.onPointerup({ ...pointer, buttons: 0 });
    await nextTick();
    assert.equal(context.defaultPrevented, true, "row right-click must suppress the WebView menu");
    const keyboardContext = new Event("contextmenu", { cancelable: true });
    Object.defineProperty(keyboardContext, "button", { value: 0 });
    row.props.onContextmenu?.(keyboardContext);
    assert.equal(keyboardContext.defaultPrevented, false, "keyboard context-menu invocation remains unchanged");
    assert.deepEqual(selectedIds(), ["item1", "item2"], "right-click on a selected row preserves the selection");
    assert.deepEqual(menuLabels(), ["Pin All Selected", "Remove All Selected"]);
    assert.equal(commands.length, beforeCommands, "right-click must not dispatch a native action or drag");
    await events.get("keydown")({ key: "Escape", stopPropagation() {}, preventDefault() {} });
    await nextTick();
    assert.deepEqual(selectedIds(), ["item1", "item2"], "the first Escape closes the menu and retains selection");
    const shiftF10 = { key: "F10", shiftKey: true, defaultPrevented: false, currentTarget: drag, preventDefault() { this.defaultPrevented = true; }, stopPropagation() {} };
    drag.props.onKeydown(shiftF10);
    await nextTick(); await settle(); await nextTick();
    assert.equal(shiftF10.defaultPrevented, true);
    assert.deepEqual(menuLabels(), ["Pin All Selected", "Remove All Selected"]);
    assert.deepEqual(selectedIds(), ["item1", "item2"], "keyboard menu preserves the complete Item Selection");
    await events.get("keydown")({ key: "Escape", stopPropagation() {}, preventDefault() {} });
    await nextTick();
    const backgroundContext = new Event("contextmenu", { cancelable: true });
    main.props.onContextmenu?.(backgroundContext);
    grid.props.onContextmenu?.(backgroundContext);
    assert.equal(backgroundContext.defaultPrevented, false);
    const item3Drag = grid.children.find(c => c.props?.["data-item-id"] === "item3").children[0];
    await gridKeydown({ key: " ", target: item3Drag, preventDefault() {}, ctrlKey: false, shiftKey: true });
    await nextTick();
    assert.deepEqual(selectedIds(), ["item2", "item3"], "right-click must preserve the item2 range anchor");
    await gridKeydown({ key: " ", target: item3Drag, preventDefault() {}, ctrlKey: true, shiftKey: false });
    await gridKeydown({ key: " ", target: grid.children.find(c => c.props?.["data-item-id"] === "item1").children[0], preventDefault() {}, ctrlKey: true, shiftKey: false });
    await nextTick();

    // Start item drag on item 1 with mixed availability to trigger confirmation
    const item1Row = grid.children.find(c => c.props?.["data-item-id"] === "item1");
    simulateDragGesture(item1Row.children[0], 20, 20);
    await settle(); await nextTick();
    assert.equal(preparedToken, "item_token_1");

    // Escape stage 1: cancels confirmation and prepared token first
    const escape = { key: "Escape", stopPropagation() {}, preventDefault() {} };
    await events.get("keydown")(escape);
    await settle(); await nextTick();
    assert.equal(preparedToken, null, "Escape cancels prepared confirmation first");
    assert.equal(summarySpan.text, `2 selected · 0/${itemCount} pinned`, "selection preserved when confirmation canceled");

    // Escape stage 2: clears non-empty Item Selection
    await events.get("keydown")(escape);
    await nextTick();
    assert.equal(summarySpan.text, `0/${itemCount} pinned`, "Escape clears non-empty Item Selection");

    // Escape stage 3: closes Manage Items
    await events.get("keydown")(escape);
    await settle();
    assert.ok(commands.some(([cmd]) => cmd === "close_manage_items"), "Escape closes Manage Items once selection is empty");
  } finally {
    app.unmount();
    delete globalThis.window;
    delete globalThis.document;
    delete globalThis.getComputedStyle;
  }
});

test("Rail-origin Manage Items context actions remain pointer-only and no-activate", async () => {
  const callbacks = new Map();
  const commands = [];
  let callbackId = 0;
  const activeBatch = {
    id: "batch1",
    createdAt: 0,
    position: 0,
    items: [
      { id: "item1", batchId: "batch1", position: 0, path: "C:\\1.txt", name: "1.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null },
      { id: "item2", batchId: "batch1", position: 1, path: "C:\\2.txt", name: "2.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null },
    ],
  };
  globalThis.document = {
    addEventListener() {}, removeEventListener() {}, querySelector: () => null, activeElement: null,
  };
  globalThis.getComputedStyle = () => ({ paddingLeft: "0", paddingRight: "0", paddingTop: "0", paddingBottom: "0", gridTemplateColumns: "1fr" });
  globalThis.window = {
    addEventListener() {}, removeEventListener() {}, setTimeout, clearTimeout,
    __TAURI_INTERNALS__: {
      metadata: { currentWindow: { label: "manage-items" } },
      transformCallback(callback) { callbacks.set(++callbackId, callback); return callbackId; },
      async invoke(command, args) {
        commands.push([command, args]);
        if (command === "plugin:event|listen") return callbackId;
        if (command === "plugin:event|unlisten") return;
        if (command === "get_manage_items") return activeBatch;
        if (command === "get_manage_items_origin") return "edge-rail";
        if (command === "get_manage_items_keyboard") return false;
        if (command === "perform_item_action" || command === "close_manage_items") return;
        throw Error(`Unexpected invoke command: ${command}`);
      },
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener() {} },
  };
  focusedNode = null;
  const root = node("root");
  const app = renderer.createApp(ManageItems);
  app.mount(root);
  try {
    await settle(); await nextTick(); await settle();
    const main = root.children[0];
    const grid = main.querySelector('[role="grid"]');
    const row = grid.children.find(child => child.props?.["data-item-id"] === "item1");
    const context = new Event("contextmenu", { cancelable: true, bubbles: true });
    Object.defineProperty(context, "button", { value: 2 });
    Object.defineProperty(context, "clientX", { value: 7 });
    Object.defineProperty(context, "clientY", { value: 9 });
    Object.defineProperty(context, "target", { value: row.children[0] });
    row.props.onContextmenu(context);
    await nextTick(); await settle(); await nextTick();
    const menu = main.querySelector(".selection-actions-menu");
    const actions = menu.children.filter(child => child.props?.role === "menuitem");
    assert.deepEqual(actions.map(action => action.text), ["Show in File Explorer", "Copy Path"]);
    assert.equal(focusedNode, null, "Rail-origin menu must not acquire focus");
    actions[1].props.onClick({ stopPropagation() {} });
    await settle(); await nextTick();
    assert.deepEqual(commands.find(([name]) => name === "perform_item_action"), ["perform_item_action", { itemId: "item1", action: "CopyPath" }]);
    assert.equal(focusedNode, null, "Rail-origin Copy Path must not activate or focus the popup");
    const row2 = grid.children.find(child => child.props?.["data-item-id"] === "item2");
    const drag2 = row2.children[0];
    const addPointer = { button: 0, buttons: 1, pointerId: 2, clientX: 0, clientY: 0, ctrlKey: true, shiftKey: false, currentTarget: drag2, preventDefault() {} };
    drag2.props.onPointerdown(addPointer);
    drag2.props.onPointerup({ ...addPointer, buttons: 0 });
    await nextTick();
    const multiContext = new Event("contextmenu", { cancelable: true, bubbles: true });
    Object.defineProperty(multiContext, "button", { value: 2 });
    Object.defineProperty(multiContext, "clientX", { value: 7 });
    Object.defineProperty(multiContext, "clientY", { value: 9 });
    Object.defineProperty(multiContext, "target", { value: row.children[0] });
    row.props.onContextmenu(multiContext);
    await nextTick(); await settle(); await nextTick();
    const multiMenu = main.querySelector(".selection-actions-menu");
    const multiActions = multiMenu.children.filter(child => child.props?.role === "menuitem");
    assert.deepEqual(multiActions.map(action => action.text), ["Pin All Selected", "Remove All Selected"]);
    assert.deepEqual(grid.children.filter(child => child.props?.["aria-selected"] === true).map(child => child.props["data-item-id"]), ["item1", "item2"]);
    assert.equal(focusedNode, null, "Rail-origin multi-selection menu must remain no-activate");
  } finally {
    app.unmount();
    delete globalThis.window;
    delete globalThis.document;
    delete globalThis.getComputedStyle;
  }
});

test("2+ selected items mixed availability: direct confirmation drag, armed re-drag, and stale mutation cancel", async () => {
  const events = new Map();
  const callbacks = new Map();
  const commands = [];
  let callbackId = 0;
  let preparedToken = null;
  let activeBatch = {
    id: "batch1",
    createdAt: 0,
    position: 0,
    items: [
      { id: "item1", batchId: "batch1", position: 0, path: "C:\\1.txt", name: "1.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null },
      { id: "item2", batchId: "batch1", position: 1, path: "C:\\2.txt", name: "2.txt", isFolder: false, isPinned: false, availability: "Unavailable", ownership: "External", managedContentKind: null },
      { id: "item3", batchId: "batch1", position: 2, path: "C:\\3.txt", name: "3.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null },
    ],
  };

  globalThis.document = {
    addEventListener: (name, listener) => events.set(name, listener),
    removeEventListener: name => events.delete(name),
    querySelector: () => null,
    activeElement: null,
  };
  globalThis.getComputedStyle = () => ({
    paddingLeft: "0",
    paddingRight: "0",
    paddingTop: "0",
    paddingBottom: "0",
    gridTemplateColumns: "1fr",
  });
  globalThis.window = {
    addEventListener() {},
    removeEventListener() {},
    setTimeout: (cb, ms = 0) => setTimeout(cb, ms),
    clearTimeout: (id) => clearTimeout(id),
    __TAURI_INTERNALS__: {
      metadata: { currentWindow: { label: "manage_items" } },
      transformCallback: callback => { callbacks.set(++callbackId, callback); return callbackId; },
      async invoke(command, args) {
        commands.push([command, args]);
        if (command === "plugin:event|listen") return callbackId;
        if (command === "plugin:event|unlisten") return;
        if (command === "get_manage_items") return activeBatch;
        if (command === "get_manage_items_origin") return "main";
        if (command === "get_manage_items_keyboard") return true;
        if (command === "close_manage_items") return;
        if (command === "prepare_item_drag") {
          preparedToken = "multi_item_token";
          return { token: preparedToken, available: 1, unavailable: 1, snapshot: { batches: [activeBatch] } };
        }
        if (command === "start_item_drag") {
          return { outcome: "copy", error: null };
        }
        if (command === "cancel_item_drag") {
          preparedToken = null;
          return { batches: [activeBatch] };
        }
        throw Error(`Unexpected invoke command: ${command}`);
      },
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener() {} },
  };

  const root = node("root");
  const app = renderer.createApp(ManageItems);
  app.mount(root);

  try {
    await settle(); await nextTick(); await settle();
    const main = root.children[0];
    function findGrid(n) {
      if (n.props?.role === "grid") return n;
      for (const c of n.children || []) {
        const found = findGrid(c);
        if (found) return found;
      }
      return null;
    }
    const grid = findGrid(main);
    const gridKeydown = grid.props.onKeydown;

    // Select item 1 and item 2 using Space and ArrowDown + Ctrl+Space
    const item1Row = grid.children.find(c => c.props?.["data-item-id"] === "item1");
    const item2Row = grid.children.find(c => c.props?.["data-item-id"] === "item2");
    await gridKeydown({ key: " ", target: item1Row.children[0], preventDefault() {}, ctrlKey: false, shiftKey: false });
    await nextTick();
    await gridKeydown({ key: " ", target: item2Row.children[0], preventDefault() {}, ctrlKey: true, shiftKey: false });
    await nextTick();
    // Drag item 2 (which is part of the 2-item selection) using real down→move→up gesture
    simulateDragGesture(item2Row.children[0], 20, 20);
    await settle(); await nextTick();

    // Verify prepare_item_drag was called with full ordered ["item1", "item2"]
    const prepareCalls = commands.filter(([cmd]) => cmd === "prepare_item_drag");
    assert.equal(prepareCalls.length, 1);
    assert.deepEqual(prepareCalls[0][1], { itemIds: ["item1", "item2"] });
    assert.equal(preparedToken, "multi_item_token");

    // Find confirmation component dialog and drag directly from it
    function findConfirmation(n) {
      if (n.props?.class?.includes("confirmation")) return n;
      for (const c of n.children || []) {
        const found = findConfirmation(c);
        if (found) return found;
      }
      return null;
    }
    const dialog = findConfirmation(main);
    assert.ok(dialog, "BatchDragConfirmation dialog should be rendered");
    const primaryBtn = dialog.children[2].children[0];

    // Direct drag from primary button dispatches the existing full prepared token
    simulateDragGesture(primaryBtn, 30, 30);
    await settle(); await nextTick();

    const startCalls = commands.filter(([cmd]) => cmd === "start_item_drag");
    assert.equal(startCalls.length, 1);
    assert.deepEqual(startCalls[0][1], { token: "multi_item_token" });
    // Successful copy clears selection
    const summarySpan = main.children[0].children[0].children[1];
    assert.equal(summarySpan.text, "0/3 pinned", "successful Copy clears Item Selection");
    // Now test armed re-drag and stale mutation cancellation:
    // Re-select items 1 and 2
    await gridKeydown({ key: " ", target: item1Row.children[0], preventDefault() {}, ctrlKey: false, shiftKey: false });
    await nextTick();
    await gridKeydown({ key: " ", target: item2Row.children[0], preventDefault() {}, ctrlKey: true, shiftKey: false });
    await nextTick();

    // Drag item 1 to trigger preparation
    simulateDragGesture(item1Row.children[0], 20, 20);
    await settle(); await nextTick();
    assert.equal(commands.filter(([cmd]) => cmd === "prepare_item_drag").length, 2);

    // Click confirm on dialog to arm it
    primaryBtn.props.onClick();
    await nextTick();

    // Re-dragging a selected item dispatches the armed token WITHOUT re-preparing
    simulateDragGesture(item1Row.children[0], 20, 20);
    await settle(); await nextTick();
    assert.equal(commands.filter(([cmd]) => cmd === "prepare_item_drag").length, 2, "armed re-drag does not re-prepare");
    assert.equal(commands.filter(([cmd]) => cmd === "start_item_drag").length, 2, "armed re-drag dispatches token");
    assert.equal(summarySpan.text, "0/3 pinned", "re-drag Copy clears Item Selection");

    // Test stale mutation: prepare again, arm, then change selection -> cancels token
    await gridKeydown({ key: " ", target: item1Row.children[0], preventDefault() {}, ctrlKey: false, shiftKey: false });
    await nextTick();
    simulateDragGesture(item1Row.children[0], 20, 20);
    await settle(); await nextTick();
    primaryBtn.props.onClick();
    await nextTick();

    const cancelsBefore = commands.filter(([cmd]) => cmd === "cancel_item_drag").length;
    // Mutate selection by selecting item 3
    const item3Row = grid.children.find(c => c.props?.["data-item-id"] === "item3");
    await gridKeydown({ key: " ", target: item3Row.children[0], preventDefault() {}, ctrlKey: true, shiftKey: false });
    await nextTick();

    const cancelsAfter = commands.filter(([cmd]) => cmd === "cancel_item_drag").length;
    assert.ok(cancelsAfter > cancelsBefore, "mutating selection must cancel the armed prepared token");
    assert.equal(summarySpan.text, "2 selected · 0/3 pinned", "selection retains surviving/added items");
  } finally {
    app.unmount();
    delete globalThis.window;
    delete globalThis.document;
    delete globalThis.getComputedStyle;
  }
});
