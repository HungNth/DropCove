<script setup lang="ts">
import { toRef, shallowRef, watch } from "vue";
import { useShellVisual, type SurfaceId } from "../../composables/useShellVisual";

const props = withDefaults(
  defineProps<{
    itemId: string;
    isFolder: boolean;
    name: string;
    size?: number;
    surface?: SurfaceId;
  }>(),
  {
    size: 32,
  }
);

const { visualUrl } = useShellVisual(
  toRef(props, "itemId"),
  toRef(props, "isFolder"),
  toRef(props, "size"),
  props.surface
);
const failed = shallowRef(false);
watch(visualUrl, () => { failed.value = false; });
</script>

<template>
  <span
    class="shell-visual inline-flex items-center justify-center overflow-hidden leading-none"
    :data-item-id="itemId"
    :style="{
      width: `${size}px`,
      height: `${size}px`,
      minWidth: `${size}px`,
      minHeight: `${size}px`,
    }"
  >
    <img
      v-if="visualUrl && !failed"
      :src="visualUrl"
      alt=""
      aria-hidden="true"
      @error="failed = true"
      class="shell-visual-img block size-full max-w-full max-h-full object-contain select-none"
      draggable="false"
    />
    <span v-else class="shell-visual-fallback inline-flex items-center justify-center text-[18px] leading-none text-dc-accent" aria-hidden="true">
      {{ isFolder ? "▰" : "▤" }}
    </span>
  </span>
</template>

