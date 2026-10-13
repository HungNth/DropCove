<script setup lang="ts">
import { computed } from "vue";
import type { ShelfBatch } from "../../composables/useShelf";

const props = defineProps<{
  batch: ShelfBatch;
  busy?: boolean;
}>();

const emit = defineEmits<{
  pin: [batchId: string, pinned: boolean];
}>();

const pinnedCount = computed(() => props.batch.items.filter((item) => item.isPinned).length);
const totalCount = computed(() => props.batch.items.length);

const isAllPinned = computed(() => totalCount.value > 0 && pinnedCount.value === totalCount.value);
const isNonePinned = computed(() => pinnedCount.value === 0);
const isMixed = computed(() => !isAllPinned.value && !isNonePinned.value);

const ariaPressed = computed<"true" | "false" | "mixed">(() => {
  if (isAllPinned.value) return "true";
  if (isMixed.value) return "mixed";
  return "false";
});

const label = computed(() => {
  if (isAllPinned.value) return "Unpin all items";
  return "Pin all items";
});

const title = computed(() => {
  if (isAllPinned.value) return "Make all items temporary; consumed after successful Copy";
  return "Keep all references after successful Copy";
});

function toggle() {
  if (props.busy) return;
  // Off or Indeterminate activates Pin all (true). On activates make all Temporary (false).
  emit("pin", props.batch.id, !isAllPinned.value);
}
</script>

<template>
  <button
    class="bulk-pin"
    type="button"
    :disabled="busy"
    :aria-pressed="ariaPressed"
    :aria-label="label"
    :title="title"
    @click="toggle"
  >{{ isAllPinned ? "Unpin All" : "Pin All" }}</button>
</template>

<style scoped>
.bulk-pin {
  border: 1px solid #46536a;
  border-radius: 4px;
  padding: 2px 6px;
  color: inherit;
  background: transparent;
  font: inherit;
  font-size: 11px;
  cursor: pointer;
}
.bulk-pin[aria-pressed="true"] {
  border-color: #a5b4fc;
  color: #a5b4fc;
}
.bulk-pin[aria-pressed="mixed"] {
  border-color: #efad70;
  color: #efad70;
}
.bulk-pin:focus-visible {
  outline: 2px solid #a5b4fc;
}
.bulk-pin:disabled {
  opacity: .45;
  cursor: default;
}
</style>
