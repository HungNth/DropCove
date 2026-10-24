import { shallowRef, watch, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ShelfBatch } from "./useShelf";

interface Preparation {
  token: string | null;
  available: number;
  unavailable: number;
  snapshot: { batches: ShelfBatch[] };
}

interface BatchDragResult {
  outcome: "copy" | "rejected" | "canceled";
  error: string | null;
}
interface Confirmation {
  token: string;
  available: number;
  unavailable: number;
  batchIds: string[];
  binding: string;
}

export function useBatchDrag(
  batches: Ref<ShelfBatch[]>,
  selectedIds: Readonly<Ref<string[]>>,
  busy: Ref<boolean>,
  status: Ref<string>,
  refresh: () => Promise<void>,
  copied: () => void,
  hold: (active: boolean) => Promise<boolean> = async () => true,
) {
  const pending = shallowRef<Confirmation | null>(null);
  let ready: Confirmation | null = null;
  let disposed = false;
  let generation = 0;
  const binding = () => JSON.stringify([selectedIds.value, batches.value]);
  let holdEpoch = 0;

  async function releaseHold(epoch: number) {
    if (!epoch || epoch !== holdEpoch) return true;
    try {
      if (await hold(false)) {
        if (epoch === holdEpoch) holdEpoch = 0;
        return true;
      }
      if (!disposed && epoch === holdEpoch) status.value = [status.value, "Rail hold release failed."].filter(Boolean).join("; ");
    } catch (error) {
      if (!disposed && epoch === holdEpoch) status.value = [status.value, `Rail hold release failed: ${error}`].filter(Boolean).join("; ");
    }
    return false;
  }


  async function cancel() {
    const activeHold = holdEpoch;
    generation++;
    const prepared = pending.value ?? ready;
    if (ready) status.value = "";
    pending.value = null;
    ready = null;
    let canceled = true;
    try {
      if (prepared) await invoke("cancel_batch_drag", { token: prepared.token });
    } catch (error) {
      canceled = false;
      if (!disposed) status.value = String(error);
    }
    const released = await releaseHold(activeHold);
    return canceled && released;
  }

  async function dispatch(token: string) {
    const activeHold = holdEpoch;
    pending.value = null;
    ready = null;
    busy.value = true;
    status.value = "";
    try {
      // Hold the Rail until native ActiveDrag has taken ownership.
      const result = await invoke<BatchDragResult>("start_batch_drag", { token });
      if (result.outcome === "copy") copied();
      try {
        await refresh();
      } catch (error) {
        if (!disposed) status.value = [result.error, String(error)].filter(Boolean).join("; ");
        return;
      }
      if (result.error && !disposed) status.value = result.error;
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally {
      await releaseHold(activeHold);
      if (!disposed) busy.value = false;
    }
  }

  async function start(batchIds: string[]) {
    if (busy.value || pending.value || disposed) return;
    busy.value = true;
    if (ready && ready.binding === binding() && JSON.stringify(ready.batchIds) === JSON.stringify(batchIds)) {
      await dispatch(ready.token);
      return;
    }
    const request = generation + 1;
    const selectedAtStart = [...selectedIds.value];
    let ownedToken: string | null = null;
    try {
      if (!await cancel()) {
        status.value = status.value || "Rail hold release failed.";
        return;
      }
      if (disposed || request !== generation) return;
      const preparation = await invoke<Preparation>("prepare_batch_drag", { batchIds });
      ownedToken = preparation.token;
      await refresh();
      if (disposed || request !== generation) return;
      if (!preparation.token || !preparation.available) {
        status.value = "No available items remain in these batches.";
        return;
      }
      const survivingSelection = selectedAtStart.filter(id => preparation.snapshot.batches.some(batch => batch.id === id));
      const preparedBinding = JSON.stringify([survivingSelection, preparation.snapshot.batches]);
      if (binding() !== preparedBinding) {
        status.value = "Shelf or selection changed while preparing this drag; drag again.";
        return;
      }
      if (preparation.unavailable) {
        if (!await hold(true)) return;
        holdEpoch = request;
        if (disposed || request !== generation || binding() !== preparedBinding) return;
        pending.value = { ...preparation, token: preparation.token, batchIds, binding: preparedBinding };
        ownedToken = null;
      } else {
        ownedToken = null;
        await dispatch(preparation.token);
      }
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally {
      const epoch = request;
      if (ownedToken) {
        try {
          await invoke("cancel_batch_drag", { token: ownedToken });
        } catch (error) {
          if (!disposed) status.value = `${status.value}; prepared drag cancellation failed: ${error}`;
        }
        await releaseHold(epoch);
      }
      if (!disposed) busy.value = false;
    }
  }

  watch(binding, () => {
    if ((pending.value || ready) && (pending.value ?? ready)?.binding !== binding()) void cancel();
  }, { flush: "sync" });

  return {
    pending,
    hasPrepared: () => Boolean(pending.value || ready),
    start,
    cancel,
    confirm: () => {
      ready = pending.value;
      pending.value = null;
      if (ready) status.value = `Ready: drag the same batches again to copy ${ready.available} available item(s).`;
    },
    dragConfirmed: () => {
      const prepared = pending.value ?? ready;
      if (prepared && prepared.binding === binding()) void dispatch(prepared.token);
    },
    dispose: () => { disposed = true; void cancel(); },
  };
}
