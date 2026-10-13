<script setup lang="ts">
import type { ResizeEdge } from "../../composables/useShelf";

const props = defineProps<{ disabled: boolean }>();
const emit = defineEmits<{ resize: [edge: ResizeEdge] }>();
const edges: ResizeEdge[] = ["North", "East", "South", "West", "NorthWest", "NorthEast", "SouthWest", "SouthEast"];

function begin(event: PointerEvent, edge: ResizeEdge) {
  if (event.button !== 0 || props.disabled) return;
  emit("resize", edge);
}
</script>

<template>
  <div class="resize-handles" aria-hidden="true">
    <div
      v-for="edge in edges"
      :key="edge"
      class="resize-handle"
      :class="edge"
      :data-resize-edge="edge"
      @pointerdown.prevent="begin($event, edge)"
    />
  </div>
</template>

<style scoped>
.resize-handles { position: absolute; inset: 0; z-index: 2; pointer-events: none; }
.resize-handle { position: absolute; pointer-events: auto; touch-action: none; }
.resize-handles:has(.resize-handle:hover) { border-radius: 11px; box-shadow: inset 0 0 0 2px #a5b4fc; }
.North, .South { left: 8px; right: 8px; height: 8px; cursor: ns-resize; }
.North { top: 0; }
.South { bottom: 0; }
.East, .West { top: 8px; bottom: 8px; width: 8px; cursor: ew-resize; }
.East { right: 0; }
.West { left: 0; }
.NorthWest, .NorthEast, .SouthWest, .SouthEast { width: 8px; height: 8px; }
.NorthWest { top: 0; left: 0; cursor: nwse-resize; }
.NorthEast { top: 0; right: 0; cursor: nesw-resize; }
.SouthWest { bottom: 0; left: 0; cursor: nesw-resize; }
.SouthEast { bottom: 0; right: 0; cursor: nwse-resize; }
</style>
