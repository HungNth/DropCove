<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, shallowRef, useTemplateRef, watch } from "vue";

const props = defineProps<{
  actions: { id: string; label: string; danger?: boolean }[];
  anchor: { x: number; y: number };
  label: string;
  keyboard: boolean;
  noActivate?: boolean;
}>();
const emit = defineEmits<{ action: [id: string]; close: [restoreFocus: boolean] }>();

const menuRef = useTemplateRef<HTMLElement>("menuRef");
const itemRefs = new Map<number, HTMLButtonElement>();
const activeIndex = shallowRef(0);
const style = shallowRef<Record<string, string>>({});

function place() {
  const width = 164;
  const height = props.actions.length * 32 + 8;
  const vw = typeof window !== "undefined" ? window.innerWidth : 300;
  const vh = typeof window !== "undefined" ? window.innerHeight : 400;
  const left = Math.max(4, Math.min(props.anchor.x, vw - width - 4));
  const top = Math.max(4, Math.min(props.anchor.y, vh - height - 4));
  style.value = { position: "fixed", left: `${left}px`, top: `${top}px`, zIndex: "9999" };
}

function focusItem(index: number) {
  if (!props.noActivate) itemRefs.get(index)?.focus();
}

function setItemRef(el: unknown, index: number) {
  if (el && typeof (el as { focus?: unknown }).focus === "function") itemRefs.set(index, el as HTMLButtonElement);
  else itemRefs.delete(index);
}

function onKeydown(event: KeyboardEvent) {
  const last = props.actions.length - 1;
  const move = (next: number) => { event.preventDefault(); event.stopPropagation(); activeIndex.value = next; focusItem(next); };
  switch (event.key) {
    case "Escape": event.preventDefault(); event.stopPropagation(); emit("close", true); break;
    case "ArrowDown": move((activeIndex.value + 1) % props.actions.length); break;
    case "ArrowUp": move((activeIndex.value - 1 + props.actions.length) % props.actions.length); break;
    case "Home": move(0); break;
    case "End": move(last); break;
    case "Tab": emit("close", false); break;
    case "Enter":
    case " ":
      event.preventDefault(); event.stopPropagation();
      if (props.actions[activeIndex.value]) emit("action", props.actions[activeIndex.value].id);
      break;
  }
}

function onPointerDownOutside(event: PointerEvent) {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) emit("close", false);
}

watch(() => props.anchor, async () => { place(); await nextTick(); focusItem(0); }, { immediate: true });
onMounted(() => window.addEventListener("pointerdown", onPointerDownOutside));
onUnmounted(() => window.removeEventListener("pointerdown", onPointerDownOutside));
</script>

<template>
  <div
    ref="menuRef"
    role="menu"
    :aria-label="label"
    class="selection-actions-menu fixed min-w-[148px] p-1 rounded-md border border-dc-border-hover bg-dc-surface-elevated shadow-dc-floating z-50 flex flex-col gap-0.5 select-none forced-colors:border-[ButtonText]"
    :style="style"
    @pointerdown.stop
    @pointerup.stop
    @click.stop
    @contextmenu.prevent
    @keydown="onKeydown"
  >
    <button
      v-for="(action, index) in actions"
      :key="action.id"
      :ref="el => setItemRef(el, index)"
      type="button"
      role="menuitem"
      tabindex="-1"
      class="menu-item w-full text-left px-2 py-1.5 rounded-xs text-[12px] font-normal bg-transparent border-0 cursor-pointer hover:bg-dc-surface-hover focus-visible:outline-none focus-visible:bg-dc-surface-hover focus-visible:ring-1 focus-visible:ring-dc-focus forced-colors:focus-visible:outline-[1px] forced-colors:focus-visible:outline-[Highlight]"
      :class="[action.danger ? 'text-dc-danger' : 'text-dc-text-primary', { 'bg-dc-surface-hover': activeIndex === index }]"
      @mouseenter="activeIndex = index"
      @click.stop="emit('action', action.id)"
    >{{ action.label }}</button>
  </div>
</template>
