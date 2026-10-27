import { computed, onMounted, onUnmounted, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { ShelfBatch } from "./useShelf";
import { useSelection } from "./useSelection";
import { usePreparedDrag } from "./usePreparedDrag";
import type { RemovalPreview } from "./selectionActions";

export function useManageItems() {
  const batch = shallowRef<ShelfBatch | null>(null);
  const status = shallowRef("");
  const busy = shallowRef(false);
  const selectionMenu = shallowRef<{ itemIds: string[]; anchor: { x: number; y: number }; keyboard: boolean; originId: string } | null>(null);
  const removalConfirmation = shallowRef<{ batchId: string; itemIds: string[]; originId: string; preview: RemovalPreview } | null>(null);
  const window = getCurrentWindow();
  const unlisteners: UnlistenFn[] = [];
  let disposed = false;
  let refreshRequest = 0;
  const items = computed(() => batch.value?.items ?? []);
  const selection = useSelection(items);
  const projection = computed(() => batch.value ? [batch.value] : []);
  const itemDrag = usePreparedDrag("item", projection, selection.ids, busy, status, refresh, selection.clear);

  async function refresh() {
    const request = ++refreshRequest;
    const projection = await invoke<ShelfBatch | null>("get_manage_items");
    if (!disposed && request === refreshRequest) batch.value = projection;
  }

  async function perform(command: string, arguments_: Record<string, unknown> = {}): Promise<boolean> {
    if (busy.value || disposed || removalConfirmation.value) return false;
    busy.value = true;
    let succeeded = false;
    try {
      await itemDrag.cancel();
      const result = await invoke<{ warning?: string | null } | null>(command, arguments_);
      succeeded = true;
      await refresh();
      if (!disposed) status.value = result?.warning ?? "";
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally {
      if (!disposed) busy.value = false;
    }
    return succeeded;
  }

  onMounted(async () => {
    try {
      for (const event of ["shelf-changed", "manage-items-opened"]) {
        const unlisten = await window.listen(event, async () => {
          try {
            if (itemDrag.hasPrepared()) await itemDrag.cancel();
            if (event === "manage-items-opened") { selection.clear(); selectionMenu.value = null; removalConfirmation.value = null; }
            await refresh();
          } catch (error) { if (!disposed) status.value = String(error); }
        });
        if (disposed) unlisten();
        else unlisteners.push(unlisten);
      }
      await refresh();
    } catch (error) {
      if (!disposed) status.value = String(error);
    }
  });

  onUnmounted(() => {
    disposed = true;
    itemDrag.dispose();
    selection.clear();
    batch.value = null;
    for (const unlisten of unlisteners) unlisten();
  });

  // Secondary-click keeps a selected target set; an unselected row replaces selection first.
  function openSelectionMenu(itemId: string, anchor: { x: number; y: number }, keyboard: boolean) {
    if (busy.value || disposed || itemDrag.pending.value || removalConfirmation.value) return;
    if (!items.value.some(item => item.id === itemId)) return;
    if (!selection.selected.value.has(itemId)) selection.select(itemId, {});
    selectionMenu.value = { itemIds: [...selection.ids.value], anchor, keyboard, originId: itemId };
  }

  // Resolves true only when the core mutation committed; failures surface through status.
  async function pinSelection(pinned: boolean): Promise<boolean> {
    const target = selectionMenu.value;
    const active = batch.value;
    selectionMenu.value = null;
    if (!target || !active || busy.value || removalConfirmation.value) return false;
    return perform("set_items_pinned", { batchId: active.id, itemIds: target.itemIds, pinned });
  }

  async function removeSelection(): Promise<boolean> {
    const target = selectionMenu.value;
    const active = batch.value;
    selectionMenu.value = null;
    if (!target || !active || busy.value || removalConfirmation.value) return false;
    if (target.itemIds.length === 1) return perform("remove_item", { itemId: target.itemIds[0] });
    busy.value = true;
    try {
      const preview = await invoke<RemovalPreview>("prepare_items_removal", { batchId: active.id, itemIds: target.itemIds });
      if (disposed || batch.value?.id !== active.id || JSON.stringify(selection.ids.value) !== JSON.stringify(target.itemIds)) {
        await invoke("cancel_selection_removal");
        return false;
      }
      removalConfirmation.value = { batchId: active.id, itemIds: target.itemIds, originId: target.originId, preview };
      return true;
    } catch (error) {
      if (!disposed) status.value = String(error);
      return false;
    } finally {
      if (!disposed) busy.value = false;
    }
  }

  async function confirmRemoval(): Promise<boolean> {
    const target = removalConfirmation.value;
    if (!target || busy.value) return false;
    removalConfirmation.value = null;
    return perform("remove_items", { batchId: target.batchId, itemIds: target.itemIds, token: target.preview.token });
  }

  async function cancelRemoval() {
    if (!removalConfirmation.value) return;
    try {
      await invoke("cancel_selection_removal");
      removalConfirmation.value = null;
    } catch (error) { if (!disposed) status.value = String(error); }
  }

  return {
    batch,
    status,
    busy,
    selectionMenu,
    openSelectionMenu,
    pinSelection,
    removeSelection,
    removalConfirmation,
    confirmRemoval,
    cancelRemoval,
    refresh,
    setItemPinned: (itemId: string, pinned: boolean) => perform("set_item_pinned", { itemId, pinned }),
    setBatchPinned: (batchId: string, pinned: boolean) => perform("set_batch_pinned", { batchId, pinned }),
    removeItem: (itemId: string) => perform("remove_item", { itemId }),
    startItemDrag: async (itemId: string) => {
      if (!removalConfirmation.value) await itemDrag.start(selection.dragIds(itemId));
    },
    cancelPreparedDrag: itemDrag.cancel,
    confirmItemDrag: itemDrag.confirm,
    dragConfirmedItem: itemDrag.dragConfirmed,
    pendingConfirmation: itemDrag.pending,
    selection,
    hasPreparedDrag: itemDrag.hasPrepared,
    selectItem: (itemId: string, modifiers: Parameters<typeof selection.select>[1]) => { if (!busy.value && !removalConfirmation.value) selection.select(itemId, modifiers); },
    selectAllItems: () => { if (!busy.value && !removalConfirmation.value) selection.selectAll(); },
    clearSelection: () => { if (!busy.value && !removalConfirmation.value) selection.clear(); },
    close: () => perform("close_manage_items"),
    dismissStatus: () => { status.value = ""; },
  };
}
