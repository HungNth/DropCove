<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, shallowRef, useTemplateRef } from "vue";
import { invoke } from "@tauri-apps/api/core";

const busy = shallowRef(false);
const error = shallowRef("");
const cancelButton = useTemplateRef<HTMLButtonElement>("cancelButton");

async function handleCancel() {
  if (busy.value) return;
  try {
    await invoke("cancel_drop_shelf_clear_temporary");
  } catch (err) {
    error.value = String(err);
  }
}

async function handleConfirm() {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    await invoke("confirm_drop_shelf_clear_temporary");
  } catch (err) {
    error.value = String(err);
  } finally {
    busy.value = false;
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    void handleCancel();
  }
}

onMounted(async () => {
  window.addEventListener("keydown", handleKeydown);
  await nextTick();
  cancelButton.value?.focus();
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <main
    class="clear-confirmation flex flex-col justify-between w-full h-full box-border p-[18px] bg-dc-surface text-dc-text-primary select-none font-['Segoe_UI',sans-serif] forced-colors:bg-[Canvas] forced-colors:text-[CanvasText]"
    role="dialog"
    aria-labelledby="clear-heading"
    aria-describedby="clear-body"
    :aria-busy="busy"
  >
    <div class="flex flex-col gap-[8px]">
      <h1 id="clear-heading" class="m-0 text-[14px] font-semibold leading-[1.3] text-dc-text-primary forced-colors:text-[CanvasText]">
        Clear temporary items?
      </h1>
      <p id="clear-body" class="m-0 text-[12px] leading-[1.4] text-dc-text-secondary forced-colors:text-[CanvasText]">
        Pinned items stay. Source files and folders aren’t deleted.
      </p>
      <div v-if="error" class="p-[6px] border border-dc-danger-border bg-dc-danger-surface text-dc-danger-text rounded-[4px] text-[11px]" role="alert">
        {{ error }}
      </div>
    </div>

    <div class="flex justify-end gap-[8px] mt-[16px]">
      <button
        ref="cancelButton"
        type="button"
        class="action min-w-[72px] h-[28px] px-[12px] text-[12px] font-normal border border-dc-border-default rounded-[4px] bg-dc-surface-elevated text-dc-text-primary cursor-pointer hover:border-dc-border-hover hover:bg-dc-surface-hover focus-visible:outline-2 focus-visible:outline-dc-focus disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:focus-visible:outline-[Highlight]"
        :disabled="busy"
        @click="handleCancel"
      >
        Cancel
      </button>
      <button
        type="button"
        class="action danger min-w-[72px] h-[28px] px-[12px] text-[12px] font-normal border border-dc-danger-border rounded-[4px] bg-dc-danger text-dc-accent-text cursor-pointer hover:bg-dc-danger-hover focus-visible:outline-2 focus-visible:outline-dc-focus disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:border-[Highlight] forced-colors:bg-[Highlight] forced-colors:text-[HighlightText] forced-colors:focus-visible:outline-[Highlight]"
        :disabled="busy"
        @click="handleConfirm"
      >
        Clear
      </button>
    </div>
  </main>
</template>
