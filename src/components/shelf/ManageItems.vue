<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, useTemplateRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { useManageItems } from "../../composables/useManageItems";
import BulkPinning from "./BulkPinning.vue";
import ShelfItemRow from "./ShelfItemRow.vue";

const { batch, status, busy, refresh, setItemPinned, setBatchPinned, removeItem, startItemDrag, close, dismissStatus } = useManageItems();

const pinnedCount = computed(() => batch.value?.items.filter((item) => item.isPinned).length ?? 0);
const totalCount = computed(() => batch.value?.items.length ?? 0);

const rootRef = useTemplateRef<HTMLElement>("root");

const window = getCurrentWindow();
const unlisteners: UnlistenFn[] = [];
let cancelKeydownSeen = false;

function getBulkElement(): HTMLElement | null {
  return rootRef.value?.querySelector<HTMLElement>(".bulk-pin") ?? null;
}

let focusRequest = 0;
let disposed = false;
async function focusBulkControl() {
  const request = ++focusRequest;
  await refresh();
  await nextTick();
  if (!disposed && request === focusRequest) getBulkElement()?.focus();
}

function getFocusableElements(): HTMLElement[] {
  if (!rootRef.value) return [];
  // Canonical order: Bulk Pinning, Close, then row Pin/Unpin and Remove controls in row order
  const elements: HTMLElement[] = [];
  const bulk = getBulkElement();
  if (bulk && !bulk.hasAttribute("disabled")) elements.push(bulk);
  const closeBtn = rootRef.value.querySelector<HTMLElement>(".header .close");
  if (closeBtn && !closeBtn.hasAttribute("disabled")) elements.push(closeBtn);

  const rows = rootRef.value.querySelectorAll<HTMLElement>(".items .item-row");
  for (const row of rows) {
    const pinBtn = row.querySelector<HTMLElement>(".item-actions .action.pin");
    if (pinBtn && !pinBtn.hasAttribute("disabled")) elements.push(pinBtn);
    const removeBtn = row.querySelector<HTMLElement>(".item-actions .action.remove");
    if (removeBtn && !removeBtn.hasAttribute("disabled")) elements.push(removeBtn);
  }
  return elements;
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Tab") {
    const focusables = getFocusableElements();
    if (focusables.length === 0) {
      event.preventDefault();
      return;
    }
    const currentIndex = focusables.findIndex((el) => el === document.activeElement || el.contains(document.activeElement));
    event.preventDefault();
    if (event.shiftKey) {
      const nextIndex = currentIndex <= 0 ? focusables.length - 1 : currentIndex - 1;
      focusables[nextIndex]?.focus();
    } else {
      const nextIndex = currentIndex === -1 || currentIndex >= focusables.length - 1 ? 0 : currentIndex + 1;
      focusables[nextIndex]?.focus();
    }
    return;
  }

  if (event.key === "Escape") {
    event.stopPropagation();
    event.preventDefault();
    if (busy.value) {
      // While item drag busy Escape cancels native drag; guard first cancellation so it doesn't close popup
      cancelKeydownSeen = true;
      return;
    }
    if (cancelKeydownSeen) {
      // Already guarded in keydown while busy, ignore residual keydown
      return;
    }
    void close();
  }
}

function onKeyup(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.stopPropagation();
    event.preventDefault();
    if (cancelKeydownSeen) {
      cancelKeydownSeen = false;
    }
  }
}

onMounted(async () => {
  document.addEventListener("keydown", onKeydown, true);
  document.addEventListener("keyup", onKeyup, true);
  try {
    const cancelListener = await window.listen("item-drag-canceled", () => { cancelKeydownSeen = true; });
    if (disposed) { cancelListener(); return; }
    unlisteners.push(cancelListener);
    const unlisten = await window.listen<boolean>("manage-items-opened", async ({ payload: keyboard }) => {
      if (!keyboard) { ++focusRequest; return; }
      try { await focusBulkControl(); }
      catch (error) { if (!disposed) status.value = String(error); }
    });
    if (disposed) { unlisten(); return; }
    unlisteners.push(unlisten);
    if (await invoke<boolean>("get_manage_items_keyboard")) await focusBulkControl();
  } catch (error) {
    if (!disposed) status.value = String(error);
  }
});

onUnmounted(() => {
  disposed = true;
  document.removeEventListener("keydown", onKeydown, true);
  document.removeEventListener("keyup", onKeyup, true);
  for (const unlisten of unlisteners) unlisten();
});
</script>

<template>
  <main
    ref="root"
    class="management"
    :aria-busy="busy"
    aria-label="Manage Items"
  >
    <header class="header">
      <div class="header-info">
        <h1 class="title">Manage Items<span v-if="batch" class="count"> · {{ batch.items.length }}</span></h1>
        <span v-if="batch" class="summary">{{ pinnedCount }}/{{ totalCount }} pinned</span>
      </div>
      <div class="header-actions">
        <BulkPinning
          v-if="batch"
          :batch="batch"
          :busy="busy"
          @pin="(id, pinned) => setBatchPinned(id, pinned)"
        />
        <button class="close" type="button" :disabled="busy" aria-label="Close Manage Items" @click="close">Close</button>
      </div>
    </header>
    <ul v-if="batch" class="items" aria-label="Batch items">
      <ShelfItemRow
        v-for="item in batch.items"
        :key="item.id"
        :item="item"
        :busy="busy"
        @pin="setItemPinned"
        @remove="removeItem"
        @drag="startItemDrag"
      />
    </ul>
    <div v-if="status" class="status" role="alert">
      <span>{{ status }}</span>
      <button class="dismiss" type="button" aria-label="Dismiss status" @click="dismissStatus">×</button>
    </div>
  </main>
</template>

<style scoped>
.management { display: flex; flex-direction: column; height: 100%; box-sizing: border-box; border: 1px solid #46536a; border-radius: 9px; color: #edf2fc; background: #141c2b; font-family: "Segoe UI", sans-serif; }
.header { display: flex; align-items: center; justify-content: space-between; gap: 6px; padding: 9px; }
.header-info { display: flex; align-items: baseline; gap: 8px; }
.title { margin: 0; font-size: 13px; }
.count { color: #a7b4cb; font-weight: normal; }
.summary { color: #a7b4cb; font-size: 11px; }
.header-actions { display: flex; align-items: center; gap: 6px; }
.close { border: 1px solid #46536a; border-radius: 4px; padding: 3px 6px; color: inherit; background: transparent; font: inherit; font-size: 11px; cursor: pointer; }
.close:focus-visible, .dismiss:focus-visible { outline: 2px solid #a5b4fc; }
.close:disabled { opacity: .45; cursor: default; }
.items { display: flex; flex-direction: column; gap: 5px; min-height: 0; overflow-y: auto; overflow-x: hidden; margin: 0; padding: 0 7px 7px; list-style: none; }
.status { display: flex; align-items: start; gap: 4px; flex-shrink: 0; margin: auto 7px 7px; padding: 6px; border: 1px solid #efad70; border-radius: 5px; font-size: 11px; overflow-wrap: anywhere; }
.dismiss { margin-left: auto; border: 0; color: inherit; background: transparent; cursor: pointer; }
</style>
