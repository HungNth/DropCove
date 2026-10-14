<script setup lang="ts">
import { computed, onMounted, onUnmounted, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { ShelfAnchor, ShelfBatch } from "../../composables/useShelf";
import ShelfBatchCard from "../shelf/ShelfBatchCard.vue";
import BatchDragConfirmation from "../shelf/BatchDragConfirmation.vue";
import ClearTemporaryConfirmation from "../shelf/ClearTemporaryConfirmation.vue";

interface ShelfSnapshot {
  batches: ShelfBatch[];
  preferredWidth: number;
  height: number;
}

interface Projection {
  edge: "Left" | "Right";
  expanded: boolean;
  width: number;
  height: number;
}

interface BatchPreparation {
  token: string;
  available: number;
  unavailable: number;
}

const edge = shallowRef<Projection["edge"]>("Right");
const expanded = shallowRef(false);
const batches = shallowRef<ShelfBatch[]>([]);
const status = shallowRef("");
const busy = shallowRef(false);
const confirmingClearTemporary = shallowRef(false);
const pendingConfirmation = shallowRef<{
  batchId: string;
  token: string;
  available: number;
  unavailable: number;
} | null>(null);
let readyConfirmation: { batchId: string; token: string } | null = null;
let unlistenRail: UnlistenFn | undefined;
let unlistenShelf: UnlistenFn | undefined;
let unlistenTransientsClosed: UnlistenFn | undefined;
let disposed = false;
const hasTemporaryItems = computed(() =>
  batches.value.some((batch) => batch.items.some((item) => !item.isPinned)),
);

async function refreshBatches() {
  try {
    const snapshot = await invoke<ShelfSnapshot>("get_shelf");
    if (!disposed) {
      batches.value = snapshot.batches;
    }
  } catch (error) {
    if (!disposed) status.value = String(error);
  }
}

async function runAction(command: string, args: Record<string, unknown> = {}) {
  if (busy.value) return;
  busy.value = true;
  try {
    await invoke(command, args);
    await refreshBatches();
    if (!disposed) status.value = "";
  } catch (error) {
    if (!disposed) status.value = String(error);
  } finally {
    if (!disposed) busy.value = false;
  }
}

async function onPointerEnter() {
  try {
    await invoke("on_rail_pointer_enter");
  } catch (error) {
    if (!disposed) status.value = String(error);
  }
}

async function onPointerLeave() {
  try {
    await invoke("on_rail_pointer_leave");
  } catch (error) {
    if (!disposed) status.value = String(error);
  }
}

async function setRailTransientHold(active: boolean, reason: "clear" | "batch-drag"): Promise<boolean> {
  try {
    await invoke("set_rail_transient_hold", { active, reason });
    return true;
  } catch (error) {
    if (!disposed) status.value = String(error);
    return false;
  }
}

async function requestClearTemporary() {
  if (busy.value || pendingConfirmation.value || !hasTemporaryItems.value) return;
  const held = await setRailTransientHold(true, "clear");
  if (!held) return;
  confirmingClearTemporary.value = true;
}

async function cancelClearTemporary() {
  confirmingClearTemporary.value = false;
  await setRailTransientHold(false, "clear");
}

async function confirmClearTemporary() {
  if (!confirmingClearTemporary.value) return;
  confirmingClearTemporary.value = false;
  await setRailTransientHold(false, "clear");
  await runAction("clear_temporary");
}

async function dispatchBatchDrag(token: string) {
  pendingConfirmation.value = null;
  readyConfirmation = null;
  await setRailTransientHold(false, "batch-drag");
  await runAction("start_batch_drag", { token });
}

async function cancelPreparedBatch() {
  const pending = pendingConfirmation.value ?? readyConfirmation;
  pendingConfirmation.value = null;
  readyConfirmation = null;
  await setRailTransientHold(false, "batch-drag");
  if (pending) await runAction("cancel_batch_drag", { token: pending.token });
}

async function startBatchDrag(batchId: string) {
  if (busy.value || pendingConfirmation.value) return;
  if (readyConfirmation?.batchId === batchId) {
    await dispatchBatchDrag(readyConfirmation.token);
    return;
  }
  if (readyConfirmation) await cancelPreparedBatch();
  busy.value = true;
  try {
    const preparation = await invoke<BatchPreparation>("prepare_batch_drag", { batchId });
    await refreshBatches();
    if (preparation.available === 0) {
      readyConfirmation = null;
      pendingConfirmation.value = null;
      return;
    }
    if (preparation.unavailable > 0) {
      const held = await setRailTransientHold(true, "batch-drag");
      if (!held) return;
      pendingConfirmation.value = {
        batchId,
        token: preparation.token,
        available: preparation.available,
        unavailable: preparation.unavailable,
      };
      return;
    }
    await invoke("start_batch_drag", { token: preparation.token });
    await refreshBatches();
  } catch (error) {
    if (!disposed) status.value = String(error);
  } finally {
    if (!disposed) busy.value = false;
  }
}

function confirmBatchDrag() {
  const pending = pendingConfirmation.value;
  if (!pending) return;
  readyConfirmation = { batchId: pending.batchId, token: pending.token };
  pendingConfirmation.value = null;
}

function dragConfirmedBatch() {
  const pending = pendingConfirmation.value;
  if (pending) void dispatchBatchDrag(pending.token);
}

async function openManageItems(batchId: string, anchor: ShelfAnchor, keyboard = false) {
  if (busy.value) return;
  try {
    await invoke("open_manage_items", { batchId, anchor, keyboard });
  } catch (error) {
    if (!disposed) status.value = String(error);
  }
}

async function openShelf() {
  try {
    await invoke("open_shelf_from_rail");
  } catch (error) {
    status.value = String(error);
  }
}

onMounted(async () => {
  document.title = "DropCove Edge Rail";
  try {
    unlistenRail = await getCurrentWindow().listen<Projection>("rail-changed", ({ payload }) => {
      if (!disposed) {
        edge.value = payload.edge;
        expanded.value = payload.expanded;
      }
    });
    unlistenShelf = await getCurrentWindow().listen("shelf-changed", async () => {
      if (!disposed) await refreshBatches();
    });
    unlistenTransientsClosed = await getCurrentWindow().listen("rail-transients-closed", async () => {
      if (disposed) return;
      confirmingClearTemporary.value = false;
      await cancelPreparedBatch();
    });
    const projection = await invoke<Projection>("get_rail");
    if (!disposed) {
      edge.value = projection.edge;
      expanded.value = projection.expanded;
    }
    await refreshBatches();
  } catch (error) {
    if (!disposed) status.value = String(error);
  }
});

onUnmounted(() => {
  disposed = true;
  unlistenRail?.();
  unlistenShelf?.();
  unlistenTransientsClosed?.();
});
</script>

<template>
  <div
    class="rail-surface"
    :class="{ left: edge === 'Left', expanded }"
    @pointerenter="onPointerEnter"
    @pointerleave="onPointerLeave"
  >
    <button
      v-if="!expanded"
      class="handle"
      :class="{ left: edge === 'Left' }"
      type="button"
      aria-label="Open Shelf"
      :title="status || 'Open Shelf'"
      @click="openShelf"
    >
      <span class="label">Open Shelf</span>
      <span v-if="status" class="error" role="alert" aria-live="assertive">!</span>
    </button>
    <section
      v-else
      class="expanded-container"
      role="region"
      aria-label="Expanded Edge Rail"
      :aria-busy="busy"
    >
      <header class="rail-header">
        <span class="rail-header-title">Retained Batches ({{ batches.length }})</span>
        <div class="rail-header-actions">
          <button
            v-if="hasTemporaryItems"
            class="clear-btn"
            type="button"
            :disabled="busy || pendingConfirmation !== null"
            @click="requestClearTemporary"
          >Clear Temporary Items</button>
          <button class="open-shelf-btn" type="button" @click="openShelf">Open Shelf</button>
        </div>
      </header>
      <div class="batch-list" role="list">
        <ShelfBatchCard
          v-for="batch in batches"
          :key="batch.id"
          :batch="batch"
          :busy="busy"
          @remove="(id) => runAction('remove_batch', { batchId: id })"
          @pin-item="(id, pinned) => runAction('set_item_pinned', { itemId: id, pinned })"
          @pin-batch="(id, pinned) => runAction('set_batch_pinned', { batchId: id, pinned })"
          @manage="openManageItems"
          @drag-item="(id) => runAction('start_item_drag', { itemId: id })"
          @drag-batch="startBatchDrag"
        />
      </div>
      <BatchDragConfirmation
        :pending="pendingConfirmation"
        @confirm="confirmBatchDrag"
        @cancel="cancelPreparedBatch"
        @drag="dragConfirmedBatch"
      />
      <ClearTemporaryConfirmation
        :open="confirmingClearTemporary"
        :focus-on-open="false"
        @confirm="confirmClearTemporary"
        @cancel="cancelClearTemporary"
      />
      <span v-if="status" class="error-banner" role="alert" aria-live="assertive">{{ status }}</span>
    </section>
  </div>
</template>

<style scoped>
.rail-surface {
  width: 100%;
  height: 100vh;
  box-sizing: border-box;
  overflow: hidden;
  user-select: none;
}
.handle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100vh;
  box-sizing: border-box;
  padding: 0;
  border: 1px solid #46536a;
  border-radius: 8px 0 0 8px;
  background: #141c2b;
  color: #edf2fc;
  font-family: "Segoe UI", sans-serif;
  cursor: pointer;
}
.handle.left {
  border-radius: 0 8px 8px 0;
}
.label {
  writing-mode: vertical-rl;
  font-size: 11px;
  line-height: 14px;
}
.handle:hover {
  background: #202b44;
  border-color: #a5b4fc;
}
.handle:focus-visible {
  outline: 1px solid #a5b4fc;
  outline-offset: -2px;
}
.error {
  position: absolute;
  bottom: 8px;
  font-weight: bold;
}
.expanded-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100vh;
  box-sizing: border-box;
  border: 1px solid #46536a;
  border-radius: 8px;
  background: #141c2b;
  color: #edf2fc;
  font-family: "Segoe UI", sans-serif;
  overflow: hidden;
}
.rail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 36px;
  padding: 0 8px;
  border-bottom: 1px solid #2d3748;
  font-size: 11px;
  font-weight: 600;
  background: #1a2333;
}
.rail-header-actions {
  align-items: center;
  gap: 4px;
}
.clear-btn {
  padding: 2px 6px;
  border: 1px solid #475569;
  border-radius: 4px;
  background: #1b2438;
  color: #f87171;
  font-size: 10px;
  cursor: pointer;
}
.clear-btn:hover:not(:disabled) {
  border-color: #f87171;
  background: #2a1b24;
}
.clear-btn:disabled {
  opacity: .45;
  cursor: default;
}
.open-shelf-btn {
  padding: 2px 6px;
  border: 1px solid #46536a;
  border-radius: 4px;
  background: #25334d;
  color: #edf2fc;
  font-size: 10px;
  cursor: pointer;
}
.open-shelf-btn:hover {
  background: #3b4d71;
}
.batch-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 4px 6px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}
.batch-list :deep(.card) {
  height: 64px;
  flex: 0 0 64px;
  box-sizing: border-box;
  border: 1px solid #2d3748;
  border-radius: 6px;
  background: #1e293b;
}
.batch-list :deep(.card-drag) {
  min-height: 100%;
  height: 100%;
  box-sizing: border-box;
}
.batch-icon {
  font-size: 18px;
  color: #94a3b8;
}
.batch-summary {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  gap: 2px;
}
.batch-name {
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.batch-sub {
  font-size: 10px;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.error-banner {
  padding: 4px 8px;
  background: #7f1d1d;
  color: #fca5a5;
  font-size: 10px;
}
@media (prefers-color-scheme: light) {
  .handle {
    background: #edf2fc;
    color: #141c2b;
  }
  .handle:hover {
    background: #dce5f6;
  }
  .expanded-container {
    background: #f8fafc;
    color: #0f172a;
    border-color: #cbd5e1;
  }
  .rail-header {
    background: #e2e8f0;
    border-color: #cbd5e1;
  }
  .open-shelf-btn {
    background: #cbd5e1;
    color: #0f172a;
    border-color: #94a3b8;
  }
  .batch-row {
    background: #ffffff;
    border-color: #e2e8f0;
  }
  .batch-sub {
    color: #64748b;
  }
}
@media (forced-colors: active) {
  .handle, .expanded-container, .batch-row {
    border-color: ButtonText;
    background: ButtonFace;
    color: ButtonText;
  }
}
</style>
