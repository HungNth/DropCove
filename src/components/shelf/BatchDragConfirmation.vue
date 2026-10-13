<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from "vue";

const props = defineProps<{ pending: { available: number; unavailable: number } | null }>();
const emit = defineEmits<{ confirm: []; cancel: []; drag: [] }>();
const dialog = useTemplateRef<HTMLDialogElement>("dialog");
let pointerStart: { id: number; x: number; y: number } | undefined;

watch(() => props.pending, async (pending) => {
  await nextTick();
  if (pending) { if (!dialog.value?.open) dialog.value?.showModal(); }
  else dialog.value?.close();
}, { immediate: true });

function pointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  pointerStart = { id: event.pointerId, x: event.clientX, y: event.clientY };
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
}

function pointerMove(event: PointerEvent) {
  if (!pointerStart || event.pointerId !== pointerStart.id || !(event.buttons & 1)) return;
  if (Math.hypot(event.clientX - pointerStart.x, event.clientY - pointerStart.y) < 6) return;
  pointerStart = undefined;
  (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
  emit("drag");
}
</script>

<template>
  <dialog ref="dialog" class="confirmation" aria-label="Drag available items confirmation" @cancel.prevent="emit('cancel')">
    <p class="question">{{ pending?.unavailable }} unavailable item(s) will remain. Drag only the {{ pending?.available }} available item(s)?</p>
    <p class="guidance">Drag the button to your destination, or confirm and then drag the batch again.</p>
    <div class="actions">
      <button type="button" class="action primary" @pointerdown="pointerDown" @pointermove="pointerMove" @pointerup="pointerStart = undefined" @pointercancel="pointerStart = undefined" @click="emit('confirm')" @dragstart.prevent>Use available items</button>
      <button type="button" class="action" @click="emit('cancel')">Cancel</button>
    </div>
  </dialog>
</template>

<style scoped>
.confirmation { max-width: calc(100% - 20px); max-height: calc(100% - 20px); box-sizing: border-box; padding: 10px; border: 1px solid #818cf8; border-radius: 8px; color: #e2e8f0; background: #1b2438; font: 11px/1.4 "Segoe UI", sans-serif; }
.confirmation::backdrop { background: rgba(0, 0, 0, .45); }
.question { margin: 0 0 6px; }
.guidance { margin: 0 0 8px; color: #bac6db; }
.actions { display: flex; flex-wrap: wrap; gap: 6px; }
.action { border: 1px solid #475569; border-radius: 4px; padding: 4px 8px; color: inherit; background: #243049; cursor: pointer; touch-action: none; user-select: none; }
.primary { border-color: #818cf8; background: #4338ca; }
.action:focus-visible { outline: 2px solid #a5b4fc; }
</style>
