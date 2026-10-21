import assert from "node:assert/strict";
import test from "node:test";
import { setImmediate as settle } from "node:timers/promises";
import { createRenderer, h, nextTick, shallowRef } from "vue";

function deferred() {
  let resolve;
  const promise = new Promise((done) => { resolve = done; });
  return { promise, resolve };
}

function node(type) {
  return { type, props: {}, children: [], parent: null };
}

const renderer = createRenderer({
  createElement: node,
  createText: () => node("text"),
  createComment: () => node("comment"),
  setText: (target, text) => { target.text = text; },
  setElementText: (target, text) => { target.text = text; },
  patchProp: (target, key, _previous, value) => { target.props[key] = value; },
  insert: (child, parent) => { child.parent = parent; parent.children.push(child); },
  remove: (child) => { child.parent.children.splice(child.parent.children.indexOf(child), 1); },
  parentNode: (target) => target.parent,
  nextSibling: () => null,
});

async function flush() {
  await settle();
  await nextTick();
}

test("concurrent Shell visuals retain loaded URLs for unchanged and unrelated generations", async () => {
  const generations = [];
  const requests = [];
  const callbacks = new Map();
  let listener;
  let nextCallback = 0;
  globalThis.window = {
    devicePixelRatio: 1,
    __TAURI_INTERNALS__: {
      metadata: { currentWindow: { label: "edge-rail" } },
      transformCallback(callback) { callbacks.set(++nextCallback, callback); return nextCallback; },
      async invoke(command, args) {
        if (command === "plugin:event|listen") { listener = callbacks.get(args.handler); return 1; }
        if (command === "plugin:event|unlisten" || command === "cancel_shelf_visual") return;
        if (command === "get_surface_generation") {
          const pending = deferred(); generations.push(pending); return pending.promise;
        }
        if (command === "get_shelf_visual") {
          const pending = deferred(); requests.push({ ...pending, ...args }); return pending.promise;
        }
        throw new Error(`Unexpected command: ${command}`);
      },
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener() {} },
  };
  const { useShellVisual } = await import("../src/composables/useShellVisual.ts");
  const Visual = {
    props: ["itemId"],
    setup(props) {
      const { visualUrl } = useShellVisual(() => props.itemId, false);
      return () => h("img", { src: visualUrl.value });
    },
  };
  const root = node("root");
  const firstItem = shallowRef("a");
  const app = renderer.createApp({ render: () => h("div", [h(Visual, { itemId: firstItem.value }), h(Visual, { itemId: "b" })]) });
  app.mount(root);
  const images = () => root.children[0].children.map((image) => image.props.src);
  try {
    await flush();
    generations[0].resolve(0);
    await flush();
    requests.find((request) => request.itemId === "a").resolve(new Uint8Array([1, 2, 3]).buffer);
    await flush();
    const firstUrl = images()[0];
    assert.match(firstUrl, /^blob:/);

    generations[1].resolve(0);
    await flush();
    assert.equal(images()[0], firstUrl, "another mounted visual must not erase an already loaded image");
    requests.find((request) => request.itemId === "b").resolve(new Uint8Array([4, 5, 6]).buffer);
    await flush();
    const loadedUrls = images();
    assert.ok(loadedUrls.every((url) => typeof url === "string" && url.startsWith("blob:")));

    listener({ event: "visual-generation-changed", id: 1, payload: { surface: "shelf", generation: 1 } });
    await flush();
    assert.deepEqual(images(), loadedUrls, "another surface's generation must not blank Edge Rail images");

    listener({ event: "visual-generation-changed", id: 1, payload: { surface: "edgeRail", generation: 1 } });
    await flush();
    assert.deepEqual(images(), loadedUrls, "refresh must keep the last good images while replacement requests are pending");
    for (const request of requests.filter((request) => request.generation === 1)) request.resolve(null);
    await flush();
    assert.deepEqual(images(), loadedUrls, "hidden or cancelled requests must not erase the last good images");

    listener({ event: "visual-generation-changed", id: 1, payload: { surface: "edgeRail", generation: 2 } });
    await flush();
    requests.find((request) => request.generation === 2 && request.itemId === "a").resolve(new Uint8Array([7, 8, 9]).buffer);
    requests.find((request) => request.generation === 2 && request.itemId === "b").resolve(new ArrayBuffer(0));
    await flush();
    assert.notEqual(images()[0], loadedUrls[0], "successful refresh must replace the old image");
    assert.match(images()[0], /^blob:/);
    assert.equal(images()[1], null, "a genuine extraction failure must show fallback rather than a stale image");
    for (const url of loadedUrls) await assert.rejects(fetch(url), "replaced or failed images must revoke stale blob URLs");

    firstItem.value = "c";
    await flush();
    assert.equal(images()[0], null, "changing item identity must not display the previous file's image");
    requests.find((request) => request.generation === 2 && request.itemId === "c").resolve(new Uint8Array([10, 11, 12]).buffer);
    await flush();
    assert.match(images()[0], /^blob:/);
  } finally {
    const remainingUrls = images().filter(Boolean);
    app.unmount();
    await flush();
    for (const url of remainingUrls) await assert.rejects(fetch(url), "unmount must release retained image URLs");
    delete globalThis.window;
  }
});
