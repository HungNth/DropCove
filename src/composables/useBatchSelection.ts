import { computed, shallowRef, watch, type Ref } from "vue";

interface SelectableBatch {
  id: string;
  items: readonly { id: string }[];
}

export interface SelectionModifiers {
  ctrlKey?: boolean;
  shiftKey?: boolean;
}

export function useBatchSelection(batches: Ref<readonly SelectableBatch[]>) {
  const selected = shallowRef<ReadonlySet<string>>(new Set());
  const anchor = shallowRef<string | null>(null);
  const ids = computed(() => batches.value.filter(batch => selected.value.has(batch.id)).map(batch => batch.id));
  const itemCount = computed(() => batches.value.reduce((count, batch) => count + (selected.value.has(batch.id) ? batch.items.length : 0), 0));

  function clear() {
    if (selected.value.size) selected.value = new Set();
    anchor.value = null;
  }

  function select(id: string, modifiers: SelectionModifiers) {
    if (!batches.value.some(batch => batch.id === id)) return;
    if (modifiers.shiftKey && anchor.value !== null) {
      const start = batches.value.findIndex(batch => batch.id === anchor.value);
      const end = batches.value.findIndex(batch => batch.id === id);
      selected.value = new Set(batches.value.slice(Math.min(start, end), Math.max(start, end) + 1).map(batch => batch.id));
      return;
    }
    if (modifiers.ctrlKey && !modifiers.shiftKey) {
      const next = new Set(selected.value);
      if (!next.delete(id)) next.add(id);
      selected.value = next;
    } else {
      selected.value = new Set([id]);
    }
    anchor.value = id;
  }

  watch(batches, current => {
    const existing = new Set(current.map(batch => batch.id));
    if (Array.from(selected.value).some(id => !existing.has(id))) {
      selected.value = new Set(Array.from(selected.value).filter(id => existing.has(id)));
    }
    if (anchor.value !== null && !existing.has(anchor.value)) anchor.value = null;
  }, { flush: "sync" });

  return {
    selected,
    ids,
    itemCount,
    select,
    clear,
    selectAll: () => { selected.value = new Set(batches.value.map(batch => batch.id)); },
    dragIds: (id: string) => selected.value.has(id) ? ids.value : [id],
  };
}
