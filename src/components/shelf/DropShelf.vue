<script setup lang="ts">
import { useShelf } from "../../composables/useShelf";
import ShelfBatchCard from "./ShelfBatchCard.vue";
import BatchDragConfirmation from "./BatchDragConfirmation.vue";

const {
  batches,
  status,
  busy,
  draggingOver,
  pendingConfirmation,
  confirmBatchDrag,
  dragConfirmedBatch,
  cancelBatchDrag,
  removeBatch,
  setItemPinned,
  openManageItems,
  startItemDrag,
  startBatchDrag,
  dismissStatus,
} = useShelf();
</script>

<template>
  <main class="shelf" :class="{ 'dragging-over': draggingOver }" :aria-busy="busy">
    <header class="header">
      <h1 class="title">DropCove</h1>
      <span class="reference-label">References, not copies</span>
    </header>
    <section v-if="batches.length" class="grid" aria-label="Shelf batches">
      <ShelfBatchCard
        v-for="batch in batches"
        :key="batch.id"
        :batch="batch"
        :busy="busy"
        @remove="removeBatch"
        @pin-item="setItemPinned"
        @manage="openManageItems"
        @drag-item="startItemDrag"
        @drag-batch="startBatchDrag"
      />
    </section>
    <section v-else class="empty" aria-label="Empty Drop Shelf">
      <span class="empty-icon" aria-hidden="true">↓</span>
      <p class="empty-text">Drop a file or folder here</p>
      <p class="empty-hint">Drag it out later to copy</p>
    </section>
    <BatchDragConfirmation :pending="pendingConfirmation" @confirm="confirmBatchDrag" @cancel="cancelBatchDrag" @drag="dragConfirmedBatch" />
    <div v-if="status" class="status" role="alert">
      <span>{{ status }}</span>
      <button type="button" class="dismiss" aria-label="Dismiss status" @click="dismissStatus">×</button>
    </div>
  </main>
</template>

<style scoped>
.shelf { position: relative; display: flex; flex-direction: column; height: 100%; box-sizing: border-box; border: 1px solid #46536a; border-radius: 12px; color: #edf2fc; background: #141c2b; font-family: "Segoe UI", sans-serif; }
.dragging-over { border-color: #a5b4fc; box-shadow: inset 0 0 0 1px #a5b4fc; background: #202b44; }
.header { display: flex; align-items: center; justify-content: space-between; gap: 6px; padding: 10px; }
.title { margin: 0; font-size: 13px; }
.reference-label { color: #a7b4cb; font-size: 10px; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(164px, 1fr)); align-content: start; gap: 4px; padding: 0 7px 7px; min-height: 0; overflow-x: hidden; overflow-y: auto; }
.empty { display: flex; flex: 1; flex-direction: column; align-items: center; justify-content: center; min-height: 70px; padding: 12px; text-align: center; }
.empty-icon { font-size: 32px; color: #a5b4fc; }
.empty-text { margin: 8px 0 0; font-size: 13px; }
.empty-hint { margin: 5px 0; color: #a7b4cb; font-size: 11px; }
.status { display: flex; align-items: start; gap: 4px; margin: auto 8px 8px; padding: 8px; border: 1px solid #efad70; border-radius: 6px; font-size: 11px; overflow-wrap: anywhere; }
.dismiss { margin-left: auto; border: 0; color: inherit; background: none; cursor: pointer; }
.dismiss:focus-visible { outline: 2px solid #a5b4fc; }
</style>
