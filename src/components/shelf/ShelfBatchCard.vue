<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ShelfAnchor, ShelfBatch } from "../../composables/useShelf";
import BulkPinning from "./BulkPinning.vue";
import ShellVisual from "./ShellVisual.vue";
import ActionIcon from "./ActionIcon.vue";

const props = defineProps<{ batch: ShelfBatch; busy: boolean }>();
const emit = defineEmits<{
  remove: [batchId: string];
  dragItem: [itemId: string];
  dragBatch: [batchId: string];
  pinItem: [itemId: string, pinned: boolean];
  pinBatch: [batchId: string, pinned: boolean];
  manage: [batchId: string, anchor: ShelfAnchor, keyboard: boolean];
}>();
let pointerStart: { id: number; x: number; y: number } | undefined;
const title = computed(() => props.batch.items.length === 1 ? props.batch.items[0].name : `${props.batch.items.length} items`);
const previews = computed(() => props.batch.items.slice(0, 3));
const singleItem = computed(() => props.batch.items.length === 1 ? props.batch.items[0] : undefined);
const card = useTemplateRef<HTMLElement>("card");

function openManagement(keyboard: boolean) {
  const bounds = card.value?.getBoundingClientRect();
  if (bounds) emit("manage", props.batch.id, { x: bounds.x, y: bounds.y, width: bounds.width, height: bounds.height }, keyboard);
}

function onManageKeydown(event: KeyboardEvent) {
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    openManagement(true);
  }
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
  <article ref="card" class="card" :data-batch-id="batch.id">
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
        <ShellVisual
          v-if="singleItem"
          :item-id="singleItem.id"
          :is-folder="singleItem.isFolder"
          :name="singleItem.name"
          :size="32"
        />
        <template v-else>
          <ShellVisual
            v-for="item in previews"
            :key="item.id"
            :item-id="item.id"
            :is-folder="item.isFolder"
            :name="item.name"
            :size="24"
          />
        </template>
      </span>
      <span class="card-info">
        <span class="card-name">{{ title }}</span>
      </span>
    </div>
    <div class="card-actions">
    <button
      v-if="singleItem"
      class="icon-action pin"
      type="button"
      :disabled="busy"
      :aria-pressed="singleItem.isPinned"
      :aria-label="`${singleItem.isPinned ? 'Unpin' : 'Pin'} ${singleItem.name}`"
      :title="singleItem.isPinned ? 'Make temporary; consumed after successful Copy' : 'Keep this reference after successful Copy'"
      @click="emit('pinItem', singleItem.id, !singleItem.isPinned)"
    ><ActionIcon name="pin" :filled="singleItem.isPinned" /></button>
    <template v-else>
      <BulkPinning
        :batch="batch"
        :busy="busy"
        @pin="(id, pinned) => emit('pinBatch', id, pinned)"
      />
      <button
        class="icon-action manage"
        type="button"
        :disabled="busy"
        :aria-label="`Manage ${title}`"
        title="Manage Items"
        @click="openManagement(false)"
        @keydown="onManageKeydown"
      ><ActionIcon name="list" /></button>
    </template>
    <button
      class="icon-action remove"
      type="button"
      :disabled="busy"
      :aria-label="`Remove batch ${title}`"
      title="Remove reference; the source is not deleted"
      @click="emit('remove', batch.id)"
    ><ActionIcon name="remove" /></button>
    </div>
  </article>
</template>

<style scoped>
.card { position: relative; display: flex; align-items: center; min-width: 0; height: 48px; box-sizing: border-box; border: 1px solid var(--dc-border-default); border-radius: 8px; background: var(--dc-surface-elevated); color: var(--dc-text-primary); }
.card-drag { display: flex; flex: 1; min-width: 0; align-items: center; gap: 4px; height: 100%; box-sizing: border-box; padding: 5px; cursor: grab; user-select: none; touch-action: none; }
.card-drag:active { cursor: grabbing; }
.card-icon { display: flex; flex: none; align-items: center; line-height: 1; }
.card-icon :deep(.shell-visual + .shell-visual) { margin-left: -14px; }
.card-info { flex: 1; min-width: 0; }
.card-name { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; color: var(--dc-text-primary); }
.card-actions { display: flex; flex: none; align-items: center; gap: 2px; padding-right: 4px; }

@media (forced-colors: active) {
  .card {
    border-color: ButtonText;
  }
}
</style>
