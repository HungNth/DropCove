<script setup lang="ts">
import { onMounted, onUnmounted, shallowRef, useTemplateRef } from "vue";
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
import { invoke } from "@tauri-apps/api/core";
const {
  batches,
  status,
  busy,
  draggingOver,
  resizing,
  resizeHover,
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
  startItemDrag,
  startBatchDrag,
  dismissDropShelf,
  startMove,
  dismissStatus,
} = useShelf();

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
  event.preventDefault();
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
</script>

<template>
  <main
    class="shelf relative flex flex-col h-full box-border border rounded-[12px] bg-dc-surface text-dc-text-primary font-['Segoe_UI',sans-serif] transition-[opacity,transform] duration-[140ms,140ms] ease-[cubic-bezier(0.16,1,0.3,1)] forced-colors:border-[ButtonText]"
    :class="{
      'border-dc-border-default': !resizeHover,
      'border-dc-drag-hover-border [box-shadow:inset_0_0_0_1px_var(--dc-drag-hover-border)]': resizeHover,
    }"
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
        :disabled="busy || resizing || moving"
        aria-label="Hide Drop Shelf"
        title="Hide Drop Shelf"
        @pointerdown.stop
        @click="() => { dismissDropShelf(); announce('Hide Drop Shelf requested'); }"
      >
        <ActionIcon name="close" />
      </button>
    </div>
    <div v-if="batches.length || draggingOver" class="flex flex-col flex-1 min-h-0 pt-[8px]">
      <section
        ref="gridRef"
        class="grid grid-cols-[repeat(auto-fill,minmax(164px,1fr))] [align-content:start] gap-[4px] pt-0 px-[7px] pb-[8px] min-h-0 overflow-x-hidden overflow-y-auto"
        role="list"
        aria-label="Shelf batches"
        :style="virtual.enabled.value ? { position: 'relative' } : {}"
        @keydown="virtual.onTab"
      >
      <div v-if="virtual.enabled.value" :style="{ height: `${virtual.totalHeight.value}px`, width: '100%', pointerEvents: 'none' }" />
      <DropPlaceholder v-if="draggingOver" :style="virtual.leadingPosition.value" />
      <ShelfBatchCard
        v-for="entry in virtual.entries.value"
        :key="entry.item.id"
        :batch="entry.item"
        :busy="busy || resizing"
        :data-virtual-index="entry.index"
        :data-virtual-id="entry.item.id"
        :role="'listitem'"
        :aria-setsize="batches.length"
        :aria-posinset="entry.index + 1"
        :style="virtual.position(entry.index)"
        @remove="(id) => { removeBatch(id); announce('Batch removed'); }"
        @pin-item="(id, p) => { setItemPinned(id, p); announce(p ? 'Item pinned' : 'Item unpinned'); }"
        @pin-batch="(id, p) => { setBatchPinned(id, p); announce(p ? 'Batch pinned' : 'Batch unpinned'); }"
        @manage="openManageItems"
        @drag-item="startItemDrag"
        @drag-batch="startBatchDrag"
      />
      </section>
    </div>
    <section v-else class="flex flex-1 flex-col items-center justify-center min-h-[70px] p-[12px] text-center" aria-label="Empty Drop Shelf">
      <ActionIcon class="size-[28px]! text-dc-accent" name="down" />
      <p class="mt-[8px] mb-0 text-[13px] text-dc-text-primary">Drop here</p>
    </section>
    <BatchDragConfirmation :pending="pendingConfirmation" @confirm="confirmBatchDrag" @cancel="cancelBatchDrag" @drag="dragConfirmedBatch" />
    <footer class="flex flex-none items-center justify-between mt-auto pt-0 px-[8px] pb-[8px]">
      <button
        v-if="hasTemporaryItems"
        type="button"
        class="clear-temporary-action inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-danger cursor-pointer [&:hover:not(:disabled)]:border-dc-danger-border [&:hover:not(:disabled)]:bg-dc-danger-surface [&:hover:not(:disabled)]:text-dc-danger aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:border-[ButtonText] forced-colors:text-dc-danger forced-colors:[&:hover:not(:disabled)]:border-dc-danger-border forced-colors:[&:hover:not(:disabled)]:bg-dc-danger-surface forced-colors:[&:hover:not(:disabled)]:text-dc-danger forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
        :disabled="busy || resizing || pendingConfirmation !== null || confirmingClearTemporary"
        aria-label="Clear Temporary Items"
        title="Clear Temporary Items; pinned references remain and source files are not deleted"
        @click="() => { requestClearTemporary(); announce('Clear temporary items requested'); }"
      >
        <ActionIcon name="trash" />
      </button>
      <div v-else class="size-[24px] flex-none" aria-hidden="true" />
      <UtilityMenu
        :disabled="busy || resizing || pendingConfirmation !== null || confirmingClearTemporary"
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
    <ShelfResizeHandles :disabled="busy || resizing || moving" @resize="startResize" />
  </main>
</template>

