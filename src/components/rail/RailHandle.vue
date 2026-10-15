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
import ActionIcon from "../shelf/ActionIcon.vue";

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
const railVirtual = useVirtualCollection(batches, railListRef, 1, 48, 4);
</script>

<template>
  <div
    class="rail-surface size-full box-border overflow-hidden select-none transition-[opacity,transform] duration-[140ms,140ms] ease-[cubic-bezier(0.16,1,0.3,1)]"
    :class="{ left: edge === 'Left', expanded }"
    @pointerenter="onPointerEnter"
    @pointerleave="onPointerLeave"
  >
    <button
      v-if="!expanded"
      class="handle flex items-center justify-center size-full box-border p-0 border border-dc-border-default bg-dc-surface text-dc-text-primary font-['Segoe_UI',sans-serif] cursor-pointer hover:bg-dc-surface-hover hover:border-dc-border-hover focus-visible:outline-2 focus-visible:outline-dc-focus focus-visible:-outline-offset-2 forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:focus-visible:outline-[Highlight]"
      :class="edge === 'Left' ? 'rounded-r-[8px] rounded-l-none' : 'rounded-l-[8px] rounded-r-none'"
      type="button"
      aria-label="Open Shelf"
      :title="status || 'Open Shelf'"
      @click="openShelf"
    >
      <span class="label [writing-mode:vertical-rl] text-[11px] leading-[14px]">Open Shelf</span>
      <span v-if="status" class="error absolute bottom-[8px] font-bold text-dc-danger" role="alert" aria-live="assertive">!</span>
    </button>
    <section
      v-else
      class="expanded-container flex flex-col size-full box-border border border-dc-border-default bg-dc-surface text-dc-text-primary font-['Segoe_UI',sans-serif] overflow-hidden forced-colors:border-[ButtonText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
      :class="edge === 'Left' ? 'rounded-r-[8px] rounded-l-none' : 'rounded-l-[8px] rounded-r-none'"
      role="region"
      aria-label="Expanded Edge Rail"
      :aria-busy="busy"
    >
      <header
        class="rail-header flex flex-none items-start h-[28px] box-border pt-[4px] pb-0 px-[6px] forced-colors:border-b-[ButtonText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
        :class="edge === 'Left' ? 'justify-end' : 'justify-start'"
      >
        <button
          class="open-shelf-btn inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-text-secondary cursor-pointer [&:hover:not(:disabled)]:border-dc-border-hover [&:hover:not(:disabled)]:bg-dc-surface-hover [&:hover:not(:disabled)]:text-dc-text-primary aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
          type="button"
          aria-label="Open Shelf"
          title="Open Shelf"
          @click="openShelf"
        ><ActionIcon :name="edge === 'Left' ? 'right' : 'left'" /></button>
      </header>
      <div class="flex flex-col flex-1 min-h-0 pt-[8px]">
        <div
          ref="railListRef"
          class="batch-list flex flex-col gap-[4px] pt-0 px-[6px] pb-[8px] overflow-y-auto flex-1 min-h-0 [&_.card]:h-[48px] [&_.card]:flex-[0_0_48px] [&_.card]:box-border [&_.card]:border [&_.card]:border-dc-border-subtle [&_.card]:rounded-[6px] [&_.card]:bg-dc-surface-elevated [&_.card]:text-dc-text-primary forced-colors:[&_.card]:border-[ButtonText] forced-colors:[&_.card]:bg-[Canvas] forced-colors:[&_.card]:text-[CanvasText] [&_.card-drag]:min-h-full [&_.card-drag]:h-full [&_.card-drag]:box-border"
          :style="railVirtual.enabled.value ? { position: 'relative' } : {}"
        >
        <div v-if="railVirtual.enabled.value" :style="{ height: `${railVirtual.totalHeight.value}px`, flex: 'none', width: '100%', pointerEvents: 'none' }" />
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
      </div>
      <footer class="rail-footer flex flex-none [align-items:end] min-h-[24px] pt-0 px-[6px] pb-[4px]">
        <button
          v-if="hasTemporaryItems"
          class="clear-btn inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-danger cursor-pointer [&:hover:not(:disabled)]:border-dc-border-hover [&:hover:not(:disabled)]:bg-dc-surface-hover [&:hover:not(:disabled)]:text-dc-text-primary aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:border-[ButtonText] forced-colors:text-dc-danger forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
          type="button"
          aria-label="Clear Temporary Items"
          title="Clear Temporary Items; pinned references remain and source files are not deleted"
          :disabled="busy || pendingConfirmation !== null"
          @click="requestClearTemporary"
        ><ActionIcon name="trash" /></button>
      </footer>
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
      <span v-if="status" class="error-banner py-[4px] px-[8px] bg-dc-danger-surface text-dc-danger-text border border-dc-danger-border text-[10px]" role="alert" aria-live="assertive">{{ status }}</span>
    </section>
  </div>
</template>
