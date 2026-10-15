<script setup lang="ts">
import { computed } from "vue";
import type { ShelfBatch } from "../../composables/useShelf";
import ActionIcon from "./ActionIcon.vue";

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

const ariaChecked = computed<"true" | "false" | "mixed">(() => {
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
    class="icon-action bulk-pin"
    type="button"
    :disabled="busy"
    role="checkbox"
    :aria-checked="ariaChecked"
    :aria-label="label"
    :title="title"
    @click="toggle"
  ><ActionIcon name="pin" :filled="isAllPinned" :mixed="isMixed" /></button>
</template>

