<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, shallowRef, useTemplateRef, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { useManageItems } from "../../composables/useManageItems";
import BulkPinning from "./BulkPinning.vue";
import ShelfItemRow from "./ShelfItemRow.vue";
import BatchDragConfirmation from "./BatchDragConfirmation.vue";
import { useAnnouncer } from "../../composables/useAnnouncer";
import { useVirtualCollection } from "../../composables/useVirtualCollection";
import ActionIcon from "./ActionIcon.vue";
import SelectionActionsMenu from "./SelectionActionsMenu.vue";
import SelectionRemovalConfirmation from "./SelectionRemovalConfirmation.vue";
import { itemContextActions, itemSelectionPinAction } from "../../composables/selectionActions";

const {
  batch,
  status,
  busy,
  selectionMenu,
  openSelectionMenu,
  pinSelection,
  removeSelection,
  removalConfirmation,
  confirmRemoval,
  cancelRemoval,
  refresh,
  setItemPinned,
  setBatchPinned,
  removeItem,
  startItemDrag,
  cancelPreparedDrag,
  confirmItemDrag,
  dragConfirmedItem,
  pendingConfirmation,
  hasPreparedDrag,
  selection,
  selectItem,
  selectAllItems,
  clearSelection,
  close,
  dismissStatus,
} = useManageItems();
const { announcement, announce } = useAnnouncer();

const pinnedCount = computed(() => batch.value?.items.filter((item) => item.isPinned).length ?? 0);
const totalCount = computed(() => batch.value?.items.length ?? 0);
const selectedCount = computed(() => selection.ids.value.length);

const rootRef = useTemplateRef<HTMLElement>("root");
const itemsContainerRef = useTemplateRef<HTMLElement>("itemsContainer");
const itemsList = computed(() => batch.value?.items ?? []);
const virtual = useVirtualCollection(itemsList, itemsContainerRef, 1, 54, 5);
const focusedItem = shallowRef<string | null>(null);
const isRailOrigin = shallowRef(false);
const window = getCurrentWindow();
const unlisteners: UnlistenFn[] = [];
let cancelKeydownSeen = false;

watch(
  () => selection.ids.value.length,
  (count, prev) => {
    if (isRailOrigin.value) return;
    if (count > 0) announce(`${count} ${count === 1 ? "item" : "items"} selected`);
    else if (prev && prev > 0) announce("Selection cleared");
  },
  { flush: "post" },
);

watch(
  itemsList,
  async (current, previous) => {
    if (!focusedItem.value) return;
    const index = previous.findIndex((item) => item.id === focusedItem.value);
    if (index < 0 || current.some((item) => item.id === focusedItem.value)) return;
    if (!current.length) {
      focusedItem.value = null;
      if (!isRailOrigin.value) getBulkElement()?.focus();
      return;
    }
    const next = Math.min(index, current.length - 1);
    focusedItem.value = current[next].id;
    if (!isRailOrigin.value) {
      if (virtual.enabled.value) await virtual.focus(next, -2);
      else {
        await nextTick();
        itemsContainerRef.value?.querySelector<HTMLElement>(`[data-item-id="${focusedItem.value}"] .item-drag`)?.focus();
      }
    }
  },
  { flush: "post" },
);
watch([itemsList, busy, pendingConfirmation], () => {
  selectionMenu.value = null;
});

const selectionMenuItems = computed(() => itemsList.value.filter(item => selectionMenu.value?.itemIds.includes(item.id)));
const selectionMenuPin = computed(() => itemSelectionPinAction(selectionMenuItems.value));
const selectionMenuActions = computed(() => itemContextActions(selectionMenuItems.value));

function openItemContextMenu(itemId: string, anchor: { x: number; y: number }, keyboard: boolean) {
  if (isRailOrigin.value && keyboard) return;
  openSelectionMenu(itemId, anchor, keyboard);
  if (selectionMenu.value && selectionMenuActions.value.length === 0) selectionMenu.value = null;
}

function focusRow(itemId: string | undefined) {
  if (itemId && !isRailOrigin.value) itemsContainerRef.value?.querySelector<HTMLElement>(`[data-item-id="${CSS.escape(itemId)}"] .item-drag`)?.focus();
}

