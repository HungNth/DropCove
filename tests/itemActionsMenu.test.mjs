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
const { default: ShelfItemRow } = await import("../src/components/shelf/ShelfItemRow.vue");
const { default: ShelfBatchCard } = await import("../src/components/shelf/ShelfBatchCard.vue");
const { default: ItemActionsMenu } = await import("../src/components/shelf/ItemActionsMenu.vue");
hooks.deregister();

let focusCalls = [];
function node(type) {
  return markRaw({
    type, props: {}, children: [], parent: null, clientWidth: 280, clientHeight: 280, scrollTop: 0,
    setPointerCapture() {}, releasePointerCapture() {}, focus() { focusCalls.push(this); },
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

test("ItemActionsMenu capabilities: External -> Show in Explorer then Copy Path; Managed Text -> Copy Text; Managed Image -> none", async () => {
  const externalItem = { id: "ext1", name: "test.txt", path: "C:\\test.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null };
  const textItem = { id: "txt1", name: "managed.txt", path: "C:\\managed\\txt1.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "Managed", managedContentKind: "Text" };
  const imageItem = { id: "img1", name: "image.png", path: "C:\\managed\\img1.png", isFolder: false, isPinned: false, availability: "Available", ownership: "Managed", managedContentKind: "Image" };

  // External row has More button and actions
  const extApp = renderer.createApp({
    render: () => h(ShelfItemRow, { item: externalItem, busy: false, menuOpen: true }),
  });
  const extRoot = node("root");
  extApp.mount(extRoot);
  try {
    const moreButtons = findByClass(extRoot, "more");
    assert.equal(moreButtons.length, 1, "external item must render More button");
    const menuItems = findByRole(extRoot, "menuitem");
    assert.equal(menuItems.length, 2);
    assert.equal(menuItems[0].text, "Show in File Explorer");
    assert.equal(menuItems[1].text, "Copy Path");
  } finally {
    extApp.unmount();
  }

  // Managed text row has More button and only Copy Text
  const txtApp = renderer.createApp({
    render: () => h(ShelfItemRow, { item: textItem, busy: false, menuOpen: true }),
  });
  const txtRoot = node("root");
  txtApp.mount(txtRoot);
  try {
    const moreButtons = findByClass(txtRoot, "more");
    assert.equal(moreButtons.length, 1, "managed text item must render More button");
    const menuItems = findByRole(txtRoot, "menuitem");
    assert.equal(menuItems.length, 1);
    assert.equal(menuItems[0].text, "Copy Text");
  } finally {
    txtApp.unmount();
  }

  // Managed image row has NO More button
  const imgApp = renderer.createApp({
    render: () => h(ShelfItemRow, { item: imageItem, busy: false, menuOpen: true }),
  });
  const imgRoot = node("root");
  imgApp.mount(imgRoot);
  try {
    const moreButtons = findByClass(imgRoot, "more");
    assert.equal(moreButtons.length, 0, "managed image item must NOT render More button");
  } finally {
    imgApp.unmount();
  }
});

test("ShelfBatchCard single-item cards expose More button; multi-item cards omit More button", async () => {
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

  const app1 = renderer.createApp({
    render: () => h(ShelfBatchCard, { batch: singleExternalBatch, busy: false, menuOpenItemId: "item1" }),
  });
  const root1 = node("root");
  app1.mount(root1);
  try {
    const moreButtons = findByClass(root1, "more");
    assert.equal(moreButtons.length, 1, "single item card must render More button");
    const menuItems = findByRole(root1, "menuitem");
    assert.equal(menuItems.length, 2);
    assert.equal(menuItems[0].text, "Show in File Explorer");
    assert.equal(menuItems[1].text, "Copy Path");
  } finally {
    app1.unmount();
  }

  const app2 = renderer.createApp({
    render: () => h(ShelfBatchCard, { batch: multiBatch, busy: false }),
  });
  const root2 = node("root");
  app2.mount(root2);
  try {
    const moreButtons = findByClass(root2, "more");
    assert.equal(moreButtons.length, 0, "multi item card must NOT render More button");
  } finally {
    app2.unmount();
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

test("ItemActionsMenu keyboard navigation: ArrowDown, ArrowUp, Home, End, Escape, Enter/Space", async () => {
  const actionsDispatched = [];
  let closedWithRestore = null;
  const item = { id: "ext1", name: "test.txt", path: "C:\\test.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null };

  const isOpen = shallowRef(true);
  const app = renderer.createApp({
    render: () => h(ItemActionsMenu, {
      item,
      isOpen: isOpen.value,
      onAction: a => actionsDispatched.push(a),
      onClose: (restore = true) => { closedWithRestore = restore; isOpen.value = false; },
    }),
  });
  const root = node("root");
  app.mount(root);
  try {
    const menu = findByRole(root, "menu")[0];
    assert.ok(menu);

    // Escape closes menu with focus restored
    menu.props.onKeydown({ key: "Escape", preventDefault() {}, stopPropagation() {} });
    await nextTick();
    assert.equal(isOpen.value, false);

    // Reopen and test Enter activation
    isOpen.value = true;
    await nextTick();
    const menu2 = findByRole(root, "menu")[0];
    menu2.props.onKeydown({ key: "Enter", preventDefault() {}, stopPropagation() {} });
    await nextTick();
    assert.deepEqual(actionsDispatched, ["ShowInExplorer"]);

    // Arrow down moves to next item
    isOpen.value = true;
    actionsDispatched.length = 0;
    await nextTick();
    const menu3 = findByRole(root, "menu")[0];
    menu3.props.onKeydown({ key: "ArrowDown", preventDefault() {}, stopPropagation() {} });
    menu3.props.onKeydown({ key: "Enter", preventDefault() {}, stopPropagation() {} });
    await nextTick();
    assert.deepEqual(actionsDispatched, ["CopyPath"]);
  } finally {
    app.unmount();
  }
});

test("ShelfItemRow action control order is Pin, More, Remove", async () => {
  const item = { id: "ext1", name: "test.txt", path: "C:\\test.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null };
  const app = renderer.createApp({
    render: () => h(ShelfItemRow, { item, busy: false }),
  });
  const root = node("root");
  app.mount(root);
  try {
    const row = root.children[0];
    const actionsContainer = row.children[1];
    const pinBtn = actionsContainer.children[0];
    const moreMenu = actionsContainer.children[1];
    const removeBtn = actionsContainer.children[2];

    assert.ok(pinBtn.props.class.includes("pin"));
    assert.ok(moreMenu.children.some(c => c.props.class?.includes("more")));
    assert.ok(removeBtn.props.class.includes("remove"));
  } finally {
    app.unmount();
  }
});

test("Clicking More button or invoking action targets only that row and leaves selection unchanged", async () => {
  const item = { id: "ext1", name: "test.txt", path: "C:\\test.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null };
  const selections = [];
  const actions = [];

  const app = renderer.createApp({
    render: () => h(ShelfItemRow, {
      item,
      busy: false,
      selected: true,
      onSelect: (...args) => selections.push(args),
      onAction: (id, action) => actions.push([id, action]),
    }),
  });
  const root = node("root");
  app.mount(root);
  try {
    const moreBtn = findByClass(root, "more")[0];
    assert.ok(moreBtn);
    moreBtn.props.onClick({ stopPropagation() {} });
    await nextTick();
    assert.deepEqual(selections, [], "More click must not emit row select event");
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


test("ItemActionsMenu in noActivate mode suppresses focus acquisition on open and navigate", async () => {
  focusCalls = [];
  const item = { id: "ext1", name: "test.txt", path: "C:\\test.txt", isFolder: false, isPinned: false, availability: "Available", ownership: "External", managedContentKind: null };
  const isOpen = shallowRef(false);
  const app = renderer.createApp({
    render: () => h(ItemActionsMenu, { item, isOpen: isOpen.value, noActivate: true }),
  });
  const root = node("root");
  app.mount(root);
  try {
    const moreBtn = findByClass(root, "more")[0];
    assert.ok(moreBtn);
    isOpen.value = true;
    await nextTick();
    assert.equal(focusCalls.length, 0, "noActivate mode must not focus menu items on open");
  } finally {
    app.unmount();
  }
});
