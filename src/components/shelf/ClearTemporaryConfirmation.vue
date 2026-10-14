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
    class="confirmation"
    aria-label="Clear temporary items confirmation"
    @cancel.prevent="emit('cancel')"
  >
    <p class="question">Clear all temporary items from the shelf?</p>
    <p class="guidance">Pinned items remain on the shelf. Source files and folders on your computer are never deleted.</p>
    <div class="actions">
      <button
        ref="cancelButton"
        type="button"
        class="action"
        @click="emit('cancel')"
      >
        Cancel
      </button>
      <button
        type="button"
        class="action danger"
        @click="emit('confirm')"
      >
        Clear Temporary Items
      </button>
    </div>
  </dialog>
</template>

<style scoped>
.confirmation {
  max-width: calc(100% - 20px);
  max-height: calc(100% - 20px);
  box-sizing: border-box;
  padding: 10px;
  border: 1px solid var(--dc-danger-border);
  border-radius: 8px;
  color: var(--dc-text-primary);
  background: var(--dc-surface);
  font: 11px/1.4 "Segoe UI", sans-serif;
}
.confirmation::backdrop {
  background: var(--dc-backdrop);
}
.question {
  margin: 0 0 6px;
  font-weight: 600;
  color: var(--dc-text-primary);
}
.guidance {
  margin: 0 0 8px;
  color: var(--dc-text-muted);
}
.actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 6px;
}
.action {
  border: 1px solid var(--dc-border-default);
  border-radius: 4px;
  padding: 4px 8px;
  color: inherit;
  background: var(--dc-surface-elevated);
  cursor: pointer;
  touch-action: none;
  user-select: none;
}
.action:hover {
  border-color: var(--dc-border-hover);
  background: var(--dc-surface-hover);
}
.danger {
  border-color: var(--dc-danger-border);
  background: var(--dc-danger);
  color: var(--dc-accent-text);
}
.danger:hover {
  background: var(--dc-danger-hover);
}
.action:focus-visible {
  outline: 2px solid var(--dc-focus);
}

@media (forced-colors: active) {
  .confirmation {
    border-color: ButtonText;
  }
  .action {
    border-color: ButtonText;
    background: ButtonFace;
    color: ButtonText;
  }
  .danger {
    border-color: Highlight;
    background: Highlight;
    color: HighlightText;
  }
  .action:focus-visible {
    outline: 2px solid Highlight !important;
  }
}
</style>
