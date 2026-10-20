import { computed, nextTick, shallowRef, toValue, watch, type MaybeRefOrGetter, type Ref } from "vue";
import { useEventListener, useResizeObserver } from "@vueuse/core";

/** Viewport ownership shared by Shelf cards, Rail cards, and Manage Items rows. */
export function useVirtualCollection<T extends { id: string }>(
  items: MaybeRefOrGetter<readonly T[]>,
  container: Ref<HTMLElement | null>,
  columns: MaybeRefOrGetter<number> = 1,
  initialHeight = 80,
  gap = 4,
  leadingItems: MaybeRefOrGetter<number> = 0,
) {
  const width = shallowRef(0);
  const contentLeft = shallowRef(0);
  const height = shallowRef(0);
  const scrollTop = shallowRef(0);
  const itemHeight = shallowRef(initialHeight);
  const focusedId = shallowRef<string | null>(null);
  const enabled = computed(() => toValue(items).length > 50);
  const columnCount = computed(() => Math.max(1, Math.floor(toValue(columns))));
  const stride = computed(() => itemHeight.value + gap);
  const leadingCount = computed(() => toValue(leadingItems));
  const totalHeight = computed(() => Math.max(0, Math.ceil((toValue(items).length + leadingCount.value) / columnCount.value) * stride.value - gap));
  const start = computed(() => Math.max(0, Math.max(0, Math.floor(scrollTop.value / stride.value) - 2) * columnCount.value - leadingCount.value));
  const end = computed(() => Math.min(toValue(items).length, (Math.ceil((scrollTop.value + height.value) / stride.value) + 2) * columnCount.value - leadingCount.value));
  const entries = computed(() => {
    const source = toValue(items);
    if (!enabled.value) return source.map((item, index) => ({ item, index }));
    const result = source.slice(start.value, end.value).map((item, offset) => ({ item, index: start.value + offset }));
    const focusedIndex = focusedId.value ? source.findIndex(item => item.id === focusedId.value) : -1;
    if (focusedIndex >= 0 && (focusedIndex < start.value || focusedIndex >= end.value)) {
      result.push({ item: source[focusedIndex], index: focusedIndex });
      result.sort((a, b) => a.index - b.index);
    }
    return result;
  });
  const itemWidth = computed(() => Math.max(0, (width.value - gap * (columnCount.value - 1)) / columnCount.value));
  function positionAt(index: number) {
    if (!enabled.value) return undefined;
    return {
      position: "absolute" as const,
      top: `${Math.floor(index / columnCount.value) * stride.value}px`,
      left: `${contentLeft.value + index % columnCount.value * (itemWidth.value + gap)}px`,
      width: `${itemWidth.value}px`,
      boxSizing: "border-box" as const,
    };
  }
  function position(index: number) {
    return positionAt(index + leadingCount.value);
  }
  const leadingPosition = computed(() => positionAt(0));
  function measure() {
    const element = container.value;
    if (!element) return;
    const style = getComputedStyle(element);
    contentLeft.value = parseFloat(style.paddingLeft);
    width.value = element.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
    height.value = element.clientHeight;
    scrollTop.value = element.scrollTop;
    // Natural row height is measured from the actual surface, not pinned to a CSS guess.
    const rows = element.querySelectorAll<HTMLElement>("[data-virtual-index]");
    for (const row of rows) itemHeight.value = Math.max(itemHeight.value, row.getBoundingClientRect().height);
  }
  useResizeObserver(container, measure);
  useEventListener(container, "scroll", measure, { passive: true });
  useEventListener(container, "focusin", (event: FocusEvent) => {
    const row = (event.target as HTMLElement).closest<HTMLElement>("[data-virtual-index]");
    focusedId.value = row?.dataset.virtualId ?? null;
  });
  useEventListener(container, "focusout", (event: FocusEvent) => {
    if (!(event.relatedTarget instanceof Node) || !container.value?.contains(event.relatedTarget)) focusedId.value = null;
  });
  watch(() => toValue(items), async () => {
    await nextTick();
    if (focusedId.value && !toValue(items).some(item => item.id === focusedId.value)) focusedId.value = null;
    measure();
  }, { flush: "post" });
  watch(entries, async () => { await nextTick(); measure(); }, { flush: "post" });
  async function focus(index: number, action = 0) {
    const source = toValue(items);
    if (index < 0 || index >= source.length || !container.value) return;
    focusedId.value = source[index].id;
    if (enabled.value) {
      const top = Math.floor((index + leadingCount.value) / columnCount.value) * stride.value;
      if (top < container.value.scrollTop) container.value.scrollTop = top;
      else if (top + itemHeight.value > container.value.scrollTop + container.value.clientHeight) {
        container.value.scrollTop = top + itemHeight.value - container.value.clientHeight;
      }
      scrollTop.value = container.value.scrollTop;
    }
    await nextTick();
    const row = container.value?.querySelector<HTMLElement>(`[data-virtual-index="${index}"]`);
    const controls = row?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)");
    controls?.[action < 0 ? controls.length - 1 : action]?.focus();
  }
  async function onTab(event: KeyboardEvent) {
    if (event.key !== "Tab" || !enabled.value) return;
    const target = event.target as HTMLElement;
    const row = target.closest<HTMLElement>("[data-virtual-index]");
    if (!row) return;
    const controls = Array.from(row.querySelectorAll<HTMLButtonElement>("button:not(:disabled)"));
    const action = controls.findIndex(control => control === target);
    if (action < 0) return;
    const index = Number(row.dataset.virtualIndex);
    const nextAction = action + (event.shiftKey ? -1 : 1);
    if (nextAction >= 0 && nextAction < controls.length) return;
    const nextIndex = index + (event.shiftKey ? -1 : 1);
    if (nextIndex < 0 || nextIndex >= toValue(items).length) return;
    event.preventDefault();
    await focus(nextIndex, event.shiftKey ? -1 : 0);
  }
  return { enabled, entries, totalHeight, width, position, leadingPosition, focus, onTab, measure };
}
