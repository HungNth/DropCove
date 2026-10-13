import { onMounted, onUnmounted, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { ShelfBatch } from "./useShelf";

export function useManageItems() {
  const batch = shallowRef<ShelfBatch | null>(null);
  const status = shallowRef("");
  const busy = shallowRef(false);
  const window = getCurrentWindow();
  const unlisteners: UnlistenFn[] = [];
  let disposed = false;
  let refreshRequest = 0;

  async function refresh() {
    const request = ++refreshRequest;
    const projection = await invoke<ShelfBatch | null>("get_manage_items");
    if (!disposed && request === refreshRequest) batch.value = projection;
  }

  async function perform(command: string, arguments_: Record<string, unknown> = {}) {
    if (busy.value || disposed) return;
    busy.value = true;
    try {
      await invoke(command, arguments_);
      await refresh();
      if (!disposed) status.value = "";
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally {
      if (!disposed) busy.value = false;
    }
  }

  onMounted(async () => {
    try {
      for (const event of ["shelf-changed", "manage-items-opened"]) {
        const unlisten = await window.listen(event, async () => {
          try { await refresh(); }
          catch (error) { if (!disposed) status.value = String(error); }
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
    batch.value = null;
    for (const unlisten of unlisteners) unlisten();
  });

  return {
    batch,
    status,
    busy,
    refresh,
    setItemPinned: (itemId: string, pinned: boolean) => perform("set_item_pinned", { itemId, pinned }),
    setBatchPinned: (batchId: string, pinned: boolean) => perform("set_batch_pinned", { batchId, pinned }),
    removeItem: (itemId: string) => perform("remove_item", { itemId }),
    startItemDrag: (itemId: string) => perform("start_item_drag", { itemId }),
    close: () => perform("close_manage_items"),
    dismissStatus: () => { status.value = ""; },
  };
}
