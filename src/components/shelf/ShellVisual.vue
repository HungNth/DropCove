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
  <span class="shell-visual" :data-item-id="itemId">
    <img
      v-if="visualUrl && !failed"
      :src="visualUrl"
      alt=""
      aria-hidden="true"
      @error="failed = true"
      class="shell-visual-img"
      draggable="false"
    />
    <span v-else class="shell-visual-fallback" aria-hidden="true">
      {{ isFolder ? "▰" : "▤" }}
    </span>
  </span>
</template>

<style scoped>
.shell-visual {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: v-bind("`${size}px`");
  height: v-bind("`${size}px`");
  min-width: v-bind("`${size}px`");
  min-height: v-bind("`${size}px`");
  overflow: hidden;
  line-height: 1;
}

.shell-visual-img {
  width: 100%;
  height: 100%;
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  display: block;
  user-select: none;
}

.shell-visual-fallback {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  line-height: 1;
  color: var(--dc-accent);
}
</style>
