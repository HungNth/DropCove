import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { registerHooks } from "node:module";
import { parse, compileScript, compileTemplate } from "vue/compiler-sfc";
import ts from "typescript";
import { createRenderer, h, markRaw, nextTick, shallowRef } from "vue";

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
const { default: ShelfItemRow } = await import("../src/components/shelf/ShelfItemRow.vue");
const { default: ShelfBatchCard } = await import("../src/components/shelf/ShelfBatchCard.vue");
hooks.deregister();

function node(type) {
  return markRaw({
    type, props: {}, children: [], parent: null, clientWidth: 280, clientHeight: 280, scrollTop: 0,
    setPointerCapture() {}, releasePointerCapture() {}, focus() {},
    addEventListener() {}, removeEventListener() {}, querySelectorAll: () => [],
    close() { this.open = false; }, showModal() { this.open = true; },
    getBoundingClientRect: () => ({ x: 0, y: 0, width: 100, height: 24, top: 0, bottom: 24, left: 0, right: 100 }),
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

function findByRole(root, role) {
  const matches = [];
  function walk(n) {
    if (n.props?.role === role) matches.push(n);
    for (const c of n.children || []) walk(c);
  }
  walk(root);
  return matches;
}

function findByClass(root, className) {
  const matches = [];
  function walk(n) {
    if (typeof n.props?.class === "string" && n.props.class.split(/\s+/).includes(className)) matches.push(n);
    for (const c of n.children || []) walk(c);
  }
  walk(root);
  return matches;
}

test("ShelfItemRow omits More for every authoritative item classification", async () => {
  const items = [
    { id: "ext1", name: "test.txt", path: "C:\\test.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null },
    { id: "txt1", name: "managed.txt", path: "C:\\managed\\txt1.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "Managed", managedContentKind: "Text" },
    { id: "img1", name: "image.png", path: "C:\\managed\\img1.png", isFolder: false, isPinned: false, availability: "Available", ownership: "Managed", managedContentKind: "Image" },
  ];
  for (const item of items) {
    const app = renderer.createApp({ render: () => h(ShelfItemRow, { item, busy: false }) });
    const root = node("root");
    app.mount(root);
    try {
      assert.equal(findByClass(root, "more").length, 0, "Manage Items rows must not render More");
      assert.equal(findByRole(root, "menuitem").length, 0, "row Item Actions open through secondary-click");
    } finally {
      app.unmount();
    }
  }
});

test("ShelfBatchCard omits More for single-item and multi-item batches", async () => {
  const singleExternalBatch = {
    id: "b1",
    items: [{ id: "item1", name: "a.txt", path: "C:\\a.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null }],
  };
  const multiBatch = {
    id: "b2",
    items: [
      { id: "item1", name: "a.txt", path: "C:\\a.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null },
      { id: "item2", name: "b.txt", path: "C:\\b.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null },
    ],
  };

  for (const batch of [singleExternalBatch, multiBatch]) {
    const app = renderer.createApp({ render: () => h(ShelfBatchCard, { batch, busy: false }) });
    const root = node("root");
    app.mount(root);
    try {
      assert.equal(findByClass(root, "more").length, 0, "batch cards must not render the redundant More trigger");
      assert.equal(findByRole(root, "menuitem").length, 0, "batch Item Actions open only through secondary-click");
    } finally {
      app.unmount();
    }
  }
});

test("Managed Image single-item cards retain only item Pin and Remove, never batch management controls", () => {
  const batch = { id: "image-batch", items: [{ id: "image", name: "image.png", path: "C:\\private\\image.png", isFolder: false, isPinned: false, availability: "Available", ownership: "Managed", managedContentKind: "Image" }] };
  const app = renderer.createApp({ render: () => h(ShelfBatchCard, { batch, busy: false }) });
  const root = node("root");
  app.mount(root);
  try {
    assert.equal(findByClass(root, "more").length, 0);
    assert.equal(findByClass(root, "manage").length, 0);
    assert.equal(findByClass(root, "bulk-pin").length, 0);
    assert.equal(findByClass(root, "pin")[0].props["aria-pressed"], false);
    assert.equal(findByClass(root, "remove").length, 1);
  } finally { app.unmount(); }
});


test("ShelfItemRow action control order is Pin, Remove", async () => {
  const item = { id: "ext1", name: "test.txt", path: "C:\\test.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null };
  const app = renderer.createApp({ render: () => h(ShelfItemRow, { item, busy: false }) });
  const root = node("root");
  app.mount(root);
  try {
    const actions = root.children[0].children[1].children;
    assert.equal(actions.length, 2);
    assert.ok(actions[0].props.class.includes("pin"));
    assert.ok(actions[1].props.class.includes("remove"));
  } finally {
    app.unmount();
  }
});


for (const [Component, dragClass] of [[ShelfItemRow, "item-drag"], [ShelfBatchCard, "card-drag"]]) {
  test(`${Component.__name} discards a loaded text preview when the target item changes`, async () => {
    const previousWindow = globalThis.window;
    globalThis.window = {
      addEventListener() {}, removeEventListener() {},
      __TAURI_INTERNALS__: {
        async invoke(command, { itemId }) {
          assert.equal(command, "get_managed_text");
          return itemId === "first" ? "First retained content" : "Second retained content";
        },
      },
    };
    const item = shallowRef({ id: "first", name: "first.txt", path: "C:\\private\\first.txt", ownership: "Managed", managedContentKind: "Text", availability: "Available", isPinned: false, isFolder: false });
    const app = renderer.createApp({ render: () => h(Component, Component === ShelfItemRow
      ? { item: item.value, busy: false }
      : { batch: { id: "batch", items: [item.value] }, busy: false }) });
    const root = node("root");
    app.mount(root);
    try {
      const drag = findByClass(root, dragClass)[0];
      await drag.props.onPointerenter();
      await nextTick();
      assert.equal(drag.props.title, "First retained content");
      item.value = { ...item.value, id: "second", name: "second.txt", path: "C:\\private\\second.txt" };
      await nextTick();
      assert.equal(drag.props.title, "second.txt", "a new item must not display the prior item's loaded content");
      await drag.props.onPointerenter();
      await nextTick();
      assert.equal(drag.props.title, "Second retained content");
    } finally {
      app.unmount();
      if (previousWindow === undefined) delete globalThis.window;
      else globalThis.window = previousWindow;
    }
  });
}


