<script setup lang="ts">
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ShelfAnchor, ShelfBatch } from "../../composables/useShelf";
import BulkPinning from "./BulkPinning.vue";
import ShellVisual from "./ShellVisual.vue";
import ActionIcon from "./ActionIcon.vue";
import ItemActionsMenu, { type ShelfItemAction } from "./ItemActionsMenu.vue";

const props = defineProps<{
  batch: ShelfBatch;
  busy: boolean;
  selected?: boolean;
  selectionTabindex?: number;
  menuOpenItemId?: string | null;
  noActivate?: boolean;
}>();
const emit = defineEmits<{
  remove: [batchId: string];
  dragBatch: [batchId: string];
  select: [batchId: string, modifiers: { ctrlKey: boolean; shiftKey: boolean }];
  focusBatch: [batchId: string];
  pinItem: [itemId: string, pinned: boolean];
  pinBatch: [batchId: string, pinned: boolean];
  manage: [batchId: string, anchor: ShelfAnchor, keyboard: boolean];
  openItemMenu: [itemId: string];
  closeItemMenu: [itemId: string];
  itemAction: [itemId: string, action: ShelfItemAction];
  selectionMenu: [batchId: string, anchor: { x: number; y: number }, keyboard: boolean];
}>();
let pointerStart: { id: number; x: number; y: number; ctrlKey: boolean; shiftKey: boolean } | undefined;
const singleItem = computed(() => props.batch.items.length === 1 ? props.batch.items[0] : undefined);
const pointerFocus = shallowRef(false);
const title = computed(() => props.batch.items.length === 1 ? props.batch.items[0].name : `${props.batch.items.length} items`);
const fullText = shallowRef<string | null>(null);
const isManagedText = computed(() => singleItem.value?.ownership === "Managed" && singleItem.value?.managedContentKind === "Text");
const isManagedImage = computed(() => singleItem.value?.ownership === "Managed" && singleItem.value?.managedContentKind === "Image");
const hasItemActions = computed(() => singleItem.value && (singleItem.value.ownership === "External" || isManagedText.value));

const hoverTitle = computed(() => {
  if (props.batch.items.length !== 1 || !singleItem.value) return title.value;
  const item = singleItem.value;
  if (item.ownership === "External") {
    return item.path;
  }
  if (isManagedText.value) {
    return fullText.value ?? item.name;
  }
  if (isManagedImage.value) {
    return `${item.name} (DropCove-created image)`;
  }
  return item.name;
});

const dragAriaLabel = computed(() => {
  if (props.batch.items.length === 1 && singleItem.value) {
    return `Drag ${title.value} to copy`;
  }
  return `Drag batch ${title.value} to copy`;
});

watch(() => singleItem.value?.id, () => {
  fullText.value = null;
});

async function loadFullText() {
  if (props.batch.items.length !== 1 || !singleItem.value || fullText.value !== null || !isManagedText.value) return;
  try {
    const text = await invoke<string | null>("get_managed_text", { itemId: singleItem.value.id });
    if (text !== null && text !== undefined) {
      fullText.value = text.length > 500 ? text.slice(0, 500) + "..." : text;
    }
  } catch {}
}
const previews = computed(() => props.batch.items.slice(0, 3));
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
  pointerStart = { id: event.pointerId, x: event.clientX, y: event.clientY, ctrlKey: event.ctrlKey, shiftKey: event.shiftKey };
  pointerFocus.value = true;
  emit("focusBatch", props.batch.id);
  (event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);
}

function pointerMove(event: PointerEvent) {
  if (!pointerStart || event.pointerId !== pointerStart.id) return;
  if (!(event.buttons & 1)) { pointerStart = undefined; return; }
  if (Math.hypot(event.clientX - pointerStart.x, event.clientY - pointerStart.y) < 6) return;
  pointerStart = undefined;
  (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
  emit("dragBatch", props.batch.id);
}

function pointerUp(event: PointerEvent) {
  const start = pointerStart;
  pointerStart = undefined;
  if (!start || start.id !== event.pointerId || props.busy) return;
  emit("select", props.batch.id, { ctrlKey: start.ctrlKey, shiftKey: start.shiftKey });
}
function onBlur() {
  pointerFocus.value = false;
}

function onKeydown(event: KeyboardEvent) {
  if (["Shift", "Control", "Alt", "Meta"].includes(event.key)) return;
  pointerFocus.value = false;
}

// Only the selection region opens Selection Actions; independent controls stay isolated.
function onContextmenu(event: MouseEvent) {
  if (event.button !== 2) return;
  const target = event.target as HTMLElement | null;
  if (target?.closest?.(".card-actions, .item-actions-menu, input, textarea")) return;
  event.preventDefault();
  if (props.busy) return;
  emit("selectionMenu", props.batch.id, { x: event.clientX, y: event.clientY }, false);
}

function onSelectionMenuKey(event: KeyboardEvent) {
  if (!(event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey))) return;
  event.preventDefault();
  event.stopPropagation();
  if (props.busy) return;
  const bounds = card.value?.getBoundingClientRect();
  if (bounds) emit("selectionMenu", props.batch.id, { x: bounds.x, y: bounds.bottom }, true);
}
</script>