async function onSelectionMenuAction(id: string) {
  const origin = selectionMenu.value?.originId;
  if (id === "ShowInExplorer" || id === "CopyPath" || id === "CopyText") {
    const item = selectionMenuItems.value.length === 1 ? selectionMenuItems.value[0] : undefined;
    selectionMenu.value = null;
    if (!item) return;
    try {
      await invoke("perform_item_action", { itemId: item.id, action: id });
      if (!isRailOrigin.value) {
        if (id === "CopyPath") announce("Path copied.");
        else if (id === "CopyText") announce("Text copied.");
        if (id !== "ShowInExplorer") {
          await nextTick();
          focusRow(origin);
        }
      }
    } catch (error) {
      status.value = String(error);
      if (!isRailOrigin.value) {
        await nextTick();
        focusRow(origin);
      }
    }
    return;
  }
  if (id === "pin") {
    const pinned = selectionMenuPin.value.pinned;
    const committed = await pinSelection(pinned);
    if (committed && !isRailOrigin.value) announce(pinned ? "Selection pinned" : "Selection unpinned");
  } else if (id === "remove") {
    const several = selectionMenuItems.value.length > 1;
    const committed = await removeSelection();
    if (committed && !several && !isRailOrigin.value) announce("Item removed");
    if (several) return;
  } else {
    return;
  }
  await nextTick();
  focusRow(origin);
}

async function onRemovalConfirm() {
  const origin = removalConfirmation.value?.originId;
  const committed = await confirmRemoval();
  if (committed && !isRailOrigin.value) announce("Selection removed");
  await nextTick();
  focusRow(origin);
}

async function onRemovalCancel() {
  const origin = removalConfirmation.value?.originId;
  await cancelRemoval();
  await nextTick();
  focusRow(origin);
}

async function closeSelectionMenu(restoreFocus: boolean) {
  const origin = selectionMenu.value?.originId;
  selectionMenu.value = null;
  if (restoreFocus) {
    await nextTick();
    focusRow(origin);
  }
}


function getBulkElement(): HTMLElement | null {
  return rootRef.value?.querySelector<HTMLElement>(".bulk-pin") ?? null;
}

let focusRequest = 0;
let disposed = false;
async function focusInitialControl() {
  const request = ++focusRequest;
  await refresh();
  await nextTick();
  if (disposed || request !== focusRequest) return;
  const first = itemsList.value[0]?.id;
  if (first) {
    focusedItem.value = first;
    if (virtual.enabled.value) await virtual.focus(0, -2);
    else itemsContainerRef.value?.querySelector<HTMLElement>(`[data-item-id="${first}"] .item-drag`)?.focus();
  } else {
    getBulkElement()?.focus();
  }
}


async function onKeydown(event: KeyboardEvent) {
  if (selectionMenu.value && event.key === "Escape") {
    event.preventDefault();
    event.stopPropagation();
    await closeSelectionMenu(true);
    return;
  }
  if (removalConfirmation.value && event.key === "Escape") {
    event.preventDefault();
    event.stopPropagation();
    await onRemovalCancel();
    return;
  }
  if (removalConfirmation.value) return;

  if (event.key === "Tab" && !pendingConfirmation.value && !busy.value) {
    event.preventDefault();
    const active = document.activeElement as HTMLElement | null;
    const row = active?.closest<HTMLElement>("[data-item-id]");
    const index = itemsList.value.findIndex(item => item.id === row?.dataset.itemId);
    const rowButtons: Array<{ rowIndex: number; controlIndex: number; kind: "pin" | "remove" }> = [];
    itemsList.value.forEach((_item, rowIndex) => {
      rowButtons.push({ rowIndex, controlIndex: 1, kind: "pin" });
      rowButtons.push({ rowIndex, controlIndex: 2, kind: "remove" });
    });
    let position = active === getBulkElement() ? 0 : active?.matches(".header .close") ? 1 : 2;
    if (index >= 0 && active?.matches("button")) {
      const kind = active.matches(".pin") ? "pin" : "remove";
      const buttonIndex = rowButtons.findIndex(b => b.rowIndex === index && b.kind === kind);
      if (buttonIndex >= 0) position = 3 + buttonIndex;
    }
    const length = 3 + rowButtons.length;
    const next = (position + (event.shiftKey ? -1 : 1) + length) % length;
    if (next === 0) getBulkElement()?.focus();
    else if (next === 1) rootRef.value?.querySelector<HTMLElement>(".header .close")?.focus();
    else if (next === 2) {
      const focused = itemsList.value.findIndex(item => item.id === focusedItem.value);
      await virtual.focus(Math.max(0, focused), -2);
    } else {
      const target = rowButtons[next - 3];
      await virtual.focus(target.rowIndex, target.controlIndex);
    }
    return;
  }

  if (event.key === "Escape") {
    event.stopPropagation();
    event.preventDefault();
    if (pendingConfirmation.value || hasPreparedDrag()) {
      await cancelPreparedDrag();
      return;
    }
    if (busy.value) {
      cancelKeydownSeen = true;
      return;
    }
    if (cancelKeydownSeen) {
      return;
    }
    if (selection.ids.value.length > 0) {
      clearSelection();
      return;
    }
    void close();
  }
}

