import { onMounted, onUnmounted, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { currentMonitor, getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";

export interface ShelfItem {
  id: string;
  batchId: string;
  position: number;
  path: string;
  name: string;
  isFolder: boolean;
  isPinned: boolean;
  availability: "Available" | "Missing" | "Unavailable";
}

export interface ShelfBatch {
  id: string;
  createdAt: number;
  position: number;
  items: ShelfItem[];
}

export interface ShelfAnchor {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface ShelfSnapshot {
  batches: ShelfBatch[];
  preferredWidth: number;
  height: number;
}

interface DropResult {
  snapshot: ShelfSnapshot | null;
  skipped: number;
}

interface BatchPreparation {
  token: string | null;
  available: number;
  unavailable: number;
  snapshot: ShelfSnapshot;
}

interface PendingDragConfirmation {
  batchId: string;
  token: string;
  available: number;
  unavailable: number;
}

export function useShelf() {
  const batches = shallowRef<ShelfBatch[]>([]);
  const status = shallowRef("");
  const busy = shallowRef(false);
  const draggingOver = shallowRef(false);
  const pendingConfirmation = shallowRef<PendingDragConfirmation | null>(null);
  let readyConfirmation: PendingDragConfirmation | null = null;
  const window = getCurrentWindow();
  const unlisteners: UnlistenFn[] = [];
  let disposed = false;
  let appliedWidth = 180;
  let refreshRequest = 0;

  async function apply(snapshot: ShelfSnapshot, request: number) {
    if (disposed) return;
    const [monitor, scale, current] = await Promise.all([currentMonitor(), window.scaleFactor(), window.innerSize()]);
    if (disposed || request !== refreshRequest) return;
    batches.value = snapshot.batches;
    const width = Math.min(snapshot.preferredWidth, monitor ? monitor.workArea.size.width / scale : snapshot.preferredWidth);
    const height = Math.min(snapshot.height, monitor ? monitor.workArea.size.height / scale : snapshot.height);
    appliedWidth = width;
    if (Math.abs(current.width / scale - width) > .5 || Math.abs(current.height / scale - height) > .5) {
      await window.setSize(new LogicalSize(width, height));
    }
  }

  async function refresh() {
    const request = ++refreshRequest;
    const snapshot = await invoke<ShelfSnapshot>("get_shelf");
    if (!disposed && request === refreshRequest) await apply(snapshot, request);
  }
  async function runAction(command: string, arguments_: Record<string, unknown> = {}) {
    busy.value = true;
    try {
      await invoke<ShelfSnapshot>(command, arguments_);
      await refresh();
      if (!disposed) status.value = "";
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally {
      if (!disposed) busy.value = false;
    }
  }

  async function perform(command: string, arguments_: Record<string, unknown> = {}) {
    if (busy.value) return;
    await runAction(command, arguments_);
  }

  async function retainListener(listener: Promise<UnlistenFn>) {
    const unlisten = await listener;
    if (disposed) unlisten();
    else unlisteners.push(unlisten);
  }

  async function dispatchBatchDrag(token: string) {
    pendingConfirmation.value = null;
    readyConfirmation = null;
    await runAction("start_batch_drag", { token });
  }

  async function cancelPreparedBatch() {
    const pending = pendingConfirmation.value ?? readyConfirmation;
    pendingConfirmation.value = null;
    readyConfirmation = null;
    if (pending) await runAction("cancel_batch_drag", { token: pending.token });
  }

  async function startBatchDrag(batchId: string) {
    if (busy.value || pendingConfirmation.value) return;
    if (readyConfirmation?.batchId === batchId) {
      await dispatchBatchDrag(readyConfirmation.token);
      return;
    }
    if (readyConfirmation) await cancelPreparedBatch();
    busy.value = true;
    try {
      const preparation = await invoke<BatchPreparation>("prepare_batch_drag", { batchId });
      await refresh();
      if (!preparation.token || preparation.available === 0) {
        status.value = "No available items remain in this batch.";
        return;
      }
      if (preparation.unavailable > 0) {
        pendingConfirmation.value = {
          batchId,
          token: preparation.token,
          available: preparation.available,
          unavailable: preparation.unavailable,
        };
        return;
      }
      await runAction("start_batch_drag", { token: preparation.token });
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally {
      if (!disposed) busy.value = false;
    }
  }

  onMounted(async () => {
    try {
      await retainListener(window.listen("shelf-changed", async () => {
        try { await refresh(); }
        catch (error) { if (!disposed) status.value = String(error); }
      }));
      await retainListener(window.listen<DropResult>("shelf-drop", async ({ payload }) => {
        draggingOver.value = false;
        if (!payload.snapshot || disposed) return;
        try {
          await refresh();
          status.value = payload.skipped ? `${payload.skipped} unsupported item(s) skipped.` : "";
        } catch (error) { status.value = String(error); }
      }));
      await retainListener(window.listen<boolean>("shelf-drag-over", ({ payload }) => {
        if (!disposed) draggingOver.value = payload;
      }));
      await retainListener(window.listen<string>("shelf-error", ({ payload }) => {
        if (!disposed) status.value = payload;
      }));
      await retainListener(window.onResized(async ({ payload }) => {
        if (disposed) return;
        try {
          const width = payload.width / await window.scaleFactor();
          if (Math.abs(width - appliedWidth) <= .5 || width < 180) return;
          appliedWidth = width;
          await invoke<ShelfSnapshot>("save_width", { width });
          await refresh();
        } catch (error) { if (!disposed) status.value = String(error); }
      }));
      await refresh();
    } catch (error) {
      if (!disposed) status.value = String(error);
    }
  });

  onUnmounted(() => {
    disposed = true;
    for (const unlisten of unlisteners) unlisten();
  });

  return {
    batches,
    status,
    busy,
    draggingOver,
    pendingConfirmation,
    confirmBatchDrag: () => {
      const pending = pendingConfirmation.value;
      if (!pending) return;
      readyConfirmation = pending;
      pendingConfirmation.value = null;
      status.value = `Ready: drag the batch again to copy ${pending.available} available item(s).`;
    },
    dragConfirmedBatch: () => {
      const pending = pendingConfirmation.value;
      if (pending) void dispatchBatchDrag(pending.token);
    },
    cancelBatchDrag: () => void cancelPreparedBatch(),
    removeBatch: (batchId: string) => perform("remove_batch", { batchId }),
    setItemPinned: (itemId: string, pinned: boolean) => perform("set_item_pinned", { itemId, pinned }),
    openManageItems: async (batchId: string, anchor: ShelfAnchor) => {
      if (busy.value) return;
      try { await invoke("open_manage_items", { batchId, anchor }); }
      catch (error) { if (!disposed) status.value = String(error); }
    },
    startItemDrag: (itemId: string) => perform("start_item_drag", { itemId }),
    startBatchDrag,
    dismissStatus: () => { status.value = ""; },
  };
}
