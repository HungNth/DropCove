<script setup lang="ts">
import { computed, onMounted, onUnmounted, shallowRef, useTemplateRef } from "vue";
import { useVirtualCollection } from "../../composables/useVirtualCollection";
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
    unlistenRail = await getCurrentWindow().listen<Projection>("rail-changed", async ({ payload }) => {
      if (!disposed) {
        edge.value = payload.edge;
        expanded.value = payload.expanded;
        if (!payload.expanded) {
          await invoke("invalidate_surface_visuals");
        }
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

const railListRef = useTemplateRef<HTMLElement>("railListRef");
const railVirtual = useVirtualCollection(batches, railListRef, 1, 72, 4);
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
      <div
        ref="railListRef"
        class="batch-list"
        role="list"
        :style="railVirtual.enabled.value ? { position: 'relative' } : {}"
      >
        <div v-if="railVirtual.enabled.value" :style="{ height: `${railVirtual.totalHeight.value}px`, width: '100%', pointerEvents: 'none' }" />
        <ShelfBatchCard
          v-for="entry in railVirtual.entries.value"
          :key="entry.item.id"
          :batch="entry.item"
          :busy="busy"
          :data-virtual-index="entry.index"
          :data-virtual-id="entry.item.id"
          :role="'listitem'"
          :aria-setsize="batches.length"
          :aria-posinset="entry.index + 1"
          :style="railVirtual.position(entry.index)"
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
  border: 1px solid var(--dc-border-default);
  border-radius: 8px 0 0 8px;
  background: var(--dc-surface);
  color: var(--dc-text-primary);
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
  background: var(--dc-surface-hover);
  border-color: var(--dc-border-hover);
}
.handle:focus-visible {
  outline: 2px solid var(--dc-focus);
  outline-offset: -2px;
}
.error {
  position: absolute;
  bottom: 8px;
  font-weight: bold;
  color: var(--dc-danger);
}
.expanded-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100vh;
  box-sizing: border-box;
  border: 1px solid var(--dc-border-default);
  border-radius: 8px;
  background: var(--dc-surface);
  color: var(--dc-text-primary);
  font-family: "Segoe UI", sans-serif;
  overflow: hidden;
}
.rail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 36px;
  padding: 0 8px;
  border-bottom: 1px solid var(--dc-border-subtle);
  font-size: 11px;
  font-weight: 600;
  background: var(--dc-surface-subtle);
  color: var(--dc-text-primary);
}
.rail-header-actions {
  align-items: center;
  gap: 4px;
}
.clear-btn {
  padding: 2px 6px;
  border: 1px solid var(--dc-border-default);
  border-radius: 4px;
  background: var(--dc-surface-elevated);
  color: var(--dc-danger);
  font-size: 10px;
  cursor: pointer;
}
.clear-btn:hover:not(:disabled) {
  border-color: var(--dc-danger-border);
  background: var(--dc-danger-surface);
}
.clear-btn:focus-visible {
  outline: 2px solid var(--dc-focus);
}
.clear-btn:disabled {
  opacity: var(--dc-disabled-opacity);
  cursor: default;
}
.open-shelf-btn {
  padding: 2px 6px;
  border: 1px solid var(--dc-border-default);
  border-radius: 4px;
  background: var(--dc-surface-hover);
  color: var(--dc-text-primary);
  font-size: 10px;
  cursor: pointer;
}
.open-shelf-btn:hover {
  background: var(--dc-surface-active);
  border-color: var(--dc-border-hover);
}
.open-shelf-btn:focus-visible {
  outline: 2px solid var(--dc-focus);
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
  border: 1px solid var(--dc-border-subtle);
  border-radius: 6px;
  background: var(--dc-surface-elevated);
  color: var(--dc-text-primary);
}
.batch-list :deep(.card-drag) {
  min-height: 100%;
  height: 100%;
  box-sizing: border-box;
}
.batch-icon {
  font-size: 18px;
  color: var(--dc-accent);
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
  color: var(--dc-text-primary);
}
.batch-sub {
  font-size: 10px;
  color: var(--dc-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.error-banner {
  padding: 4px 8px;
  background: var(--dc-danger-surface);
  color: var(--dc-danger-text);
  border: 1px solid var(--dc-danger-border);
  font-size: 10px;
}

@media (forced-colors: active) {
  .handle, .expanded-container, .batch-list :deep(.card) {
    border-color: ButtonText;
    background: Canvas;
    color: CanvasText;
  }
  .rail-header {
    border-bottom-color: ButtonText;
    background: Canvas;
    color: CanvasText;
  }
  .handle {
    background: ButtonFace;
    color: ButtonText;
  }
  .clear-btn, .open-shelf-btn {
    border-color: ButtonText;
    background: ButtonFace;
    color: ButtonText;
  }
  .clear-btn:focus-visible, .open-shelf-btn:focus-visible, .handle:focus-visible {
    outline: 2px solid Highlight !important;
  }
}
</style>
