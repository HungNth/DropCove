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
  <dialog
    ref="dialog"
    class="confirmation max-w-[calc(100%-20px)] max-h-[calc(100%-20px)] box-border p-[10px] border border-dc-accent rounded-[8px] text-dc-text-primary bg-dc-surface text-[11px] leading-[1.4] font-['Segoe_UI',sans-serif] forced-colors:border-[ButtonText]"
    aria-label="Drag available items confirmation"
    @cancel.prevent="emit('cancel')"
  >
    <p class="m-0 mb-[6px] font-semibold text-dc-text-primary">
      {{ pending?.unavailable }} unavailable item(s) will remain. Drag only the {{ pending?.available }} available item(s)?
    </p>
    <p class="m-0 mb-[8px] text-dc-text-muted">
      Drag the button to your destination, or confirm and then drag the same selection again.
    </p>
    <div class="flex flex-wrap gap-[6px]">
      <button
        type="button"
        class="border border-dc-action rounded-[4px] py-[4px] px-[8px] text-dc-accent-text bg-dc-action cursor-pointer touch-none select-none hover:border-dc-border-hover hover:bg-dc-accent-hover focus-visible:outline-2 focus-visible:outline-dc-focus forced-colors:border-[Highlight] forced-colors:bg-[Highlight] forced-colors:text-[HighlightText] forced-colors:focus-visible:outline-[2px] forced-colors:focus-visible:outline-[Highlight]! forced-colors:hover:border-dc-border-hover"
        @pointerdown="pointerDown"
        @pointermove="pointerMove"
        @pointerup="pointerStart = undefined"
        @pointercancel="pointerStart = undefined"
        @click="emit('confirm')"
        @dragstart.prevent
      >
        Use available items
      </button>
      <button
        type="button"
        class="border border-dc-border-default rounded-[4px] py-[4px] px-[8px] text-inherit bg-dc-surface-elevated cursor-pointer touch-none select-none hover:border-dc-border-hover hover:bg-dc-surface-hover focus-visible:outline-2 focus-visible:outline-dc-focus forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:focus-visible:outline-[2px] forced-colors:focus-visible:outline-[Highlight]!"
        @click="emit('cancel')"
      >
        Cancel
      </button>
    </div>
  </dialog>
</template>

<style scoped>
.confirmation::backdrop {
  background: var(--dc-backdrop);
}
</style>
