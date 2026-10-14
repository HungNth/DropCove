<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, useTemplateRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { useManageItems } from "../../composables/useManageItems";
import BulkPinning from "./BulkPinning.vue";
import ShelfItemRow from "./ShelfItemRow.vue";
import { useAnnouncer } from "../../composables/useAnnouncer";
import { useVirtualCollection } from "../../composables/useVirtualCollection";

const { batch, status, busy, refresh, setItemPinned, setBatchPinned, removeItem, startItemDrag, close, dismissStatus } = useManageItems();
const { announcement, announce } = useAnnouncer();

const pinnedCount = computed(() => batch.value?.items.filter((item) => item.isPinned).length ?? 0);
const totalCount = computed(() => batch.value?.items.length ?? 0);

const rootRef = useTemplateRef<HTMLElement>("root");
const itemsContainerRef = useTemplateRef<HTMLElement>("itemsContainer");
const itemsList = computed(() => batch.value?.items ?? []);
const virtual = useVirtualCollection(itemsList, itemsContainerRef, 1, 60, 5);
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

async function onKeydown(event: KeyboardEvent) {
  if (event.key === "Tab") {
    const bulk = getBulkElement();
    const closeBtn = rootRef.value?.querySelector<HTMLElement>(".header .close");
    const active = document.activeElement;

    if (virtual.enabled.value) {
      if (active === bulk && !event.shiftKey) {
        event.preventDefault();
        closeBtn?.focus();
        return;
      }
      if (active === closeBtn) {
        event.preventDefault();
        if (event.shiftKey) bulk?.focus();
        else await virtual.focus(0, 0);
        return;
      }
      const isInsideRows = itemsContainerRef.value?.contains(active);
      if (isInsideRows) {
        const row = (active as HTMLElement)?.closest<HTMLElement>("[data-virtual-index]");
        const index = row ? Number(row.dataset.virtualIndex) : -1;
        const controls = row ? Array.from(row.querySelectorAll<HTMLButtonElement>("button:not(:disabled)")) : [];
        const actionIdx = controls.findIndex(c => c === active);
        if (event.shiftKey && index === 0 && actionIdx === 0) {
          event.preventDefault();
          closeBtn?.focus();
          return;
        }
        if (!event.shiftKey && index === itemsList.value.length - 1 && actionIdx === controls.length - 1) {
          event.preventDefault();
          bulk?.focus();
          return;
        }
        await virtual.onTab(event);
        return;
      }
    }

    const focusables = getFocusableElements();
    if (focusables.length === 0) {
      event.preventDefault();
      return;
    }
    const currentIndex = focusables.findIndex((el) => el === active || el.contains(active));
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
    <ul
      v-if="batch"
      ref="itemsContainer"
      class="items"
      role="list"
      :aria-label="`Batch items (${batch.items.length})`"
      :style="virtual.enabled.value ? { position: 'relative' } : {}"
    >
      <div v-if="virtual.enabled.value" :style="{ height: `${virtual.totalHeight.value}px`, width: '100%', pointerEvents: 'none' }" />
      <ShelfItemRow
        v-for="entry in virtual.entries.value"
        :key="entry.item.id"
        :item="entry.item"
        :busy="busy"
        :data-virtual-index="entry.index"
        :data-virtual-id="entry.item.id"
        :role="'listitem'"
        :aria-setsize="batch.items.length"
        :aria-posinset="entry.index + 1"
        :style="virtual.position(entry.index)"
        @pin="(id, p) => { setItemPinned(id, p); announce(p ? 'Item pinned' : 'Item unpinned'); }"
        @remove="(id) => { removeItem(id); announce('Item removed'); }"
        @drag="startItemDrag"
      />
    </ul>
    <div v-if="status" class="status" role="alert">
      <span>{{ status }}</span>
      <button class="dismiss" type="button" aria-label="Dismiss status" @click="dismissStatus">×</button>
    </div>
    <div class="sr-only" role="status" aria-live="polite" aria-atomic="true">
      {{ announcement }}
    </div>
  </main>
</template>

<style scoped>
.management { display: flex; flex-direction: column; height: 100%; box-sizing: border-box; border: 1px solid var(--dc-border-default); border-radius: 9px; color: var(--dc-text-primary); background: var(--dc-surface); font-family: "Segoe UI", sans-serif; }
.header { display: flex; align-items: center; justify-content: space-between; gap: 6px; padding: 9px; }
.header-info { display: flex; align-items: baseline; gap: 8px; }
.title { margin: 0; font-size: 13px; color: var(--dc-text-primary); }
.count { color: var(--dc-text-muted); font-weight: normal; }
.summary { color: var(--dc-text-muted); font-size: 11px; }
.header-actions { display: flex; align-items: center; gap: 6px; }
.close { border: 1px solid var(--dc-border-default); border-radius: 4px; padding: 3px 6px; color: inherit; background: transparent; font: inherit; font-size: 11px; cursor: pointer; }
.close:hover { border-color: var(--dc-border-hover); background: var(--dc-surface-hover); }
.close:focus-visible, .dismiss:focus-visible { outline: 2px solid var(--dc-focus); }
.close:disabled { opacity: var(--dc-disabled-opacity); cursor: default; }
.items { display: flex; flex-direction: column; gap: 5px; min-height: 0; overflow-y: auto; overflow-x: hidden; margin: 0; padding: 0 7px 7px; list-style: none; }
.status { display: flex; align-items: start; gap: 4px; flex-shrink: 0; margin: auto 7px 7px; padding: 6px; border: 1px solid var(--dc-warning-border); background: var(--dc-warning-surface); color: var(--dc-warning-text); border-radius: 5px; font-size: 11px; overflow-wrap: anywhere; }
.dismiss { margin-left: auto; border: 0; color: inherit; background: transparent; cursor: pointer; }

@media (forced-colors: active) {
  .management {
    border-color: ButtonText;
  }
  .close {
    border-color: ButtonText;
    background: ButtonFace;
    color: ButtonText;
  }
  .close:focus-visible, .dismiss:focus-visible {
    outline: 2px solid Highlight !important;
  }
}
</style>
