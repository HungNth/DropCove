<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, shallowRef, useTemplateRef, watch } from "vue";
import { useResizeObserver } from "@vueuse/core";
import { useAnnouncer } from "../../composables/useAnnouncer";
import { useVirtualCollection } from "../../composables/useVirtualCollection";
import { useShelf } from "../../composables/useShelf";
import ShelfBatchCard from "./ShelfBatchCard.vue";
import DropPlaceholder from "./DropPlaceholder.vue";
import BatchDragConfirmation from "./BatchDragConfirmation.vue";
import ShelfResizeHandles from "./ShelfResizeHandles.vue";
import ActionIcon from "./ActionIcon.vue";
import UtilityMenu from "./UtilityMenu.vue";
import SelectionActionsMenu from "./SelectionActionsMenu.vue";
import SelectionRemovalConfirmation from "./SelectionRemovalConfirmation.vue";
import { batchSelectionPinAction } from "../../composables/selectionActions";
import { invoke } from "@tauri-apps/api/core";
import type { ShelfItemAction } from "./ItemActionsMenu.vue";
const {
  batches,
  activeItemActionsMenuId,
  selectionMenu,
  openSelectionMenu,
  pinSelection,
  removeSelection,
  removalConfirmation,
  confirmRemoval,
  cancelRemoval,
  selection,
  selectBatch,
  status,
  busy,
  draggingOver,
  resizing,
  moving,
  startResize,
  pendingConfirmation,
  hasTemporaryItems,
  confirmingClearTemporary,
  requestClearTemporary,
  confirmBatchDrag,
  dragConfirmedBatch,
  cancelBatchDrag,
  removeBatch,
  setItemPinned,
  setBatchPinned,
  openManageItems,
  startBatchDrag,
  dismissDropShelf,
  startMove,
  dismissStatus,
} = useShelf();

async function handlePerformItemAction(itemId: string, action: ShelfItemAction) {
  try {
    await invoke("perform_item_action", { itemId, action });
    activeItemActionsMenuId.value = null;
    if (action === "CopyPath") {
      announce("Path copied.");
    } else if (action === "CopyText") {
      announce("Text copied.");
    }
    if (action !== "ShowInExplorer") {
      await nextTick();
      const batch = batches.value.find(batch => batch.items.some(item => item.id === itemId));
      if (batch) gridRef.value?.querySelector<HTMLButtonElement>(`[data-batch-id="${CSS.escape(batch.id)}"] .more`)?.focus();
    }
  } catch (error) {
    activeItemActionsMenuId.value = null;
    status.value = String(error);
  }
}
async function openSettingsFromMenu() {
  try {
    await invoke("open_settings");
  } catch (error) {
    status.value = String(error);
    document.querySelector<HTMLButtonElement>(".more-action")?.focus();
  }
}

async function openAboutFromMenu() {
  try {
    await invoke("open_about");
  } catch (error) {
    status.value = String(error);
    document.querySelector<HTMLButtonElement>(".more-action")?.focus();
  }
}

function onPointerDownOutside(event: PointerEvent) {
  const target = event.target as HTMLElement | null;
  if (target?.closest(".manage")) return;
  void invoke("close_manage_items");
}

onMounted(() => {
  window.addEventListener("pointerdown", onPointerDownOutside);
});

onUnmounted(() => {
  window.removeEventListener("pointerdown", onPointerDownOutside);
});

const { announcement, announce } = useAnnouncer();
const gridRef = useTemplateRef<HTMLElement>("gridRef");
const columns = shallowRef(1);
useResizeObserver(gridRef, () => {
  if (gridRef.value) columns.value = getComputedStyle(gridRef.value).gridTemplateColumns.split(" ").length;
});
const virtual = useVirtualCollection(batches, gridRef, columns, 48, 4, () => Number(draggingOver.value));
const focusedBatch = shallowRef<string | null>(null);

watch([batches, busy, resizing, moving, pendingConfirmation, confirmingClearTemporary], () => {
  activeItemActionsMenuId.value = null;
  selectionMenu.value = null;
});

