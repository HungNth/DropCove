import { computed, shallowRef, watch, type Ref } from "vue";

export interface SelectableItem {
  id: string;
  items?: readonly { id: string }[];
}

export interface SelectionModifiers {
  ctrlKey?: boolean;
  shiftKey?: boolean;
}

export function useSelection<T extends SelectableItem>(items: Ref<readonly T[]>) {
  const selected = shallowRef<ReadonlySet<string>>(new Set());
  const anchor = shallowRef<string | null>(null);
  const ids = computed(() => items.value.filter(item => selected.value.has(item.id)).map(item => item.id));
  const itemCount = computed(() => items.value.reduce((count, item) => count + (selected.value.has(item.id) ? (item.items ? item.items.length : 1) : 0), 0));

  function clear() {
    if (selected.value.size) selected.value = new Set();
    anchor.value = null;
  }

  function select(id: string, modifiers: SelectionModifiers) {
    if (!items.value.some(item => item.id === id)) return;
    if (modifiers.shiftKey && anchor.value !== null) {
      const start = items.value.findIndex(item => item.id === anchor.value);
      const end = items.value.findIndex(item => item.id === id);
      selected.value = new Set(items.value.slice(Math.min(start, end), Math.max(start, end) + 1).map(item => item.id));
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

  watch(items, current => {
    const existing = new Set(current.map(item => item.id));
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
    selectAll: () => { selected.value = new Set(items.value.map(item => item.id)); },
    dragIds: (id: string) => selected.value.has(id) ? ids.value : [id],
  };
}