async function onGridKeydown(event: KeyboardEvent) {
  if (event.defaultPrevented || event.key === "Tab") return;
  const target = event.target as HTMLElement;
  if (!target.matches(".item-drag") || busy.value) return;
  const row = target.closest<HTMLElement>("[data-item-id]");
  const index = itemsList.value.findIndex((item) => item.id === row?.dataset.itemId);
  if (index < 0) return;
  const id = itemsList.value[index].id;

  if (event.key === " ") {
    event.preventDefault();
    selectItem(id, { ctrlKey: event.ctrlKey, shiftKey: event.shiftKey });
    return;
  }
  if (event.ctrlKey && event.key.toLowerCase() === "a") {
    event.preventDefault();
    selectAllItems();
    return;
  }
  let next = index;
  switch (event.key) {
    case "ArrowUp":
      next--;
      break;
    case "ArrowDown":
      next++;
      break;
    case "Home":
      next = 0;
      break;
    case "End":
      next = itemsList.value.length - 1;
      break;
    default:
      return;
  }
  event.preventDefault();
  next = Math.max(0, Math.min(next, itemsList.value.length - 1));
  focusedItem.value = itemsList.value[next].id;
  if (virtual.enabled.value) await virtual.focus(next, -2);
  else {
    await nextTick();
    itemsContainerRef.value?.querySelector<HTMLElement>(`[data-item-id="${focusedItem.value}"] .item-drag`)?.focus();
  }
}

function onKeyup(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.stopPropagation();
    event.preventDefault();
    if (cancelKeydownSeen) {
      cancelKeydownSeen = false;
    }
  }
}

onMounted(async () => {
  document.addEventListener("keydown", onKeydown, true);
  document.addEventListener("keyup", onKeyup, true);
  try {
    const cancelListener = await window.listen("item-drag-canceled", () => { cancelKeydownSeen = true; });
    if (disposed) { cancelListener(); return; }
    unlisteners.push(cancelListener);
    isRailOrigin.value = await invoke<string | null>("get_manage_items_origin") === "edge-rail";

    const unlisten = await window.listen<boolean>("manage-items-opened", async ({ payload: keyboard }) => {
      isRailOrigin.value = await invoke<string | null>("get_manage_items_origin") === "edge-rail";
      if (!keyboard || isRailOrigin.value) {
        ++focusRequest;
        return;
      }
      try {
        await focusInitialControl();
      } catch (error) {
        if (!disposed) status.value = String(error);
      }
    });
    if (disposed) {
      unlisten();
      return;
    }
    unlisteners.push(unlisten);
    if (!isRailOrigin.value && (await invoke<boolean>("get_manage_items_keyboard"))) {
      await focusInitialControl();
    }
  } catch (error) {
    if (!disposed) status.value = String(error);
  }
});

onUnmounted(() => {
  disposed = true;
  document.removeEventListener("keydown", onKeydown, true);
  document.removeEventListener("keyup", onKeyup, true);
  for (const unlisten of unlisteners) unlisten();
});
</script>

