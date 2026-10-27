<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ShelfItem } from "../../composables/useShelf";
import ShellVisual from "./ShellVisual.vue";
import ActionIcon from "./ActionIcon.vue";
import ItemActionsMenu, { type ShelfItemAction } from "./ItemActionsMenu.vue";

const props = defineProps<{
  item: ShelfItem;
  busy: boolean;
  selected?: boolean;
  selectionTabindex?: number;
  menuOpen?: boolean;
  noActivate?: boolean;
}>();
const emit = defineEmits<{
  pin: [itemId: string, pinned: boolean];
  remove: [itemId: string];
  drag: [itemId: string];
  select: [itemId: string, modifiers: { ctrlKey: boolean; shiftKey: boolean }];
  focusItem: [itemId: string];
  openMenu: [itemId: string];
  closeMenu: [itemId: string];
  action: [itemId: string, action: ShelfItemAction];
  selectionMenu: [itemId: string, anchor: { x: number; y: number }, keyboard: boolean];
}>();
let pointerStart: { id: number; x: number; y: number; ctrlKey: boolean; shiftKey: boolean } | undefined;
const pointerFocus = shallowRef(false);

function pointerDown(event: PointerEvent) {
  if (event.button !== 0 || props.busy) return;
  pointerStart = { id: event.pointerId, x: event.clientX, y: event.clientY, ctrlKey: event.ctrlKey, shiftKey: event.shiftKey };
  pointerFocus.value = true;
  emit("focusItem", props.item.id);
  (event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);
}

function pointerMove(event: PointerEvent) {
  if (!pointerStart || event.pointerId !== pointerStart.id) return;
  if (!(event.buttons & 1)) { pointerStart = undefined; return; }
  if (Math.hypot(event.clientX - pointerStart.x, event.clientY - pointerStart.y) < 6) return;
  pointerStart = undefined;
  (event.currentTarget as HTMLElement).releasePointerCapture?.(event.pointerId);
  emit("drag", props.item.id);
}

function pointerUp(event: PointerEvent) {
  const start = pointerStart;
  pointerStart = undefined;
  if (!start || start.id !== event.pointerId || props.busy) return;
  emit("select", props.item.id, { ctrlKey: start.ctrlKey, shiftKey: start.shiftKey });
}

function onBlur() {
  pointerFocus.value = false;
}

// Only the selection region opens Selection Actions; the browser menu stays suppressed everywhere on the row.
function onContextmenu(event: MouseEvent) {
  if (event.button !== 2) return;
  event.preventDefault();
  const target = event.target as HTMLElement | null;
  if (target?.closest?.(".item-actions, .item-actions-menu, input, textarea")) return;
  if (props.busy) return;
  emit("selectionMenu", props.item.id, { x: event.clientX, y: event.clientY }, false);
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey)) {
    event.preventDefault();
    event.stopPropagation();
    if (props.busy) return;
    const bounds = (event.currentTarget as HTMLElement | null)?.getBoundingClientRect?.();
    emit("selectionMenu", props.item.id, { x: bounds?.x ?? 0, y: bounds?.bottom ?? 0 }, true);
    return;
  }
  if (["Shift", "Control", "Alt", "Meta"].includes(event.key)) return;
  pointerFocus.value = false;
}
const fullText = shallowRef<string | null>(null);
const isManagedText = computed(() => props.item.ownership === "Managed" && props.item.managedContentKind === "Text");
const isManagedImage = computed(() => props.item.ownership === "Managed" && props.item.managedContentKind === "Image");
const hasItemActions = computed(() => props.item.ownership === "External" || isManagedText.value);

const rowAriaLabel = computed(() => {
  if (props.item.ownership === "External") {
    return `${props.item.name}, ${props.item.path}, ${props.item.availability}`;
  }
  if (isManagedImage.value) {
    return `${props.item.name}, DropCove-created image, ${props.item.availability}`;
  }
  return `${props.item.name}, ${props.item.availability}`;
});

const hoverTitle = computed(() => {
  if (props.item.ownership === "External") {
    return props.item.path;
  }
  if (isManagedText.value) {
    return fullText.value ?? props.item.name;
  }
  if (isManagedImage.value) {
    return `${props.item.name} (DropCove-created image)`;
  }
  return props.item.name;
});

const dragAriaLabel = computed(() => {
  if (props.item.ownership === "External") {
    return `Drag ${props.item.path} to copy`;
  }
  return `Drag ${props.item.name} to copy`;
});

