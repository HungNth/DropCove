<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ShelfAnchor, ShelfBatch } from "../../composables/useShelf";

const props = defineProps<{ batch: ShelfBatch; busy: boolean }>();
const emit = defineEmits<{
  remove: [batchId: string];
  dragItem: [itemId: string];
  dragBatch: [batchId: string];
  pinItem: [itemId: string, pinned: boolean];
  manage: [batchId: string, anchor: ShelfAnchor];
}>();
let pointerStart: { id: number; x: number; y: number } | undefined;
const title = computed(() => props.batch.items.length === 1 ? props.batch.items[0].name : `${props.batch.items.length} items`);
const previews = computed(() => props.batch.items.slice(0, 3));
const singleItem = computed(() => props.batch.items.length === 1 ? props.batch.items[0] : undefined);
const card = useTemplateRef<HTMLElement>("card");

function openManagement() {
  const bounds = card.value?.getBoundingClientRect();
  if (bounds) emit("manage", props.batch.id, { x: bounds.x, y: bounds.y, width: bounds.width, height: bounds.height });
}

function pointerDown(event: PointerEvent) {
  if (event.button !== 0 || props.busy) return;
  pointerStart = { id: event.pointerId, x: event.clientX, y: event.clientY };
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
}

function pointerMove(event: PointerEvent) {
  if (!pointerStart || event.pointerId !== pointerStart.id) return;
  if (!(event.buttons & 1)) { pointerStart = undefined; return; }
  if (Math.hypot(event.clientX - pointerStart.x, event.clientY - pointerStart.y) < 6) return;
  pointerStart = undefined;
  (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
  if (props.batch.items.length === 1) {
    emit("dragItem", props.batch.items[0].id);
  } else {
    emit("dragBatch", props.batch.id);
  }
}
</script>

<template>
  <article ref="card" class="card" :class="{ 'single-item': singleItem, 'multi-item': !singleItem }">
    <div
      class="card-drag"
      :title="batch.items.length === 1 ? batch.items[0].path : title"
      :aria-label="batch.items.length === 1 ? `Drag ${title} to copy` : `Drag batch ${title} to copy`"
      @pointerdown.prevent="pointerDown"
      @pointermove="pointerMove"
      @pointerup="pointerStart = undefined"
      @pointercancel="pointerStart = undefined"
      @dragstart.prevent
    >
      <span class="card-icon" aria-hidden="true">
        <span v-for="item in previews" :key="item.id">{{ item.isFolder ? "▰" : "▤" }}</span>
      </span>
      <span class="card-info">
        <span class="card-name">{{ title }}</span>
        <span class="card-hint">{{ singleItem?.isPinned ? "Pinned · Drag to copy" : batch.items.length === 1 ? "Drag to copy" : "Drag batch to copy" }}</span>
      </span>
    </div>
    <button
      v-if="singleItem"
      class="pin"
      type="button"
      :disabled="busy"
      :aria-pressed="singleItem.isPinned"
      :aria-label="`${singleItem.isPinned ? 'Unpin' : 'Pin'} ${singleItem.name}`"
      :title="singleItem.isPinned ? 'Make temporary; consumed after successful Copy' : 'Keep this reference after successful Copy'"
      @click="emit('pinItem', singleItem.id, !singleItem.isPinned)"
    >{{ singleItem.isPinned ? "Unpin" : "Pin" }}</button>
    <button
      v-else
      class="manage"
      type="button"
      :disabled="busy"
      :aria-label="`Manage ${title}`"
      @click="openManagement"
    >Manage Items</button>
    <button
      class="remove"
      type="button"
      :disabled="busy"
      :aria-label="`Remove batch ${title}`"
      title="Remove reference; the source is not deleted"
      @click="emit('remove', batch.id)"
    >×</button>
  </article>
</template>

<style scoped>
.card { position: relative; min-width: 0; border: 1px solid #3b475c; border-radius: 10px; background: #202a3b; }
.card-drag { display: flex; align-items: center; gap: 8px; min-height: 54px; box-sizing: border-box; padding: 8px 24px 8px 8px; cursor: grab; user-select: none; touch-action: none; }
.card-drag:active { cursor: grabbing; }
.card-icon { display: flex; color: #a5b4fc; font-size: 18px; letter-spacing: -6px; padding-right: 6px; line-height: 1; }
.card-info { display: flex; flex: 1; min-width: 0; flex-direction: column; gap: 3px; }
.card-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; }
.card-hint { color: #a7b4cb; font-size: 10px; }
.remove { position: absolute; top: 3px; right: 3px; border: 0; padding: 2px 6px; color: #cbd5e1; background: transparent; font-size: 18px; cursor: pointer; }
.single-item .card-drag { padding-right: 48px; }
.pin { position: absolute; bottom: 4px; right: 4px; border: 1px solid #46536a; border-radius: 4px; padding: 2px 4px; color: #cbd5e1; background: #202a3b; font: inherit; font-size: 10px; cursor: pointer; }
.pin[aria-pressed="true"] { border-color: #a5b4fc; color: #a5b4fc; }
.multi-item .card-drag { padding-bottom: 26px; }
.manage { position: absolute; bottom: 4px; right: 4px; border: 1px solid #46536a; border-radius: 4px; padding: 2px 4px; color: #cbd5e1; background: #202a3b; font: inherit; font-size: 10px; cursor: pointer; }
.remove:focus-visible, .pin:focus-visible, .manage:focus-visible { outline: 2px solid #a5b4fc; border-radius: 4px; }
.remove:disabled, .pin:disabled, .manage:disabled { opacity: .45; cursor: default; }
</style>
