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
const { default: ShelfBatchCard } = await import("../src/components/shelf/ShelfBatchCard.vue");
const { default: ShelfItemRow } = await import("../src/components/shelf/ShelfItemRow.vue");
hooks.deregister();

function node(type) {
  return markRaw({
    type, props: {}, children: [], parent: null, clientWidth: 280, clientHeight: 280, scrollTop: 0,
    setPointerCapture() {}, releasePointerCapture() {}, focus() {},
    addEventListener() {}, removeEventListener() {}, querySelectorAll: () => [],
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

test("ShelfItemRow tooltips and accessibility labels respect path privacy based on authoritative classification", async () => {
  const externalItem = {
    id: "ext1",
    batchId: "b1",
    position: 0,
    name: "external.txt",
    path: "C:\\Users\\User\\Documents\\external.txt",
    isFolder: false,
    isPinned: false,
    availability: "Available",
    ownership: "External",
    managedContentKind: null,
  };
  const managedTextItem = {
    id: "txt1",
    batchId: "b1",
    position: 1,
    name: "managed.txt",
    path: "C:\\Users\\User\\AppData\\Local\\DropCove\\managed\\txt1.txt",
    isFolder: false,
    isPinned: false,
    availability: "Available",
    ownership: "Managed",
    managedContentKind: "Text",
  };
  const managedImageItem = {
    id: "img1",
    batchId: "b1",
    position: 2,
    name: "managed.png",
    path: "C:\\Users\\User\\AppData\\Local\\DropCove\\managed\\img1.png",
    isFolder: false,
    isPinned: false,
    availability: "Available",
    ownership: "Managed",
    managedContentKind: "Image",
  };

  const app = renderer.createApp({
    render: () => h("div", [
      h(ShelfItemRow, { item: externalItem, busy: false }),
      h(ShelfItemRow, { item: managedTextItem, busy: false }),
      h(ShelfItemRow, { item: managedImageItem, busy: false }),
    ]),
  });
  const root = node("root");
  app.mount(root);
  try {
    const [extRow, txtRow, imgRow] = root.children[0].children;

    // External item retains complete path in aria-label and hoverTitle
    assert.equal(extRow.props["aria-label"], `${externalItem.name}, ${externalItem.path}, ${externalItem.availability}`);
    const extDrag = extRow.children[0];
    assert.equal(extDrag.props.title, externalItem.path);
    assert.equal(extDrag.props["aria-label"], `Drag ${externalItem.path} to copy`);

    // Managed text item MUST NOT expose profile data path in aria-label or hover title
    assert.ok(!txtRow.props["aria-label"].includes(managedTextItem.path), "managed text row aria-label must not include path");
    assert.ok(!txtRow.props["aria-label"].includes("AppData"), "managed text row aria-label must not include profile data");
    assert.equal(txtRow.props["aria-label"], `${managedTextItem.name}, ${managedTextItem.availability}`);
    const txtDrag = txtRow.children[0];
    assert.ok(!txtDrag.props.title?.includes("AppData"), "managed text drag title must not include profile data");
    assert.equal(txtDrag.props.title, managedTextItem.name);
    assert.equal(txtDrag.props["aria-label"], `Drag ${managedTextItem.name} to copy`);

    // Managed image item MUST NOT expose profile data path, should use name + 'DropCove-created image'
    assert.ok(!imgRow.props["aria-label"].includes(managedImageItem.path), "managed image row aria-label must not include path");
    assert.ok(!imgRow.props["aria-label"].includes("AppData"), "managed image row aria-label must not include profile data");
    assert.equal(imgRow.props["aria-label"], `${managedImageItem.name}, DropCove-created image, ${managedImageItem.availability}`);
    const imgDrag = imgRow.children[0];
    assert.ok(!imgDrag.props.title?.includes("AppData"), "managed image drag title must not include profile data");
    assert.equal(imgDrag.props.title, `${managedImageItem.name} (DropCove-created image)`);
    assert.equal(imgDrag.props["aria-label"], `Drag ${managedImageItem.name} to copy`);
  } finally {
    app.unmount();
  }
});