watch(() => props.item.id, () => {
  fullText.value = null;
});

async function loadFullText() {
  if (fullText.value !== null || !isManagedText.value) return;
  try {
    const text = await invoke<string | null>("get_managed_text", { itemId: props.item.id });
    if (text !== null && text !== undefined) {
      fullText.value = text.length > 500 ? text.slice(0, 500) + "..." : text;
    }
  } catch {}
}
</script>

<template>
  <li
    class="item-row flex flex-none items-center gap-[5px] min-w-0 h-[54px] box-border border border-dc-border-default rounded-[7px] p-[5px] bg-dc-surface-elevated text-dc-text-primary transition-[opacity,transform] duration-[140ms,140ms] ease-[cubic-bezier(0.16,1,0.3,1)] forced-colors:border-[ButtonText]"
    :data-item-id="item.id"
    :data-selected="selected || undefined"
    :aria-selected="selectionTabindex !== undefined ? Boolean(selected) : undefined"
    :aria-label="rowAriaLabel"
    @contextmenu="onContextmenu"
  >
    <div
      class="item-drag flex flex-1 min-w-0 items-center gap-[5px] cursor-grab select-none touch-none active:cursor-grabbing"
      :title="hoverTitle"
      :aria-label="dragAriaLabel"
      :data-pointer-focus="pointerFocus || undefined"
      :role="selectionTabindex !== undefined ? 'gridcell' : undefined"
      :tabindex="selectionTabindex"
      @focus="emit('focusItem', item.id)"
      @blur="onBlur"
      @keydown="onKeydown"
      @pointerenter="loadFullText"
      @pointerdown="pointerDown"
      @pointermove="pointerMove"
      @pointerup="pointerUp"
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
        <span class="availability whitespace-nowrap text-dc-text-muted text-[10px]">{{ item.availability }}</span>
      </span>
    </div>
    <div class="item-actions flex flex-none items-center gap-[3px]" :role="selectionTabindex !== undefined ? 'gridcell' : undefined">
      <button
        class="action pin inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-text-secondary cursor-pointer [&:hover:not(:disabled)]:border-dc-border-hover [&:hover:not(:disabled)]:bg-dc-surface-hover [&:hover:not(:disabled)]:text-dc-text-primary aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
        type="button"
        :disabled="busy"
        :aria-label="`${item.isPinned ? 'Unpin' : 'Pin'} ${item.name}`"
        :aria-pressed="item.isPinned"
        :title="item.isPinned ? 'Make temporary; consumed after successful Copy' : 'Keep this reference after successful Copy'"
        @click="emit('pin', item.id, !item.isPinned)"
      ><ActionIcon name="pin" :filled="item.isPinned" /></button>
      <ItemActionsMenu
        v-if="hasItemActions"
        :item="item"
        :disabled="busy"
        :is-open="menuOpen"
        :no-activate="noActivate"
        @open="emit('openMenu', item.id)"
        @close="emit('closeMenu', item.id)"
        @action="a => emit('action', item.id, a)"
      />
      <button
        class="action remove inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-text-secondary cursor-pointer [&.remove:hover:not(:disabled)]:border-dc-danger-border [&:hover:not(:disabled)]:bg-dc-surface-hover [&.remove:hover:not(:disabled)]:text-dc-danger aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
        type="button"
        :disabled="busy"
        :aria-label="`Remove ${item.name}`"
        title="Remove item and its DropCove-created file; external sources stay"
        @click="emit('remove', item.id)"
      ><ActionIcon name="remove" /></button>
    </div>
  </li>
  </template>

<style scoped>
.item-row[data-selected] {
  border-color: var(--dc-accent);
  background: var(--dc-accent-subtle);
}
.item-drag:focus-visible:not([data-pointer-focus]) {
  outline: 2px solid var(--dc-focus);
  outline-offset: -3px;
  border-radius: 6px;
}
.item-drag[data-pointer-focus]:focus-visible {
  outline: none !important;
}
@media (forced-colors: active) {
  .item-row[data-selected] {
    forced-color-adjust: none;
    border-color: Highlight;
    background: Highlight;
    color: HighlightText;
  }
  .item-row[data-selected] :deep(.item-name) { color: HighlightText; }
  .item-row[data-selected] .item-drag:focus-visible:not([data-pointer-focus]) { outline-color: HighlightText !important; }
}
</style>

