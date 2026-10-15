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
  <article
    ref="card"
    class="card relative flex items-center min-w-0 h-[48px] box-border border border-dc-border-default rounded-[8px] bg-dc-surface-elevated text-dc-text-primary transition-[opacity,transform] duration-[140ms,140ms] ease-[cubic-bezier(0.16,1,0.3,1)] forced-colors:border-[ButtonText]"
    :data-batch-id="batch.id"
  >
    <div
      class="card-drag flex flex-1 min-w-0 items-center gap-[4px] h-full box-border p-[5px] cursor-grab select-none touch-none active:cursor-grabbing"
      :title="batch.items.length === 1 ? batch.items[0].path : title"
      :aria-label="batch.items.length === 1 ? `Drag ${title} to copy` : `Drag batch ${title} to copy`"
      @pointerdown.prevent="pointerDown"
      @pointermove="pointerMove"
      @pointerup="pointerStart = undefined"
      @pointercancel="pointerStart = undefined"
      @dragstart.prevent
    >
      <span class="card-icon flex flex-none items-center leading-none [&_.shell-visual+.shell-visual]:-ml-[14px]" aria-hidden="true">
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
      <span class="card-info flex-1 min-w-0">
        <span class="card-name block overflow-hidden text-ellipsis whitespace-nowrap text-[12px] text-dc-text-primary">{{ title }}</span>
      </span>
    </div>
    <div class="card-actions flex flex-none items-center gap-[2px] pr-[4px]">
    <button
      v-if="singleItem"
      class="pin inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-dc-border-default rounded-[4px] bg-dc-surface-elevated text-dc-text-secondary cursor-pointer [&:hover:not(:disabled)]:border-dc-border-hover [&:hover:not(:disabled)]:bg-dc-surface-hover [&:hover:not(:disabled)]:text-dc-text-primary aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
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
        class="manage inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-dc-border-default rounded-[4px] bg-dc-surface-elevated text-dc-text-secondary cursor-pointer [&:hover:not(:disabled)]:border-dc-border-hover [&:hover:not(:disabled)]:bg-dc-surface-hover [&:hover:not(:disabled)]:text-dc-text-primary aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
        type="button"
        :disabled="busy"
        :aria-label="`Manage ${title}`"
        title="Manage Items"
        @click="openManagement(false)"
        @keydown="onManageKeydown"
      ><ActionIcon name="list" /></button>
    </template>
    <button
      class="remove inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-dc-border-default rounded-[4px] bg-dc-surface-elevated text-dc-text-secondary cursor-pointer [&.remove:hover:not(:disabled)]:border-dc-danger-border [&:hover:not(:disabled)]:bg-dc-surface-hover [&.remove:hover:not(:disabled)]:text-dc-danger aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
      type="button"
      :disabled="busy"
      :aria-label="`Remove batch ${title}`"
      title="Remove reference; the source is not deleted"
      @click="emit('remove', batch.id)"
    ><ActionIcon name="remove" /></button>
    </div>
  </article>
</template>

