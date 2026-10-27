<script setup lang="ts">
import { computed, nextTick, onUnmounted, useTemplateRef, watch } from "vue";
import { removalConfirmationCopy, type RemovalPreview } from "../../composables/selectionActions";

const props = defineProps<{
  preview: RemovalPreview | null;
  domain: "batches" | "items";
  noActivate?: boolean;
}>();
const emit = defineEmits<{ confirm: []; cancel: [] }>();

const dialog = useTemplateRef<HTMLDialogElement>("dialog");
const cancelButton = useTemplateRef<HTMLButtonElement>("cancelButton");
const copy = computed(() => props.preview ? removalConfirmationCopy(props.preview, props.domain) : null);

watch(() => props.preview, async (preview) => {
  await nextTick();
  if (preview) {
    if (!props.noActivate) {
      if (!dialog.value?.open) dialog.value?.showModal?.();
      cancelButton.value?.focus();
    }
  } else if (dialog.value?.open) {
    dialog.value.close();
  }
}, { immediate: true });

onUnmounted(() => { if (dialog.value?.open) dialog.value.close(); });
</script>

<template>
  <component
    :is="noActivate ? 'div' : 'dialog'"
    :role="noActivate ? 'alertdialog' : undefined"
    :aria-modal="noActivate ? 'false' : undefined"
    :class="noActivate ? 'absolute inset-x-[6px] bottom-[6px] z-50 shadow-dc-floating' : ''"
    v-if="preview && copy"
    ref="dialog"
    class="selection-removal-confirmation max-w-[calc(100%-20px)] box-border p-[18px] border border-dc-border-subtle rounded-[8px] text-dc-text-primary bg-dc-surface text-[12px] leading-[1.4] select-none font-['Segoe_UI',sans-serif] forced-colors:bg-[Canvas] forced-colors:border-[ButtonText] forced-colors:text-[CanvasText]"
    aria-labelledby="selection-removal-heading"
    aria-describedby="selection-removal-body"
    @cancel.prevent="emit('cancel')"
  >
    <div class="flex flex-col gap-[8px]">
      <h1 id="selection-removal-heading" class="m-0 text-[14px] font-semibold leading-[1.3]">Remove {{ copy.scope }}?</h1>
      <p id="selection-removal-body" class="m-0 text-dc-text-secondary forced-colors:text-[CanvasText]">{{ copy.body }}</p>
    </div>
    <div class="flex justify-end gap-[8px] mt-[16px]">
      <button
        ref="cancelButton"
        type="button"
        class="action min-w-[72px] h-[28px] px-[12px] text-[12px] border border-dc-border-default rounded-[4px] bg-dc-surface-elevated text-dc-text-primary cursor-pointer hover:bg-dc-surface-hover focus-visible:outline-2 focus-visible:outline-dc-focus forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText]"
        @click="emit('cancel')"
      >Cancel</button>
      <button
        type="button"
        class="action danger min-w-[72px] h-[28px] px-[12px] text-[12px] border border-dc-danger-border rounded-[4px] bg-dc-danger-action text-dc-accent-text cursor-pointer hover:bg-dc-danger-hover focus-visible:outline-2 focus-visible:outline-dc-focus forced-colors:border-[Highlight] forced-colors:bg-[Highlight] forced-colors:text-[HighlightText]"
        @click="emit('confirm')"
      >Remove</button>
    </div>
  </component>
</template>

<style scoped>
.selection-removal-confirmation::backdrop { background: var(--dc-backdrop); }
</style>
