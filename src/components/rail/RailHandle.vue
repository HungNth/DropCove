<script setup lang="ts">
import { computed, onMounted, onUnmounted, shallowRef, useTemplateRef } from "vue";
import { useVirtualCollection } from "../../composables/useVirtualCollection";
import { useBatchSelection } from "../../composables/useBatchSelection";
import { useBatchDrag } from "../../composables/useBatchDrag";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { ShelfAnchor, ShelfBatch } from "../../composables/useShelf";
import ShelfBatchCard from "../shelf/ShelfBatchCard.vue";
import DropPlaceholder from "../shelf/DropPlaceholder.vue";
import BatchDragConfirmation from "../shelf/BatchDragConfirmation.vue";
import ClearTemporaryConfirmation from "../shelf/ClearTemporaryConfirmation.vue";
import ActionIcon from "../shelf/ActionIcon.vue";

interface ShelfSnapshot {
  batches: ShelfBatch[];
  preferredWidth: number;
  height: number;
  warning?: string | null;
}

interface Projection {
  edge: "Left" | "Right";
  expanded: boolean;
  width: number;
  height: number;
}


const edge = shallowRef<Projection["edge"]>("Right");
const expanded = shallowRef(false);
const batches = shallowRef<ShelfBatch[]>([]);
const status = shallowRef("");
const busy = shallowRef(false);
const draggingOver = shallowRef(false);
const confirmingClearTemporary = shallowRef(false);
const selection = useBatchSelection(batches);
const batchDrag = useBatchDrag(batches, selection.ids, busy, status, refreshBatches, selection.clear, active => setRailTransientHold(active, "batch-drag"));
const pendingConfirmation = batchDrag.pending;
let unlistenRail: UnlistenFn | undefined;
let unlistenShelf: UnlistenFn | undefined;
let unlistenTransientsClosed: UnlistenFn | undefined;
let unlistenDragOver: UnlistenFn | undefined;
let unlistenError: UnlistenFn | undefined;
let unlistenDrop: UnlistenFn | undefined;
let disposed = false;
let pendingWarning = "";
const hasTemporaryItems = computed(() =>
  batches.value.some((batch) => batch.items.some((item) => !item.isPinned)),
);

