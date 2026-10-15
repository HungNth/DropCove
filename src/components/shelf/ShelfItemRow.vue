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
  <li
    class="item-row flex items-center gap-[5px] min-w-0 h-[54px] box-border border border-dc-border-default rounded-[7px] p-[5px] bg-dc-surface-elevated text-dc-text-primary transition-[opacity,transform] duration-[140ms,140ms] ease-[cubic-bezier(0.16,1,0.3,1)] forced-colors:border-[ButtonText]"
    :data-item-id="item.id"
    :aria-label="`${item.name}, ${item.path}, ${item.availability}`"
  >
    <div
      class="item-drag flex flex-1 min-w-0 items-center gap-[5px] cursor-grab select-none touch-none active:cursor-grabbing"
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
      <span class="item-info flex min-w-0 flex-col gap-[2px]">
        <span class="item-name overflow-hidden text-ellipsis whitespace-nowrap text-[12px] text-dc-text-primary">{{ item.name }}</span>
        <span class="item-path overflow-hidden text-ellipsis whitespace-nowrap text-dc-text-muted text-[10px]">{{ item.path }}</span>
      </span>
    </div>
    <div class="item-actions flex flex-none items-center gap-[3px]">
      <span class="availability max-w-[60px] text-dc-text-muted text-[10px] [overflow-wrap:anywhere]">{{ item.isPinned ? "Pinned · " : "" }}{{ item.availability }}</span>
      <button
        class="action pin inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-dc-border-default rounded-[4px] bg-dc-surface-elevated text-dc-text-secondary cursor-pointer [&:hover:not(:disabled)]:border-dc-border-hover [&:hover:not(:disabled)]:bg-dc-surface-hover [&:hover:not(:disabled)]:text-dc-text-primary aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
        type="button"
        :disabled="busy"
        :aria-label="`${item.isPinned ? 'Unpin' : 'Pin'} ${item.name}`"
        :aria-pressed="item.isPinned"
        :title="item.isPinned ? 'Make temporary; consumed after successful Copy' : 'Keep this reference after successful Copy'"
        @click="emit('pin', item.id, !item.isPinned)"
      ><ActionIcon name="pin" :filled="item.isPinned" /></button>
      <button
        class="action remove inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-dc-border-default rounded-[4px] bg-dc-surface-elevated text-dc-text-secondary cursor-pointer [&.remove:hover:not(:disabled)]:border-dc-danger-border [&:hover:not(:disabled)]:bg-dc-surface-hover [&.remove:hover:not(:disabled)]:text-dc-danger aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
        type="button"
        :disabled="busy"
        :aria-label="`Remove ${item.name}`"
        title="Remove reference; the source is not deleted"
        @click="emit('remove', item.id)"
      ><ActionIcon name="remove" /></button>
    </div>
  </li>
  </template>

