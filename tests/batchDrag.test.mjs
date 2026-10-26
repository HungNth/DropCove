import assert from "node:assert/strict";
import test from "node:test";
import { setImmediate as settle } from "node:timers/promises";
import { effectScope, shallowRef } from "vue";
import { usePreparedDrag } from "../src/composables/usePreparedDrag.ts";

function fixture(unavailable = 1) {
  const batches = shallowRef([{ id: "a", items: [{ id: "1" }, { id: "2" }] }]);
  const ids = shallowRef(["a"]);
  const busy = shallowRef(false);
  const status = shallowRef("");
  const calls = [];
  const hooks = { refresh: async () => {}, hold: async () => true };
  let copied = 0;
  const preparation = () => ({ token: "prepared", available: 1, unavailable, snapshot: { batches: structuredClone(batches.value) } });
  const handlers = {
    prepare_batch_drag: async () => preparation(),
    start_batch_drag: async () => ({ outcome: "copy", error: null }),
    cancel_batch_drag: async () => {},
  };
  globalThis.window = { __TAURI_INTERNALS__: { async invoke(command) {
    calls.push(command);
    if (!(command in handlers)) throw Error(command);
    return handlers[command]();
  } } };
  const scope = effectScope();
  const drag = scope.run(() => usePreparedDrag("batch", batches, ids, busy, status, () => hooks.refresh(), () => { copied++; ids.value = []; }, active => hooks.hold(active)));
  return {
    batches, ids, busy, status, calls, hooks, handlers, preparation, drag,
    get copied() { return copied; },
    async close() { await drag.cancel(); scope.stop(); delete globalThis.window; },
  };
}

for (const armed of [false, true]) {
  test(`unavailable confirmation ${armed ? "armed" : "direct"} drag completes Copy`, async () => {
    const f = fixture();
    try {
      await f.drag.start(["a"]);
      assert.equal(f.drag.pending.value.unavailable, 1);
      if (armed) f.drag.confirm();
      f.drag.dragConfirmed();
      await settle();
      assert.equal(f.calls.filter(c => c === "start_batch_drag").length, 1);
      assert.equal(f.copied, 1);
      assert.deepEqual(f.ids.value, []);
      assert.equal(f.drag.pending.value, null);
      assert.equal(f.busy.value, false);
    } finally { await f.close(); }
  });
}

test("changing selected identities cancels an armed unavailable preparation", async () => {
  const f = fixture();
  try {
    await f.drag.start(["a"]);
    f.drag.confirm();
    f.ids.value = ["b"];
    f.drag.dragConfirmed();
    await settle();
    assert.equal(f.calls.filter(c => c === "cancel_batch_drag").length, 1);
    assert.equal(f.calls.filter(c => c === "start_batch_drag").length, 0);
  } finally { await f.close(); }
});

test("armed card retry dispatches once without preparing another drag", async () => {
  const f = fixture();
  try {
    await f.drag.start(["a"]);
    f.drag.confirm();
    await f.drag.start(["a"]);
    assert.equal(f.calls.filter(c => c === "prepare_batch_drag").length, 1);
    assert.equal(f.calls.filter(c => c === "start_batch_drag").length, 1);
  } finally { await f.close(); }
});

test("hiding during preparation cancels the late token without starting OLE", async () => {
  const f = fixture(0);
  let finish;
  f.handlers.prepare_batch_drag = () => new Promise(resolve => { finish = resolve; });
  try {
    const pending = f.drag.start(["a"]);
    await settle();
    await f.drag.cancel();
    finish(f.preparation());
    await pending;
    assert.equal(f.calls.filter(c => c === "cancel_batch_drag").length, 1);
    assert.equal(f.calls.filter(c => c === "start_batch_drag").length, 0);
  } finally { await f.close(); }
});

for (const failure of ["refresh", "hold"]) {
  test(`${failure} failure releases native preparation and preserves selection`, async () => {
    const f = fixture();
    if (failure === "refresh") f.hooks.refresh = async () => { throw Error("refresh failed"); };
    else f.hooks.hold = async active => { if (active) throw Error("hold failed"); return true; };
    try {
      await f.drag.start(["a"]);
      assert.equal(f.calls.filter(c => c === "cancel_batch_drag").length, 1);
      assert.equal(f.calls.filter(c => c === "start_batch_drag").length, 0);
      assert.deepEqual(f.ids.value, ["a"]);
      assert.equal(f.drag.pending.value, null);
      assert.equal(f.busy.value, false);
    } finally { await f.close(); }
  });
}

