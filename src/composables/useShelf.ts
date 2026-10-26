import { computed, nextTick, onMounted, onUnmounted, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { currentMonitor, getCurrentWindow, LogicalSize, PhysicalPosition } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { useSelection } from "./useSelection.ts";
import { usePreparedDrag } from "./usePreparedDrag.ts";

export interface ShelfItem {
  id: string;
  batchId: string;
  position: number;
  path: string;
  name: string;
  isFolder: boolean;
  isPinned: boolean;
  availability: "Available" | "Missing" | "Unavailable";
  ownership: "External" | "Managed";
  managedContentKind: "Text" | "Image" | null;
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
  warning?: string | null;
}
interface DropResult {
  snapshot: ShelfSnapshot | null;
  skipped: number;
  message?: string;
}

interface SelectionModifiers {
  ctrlKey?: boolean;
  shiftKey?: boolean;
}

export type ResizeEdge = "North" | "NorthEast" | "East" | "SouthEast" | "South" | "SouthWest" | "West" | "NorthWest";

export function useShelf() {
  const batches = shallowRef<ShelfBatch[]>([]);
  const status = shallowRef("");
  const busy = shallowRef(false);
  const draggingOver = shallowRef(false);
  const resizing = shallowRef(false);
  const moving = shallowRef(false);
  const confirmingClearTemporary = shallowRef(false);
  const activeItemActionsMenuId = shallowRef<string | null>(null);
  let pendingRecoveryWarning: string | null = null;
  const window = getCurrentWindow();
  const unlisteners: UnlistenFn[] = [];
  let disposed = false;
  let refreshRequest = 0;
  const selection = useSelection(batches);
  const batchDrag = usePreparedDrag("batch", batches, selection.ids, busy, status, refresh, selection.clear);
  const pendingConfirmation = batchDrag.pending;

  const hasTemporaryItems = computed(() =>
    batches.value.some((batch) => batch.items.some((item) => !item.isPinned))
  );

  async function apply(snapshot: ShelfSnapshot, request: number) {
    if (disposed) return;
    const [monitor, scale, current, outer, position] = await Promise.all([
      currentMonitor(), window.scaleFactor(), window.innerSize(), window.outerSize(), window.outerPosition(),
    ]);
    if (disposed || request !== refreshRequest) return;
    batches.value = snapshot.batches;
    if (resizing.value || moving.value || await invoke<boolean>("get_shelf_resizing")) return;
    if (disposed || request !== refreshRequest) return;
    const borderWidth = outer.width - current.width;
    const borderHeight = outer.height - current.height;
    const width = Math.min(snapshot.preferredWidth, monitor ? (monitor.workArea.size.width - borderWidth) / scale : snapshot.preferredWidth);
    const height = Math.min(snapshot.height, monitor ? (monitor.workArea.size.height - borderHeight) / scale : snapshot.height);
    if (Math.abs(current.width / scale - width) > .5 || Math.abs(current.height / scale - height) > .5) {
      await window.setSize(new LogicalSize(width, height));
    }
    if (monitor && !disposed && request === refreshRequest) {
      const area = monitor.workArea;
      const x = Math.max(area.position.x, Math.min(position.x, area.position.x + area.size.width - width * scale - borderWidth));
      const y = Math.max(area.position.y, Math.min(position.y, area.position.y + area.size.height - height * scale - borderHeight));
      if (Math.abs(x - position.x) > .5 || Math.abs(y - position.y) > .5) await window.setPosition(new PhysicalPosition(x, y));
    }
  }

  async function requestClearTemporary() {
    if (busy.value || pendingConfirmation.value || !hasTemporaryItems.value) return;
    try {
      await invoke("request_drop_shelf_clear_temporary");
    } catch (error) {
      if (!disposed) status.value = String(error);
    }
  }

  async function refresh() {
    const request = ++refreshRequest;
    const snapshot = await invoke<ShelfSnapshot>("get_shelf");
    if (!disposed && request === refreshRequest) {
      await apply(snapshot, request);
      const previousWarning = pendingRecoveryWarning;
      pendingRecoveryWarning = snapshot.warning ?? null;
      if (!disposed && (!status.value || status.value === previousWarning)) {
        status.value = pendingRecoveryWarning ?? "";
      }
    }
  }
  async function runAction(command: string, arguments_: Record<string, unknown> = {}) {
    const control = document.activeElement?.closest(".card-actions") ? document.activeElement as HTMLElement : null;
    busy.value = true;
    try {
      await invoke<ShelfSnapshot>(command, arguments_);
      await refresh();
      if (!disposed) status.value = pendingRecoveryWarning ?? "";
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally {
      if (!disposed) {
        busy.value = false;
        await nextTick();
        if (control?.isConnected && document.activeElement === document.body) control.focus();
      }
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

  async function onKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape" || event.defaultPrevented || event.repeat) return;
    if (batchDrag.hasPrepared()) {
      await batchDrag.cancel();
      return;
    }
    if (
      confirmingClearTemporary.value ||
      busy.value ||
      resizing.value ||
      moving.value
    ) {
      return;
    }
    const selectedAtEscape = selection.ids.value.join(",");
    try {
      const active = await invoke<ShelfBatch | null>("get_manage_items");
      if (active) {
        await invoke("close_manage_items");
        return;
      }
      if (selectedAtEscape) {
        if (selection.ids.value.join(",") === selectedAtEscape) selection.clear();
        return;
      }
      if (selection.ids.value.length) return;
      await invoke("dismiss_drop_shelf");
    } catch (error) {
      if (!disposed) status.value = String(error);
    }
  }

  onMounted(async () => {
    document.addEventListener("keydown", onKeydown);
    try {
      await retainListener(window.listen("shelf-hidden", () => {
        activeItemActionsMenuId.value = null;
        selection.clear();
        void batchDrag.cancel();
      }));
      await retainListener(window.listen<boolean>("shelf-resize-state", async ({ payload }) => {
        if (disposed) return;
        resizing.value = payload;
        if (payload && batchDrag.hasPrepared()) await batchDrag.cancel();
        if (!payload) {
          moving.value = false;
          try { await refresh(); }
          catch (error) { if (!disposed) status.value = String(error); }
        }
      }));
      await retainListener(window.listen("shelf-changed", async () => {
        moving.value = false;
        try { await refresh(); }
        catch (error) { if (!disposed) status.value = String(error); }
      }));
      await retainListener(window.listen<DropResult>("shelf-drop", async ({ payload }) => {
        draggingOver.value = false;
        if (!payload.snapshot || disposed) return;
        try {
          await refresh();
          status.value = payload.message ?? (payload.skipped ? `${payload.skipped} unsupported item(s) skipped.` : (pendingRecoveryWarning ?? ""));
        } catch (error) { status.value = String(error); }
      }));
      await retainListener(window.listen<boolean>("shelf-drag-over", ({ payload }) => {
        if (!disposed) draggingOver.value = payload;
      }));
      await retainListener(window.listen<string>("shelf-error", ({ payload }) => {
        if (!disposed) status.value = payload;
      }));
      await retainListener(window.listen<string>("manage-items-closed", ({ payload: batchId }) => {
        if (disposed) return;
        const escaped = CSS.escape(batchId);
        const card = document.querySelector<HTMLElement>(`[data-batch-id="${escaped}"]`);
        const target = card?.querySelector<HTMLElement>(".manage") ?? card?.querySelector<HTMLElement>(".card-drag");
        target?.focus();
      }));
      await retainListener(window.listen<boolean>("shelf-clear-confirmation-state", ({ payload }) => {
        if (disposed) return;
        confirmingClearTemporary.value = payload;
      }));
      await retainListener(window.listen("shelf-clear-confirmation-closed", async () => {
        if (disposed) return;
        confirmingClearTemporary.value = false;
        await nextTick();
        const clearButton = document.querySelector<HTMLElement>(".clear-temporary-action");
        clearButton?.focus();
      }));
      confirmingClearTemporary.value = await invoke<boolean>("get_drop_shelf_clear_temporary_state");
      await refresh();
    } catch (error) {
      if (!disposed) status.value = String(error);
    }
  });

  onUnmounted(() => {
    batchDrag.dispose();
    disposed = true;
    document.removeEventListener("keydown", onKeydown);
    for (const unlisten of unlisteners) unlisten();
  });

  return {
    batches,
    activeItemActionsMenuId,
    selection,
    selectBatch: (batchId: string, modifiers: SelectionModifiers) => {
      if (!busy.value && !resizing.value && !moving.value && !confirmingClearTemporary.value) selection.select(batchId, modifiers);
    },
    status,
    busy,
    draggingOver,
    resizing,
    moving,
    startResize: async (edge: ResizeEdge) => {
      if (busy.value || resizing.value || moving.value) return;
      try { await invoke("start_shelf_resize", { edge }); }
      catch (error) { if (!disposed) status.value = String(error); }
    },
    startMove: async (event: PointerEvent) => {
      if (
        event.button !== 0 ||
        busy.value ||
        resizing.value ||
        moving.value ||
        pendingConfirmation.value ||
        confirmingClearTemporary.value ||
        draggingOver.value
      ) {
        return;
      }
      moving.value = true;
      try {
        await invoke("prepare_shelf_move");
        await window.startDragging();
      } catch (error) {
        if (!disposed) status.value = String(error);
      } finally {
        if (!disposed) moving.value = false;
      }
    },
    pendingConfirmation,
    hasTemporaryItems,
    confirmingClearTemporary,
    requestClearTemporary,
    confirmBatchDrag: batchDrag.confirm,
    dragConfirmedBatch: batchDrag.dragConfirmed,
    cancelBatchDrag: batchDrag.cancel,
    removeBatch: (batchId: string) => perform("remove_batch", { batchId }),
    setItemPinned: (itemId: string, pinned: boolean) => perform("set_item_pinned", { itemId, pinned }),
    setBatchPinned: (batchId: string, pinned: boolean) => perform("set_batch_pinned", { batchId, pinned }),
    openManageItems: async (batchId: string, anchor: ShelfAnchor, keyboard = false) => {
      if (busy.value) return;
      try { await invoke("open_manage_items", { batchId, anchor, keyboard }); }
      catch (error) { if (!disposed) status.value = String(error); }
    },
    startItemDrag: (itemId: string) => perform("start_item_drag", { itemId }),
    startBatchDrag: (batchId: string) => batchDrag.start(selection.dragIds(batchId)),
    dismissDropShelf: async () => {
      if (busy.value) return;
      try {
        await invoke("dismiss_drop_shelf");
      } catch (error) {
        if (!disposed) status.value = String(error);
      }
    },
    dismissStatus: () => {
      const current = status.value;
      if (pendingRecoveryWarning && current === pendingRecoveryWarning) {
        pendingRecoveryWarning = null;
        status.value = "";
        void invoke("acknowledge_shelf_warning");
      } else {
        status.value = pendingRecoveryWarning ?? "";
      }
    },
  };
}
