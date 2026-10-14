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
.confirmation { max-width: calc(100% - 20px); max-height: calc(100% - 20px); box-sizing: border-box; padding: 10px; border: 1px solid var(--dc-accent); border-radius: 8px; color: var(--dc-text-primary); background: var(--dc-surface); font: 11px/1.4 "Segoe UI", sans-serif; }
.confirmation::backdrop { background: var(--dc-backdrop); }
.question { margin: 0 0 6px; font-weight: 600; color: var(--dc-text-primary); }
.guidance { margin: 0 0 8px; color: var(--dc-text-muted); }
.actions { display: flex; flex-wrap: wrap; gap: 6px; }
.action { border: 1px solid var(--dc-border-default); border-radius: 4px; padding: 4px 8px; color: inherit; background: var(--dc-surface-elevated); cursor: pointer; touch-action: none; user-select: none; }
.action:hover { border-color: var(--dc-border-hover); background: var(--dc-surface-hover); }
.primary { border-color: var(--dc-accent); background: var(--dc-accent); color: var(--dc-accent-text); }
.primary:hover { background: var(--dc-accent-hover); }
.action:focus-visible { outline: 2px solid var(--dc-focus); }

@media (forced-colors: active) {
  .confirmation {
    border-color: ButtonText;
  }
  .action {
    border-color: ButtonText;
    background: ButtonFace;
    color: ButtonText;
  }
  .primary {
    border-color: Highlight;
    background: Highlight;
    color: HighlightText;
  }
  .action:focus-visible {
    outline: 2px solid Highlight !important;
  }
}
</style>