<template>
  <main
    ref="root"
    class="management flex flex-col h-full box-border border border-dc-border-default rounded-[9px] text-dc-text-primary bg-dc-surface font-['Segoe_UI',sans-serif] transition-[opacity,transform] duration-[140ms,140ms] ease-[cubic-bezier(0.16,1,0.3,1)] forced-colors:border-[ButtonText]"
    :aria-busy="busy"
    aria-label="Manage Items"
  >
    <header class="header flex items-center justify-between gap-[6px] p-[9px]">
      <div class="header-info flex min-w-0 flex-col gap-[2px]">
        <h1 class="title m-0 text-[13px] text-dc-text-primary">Manage Items<span v-if="batch" class="count text-dc-text-muted font-normal"> · {{ batch.items.length }}</span></h1>
        <span v-if="batch" class="summary text-dc-text-muted text-[11px]">{{ selectedCount > 0 ? `${selectedCount} selected · ${pinnedCount}/${totalCount} pinned` : `${pinnedCount}/${totalCount} pinned` }}</span>
      </div>
      <div class="header-actions flex items-center gap-[6px]">
        <BulkPinning
          v-if="batch"
          :batch="batch"
          :busy="busy || removalConfirmation !== null"
          @pin="(id, pinned) => setBatchPinned(id, pinned)"
        />
        <button
          class="close inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-text-secondary cursor-pointer [&:hover:not(:disabled)]:border-dc-border-hover [&:hover:not(:disabled)]:bg-dc-surface-hover [&:hover:not(:disabled)]:text-dc-text-primary aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
          type="button"
          :disabled="busy || removalConfirmation !== null"
          aria-label="Close Manage Items"
          title="Close Manage Items"
          @click="close"
        ><ActionIcon name="close" /></button>
      </div>
    </header>
    <div v-if="batch" class="flex flex-col flex-1 min-h-0 pt-[4px]">
      <ul
        ref="itemsContainer"
        class="items flex flex-col gap-[5px] min-h-0 overflow-y-auto overflow-x-hidden m-0 pt-0 px-[7px] pb-[3px] list-none"
        role="grid"
        aria-multiselectable="true"
        :aria-label="`Batch items (${batch.items.length})`"
        :aria-rowcount="batch.items.length"
        :style="virtual.enabled.value ? { position: 'relative' } : {}"
        @scroll="selectionMenu = null"
        @keydown="onGridKeydown"
        @focusin="(event) => { const id = (event.target as HTMLElement).closest<HTMLElement>('[data-item-id]')?.dataset.itemId; if (id) focusedItem = id; }"
        @pointerup="(event) => { if (!busy && !(event.target as HTMLElement).closest('.item-row')) clearSelection(); }"
      >
      <div v-if="virtual.enabled.value" :style="{ height: `${virtual.totalHeight.value}px`, flex: 'none', width: '100%', pointerEvents: 'none' }" />
      <ShelfItemRow
        v-for="entry in virtual.entries.value"
        :key="entry.item.id"
        :item="entry.item"
        :busy="busy || removalConfirmation !== null"
        :selected="selection.selected.value.has(entry.item.id)"
        :selection-tabindex="(focusedItem ?? itemsList[0]?.id) === entry.item.id ? 0 : -1"
        :data-virtual-index="entry.index"
        :data-virtual-id="entry.item.id"
        role="row"
        :aria-rowindex="entry.index + 1"
        :style="virtual.position(entry.index)"
        @select="selectItem"
        @focus-item="(id) => focusedItem = id"
        @pin="(id, p) => { setItemPinned(id, p); if (!isRailOrigin) announce(p ? 'Item pinned' : 'Item unpinned'); }"
        @remove="(id) => { removeItem(id); if (!isRailOrigin) announce('Item removed'); }"
        @drag="startItemDrag"
        @selection-menu="openItemContextMenu"
      />
      </ul>
    </div>
    <SelectionActionsMenu
      v-if="selectionMenu"
      :actions="selectionMenuActions"
      :anchor="selectionMenu.anchor"
      :keyboard="selectionMenu.keyboard"
      :no-activate="isRailOrigin"
      label="Selection actions"
      @action="onSelectionMenuAction"
      @close="closeSelectionMenu"
    />
    <SelectionRemovalConfirmation
      :preview="removalConfirmation?.preview ?? null"
      domain="items"
      :no-activate="isRailOrigin"
      @confirm="onRemovalConfirm"
      @cancel="onRemovalCancel"
    />
    <BatchDragConfirmation :pending="pendingConfirmation" @confirm="confirmItemDrag" @cancel="cancelPreparedDrag" @drag="dragConfirmedItem" />
    <div v-if="status" class="status flex [align-items:start] gap-[4px] shrink-0 mt-auto mx-[7px] mb-[7px] p-[6px] border border-dc-warning-border bg-dc-warning-surface text-dc-warning-text rounded-[5px] text-[11px] [overflow-wrap:anywhere]" role="alert">
      <span>{{ status }}</span>
      <button class="dismiss ml-auto border-0 border-none text-inherit bg-transparent cursor-pointer" type="button" aria-label="Dismiss status" @click="dismissStatus">×</button>
    </div>
    <div v-if="!isRailOrigin" class="sr-only" role="status" aria-live="polite" aria-atomic="true">
      {{ announcement }}
    </div>
  </main>
</template>

