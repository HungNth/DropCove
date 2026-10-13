<script setup lang="ts">
import { nextTick, onUnmounted, useTemplateRef, watch } from "vue";

const props = defineProps<{
  open: boolean;
}>();

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
    cancelButton.value?.focus();
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
  border: 1px solid #f87171;
  border-radius: 8px;
  color: #e2e8f0;
  background: #1b2438;
  font: 11px/1.4 "Segoe UI", sans-serif;
}
.confirmation::backdrop {
  background: rgba(0, 0, 0, .45);
}
.question {
  margin: 0 0 6px;
  font-weight: 600;
}
.guidance {
  margin: 0 0 8px;
  color: #bac6db;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 6px;
}
.action {
  border: 1px solid #475569;
  border-radius: 4px;
  padding: 4px 8px;
  color: inherit;
  background: #243049;
  cursor: pointer;
  touch-action: none;
  user-select: none;
}
.danger {
  border-color: #f87171;
  background: #991b1b;
}
.danger:hover {
  background: #b91c1c;
}
.action:focus-visible {
  outline: 2px solid #a5b4fc;
}
</style>