<template>
  <article
    ref="card"
    class="card relative flex items-center min-w-0 h-[48px] box-border border border-dc-border-default rounded-[8px] bg-dc-card shadow-dc-card text-dc-text-primary transition-[opacity,transform] duration-[140ms,140ms] ease-[cubic-bezier(0.16,1,0.3,1)] forced-colors:border-[ButtonText]"
    :data-batch-id="batch.id"
    :data-selected="selected || undefined"
    :aria-selected="selectionTabindex !== undefined ? Boolean(selected) : undefined"
    @contextmenu="onContextmenu"
  >
    <div
      class="card-drag flex flex-1 min-w-0 items-center gap-[4px] h-full box-border p-[5px] cursor-grab select-none touch-none active:cursor-grabbing"
      :title="hoverTitle"
      :aria-label="dragAriaLabel"
      :data-pointer-focus="pointerFocus || undefined"
      :role="selectionTabindex !== undefined ? 'gridcell' : undefined"
      :tabindex="selectionTabindex"
      @focus="emit('focusBatch', batch.id)"
      @blur="onBlur"
      @keydown="event => { onSelectionMenuKey(event); onKeydown(event); }"
      @pointerenter="loadFullText"
      @pointerdown="pointerDown"
      @pointermove="pointerMove"
      @pointerup="pointerUp"
      @pointercancel="pointerStart = undefined"
      @dragstart.prevent
    >
      <span class="card-icon relative isolate flex flex-none items-center leading-none pointer-events-none" :class="{ 'size-[30px]': !singleItem }" aria-hidden="true">
        <ShellVisual
          v-if="singleItem"
          :item-id="singleItem.id"
          :is-folder="singleItem.isFolder"
          :name="singleItem.name"
          :size="32"
        />
        <template v-else>
          <ShellVisual
            v-for="(item, index) in previews"
            :key="item.id"
            class="absolute"
            :style="{ left: `${index * 3}px`, bottom: `${index * 3}px`, zIndex: previews.length - index }"
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
    <div class="card-actions flex flex-none items-center gap-[2px] pr-[4px]" :role="selectionTabindex !== undefined ? 'gridcell' : undefined">
    <button
      v-if="singleItem"
      class="pin inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-text-secondary cursor-pointer [&:hover:not(:disabled)]:border-dc-border-hover [&:hover:not(:disabled)]:bg-dc-surface-hover [&:hover:not(:disabled)]:text-dc-text-primary aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
      type="button"
      :disabled="busy"
      :aria-pressed="singleItem.isPinned"
      :aria-label="`${singleItem.isPinned ? 'Unpin' : 'Pin'} ${singleItem.name}`"
      :title="singleItem.isPinned ? 'Make temporary; consumed after successful Copy' : 'Keep this reference after successful Copy'"
      @click="emit('pinItem', singleItem.id, !singleItem.isPinned)"
    ><ActionIcon name="pin" :filled="singleItem.isPinned" /></button>
    <ItemActionsMenu
      v-if="singleItem && hasItemActions"
      :item="singleItem"
      :disabled="busy"
      :is-open="menuOpenItemId === singleItem.id"
      :no-activate="noActivate"
      @open="emit('openItemMenu', singleItem.id)"
      @close="emit('closeItemMenu', singleItem.id)"
      @action="singleItem && emit('itemAction', singleItem.id, $event)"
    />
    <template v-if="!singleItem">
      <BulkPinning
        :batch="batch"
        :busy="busy"
        @pin="(id, pinned) => emit('pinBatch', id, pinned)"
      />
      <button
        class="manage inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-text-secondary cursor-pointer [&:hover:not(:disabled)]:border-dc-border-hover [&:hover:not(:disabled)]:bg-dc-surface-hover [&:hover:not(:disabled)]:text-dc-text-primary aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
        type="button"
        :disabled="busy"
        :aria-label="`Manage ${title}`"
        title="Manage Items"
        @click="openManagement(false)"
        @keydown="onManageKeydown"
      ><ActionIcon name="list" /></button>
    </template>
    <button
      class="remove inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-text-secondary cursor-pointer [&.remove:hover:not(:disabled)]:border-dc-danger-border [&:hover:not(:disabled)]:bg-dc-surface-hover [&.remove:hover:not(:disabled)]:text-dc-danger aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
      type="button"
      :disabled="busy"
      :aria-label="`Remove batch ${title}`"
      title="Remove batch and its DropCove-created files; external sources stay"
      @click="emit('remove', batch.id)"
    ><ActionIcon name="remove" /></button>
    </div>
  </article>
</template>


<style scoped>
.card[data-selected] {
  border-color: var(--dc-accent);
  background: var(--dc-accent-subtle);
}
.card-drag:focus-visible:not([data-pointer-focus]) {
  outline: 2px solid var(--dc-focus);
  outline-offset: -3px;
  border-radius: 6px;
}
.card-drag[data-pointer-focus]:focus-visible {
  outline: none !important;
}
@media (forced-colors: active) {
  .card[data-selected] {
    forced-color-adjust: none;
    border-color: Highlight;
    background: Highlight;
    color: HighlightText;
  }
  .card[data-selected] :deep(.card-name) { color: HighlightText; }
  .card[data-selected] .card-drag:focus-visible:not([data-pointer-focus]) { outline-color: HighlightText !important; }
}
</style>
