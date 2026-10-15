<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import { useAnnouncer } from "../../composables/useAnnouncer";
import { useVirtualCollection } from "../../composables/useVirtualCollection";
import { useShelf } from "../../composables/useShelf";
import ShelfBatchCard from "./ShelfBatchCard.vue";
import BatchDragConfirmation from "./BatchDragConfirmation.vue";
import ClearTemporaryConfirmation from "./ClearTemporaryConfirmation.vue";
import ShelfResizeHandles from "./ShelfResizeHandles.vue";
import ActionIcon from "./ActionIcon.vue";
const {
  batches,
  status,
  busy,
  draggingOver,
  resizing,
  resizeHover,
  startResize,
  pendingConfirmation,
  hasTemporaryItems,
  confirmingClearTemporary,
  requestClearTemporary,
  cancelClearTemporary,
  confirmClearTemporary,
  confirmBatchDrag,
  dragConfirmedBatch,
  cancelBatchDrag,
  removeBatch,
  setItemPinned,
  setBatchPinned,
  openManageItems,
  startItemDrag,
  startBatchDrag,
  dismissStatus,
} = useShelf();
const { announcement, announce } = useAnnouncer();
const gridRef = useTemplateRef<HTMLElement>("gridRef");
const columns = computed(() => {
  if (!gridRef.value) return 1;
  return Math.max(1, Math.floor(gridRef.value.clientWidth / 164));
});
const virtual = useVirtualCollection(batches, gridRef, columns, 48, 4);
</script>

<template>
  <main class="shelf" :class="{ 'dragging-over': draggingOver, 'resize-hover': resizeHover }" :aria-busy="busy || resizing">
    <section
      v-if="batches.length"
      ref="gridRef"
      class="grid"
      role="list"
      aria-label="Shelf batches"
      :style="virtual.enabled.value ? { position: 'relative' } : {}"
      @keydown="virtual.onTab"
    >
      <div v-if="virtual.enabled.value" :style="{ height: `${virtual.totalHeight.value}px`, width: '100%', pointerEvents: 'none' }" />
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
    <section v-else class="empty" aria-label="Empty Drop Shelf">
      <ActionIcon class="empty-icon" name="down" />
      <p class="empty-text">Drop here</p>
    </section>
    <BatchDragConfirmation :pending="pendingConfirmation" @confirm="confirmBatchDrag" @cancel="cancelBatchDrag" @drag="dragConfirmedBatch" />
    <ClearTemporaryConfirmation :open="confirmingClearTemporary" :focus-on-open="true" @confirm="confirmClearTemporary" @cancel="cancelClearTemporary" />
    <footer v-if="hasTemporaryItems" class="footer">
      <button
        type="button"
        class="icon-action clear-temporary-action"
        :disabled="busy || resizing || pendingConfirmation !== null"
        aria-label="Clear Temporary Items"
        title="Clear Temporary Items; pinned references remain and source files are not deleted"
        @click="() => { requestClearTemporary(); announce('Clear temporary items requested'); }"
      >
        <ActionIcon name="trash" />
      </button>
    </footer>
    <div v-if="status" class="status" role="alert">
      <span>{{ status }}</span>
      <button type="button" class="dismiss" aria-label="Dismiss status" @click="dismissStatus">×</button>
    </div>
    <div class="sr-only" role="status" aria-live="polite" aria-atomic="true">
      {{ announcement }}
    </div>
    <ShelfResizeHandles :disabled="busy || resizing" @resize="startResize" />
  </main>
</template>

<style scoped>
.shelf { position: relative; display: flex; flex-direction: column; height: 100%; box-sizing: border-box; border: 1px solid var(--dc-border-default); border-radius: 12px; color: var(--dc-text-primary); background: var(--dc-surface); font-family: "Segoe UI", sans-serif; }
.dragging-over { border-color: var(--dc-drag-hover-border); box-shadow: inset 0 0 0 1px var(--dc-drag-hover-border); background: var(--dc-drag-hover-bg); }
.resize-hover { border-color: var(--dc-drag-hover-border); box-shadow: inset 0 0 0 1px var(--dc-drag-hover-border); }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(164px, 1fr)); align-content: start; gap: 4px; padding: 7px; min-height: 0; overflow-x: hidden; overflow-y: auto; }
.empty { display: flex; flex: 1; flex-direction: column; align-items: center; justify-content: center; min-height: 70px; padding: 12px; text-align: center; }
.empty-icon { width: 32px; height: 32px; color: var(--dc-accent); }
.empty-text { margin: 8px 0 0; font-size: 13px; color: var(--dc-text-primary); }
.status { display: flex; align-items: start; gap: 4px; margin: auto 8px 8px; padding: 8px; border: 1px solid var(--dc-warning-border); background: var(--dc-warning-surface); color: var(--dc-warning-text); border-radius: 6px; font-size: 11px; overflow-wrap: anywhere; }
.dismiss { margin-left: auto; border: 0; color: inherit; background: none; cursor: pointer; }
.dismiss:focus-visible { outline: 2px solid var(--dc-focus); }
.footer { display: flex; justify-content: flex-start; margin-top: auto; padding: 4px 8px 8px; }
.clear-temporary-action { color: var(--dc-danger); }
.clear-temporary-action:hover:not(:disabled) { border-color: var(--dc-danger-border); background: var(--dc-danger-surface); color: var(--dc-danger); }

@media (forced-colors: active) {
  .shelf {
    border-color: ButtonText;
  }
  .clear-temporary-action:focus-visible, .dismiss:focus-visible {
    outline: 2px solid Highlight !important;
  }
}
</style>
