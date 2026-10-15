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
import ActionIcon from "./ActionIcon.vue";

const { batch, status, busy, refresh, setItemPinned, setBatchPinned, removeItem, startItemDrag, close, dismissStatus } = useManageItems();
const { announcement, announce } = useAnnouncer();

const pinnedCount = computed(() => batch.value?.items.filter((item) => item.isPinned).length ?? 0);
const totalCount = computed(() => batch.value?.items.length ?? 0);

const rootRef = useTemplateRef<HTMLElement>("root");
const itemsContainerRef = useTemplateRef<HTMLElement>("itemsContainer");
const itemsList = computed(() => batch.value?.items ?? []);
const virtual = useVirtualCollection(itemsList, itemsContainerRef, 1, 54, 5);
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
    class="management flex flex-col h-full box-border border border-dc-border-default rounded-[9px] text-dc-text-primary bg-dc-surface font-['Segoe_UI',sans-serif] transition-[opacity,transform] duration-[140ms,140ms] ease-[cubic-bezier(0.16,1,0.3,1)] forced-colors:border-[ButtonText]"
    :aria-busy="busy"
    aria-label="Manage Items"
  >
    <header class="header flex items-center justify-between gap-[6px] p-[9px]">
      <div class="header-info flex min-w-0 flex-col gap-[2px]">
        <h1 class="title m-0 text-[13px] text-dc-text-primary">Manage Items<span v-if="batch" class="count text-dc-text-muted font-normal"> · {{ batch.items.length }}</span></h1>
        <span v-if="batch" class="summary text-dc-text-muted text-[11px]">{{ pinnedCount }}/{{ totalCount }} pinned</span>
      </div>
      <div class="header-actions flex items-center gap-[6px]">
        <BulkPinning
          v-if="batch"
          :batch="batch"
          :busy="busy"
          @pin="(id, pinned) => setBatchPinned(id, pinned)"
        />
        <button
          class="close inline-flex items-center justify-center flex-none size-[24px] box-border p-0 border border-transparent rounded-[4px] bg-transparent text-dc-text-secondary cursor-pointer [&:hover:not(:disabled)]:border-dc-border-hover [&:hover:not(:disabled)]:bg-dc-surface-hover [&:hover:not(:disabled)]:text-dc-text-primary aria-pressed:border-dc-accent aria-pressed:text-dc-accent aria-checked:border-dc-accent aria-checked:text-dc-accent aria-[checked=mixed]:border-dc-warning-border aria-[checked=mixed]:text-dc-warning disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:border-[ButtonText] forced-colors:[&:hover:not(:disabled)]:bg-[Highlight] forced-colors:[&:hover:not(:disabled)]:text-[HighlightText] forced-colors:aria-pressed:text-[Highlight] forced-colors:aria-pressed:border-[Highlight] forced-colors:aria-checked:text-[Highlight] forced-colors:aria-checked:border-[Highlight] forced-colors:aria-[checked=mixed]:text-[LinkText] forced-colors:aria-[checked=mixed]:border-[LinkText]"
          type="button"
          :disabled="busy"
          aria-label="Close Manage Items"
          title="Close Manage Items"
          @click="close"
        ><ActionIcon name="close" /></button>
      </div>
    </header>
    <div v-if="batch" class="flex flex-col flex-1 min-h-0 pt-[4px]">
      <ul
        ref="itemsContainer"
        class="items flex flex-col gap-[5px] min-h-0 overflow-y-auto overflow-x-hidden m-0 pt-0 px-[7px] pb-[3px] list-none"
        role="list"
        :aria-label="`Batch items (${batch.items.length})`"
        :style="virtual.enabled.value ? { position: 'relative' } : {}"
      >
      <div v-if="virtual.enabled.value" :style="{ height: `${virtual.totalHeight.value}px`, flex: 'none', width: '100%', pointerEvents: 'none' }" />
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
    </div>
    <div v-if="status" class="status flex [align-items:start] gap-[4px] shrink-0 mt-auto mx-[7px] mb-[7px] p-[6px] border border-dc-warning-border bg-dc-warning-surface text-dc-warning-text rounded-[5px] text-[11px] [overflow-wrap:anywhere]" role="alert">
      <span>{{ status }}</span>
      <button class="dismiss ml-auto border-0 border-none text-inherit bg-transparent cursor-pointer" type="button" aria-label="Dismiss status" @click="dismissStatus">×</button>
    </div>
    <div class="sr-only" role="status" aria-live="polite" aria-atomic="true">
      {{ announcement }}
    </div>
  </main>
</template>

