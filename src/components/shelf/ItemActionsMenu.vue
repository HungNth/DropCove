<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, shallowRef, useTemplateRef, watch } from "vue";
import type { ShelfItem } from "../../composables/useShelf";
import ActionIcon from "./ActionIcon.vue";

export type ShelfItemAction = "ShowInExplorer" | "CopyPath" | "CopyText";

const props = defineProps<{
  item: ShelfItem;
  disabled?: boolean;
  isOpen?: boolean;
  noActivate?: boolean;
}>();

const emit = defineEmits<{
  open: [];
  close: [];
  action: [action: ShelfItemAction];
}>();

const activeIndex = shallowRef(0);
const menuStyle = shallowRef<Record<string, string>>({});
const moreButtonRef = useTemplateRef<HTMLButtonElement>("moreButtonRef");
const menuRef = useTemplateRef<HTMLElement>("menuRef");
const itemRefs = new Map<number, HTMLButtonElement>();
interface ActionOption {
  id: ShelfItemAction;
  label: string;
}

const actions = computed<ActionOption[]>(() => {
  if (props.item.ownership === "External") {
    return [
      { id: "ShowInExplorer", label: "Show in File Explorer" },
      { id: "CopyPath", label: "Copy Path" },
    ];
  }
  if (props.item.ownership === "Managed" && props.item.managedContentKind === "Text") {
    return [
      { id: "CopyText", label: "Copy Text" },
    ];
  }
  return [];
});

function setItemRef(el: unknown, index: number) {
  if (el && typeof (el as { focus?: unknown }).focus === "function") {
    itemRefs.set(index, el as HTMLButtonElement);
  } else {
    itemRefs.delete(index);
  }
}

function focusItem(index: number) {
  if (props.noActivate) return;
  const button = itemRefs.get(index);
  button?.focus();
}

function updatePosition() {
  if (!moreButtonRef.value) return;
  const rect = moreButtonRef.value.getBoundingClientRect();
  const viewportWidth = typeof window !== "undefined" ? window.innerWidth : 300;
  const viewportHeight = typeof window !== "undefined" ? window.innerHeight : 400;
  const menuWidth = 148;
  const estimatedHeight = actions.value.length * 32 + 8;
  const style: Record<string, string> = { position: "fixed", zIndex: "9999" };

  const spaceBelow = viewportHeight - rect.bottom;
  const spaceAbove = rect.top;
  if (spaceBelow < estimatedHeight + 8 && spaceAbove > spaceBelow) {
    style.bottom = `${Math.max(4, viewportHeight - rect.top + 4)}px`;
  } else {
    style.top = `${Math.max(4, Math.min(rect.bottom + 4, viewportHeight - estimatedHeight - 4))}px`;
  }

  const rightOffset = viewportWidth - rect.right;
  if (rect.right - menuWidth < 4) {
    style.left = "4px";
  } else if (rightOffset < 4) {
    style.right = "4px";
  } else {
    style.right = `${Math.max(4, rightOffset)}px`;
  }

  menuStyle.value = style;
}

function openMenu() {
  if (props.disabled || actions.value.length === 0) return;
  emit("open");
}

watch(() => props.isOpen, async (open) => {
  if (!open) return;
  activeIndex.value = 0;
  await nextTick();
  if (!props.isOpen) return;
  updatePosition();
  focusItem(0);
}, { immediate: true });

function closeMenu(restoreFocus = true) {
  emit("close");
  activeIndex.value = -1;
  if (restoreFocus && !props.noActivate) {
    moreButtonRef.value?.focus();
  }
}

function toggleMenu() {
  if (props.disabled) return;
  if (props.isOpen) {
    closeMenu(true);
  } else {
    void openMenu();
  }
}

function triggerAction(actionId: ShelfItemAction) {
  closeMenu(true);
  emit("action", actionId);
}

function onButtonKeydown(event: KeyboardEvent) {
  if (props.disabled) return;
  if (event.key === "ArrowUp" || event.key === "ArrowDown" || event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    event.stopPropagation();
    void openMenu();
  }
}

