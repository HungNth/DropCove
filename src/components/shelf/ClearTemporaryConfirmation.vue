<script setup lang="ts">
import { nextTick, onUnmounted, useTemplateRef, watch } from "vue";

const props = withDefaults(defineProps<{
  open: boolean;
  focusOnOpen?: boolean;
}>(), {
  focusOnOpen: true,
});

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();

const dialog = useTemplateRef<HTMLDialogElement>("dialog");
const cancelButton = useTemplateRef<HTMLButtonElement>("cancelButton");

watch(() => props.open, async (isOpen) => {
  await nextTick();
  if (isOpen) {
    if (!dialog.value?.open) {
      dialog.value?.showModal();
    }
    if (props.focusOnOpen) {
      cancelButton.value?.focus();
    }
  } else {
    if (dialog.value?.open) {
      dialog.value.close();
    }
  }
}, { immediate: true });

onUnmounted(() => {
  if (dialog.value?.open) {
    dialog.value.close();
  }
});
</script>

<template>
  <dialog
    ref="dialog"
    class="confirmation max-w-[calc(100%-20px)] max-h-[calc(100%-20px)] box-border p-[18px] border border-dc-border-subtle rounded-[8px] text-dc-text-primary bg-dc-surface text-[12px] leading-[1.4] select-none font-['Segoe_UI',sans-serif] forced-colors:bg-[Canvas] forced-colors:border-[ButtonText] forced-colors:text-[CanvasText]"
    aria-labelledby="rail-clear-heading"
    aria-describedby="rail-clear-body"
    @cancel.prevent="emit('cancel')"
  >
    <div class="flex flex-col gap-[8px]">
      <h1 id="rail-clear-heading" class="m-0 text-[14px] font-semibold leading-[1.3] text-dc-text-primary forced-colors:text-[CanvasText]">
        Clear temporary items?
      </h1>
      <p id="rail-clear-body" class="m-0 text-[12px] leading-[1.4] text-dc-text-secondary forced-colors:text-[CanvasText]">
        Pinned items stay. Source files and folders aren’t deleted.
      </p>
    </div>
    <div class="flex justify-end gap-[8px] mt-[16px]">
      <button
        ref="cancelButton"
        type="button"
        class="action min-w-[72px] h-[28px] px-[12px] text-[12px] font-normal border border-dc-border-default rounded-[4px] bg-dc-surface-elevated text-dc-text-primary cursor-pointer hover:border-dc-border-hover hover:bg-dc-surface-hover focus-visible:outline-2 focus-visible:outline-dc-focus forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:focus-visible:outline-[Highlight]"
        @click="emit('cancel')"
      >
        Cancel
      </button>
      <button
        type="button"
        class="action danger min-w-[72px] h-[28px] px-[12px] text-[12px] font-normal border border-dc-danger-border rounded-[4px] bg-dc-danger text-dc-accent-text cursor-pointer hover:bg-dc-danger-hover focus-visible:outline-2 focus-visible:outline-dc-focus forced-colors:border-[Highlight] forced-colors:bg-[Highlight] forced-colors:text-[HighlightText] forced-colors:focus-visible:outline-[Highlight]"
        @click="emit('confirm')"
      >
        Clear
      </button>
    </div>
  </dialog>
</template>

<style scoped>
.confirmation::backdrop {
  background: var(--dc-backdrop);
}
</style>
