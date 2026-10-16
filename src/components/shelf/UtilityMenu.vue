<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted, useTemplateRef } from "vue";
import ActionIcon from "./ActionIcon.vue";

const props = defineProps<{
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "open-settings"): void;
  (e: "open-about"): void;
}>();

const isOpen = ref(false);
const activeIndex = ref<number>(-1);

const moreButtonRef = useTemplateRef<HTMLButtonElement>("moreButtonRef");
const menuRef = useTemplateRef<HTMLElement>("menuRef");
const settingsItemRef = useTemplateRef<HTMLButtonElement>("settingsItemRef");
const aboutItemRef = useTemplateRef<HTMLButtonElement>("aboutItemRef");

const menuItems = [
  { id: "settings", label: "Settings", action: () => emit("open-settings") },
  { id: "about", label: "About", action: () => emit("open-about") },
];

watch(
  () => props.disabled,
  (disabled) => {
    if (disabled && isOpen.value) {
      closeMenu(false);
    }
  }
);

function toggleMenu() {
  if (props.disabled) return;
  if (isOpen.value) {
    closeMenu(true);
  } else {
    openMenu();
  }
}

async function openMenu() {
  if (props.disabled) return;
  isOpen.value = true;
  activeIndex.value = 0;
  await nextTick();
  focusItem(0);
}

function closeMenu(restoreFocus = true) {
  if (!isOpen.value) return;
  isOpen.value = false;
  activeIndex.value = -1;
  if (restoreFocus) {
    moreButtonRef.value?.focus();
  }
}

async function triggerItem(index: number) {
  const item = menuItems[index];
  if (!item) return;
  closeMenu(false);
  item.action();
}

function onButtonKeydown(event: KeyboardEvent) {
  if (props.disabled) return;
  if (event.key === "ArrowUp" || event.key === "ArrowDown" || event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    event.stopPropagation();
    openMenu();
  }
}

function onMenuKeydown(event: KeyboardEvent) {
  if (!isOpen.value) return;

  switch (event.key) {
    case "Escape":
      event.preventDefault();
      event.stopPropagation();
      closeMenu(true);
      break;

    case "ArrowDown":
      event.preventDefault();
      event.stopPropagation();
      activeIndex.value = (activeIndex.value + 1) % menuItems.length;
      focusItem(activeIndex.value);
      break;

    case "ArrowUp":
      event.preventDefault();
      event.stopPropagation();
      activeIndex.value = (activeIndex.value - 1 + menuItems.length) % menuItems.length;
      focusItem(activeIndex.value);
      break;

    case "Home":
      event.preventDefault();
      event.stopPropagation();
      activeIndex.value = 0;
      focusItem(activeIndex.value);
      break;

    case "End":
      event.preventDefault();
      event.stopPropagation();
      activeIndex.value = menuItems.length - 1;
      focusItem(activeIndex.value);
      break;

    case "Tab":
      closeMenu(true);
      break;

    case "Enter":
    case " ":
      event.preventDefault();
      event.stopPropagation();
      if (activeIndex.value >= 0) {
        void triggerItem(activeIndex.value);
      }
      break;
  }
}

function focusItem(index: number) {
  if (index === 0) {
    settingsItemRef.value?.focus();
  } else if (index === 1) {
    aboutItemRef.value?.focus();
  }
}

function onPointerDownOutside(event: PointerEvent) {
  if (!isOpen.value) return;
  const target = event.target as Node | null;
  if (menuRef.value && !menuRef.value.contains(target) && moreButtonRef.value && !moreButtonRef.value.contains(target)) {
    // Keep the ensuing pointer default from blurring the restored trigger focus.
    event.preventDefault();
    closeMenu(true);
  }
}

onMounted(() => {
  window.addEventListener("pointerdown", onPointerDownOutside);
});

onUnmounted(() => {
  window.removeEventListener("pointerdown", onPointerDownOutside);
});
</script>

<template>
  <div class="relative inline-flex items-center">
    <!-- Utility Menu Popover -->
    <div
      v-if="isOpen"
      ref="menuRef"
      role="menu"
      aria-label="Drop Shelf Utility Menu"
      class="absolute right-0 bottom-[calc(100%+4px)] w-[132px] p-1 rounded-md border border-dc-border-subtle bg-dc-surface-elevated shadow-md z-50 flex flex-col gap-0.5 select-none"
      @keydown="onMenuKeydown"
    >
      <button
        ref="settingsItemRef"
        type="button"
        role="menuitem"
        tabindex="-1"
        class="w-full text-left px-2 py-1.5 rounded-xs text-[12px] font-normal text-dc-text-primary bg-transparent border-0 cursor-pointer hover:bg-dc-surface-hover hover:text-dc-text-primary focus-visible:outline-none focus-visible:bg-dc-surface-hover focus-visible:text-dc-text-primary focus-visible:ring-1 focus-visible:ring-dc-focus forced-colors:focus-visible:outline-[1px] forced-colors:focus-visible:outline-[Highlight]"
        :class="{ 'bg-dc-surface-hover': activeIndex === 0 }"
        @mouseenter="activeIndex = 0"
        @click="triggerItem(0)"
      >
        Settings
      </button>

      <button
        ref="aboutItemRef"
        type="button"
        role="menuitem"
        tabindex="-1"
        class="w-full text-left px-2 py-1.5 rounded-xs text-[12px] font-normal text-dc-text-primary bg-transparent border-0 cursor-pointer hover:bg-dc-surface-hover hover:text-dc-text-primary focus-visible:outline-none focus-visible:bg-dc-surface-hover focus-visible:text-dc-text-primary focus-visible:ring-1 focus-visible:ring-dc-focus forced-colors:focus-visible:outline-[1px] forced-colors:focus-visible:outline-[Highlight]"
        :class="{ 'bg-dc-surface-hover': activeIndex === 1 }"
        @mouseenter="activeIndex = 1"
        @click="triggerItem(1)"
      >
        About
      </button>
    </div>

    <!-- More Action Button -->
    <button
      ref="moreButtonRef"
      type="button"
      class="more-action inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-text-muted cursor-pointer hover:text-dc-text-primary hover:border-dc-border-hover hover:bg-dc-surface-hover focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-dc-focus disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText]"
      :disabled="disabled"
      aria-label="More"
      title="More"
      aria-haspopup="menu"
      :aria-expanded="isOpen"
      @click="toggleMenu"
      @keydown="onButtonKeydown"
    >
      <ActionIcon name="more" />
    </button>
  </div>
</template>
