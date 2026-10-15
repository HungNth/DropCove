<script setup lang="ts">
import type { ShelfItem } from "../../composables/useShelf";
import ShellVisual from "./ShellVisual.vue";
import ActionIcon from "./ActionIcon.vue";

const props = defineProps<{ item: ShelfItem; busy: boolean }>();
const emit = defineEmits<{
  pin: [itemId: string, pinned: boolean];
  remove: [itemId: string];
  drag: [itemId: string];
}>();
let pointerStart: { id: number; x: number; y: number } | undefined;

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
  emit("drag", props.item.id);
}
</script>

<template>
  <li class="item-row" :data-item-id="item.id" :aria-label="`${item.name}, ${item.path}, ${item.availability}`">
    <div
      class="item-drag"
      :title="item.path"
      :aria-label="`Drag ${item.path} to copy`"
      @pointerdown.prevent="pointerDown"
      @pointermove="pointerMove"
      @pointerup="pointerStart = undefined"
      @pointercancel="pointerStart = undefined"
      @dragstart.prevent
    >
      <ShellVisual
        :item-id="item.id"
        :is-folder="item.isFolder"
        :name="item.name"
        :size="32"
        surface="manageItems"
      />
      <span class="item-info">
        <span class="item-name">{{ item.name }}</span>
        <span class="item-path">{{ item.path }}</span>
      </span>
    </div>
    <div class="item-actions">
      <span class="availability">{{ item.isPinned ? "Pinned · " : "" }}{{ item.availability }}</span>
      <button
        class="icon-action action pin"
        type="button"
        :disabled="busy"
        :aria-label="`${item.isPinned ? 'Unpin' : 'Pin'} ${item.name}`"
        :aria-pressed="item.isPinned"
        :title="item.isPinned ? 'Make temporary; consumed after successful Copy' : 'Keep this reference after successful Copy'"
        @click="emit('pin', item.id, !item.isPinned)"
      ><ActionIcon name="pin" :filled="item.isPinned" /></button>
      <button
        class="icon-action action remove"
        type="button"
        :disabled="busy"
        :aria-label="`Remove ${item.name}`"
        title="Remove reference; the source is not deleted"
        @click="emit('remove', item.id)"
      ><ActionIcon name="remove" /></button>
    </div>
  </li>
</template>

<style scoped>
.item-row { display: flex; align-items: center; gap: 5px; min-width: 0; height: 54px; box-sizing: border-box; border: 1px solid var(--dc-border-default); border-radius: 7px; padding: 5px; background: var(--dc-surface-elevated); color: var(--dc-text-primary); }
.item-drag { display: flex; flex: 1; min-width: 0; align-items: center; gap: 5px; cursor: grab; user-select: none; touch-action: none; }
.item-drag:active { cursor: grabbing; }
.item-icon { flex-shrink: 0; color: var(--dc-accent); font-size: 20px; }
.item-info { display: flex; min-width: 0; flex-direction: column; gap: 2px; }
.item-name, .item-path { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.item-name { font-size: 12px; color: var(--dc-text-primary); }
.item-path { color: var(--dc-text-muted); font-size: 10px; }
.item-actions { display: flex; flex: none; align-items: center; gap: 3px; }
.availability { max-width: 60px; color: var(--dc-text-muted); font-size: 10px; overflow-wrap: anywhere; }

@media (forced-colors: active) {
  .item-row {
    border-color: ButtonText;
  }
}
</style>
