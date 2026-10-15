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
    class="confirmation max-w-[calc(100%-20px)] max-h-[calc(100%-20px)] box-border p-[10px] border border-dc-danger-border rounded-[8px] text-dc-text-primary bg-dc-surface text-[11px] leading-[1.4] font-['Segoe_UI',sans-serif] forced-colors:border-[ButtonText]"
    aria-label="Clear temporary items confirmation"
    @cancel.prevent="emit('cancel')"
  >
    <p class="m-0 mb-[6px] font-semibold text-dc-text-primary">Clear all temporary items from the shelf?</p>
    <p class="m-0 mb-[8px] text-dc-text-muted">Pinned items remain on the shelf. Source files and folders on your computer are never deleted.</p>
    <div class="flex flex-wrap justify-end gap-[6px]">
      <button
        ref="cancelButton"
        type="button"
        class="action border border-dc-border-default rounded-[4px] py-[4px] px-[8px] text-inherit bg-dc-surface-elevated cursor-pointer touch-none select-none hover:border-dc-border-hover hover:bg-dc-surface-hover focus-visible:outline-2 focus-visible:outline-dc-focus forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:focus-visible:outline-[2px] forced-colors:focus-visible:outline-[Highlight]!"
        @click="emit('cancel')"
      >
        Cancel
      </button>
      <button
        type="button"
        class="action danger border border-dc-danger-border rounded-[4px] py-[4px] px-[8px] text-dc-accent-text bg-dc-danger cursor-pointer touch-none select-none hover:border-dc-border-hover hover:bg-dc-danger-hover focus-visible:outline-2 focus-visible:outline-dc-focus forced-colors:border-[Highlight] forced-colors:bg-[Highlight] forced-colors:text-[HighlightText] forced-colors:focus-visible:outline-[2px] forced-colors:focus-visible:outline-[Highlight]! forced-colors:hover:border-dc-border-hover"
        @click="emit('confirm')"
      >
        Clear Temporary Items
      </button>
    </div>
  </dialog>
</template>

<style scoped>
.confirmation::backdrop {
  background: var(--dc-backdrop);
}
</style>