async function refreshBatches() {
  try {
    const snapshot = await invoke<ShelfSnapshot>("get_shelf");
    if (!disposed) {
      batches.value = snapshot.batches;
      const previousWarning = pendingWarning;
      pendingWarning = snapshot.warning ?? "";
      if (pendingWarning || status.value === previousWarning) status.value = pendingWarning;
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
    if (!disposed) status.value = pendingWarning;
  } catch (error) {
    if (!disposed) status.value = String(error);
  } finally {
    if (!disposed) busy.value = false;
  }
}

async function onBodyPointerEnter() {
  try {
    await invoke("on_rail_body_pointer_enter");
  } catch (error) {
    if (!disposed) status.value = String(error);
  }
}

async function onButtonPointerEnter() {
  try {
    await invoke("on_rail_button_pointer_enter");
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

async function startBatchDrag(batchId: string) {
  await batchDrag.start(selection.dragIds(batchId));
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

function onPointerDownOutside(event: PointerEvent) {
  const target = event.target as HTMLElement | null;
  if (target?.closest(".manage")) return;
  event.preventDefault();
  void invoke("close_manage_items");
}

onMounted(async () => {
  window.addEventListener("pointerdown", onPointerDownOutside);
  document.title = "DropCove Edge Rail";
  try {
    unlistenDragOver = await getCurrentWindow().listen<boolean>("shelf-drag-over", ({ payload }) => {
      if (!disposed) draggingOver.value = payload;
    });
    unlistenError = await getCurrentWindow().listen<string>("shelf-error", ({ payload }) => {
      if (!disposed) status.value = payload;
    });
    unlistenDrop = await getCurrentWindow().listen<{ snapshot: ShelfSnapshot | null; skipped: number; message?: string }>("shelf-drop", ({ payload }) => {
      if (disposed || !payload.snapshot) return;
      draggingOver.value = false;
      status.value = payload.message ?? (payload.skipped ? `${payload.skipped} unsupported item(s) skipped.` : pendingWarning);
    });
    unlistenRail = await getCurrentWindow().listen<Projection>("rail-changed", async ({ payload }) => {
      if (!disposed) {
        edge.value = payload.edge;
        expanded.value = payload.expanded;
        if (!payload.expanded) {
          selection.clear();
          await batchDrag.cancel();
          await invoke("invalidate_surface_visuals");
        }
      }
    });
    unlistenShelf = await getCurrentWindow().listen("shelf-changed", async () => {
      if (!disposed) await refreshBatches();
    });
    unlistenTransientsClosed = await getCurrentWindow().listen("rail-transients-closed", async () => {
      if (disposed) return;
      expanded.value = false;
      selection.clear();
      confirmingClearTemporary.value = false;
      await batchDrag.cancel();
      await invoke("invalidate_surface_visuals");
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
  batchDrag.dispose();
  disposed = true;
  window.removeEventListener("pointerdown", onPointerDownOutside);
  unlistenRail?.();
  unlistenShelf?.();
  unlistenTransientsClosed?.();
  unlistenDragOver?.();
  unlistenError?.();
  unlistenDrop?.();
});
const railListRef = useTemplateRef<HTMLElement>("railListRef");
const railVirtual = useVirtualCollection(batches, railListRef, 1, 48, 4, () => Number(draggingOver.value));
</script>

<template>
  <div
    class="rail-surface size-full box-border overflow-hidden select-none transition-[opacity,transform] duration-[140ms,140ms] ease-[cubic-bezier(0.16,1,0.3,1)]"
    :class="{ left: edge === 'Left', expanded }"
    @pointerleave="onPointerLeave"
  >
    <div
      v-if="!expanded"
      class="handle flex flex-col size-full box-border p-0 border border-dc-border-default bg-dc-surface text-dc-text-primary font-['Segoe_UI',sans-serif] select-none forced-colors:border-[ButtonText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
      :class="edge === 'Left' ? 'rounded-r-[8px] rounded-l-none' : 'rounded-l-[8px] rounded-r-none'"
    >
      <!-- 15px button height + 1px top handle border forms the exact 16px outer hit region -->
      <button
        class="open-shelf-btn relative flex items-center justify-center w-full h-[15px] flex-none box-border p-0 border-none bg-transparent text-dc-text-secondary cursor-pointer hover:bg-dc-surface-hover hover:text-dc-text-primary focus-visible:outline-2 focus-visible:outline-dc-focus focus-visible:-outline-offset-2 forced-colors:bg-transparent forced-colors:text-[ButtonText] forced-colors:hover:bg-[Highlight] forced-colors:hover:text-[HighlightText] forced-colors:focus-visible:outline-[Highlight]"
        :class="edge === 'Left' ? 'rounded-tr-[7px]' : 'rounded-tl-[7px]'"
        aria-label="Open Shelf"
        :title="status || 'Open Shelf'"
        @pointerenter="onButtonPointerEnter"
        @click="openShelf"
      >
        <ActionIcon :name="edge === 'Left' ? 'right' : 'left'" :size="10" />
        <span v-if="status" class="error absolute bottom-0 font-bold text-[10px] leading-none text-dc-danger">!</span>
      </button>
      <div
        class="expansion-body flex flex-1 flex-col items-center justify-center w-full min-h-0 box-border cursor-default hover:bg-dc-surface-hover forced-colors:hover:bg-[Highlight]"
        :class="edge === 'Left' ? 'rounded-br-[7px]' : 'rounded-bl-[7px]'"
        @pointerenter="onBodyPointerEnter"
      >
        <div
          class="grip flex flex-col items-center justify-center gap-[4px] pointer-events-none select-none"
          aria-hidden="true"
        >
          <span class="w-[2px] h-[32px] rounded-full bg-dc-border-default forced-colors:bg-[ButtonText]" />
        </div>
      </div>
    </div>
    <section
      v-else
      class="expanded-container flex flex-col size-full box-border border border-dc-border-default bg-dc-surface text-dc-text-primary font-['Segoe_UI',sans-serif] overflow-hidden forced-colors:border-[ButtonText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
      :class="edge === 'Left' ? 'rounded-r-[8px] rounded-l-none' : 'rounded-l-[8px] rounded-r-none'"
      role="region"
      aria-label="Expanded Edge Rail"
      :aria-busy="busy"
      @pointerenter="onBodyPointerEnter"
    >
      <header
        class="rail-header flex flex-none items-start h-[28px] box-border pt-[4px] pb-0 px-[6px] forced-colors:border-b-[ButtonText] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
        :class="edge === 'Left' ? 'justify-end' : 'justify-start'"
      >
        <button
          class="open-shelf-btn inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-text-secondary cursor-pointer [&:hover:not(:disabled)]:border-dc-border-hover [&:hover:not(:disabled)]:bg-dc-surface-hover [&:hover:not(:disabled)]:text-dc-text-primary aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
          aria-label="Open Shelf"
          title="Open Shelf"
          @pointerenter="onButtonPointerEnter"
          @click="openShelf"
        ><ActionIcon :name="edge === 'Left' ? 'right' : 'left'" /></button>
      </header>
      <div class="flex flex-col flex-1 min-h-0 pt-[8px]">
        <div
          ref="railListRef"
          class="batch-list flex flex-col gap-[4px] pt-0 px-[6px] pb-[8px] overflow-y-auto flex-1 min-h-0 [&_.card]:h-[48px] [&_.card]:flex-[0_0_48px] [&_.card]:box-border [&_.card]:border [&_.card]:border-dc-border-subtle [&_.card]:rounded-[6px] [&_.card]:bg-dc-card [&_.card]:text-dc-text-primary forced-colors:[&_.card]:border-[ButtonText] forced-colors:[&_.card]:bg-[Canvas] forced-colors:[&_.card]:text-[CanvasText] [&_.card-drag]:min-h-full [&_.card-drag]:h-full [&_.card-drag]:box-border"
          :style="railVirtual.enabled.value ? { position: 'relative' } : {}"
          @pointerup="event => { if (!busy && !(event.target as HTMLElement).closest('.card')) selection.clear(); }"
        >
        <div v-if="railVirtual.enabled.value" :style="{ height: `${railVirtual.totalHeight.value}px`, flex: 'none', width: '100%', pointerEvents: 'none' }" />
        <DropPlaceholder v-if="draggingOver" class="rounded-[6px]" :style="railVirtual.leadingPosition.value" />
        <ShelfBatchCard
          v-for="entry in railVirtual.entries.value"
          :key="entry.item.id"
          :batch="entry.item"
          :busy="busy"
          :selected="selection.selected.value.has(entry.item.id)"
          @select="(id, modifiers) => { if (!busy && !confirmingClearTemporary) selection.select(id, modifiers); }"
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
          title="Clear temporary items and DropCove-created files; pinned items and external sources stay"
          :disabled="busy || pendingConfirmation !== null"
          @click="requestClearTemporary"
        ><ActionIcon name="trash" /></button>
        <span v-if="selection.ids.value.length" class="min-w-0 flex-1 px-[4px] text-center text-[10px] text-dc-text-secondary">{{ selection.ids.value.length }} batches · {{ selection.itemCount.value }} items</span>
      </footer>
      <BatchDragConfirmation
        :pending="pendingConfirmation"
        @confirm="batchDrag.confirm"
        @cancel="batchDrag.cancel"
        @drag="batchDrag.dragConfirmed"
      />
      <ClearTemporaryConfirmation
        :open="confirmingClearTemporary"
        :focus-on-open="false"
        @confirm="confirmClearTemporary"
        @cancel="cancelClearTemporary"
      />
      <span v-if="status" class="error-banner py-[4px] px-[8px] bg-dc-danger-surface text-dc-danger-text border border-dc-danger-border text-[10px]">{{ status }}</span>
    </section>
  </div>
</template>