function onMenuKeydown(event: KeyboardEvent) {
  if (!props.isOpen) return;

  switch (event.key) {
    case "Escape":
      event.preventDefault();
      event.stopPropagation();
      closeMenu(true);
      break;

    case "ArrowDown":
      event.preventDefault();
      event.stopPropagation();
      activeIndex.value = (activeIndex.value + 1) % actions.value.length;
      focusItem(activeIndex.value);
      break;

    case "ArrowUp":
      event.preventDefault();
      event.stopPropagation();
      activeIndex.value = (activeIndex.value - 1 + actions.value.length) % actions.value.length;
      focusItem(activeIndex.value);
      break;

    case "Home":
      event.preventDefault();
      event.stopPropagation();
      activeIndex.value = 0;
      focusItem(0);
      break;

    case "End":
      event.preventDefault();
      event.stopPropagation();
      activeIndex.value = actions.value.length - 1;
      focusItem(activeIndex.value);
      break;

    case "Tab":
      closeMenu(false);
      break;

    case "Enter":
    case " ":
      event.preventDefault();
      event.stopPropagation();
      if (activeIndex.value >= 0 && activeIndex.value < actions.value.length) {
        triggerAction(actions.value[activeIndex.value].id);
      }
      break;
  }
}

function onPointerDownOutside(event: PointerEvent) {
  if (!props.isOpen) return;
  const target = event.target as Node | null;
  if (menuRef.value && !menuRef.value.contains(target) && moreButtonRef.value && !moreButtonRef.value.contains(target)) {
    closeMenu(!props.noActivate);
  }
}

onMounted(() => {
  if (typeof window !== "undefined") {
    window.addEventListener("pointerdown", onPointerDownOutside);
  }
});

onUnmounted(() => {
  if (typeof window !== "undefined") {
    window.removeEventListener("pointerdown", onPointerDownOutside);
  }
});
</script>

<template>
  <div class="relative inline-flex items-center" :class="{ 'z-50': isOpen }">
    <!-- Item Actions Popover Menu -->
    <div
      v-if="isOpen && actions.length"
      ref="menuRef"
      role="menu"
      :aria-label="`Actions for ${item.name}`"
      class="item-actions-menu fixed min-w-[148px] p-1 rounded-md border border-dc-border-hover bg-dc-surface-elevated shadow-dc-floating z-50 flex flex-col gap-0.5 select-none forced-colors:border-[ButtonText]"
      :style="menuStyle"
      @pointerdown.stop
      @pointerup.stop
      @click.stop
      @keydown="onMenuKeydown"
    >
      <button
        v-for="(actionItem, index) in actions"
        :key="actionItem.id"
        :ref="el => setItemRef(el, index)"
        type="button"
        role="menuitem"
        tabindex="-1"
        class="menu-item w-full text-left px-2 py-1.5 rounded-xs text-[12px] font-normal text-dc-text-primary bg-transparent border-0 cursor-pointer hover:bg-dc-surface-hover hover:text-dc-text-primary focus-visible:outline-none focus-visible:bg-dc-surface-hover focus-visible:text-dc-text-primary focus-visible:ring-1 focus-visible:ring-dc-focus forced-colors:focus-visible:outline-[1px] forced-colors:focus-visible:outline-[Highlight]"
        :class="{ 'bg-dc-surface-hover': activeIndex === index }"
        @mouseenter="activeIndex = index"
        @click.stop="triggerAction(actionItem.id)"
      >
        {{ actionItem.label }}
      </button>
    </div>

    <!-- More Action Trigger Button -->
    <button
      ref="moreButtonRef"
      type="button"
      class="more action inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-text-secondary cursor-pointer hover:border-dc-border-hover hover:bg-dc-surface-hover hover:text-dc-text-primary aria-pressed:border-dc-accent aria-pressed:text-dc-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-dc-focus disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText]"
      :disabled="disabled"
      :aria-label="`More actions for ${item.name}`"
      :title="`More actions for ${item.name}`"
      aria-haspopup="menu"
      :aria-expanded="isOpen"
      @pointerdown.stop
      @pointerup.stop
      @click.stop="toggleMenu"
      @keydown="onButtonKeydown"
    >
      <ActionIcon name="more" />
    </button>
  </div>
</template>
