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
    if (url.endsWith("/ShellVisual.vue") || url.endsWith("/ActionIcon.vue")) {
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
const { default: ShelfItemRow } = await import("../src/components/shelf/ShelfItemRow.vue");
hooks.deregister();

let focusCalls = 0;
function node(type) {
  return markRaw({
    type, props: {}, children: [], parent: null, clientWidth: 240, clientHeight: 54, scrollTop: 0,
    setPointerCapture() {}, releasePointerCapture() {}, focus() { focusCalls++; },
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

test("ShelfItemRow preserves selected drag scope until threshold and exposes row selection", async () => {
  const selected = shallowRef(true);
  const actions = [];
  const item = {
    id: "item1",
    batchId: "batch1",
    position: 0,
    name: "item1.txt",
    path: "C:\\item1.txt",
    isFolder: false,
    isPinned: false,
    availability: "Available",
  };
  const app = renderer.createApp({
    render: () =>
      h(ShelfItemRow, {
        item,
        busy: false,
        selected: selected.value,
        selectionTabindex: 0,
        role: "row",
        onSelect: (...args) => actions.push(["select", ...args]),
        onDrag: id => actions.push(["drag", id]),
        onPin: (...args) => actions.push(["pin", ...args]),
        onRemove: id => actions.push(["remove", id]),
      }),
  });
  const root = node("root");
  app.mount(root);
  try {
    const row = root.children[0];
    const drag = row.children[0];
    const event = (x, modifiers = {}) => ({
      button: 0,
      buttons: 1,
      pointerId: 1,
      clientX: x,
      clientY: 0,
      currentTarget: drag,
      preventDefault() {},
      ctrlKey: false,
      shiftKey: false,
      ...modifiers,
    });
    assert.equal(row.props.role, "row");
    assert.equal(row.props["aria-selected"], true);
    assert.equal(row.props["data-selected"], true);

    // Pointer down does not select or drag yet
    drag.props.onPointerdown(event(0));
    assert.deepEqual(actions, [], "pointer-down must not prematurely mutate selection");

    // Movement below threshold does not drag
    drag.props.onPointermove(event(5));
    assert.deepEqual(actions, []);

    // Movement crossing threshold triggers drag
    drag.props.onPointermove(event(6));
    drag.props.onPointerup(event(6));
    assert.deepEqual(actions, [["drag", "item1"]]);

    // Pointer up below threshold emits selection
    actions.length = 0;
    drag.props.onPointerdown(event(0, { ctrlKey: true }));
    drag.props.onPointerup(event(0));
    assert.deepEqual(actions, [["select", "item1", { ctrlKey: true, shiftKey: false }]]);

    // Updating selected changes aria-selected and data-selected
    selected.value = false;
    await nextTick();
    assert.equal(row.props["aria-selected"], false);
    assert.equal(row.props["data-selected"], undefined);

    // Row action buttons (pin, remove) do not select or drag
    actions.length = 0;
    const actionContainer = row.children[1];
    const pinBtn = actionContainer.children[0];
    pinBtn.props.onClick();
    assert.deepEqual(actions, [["pin", "item1", true]], "pin button must not trigger row selection or drag");

    actions.length = 0;
    const removeBtn = actionContainer.children[1];
    removeBtn.props.onClick();
    assert.deepEqual(actions, [["remove", "item1"]], "remove button must not trigger row selection or drag");

    // Roving tabindex and pointer focus behavior
    assert.equal(drag.props.tabindex, 0);
    assert.equal(drag.props.role, "gridcell");

    drag.props.onBlur({});
    await nextTick();
    assert.ok(!drag.props["data-pointer-focus"]);

    drag.props.onPointerdown(event(0, { shiftKey: true }));
    await nextTick();
    assert.equal(drag.props["data-pointer-focus"], true);

    // Modifier keys do not clear pointer-focus
    drag.props.onKeydown({ key: "Shift" });
    await nextTick();
    assert.equal(drag.props["data-pointer-focus"], true);

    // Navigation keys clear pointer-focus
    drag.props.onKeydown({ key: "ArrowDown" });
    await nextTick();
    assert.ok(!drag.props["data-pointer-focus"]);
    assert.equal(focusCalls, 0, "pointer interactions must rely on native focus without imperative .focus()");
  } finally {
    app.unmount();
  }
});