for (const change of ["selection", "shelf"]) {
  test(`${change} change during preparation cannot retarget the returned token`, async () => {
    const f = fixture();
    f.hooks.refresh = async () => {
      if (change === "selection") f.ids.value = [];
      else f.batches.value = [...f.batches.value, { id: "b", items: [{ id: "3" }] }];
    };
    try {
      await f.drag.start(["a"]);
      assert.equal(f.calls.filter(c => c === "cancel_batch_drag").length, 1);
      assert.equal(f.calls.filter(c => c === "start_batch_drag").length, 0);
      assert.equal(f.drag.pending.value, null);
      assert.equal(f.busy.value, false);
    } finally { await f.close(); }
  });
}

test("accepted Copy clears selection even when completion and refresh both fail", async () => {
  const f = fixture(0);
  let refreshes = 0;
  f.handlers.start_batch_drag = async () => ({ outcome: "copy", error: "consumption failed" });
  f.hooks.refresh = async () => { if (++refreshes > 1) throw Error("refresh failed"); };
  try {
    await f.drag.start(["a"]);
    assert.equal(f.copied, 1);
    assert.deepEqual(f.ids.value, []);
    assert.match(f.status.value, /consumption failed/);
    assert.match(f.status.value, /refresh failed/);
    assert.equal(f.busy.value, false);
  } finally { await f.close(); }
});

test("accepted Copy completion warning takes precedence over a refresh status", async () => {
  const f = fixture(0);
  f.handlers.start_batch_drag = async () => ({ outcome: "copy", error: "publish failed" });
  f.hooks.refresh = async () => { f.status.value = "unrelated refresh status"; };
  try {
    await f.drag.start(["a"]);
    assert.deepEqual(f.ids.value, []);
    assert.equal(f.copied, 1);
    assert.equal(f.status.value, "publish failed");
  } finally { await f.close(); }
});

for (const outcome of ["rejected", "canceled"]) {
  test(`${outcome} native outcome retains selection and does not consume Copy state`, async () => {
    const f = fixture(0);
    f.handlers.start_batch_drag = async () => ({ outcome, error: null });
    try {
      await f.drag.start(["a"]);
      assert.deepEqual(f.ids.value, ["a"]);
      assert.equal(f.copied, 0);
      assert.equal(f.busy.value, false);
    } finally { await f.close(); }
  });
}

test("failed Rail hold release aborts preparation before allocating a native token", async () => {
  const f = fixture();
  await f.drag.start(["a"]);
  f.drag.confirm();
  f.hooks.hold = async () => false;
  try {
    await f.drag.start(["other"]);
    assert.equal(f.calls.filter(c => c === "prepare_batch_drag").length, 1);
    assert.deepEqual(f.ids.value, ["a"]);
    assert.equal(f.busy.value, false);
    assert.match(f.status.value, /hold release failed/i);
  } finally { f.hooks.hold = async () => true; await f.close(); }
});

test("native start failure preserves selection and its original error context", async () => {
  const f = fixture(0);
  f.handlers.start_batch_drag = async () => { throw Error("Windows drag failed 0x80004005; Manage Items synchronization failed: unavailable"); };
  try {
    await f.drag.start(["a"]);
    assert.deepEqual(f.ids.value, ["a"]);
    assert.equal(f.copied, 0);
    assert.equal(f.busy.value, false);
    assert.match(f.status.value, /Windows drag failed 0x80004005/);
    assert.match(f.status.value, /synchronization failed/);
  } finally { await f.close(); }
});

test("stale asynchronous cancellation cannot release a newer preparation's Rail hold", async () => {
  const f = fixture();
  let holdActive = false;
  const holdCalls = [];
  f.hooks.hold = async active => { holdActive = active; holdCalls.push(active); return true; };
  let finishFirstCancel;
  const delayCancel = new Promise(resolve => { finishFirstCancel = resolve; });
  let cancelCalls = 0;
  globalThis.window.__TAURI_INTERNALS__.invoke = async command => {
    if (command === "cancel_batch_drag") {
      if (++cancelCalls === 1) await delayCancel;
      return;
    }
    return f.handlers[command]();
  };
  try {
    await f.drag.start(["a"]);
    assert.equal(holdActive, true);
    const staleCancel = f.drag.cancel();
    await f.drag.start(["a"]);
    assert.equal(holdActive, true);
    finishFirstCancel();
    await staleCancel;
    await settle();
    assert.equal(holdActive, true, "older cancellation must not drop a newer pending preparation's hold");
    assert.notEqual(f.drag.pending.value, null);
  } finally { await f.close(); }
});

test("ordinary single cancellation releases an active Rail hold", async () => {
  const f = fixture();
  let holdActive = false;
  f.hooks.hold = async active => { holdActive = active; return true; };
  try {
    await f.drag.start(["a"]);
    assert.equal(holdActive, true);
    await f.drag.cancel();
    assert.equal(holdActive, false, "single cancel must release the active hold");
  } finally { await f.close(); }
});