const selectionMenuBatches = computed(() => batches.value.filter(batch => selectionMenu.value?.batchIds.includes(batch.id)));
const selectionMenuPin = computed(() => batchSelectionPinAction(selectionMenuBatches.value));
// Singular wording follows the selection unit, not the affected item count.
const selectionMenuActions = computed(() => [
  { id: "pin", label: selectionMenuPin.value.label },
  { id: "remove", label: selectionMenuBatches.value.length > 1 ? "Remove Selected" : "Remove Batch", danger: true },
]);

function focusOrigin(origin: string | undefined) {
  if (origin) gridRef.value?.querySelector<HTMLElement>(`[data-batch-id="${CSS.escape(origin)}"] .card-drag`)?.focus();
}

async function onSelectionMenuAction(id: string) {
  const origin = selectionMenu.value?.originId;
  if (id === "pin") {
    const pinned = selectionMenuPin.value.pinned;
    const committed = await pinSelection(pinned);
    if (committed) announce(pinned ? "Selection pinned" : "Selection unpinned");
  } else if (id === "remove") {
    const several = selectionMenuBatches.value.length > 1;
    const committed = await removeSelection();
    if (committed && !several) announce("Batch removed");
    if (several) return;
  } else {
    return;
  }
  await nextTick();
  focusOrigin(origin);
}

async function onRemovalConfirm() {
  const origin = removalConfirmation.value?.originId;
  const committed = await confirmRemoval();
  if (committed) announce("Selection removed");
  await nextTick();
  focusOrigin(origin);
}

async function onRemovalCancel() {
  const origin = removalConfirmation.value?.originId;
  await cancelRemoval();
  await nextTick();
  focusOrigin(origin);
}

async function closeSelectionMenu(restoreFocus: boolean) {
  const origin = selectionMenu.value?.originId;
  selectionMenu.value = null;
  if (restoreFocus && origin) {
    await nextTick();
    gridRef.value?.querySelector<HTMLElement>(`[data-batch-id="${CSS.escape(origin)}"] .card-drag`)?.focus();
  }
}
watch(() => `${selection.ids.value.join(',')}:${selection.itemCount.value}`, () => {
  const count = selection.ids.value.length;
  announce(count ? `${count} batches, ${selection.itemCount.value} items selected` : "Selection cleared");
});

async function onGridKeydown(event: KeyboardEvent) {
  const target = event.target as HTMLElement;
  if (event.key === "Tab") {
    await virtual.onTab(event);
    return;
  }
  if (!target.matches(".card-drag") || busy.value || resizing.value) {
    await virtual.onTab(event);
    return;
  }
  const card = target.closest<HTMLElement>("[data-batch-id]");
  const index = batches.value.findIndex(batch => batch.id === card?.dataset.batchId);
  if (index < 0) return;
  const id = batches.value[index].id;
  if (event.key === " ") {
    event.preventDefault();
    selectBatch(id, event);
    return;
  }
  if (event.ctrlKey && event.key.toLowerCase() === "a") {
    event.preventDefault();
    selection.selectAll();
    return;
  }
  let next = index;
  switch (event.key) {
    case "ArrowLeft": next--; break;
    case "ArrowRight": next++; break;
    case "ArrowUp": next -= columns.value; break;
    case "ArrowDown": next += columns.value; break;
    case "Home": next = 0; break;
    case "End": next = batches.value.length - 1; break;
    default: return;
  }
  event.preventDefault();
  next = Math.max(0, Math.min(next, batches.value.length - 1));
  focusedBatch.value = batches.value[next].id;
  await virtual.focus(next, -2);
}

watch(batches, async (current, previous) => {
  const index = previous.findIndex(batch => batch.id === focusedBatch.value);
  if (index < 0 || current.some(batch => batch.id === focusedBatch.value)) return;
  if (!current.length) {
    focusedBatch.value = null;
    document.querySelector<HTMLElement>(".more-action")?.focus();
    return;
  }
  const next = Math.min(index, current.length - 1);
  focusedBatch.value = current[next].id;
  await virtual.focus(next, -2);
}, { flush: "post" });
</script>

