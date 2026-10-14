<script setup lang="ts">
import type { ShelfItem } from "../../composables/useShelf";
import ShellVisual from "./ShellVisual.vue";

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
        class="action pin"
        type="button"
        :disabled="busy"
        :aria-label="`${item.isPinned ? 'Unpin' : 'Pin'} ${item.name}`"
        :aria-pressed="item.isPinned"
        @click="emit('pin', item.id, !item.isPinned)"
      >{{ item.isPinned ? "Unpin" : "Pin" }}</button>
      <button
        class="action remove"
        type="button"
        :disabled="busy"
        :aria-label="`Remove ${item.name}`"
        title="Remove reference; the source is not deleted"
        @click="emit('remove', item.id)"
      >Remove</button>
    </div>
  </li>
</template>

<style scoped>
.item-row { min-width: 0; border: 1px solid var(--dc-border-default); border-radius: 7px; padding: 6px; background: var(--dc-surface-elevated); color: var(--dc-text-primary); }
.item-drag { display: flex; align-items: center; gap: 7px; cursor: grab; user-select: none; touch-action: none; }
.item-drag:active { cursor: grabbing; }
.item-icon { flex-shrink: 0; color: var(--dc-accent); font-size: 20px; }
.item-info { display: flex; min-width: 0; flex-direction: column; gap: 2px; }
.item-name, .item-path { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.item-name { font-size: 12px; color: var(--dc-text-primary); }
.item-path { color: var(--dc-text-muted); font-size: 10px; }
.item-actions { display: flex; align-items: center; gap: 5px; margin-top: 5px; }
.availability { flex: 1; color: var(--dc-text-muted); font-size: 10px; }
.action { border: 1px solid var(--dc-border-default); border-radius: 4px; padding: 2px 5px; color: inherit; background: transparent; font: inherit; font-size: 10px; cursor: pointer; }
.action:hover { border-color: var(--dc-border-hover); background: var(--dc-surface-hover); }
.pin[aria-pressed="true"] { border-color: var(--dc-accent); color: var(--dc-accent); }
.action:focus-visible { outline: 2px solid var(--dc-focus); }
.action:disabled { opacity: var(--dc-disabled-opacity); cursor: default; }

@media (forced-colors: active) {
  .item-row {
    border-color: ButtonText;
  }
  .action {
    border-color: ButtonText;
    background: ButtonFace;
    color: ButtonText;
  }
  .pin[aria-pressed="true"] {
    border-color: Highlight;
    color: Highlight;
  }
  .action:focus-visible {
    outline: 2px solid Highlight !important;
  }
}
</style>