<template>
  <main
    class="shelf relative flex flex-col h-full box-border border border-dc-border-default rounded-[12px] bg-dc-surface text-dc-text-primary font-['Segoe_UI',sans-serif] transition-[opacity,transform] duration-[140ms,140ms] ease-[cubic-bezier(0.16,1,0.3,1)] forced-colors:border-[ButtonText]"
    :aria-busy="busy || resizing || moving"
  >
    <div class="control-strip relative flex flex-none items-center justify-center h-[28px] box-border select-none pointer-events-auto" role="toolbar" aria-label="Drop Shelf controls">
      <div
        class="drag-region flex w-full h-full items-center justify-center pt-[4px] box-border cursor-default"
        aria-hidden="true"
        @pointerdown="startMove"
      >
        <ActionIcon name="grip" class="text-dc-text-muted/60 pointer-events-none forced-colors:text-[ButtonText]" />
      </div>
      <button
        type="button"
        class="dismiss-action absolute top-[4px] right-[4px] z-[3] inline-flex items-center justify-center size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-text-secondary cursor-pointer [&:hover:not(:disabled)]:border-dc-border-hover [&:hover:not(:disabled)]:bg-dc-surface-hover [&:hover:not(:disabled)]:text-dc-text-primary focus-visible:outline-2 focus-visible:outline-dc-focus disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText]"
        :disabled="busy || resizing || moving || removalConfirmation !== null"
        aria-label="Hide Drop Shelf"
        title="Hide Drop Shelf"
        @pointerdown.stop
        @click="() => { activeItemActionsMenuId = null; dismissDropShelf(); announce('Hide Drop Shelf requested'); }"
      >
        <ActionIcon name="close" />
      </button>
    </div>
    <div v-if="batches.length || draggingOver" class="flex flex-col flex-1 min-h-0 pt-[8px]">
      <section
        ref="gridRef"
        class="grid grid-cols-[repeat(auto-fill,minmax(164px,1fr))] [align-content:start] gap-[4px] pt-0 px-[7px] pb-[8px] min-h-0 overflow-x-hidden overflow-y-auto"
        role="grid"
        aria-multiselectable="true"
        aria-label="Shelf batches"
        :aria-rowcount="batches.length"
        :style="virtual.enabled.value ? { position: 'relative' } : {}"
        @scroll="() => { activeItemActionsMenuId = null; selectionMenu = null; }"
        @keydown="onGridKeydown"
        @focusin="event => { const id = (event.target as HTMLElement).closest<HTMLElement>('[data-batch-id]')?.dataset.batchId; if (id) focusedBatch = id; }"
        @pointerup="event => { if (!busy && !resizing && !(event.target as HTMLElement).closest('.card, .item-actions-menu, .more')) selection.clear(); }"
      >
      <div v-if="virtual.enabled.value" :style="{ height: `${virtual.totalHeight.value}px`, width: '100%', pointerEvents: 'none' }" />
      <DropPlaceholder v-if="draggingOver" :style="virtual.leadingPosition.value" />
      <ShelfBatchCard
        v-for="entry in virtual.entries.value"
        :key="entry.item.id"
        :batch="entry.item"
        :busy="busy || resizing || removalConfirmation !== null"
        :selected="selection.selected.value.has(entry.item.id)"
        :selection-tabindex="(focusedBatch ?? batches[0]?.id) === entry.item.id ? 0 : -1"
        @select="selectBatch"
        @focus-batch="id => focusedBatch = id"
        :data-virtual-index="entry.index"
        :data-virtual-id="entry.item.id"
        role="row"
        :aria-rowindex="entry.index + 1"
        :style="virtual.position(entry.index)"
        @remove="(id) => { removeBatch(id); announce('Batch removed'); }"
        @pin-item="(id, p) => { setItemPinned(id, p); announce(p ? 'Item pinned' : 'Item unpinned'); }"
        @pin-batch="(id, p) => { setBatchPinned(id, p); announce(p ? 'Batch pinned' : 'Batch unpinned'); }"
        :menu-open-item-id="activeItemActionsMenuId"
        @open-item-menu="id => activeItemActionsMenuId = id"
        @close-item-menu="id => { if (activeItemActionsMenuId === id) activeItemActionsMenuId = null; }"
        @item-action="handlePerformItemAction"
        @selection-menu="(id, anchor, keyboard) => { activeItemActionsMenuId = null; openSelectionMenu(id, anchor, keyboard); }"
        @manage="(id, a, k) => { activeItemActionsMenuId = null; openManageItems(id, a, k); }"
        @drag-batch="id => { activeItemActionsMenuId = null; startBatchDrag(id); }"
      />
      </section>
    </div>
    <section v-else class="flex flex-1 flex-col items-center justify-center min-h-[70px] p-[12px] text-center" aria-label="Empty Drop Shelf">
      <ActionIcon class="size-[28px]! text-dc-accent" name="down" />
      <p class="mt-[8px] mb-0 text-[13px] text-dc-text-primary">Drop here</p>
    </section>
    <SelectionActionsMenu
      v-if="selectionMenu"
      :actions="selectionMenuActions"
      :anchor="selectionMenu.anchor"
      :keyboard="selectionMenu.keyboard"
      label="Selection actions"
      @action="onSelectionMenuAction"
      @close="closeSelectionMenu"
    />
    <SelectionRemovalConfirmation
      :preview="removalConfirmation?.preview ?? null"
      domain="batches"
      @confirm="onRemovalConfirm"
      @cancel="onRemovalCancel"
    />
    <BatchDragConfirmation :pending="pendingConfirmation" @confirm="confirmBatchDrag" @cancel="cancelBatchDrag" @drag="dragConfirmedBatch" />
    <footer class="flex flex-none items-center justify-between mt-auto pt-0 px-[8px] pb-[8px]">
      <button
        v-if="hasTemporaryItems"
        type="button"
        class="clear-temporary-action inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-danger cursor-pointer [&:hover:not(:disabled)]:border-dc-danger-border [&:hover:not(:disabled)]:bg-dc-danger-surface [&:hover:not(:disabled)]:text-dc-danger aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:border-[ButtonText] forced-colors:text-dc-danger forced-colors:[&:hover:not(:disabled)]:border-dc-danger-border forced-colors:[&:hover:not(:disabled)]:bg-dc-danger-surface forced-colors:[&:hover:not(:disabled)]:text-dc-danger forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
        :disabled="busy || resizing || pendingConfirmation !== null || confirmingClearTemporary || removalConfirmation !== null"
        aria-label="Clear Temporary Items"
        title="Clear temporary items and DropCove-created files; pinned items and external sources stay"
        @click="() => { requestClearTemporary(); announce('Clear temporary items requested'); }"
      >
        <ActionIcon name="trash" />
      </button>
      <div v-else class="size-[24px] flex-none" aria-hidden="true" />
      <span v-if="selection.ids.value.length" class="min-w-0 flex-1 px-[4px] text-center text-[10px] text-dc-text-secondary" aria-hidden="true">{{ selection.ids.value.length }} batches · {{ selection.itemCount.value }} items</span>
      <UtilityMenu
        :disabled="busy || resizing || pendingConfirmation !== null || confirmingClearTemporary || removalConfirmation !== null"
        @open-settings="openSettingsFromMenu"
        @open-about="openAboutFromMenu"
      />
    </footer>
    <div v-if="status" class="flex [align-items:start] gap-[4px] mt-auto mx-[8px] mb-[8px] p-[8px] border border-dc-warning-border bg-dc-warning-surface text-dc-warning-text rounded-[6px] text-[11px] [overflow-wrap:anywhere]" role="alert">
      <span>{{ status }}</span>
      <button type="button" class="dismiss ml-auto border-0 border-none text-inherit bg-transparent cursor-pointer" aria-label="Dismiss status" @click="dismissStatus">×</button>
    </div>
    <div class="sr-only" role="status" aria-live="polite" aria-atomic="true">
      {{ announcement }}
    </div>
    <ShelfResizeHandles :disabled="busy || resizing || moving || removalConfirmation !== null" @resize="startResize" />
  </main>
</template>

