<script setup lang="ts">
import type { ResizeEdge } from "../../composables/useShelf";

const props = defineProps<{ disabled: boolean }>();
const emit = defineEmits<{ resize: [edge: ResizeEdge] }>();

function begin(event: PointerEvent, edge: ResizeEdge) {
  if (event.button !== 0 || props.disabled) return;
  emit("resize", edge);
}
</script>

<template>
  <div class="resize-handles absolute inset-0 z-[2] pointer-events-none has-[.resize-handle:hover]:rounded-[11px] has-[.resize-handle:hover]:[box-shadow:inset_0_0_0_2px_var(--dc-focus)] forced-colors:has-[.resize-handle:hover]:[box-shadow:inset_0_0_0_2px_Highlight]" aria-hidden="true">
    <div class="resize-handle absolute pointer-events-auto touch-none top-0 left-[8px] right-[8px] h-[8px] cursor-ns-resize" data-resize-edge="North" @pointerdown.prevent="begin($event, 'North')" />
    <div class="resize-handle absolute pointer-events-auto touch-none top-[8px] bottom-[8px] right-0 w-[8px] cursor-ew-resize" data-resize-edge="East" @pointerdown.prevent="begin($event, 'East')" />
    <div class="resize-handle absolute pointer-events-auto touch-none bottom-0 left-[8px] right-[8px] h-[8px] cursor-ns-resize" data-resize-edge="South" @pointerdown.prevent="begin($event, 'South')" />
    <div class="resize-handle absolute pointer-events-auto touch-none top-[8px] bottom-[8px] left-0 w-[8px] cursor-ew-resize" data-resize-edge="West" @pointerdown.prevent="begin($event, 'West')" />
    <div class="resize-handle absolute pointer-events-auto touch-none top-0 left-0 size-[8px] cursor-nwse-resize" data-resize-edge="NorthWest" @pointerdown.prevent="begin($event, 'NorthWest')" />
    <div class="resize-handle absolute pointer-events-auto touch-none top-0 right-0 size-[8px] cursor-nesw-resize" data-resize-edge="NorthEast" @pointerdown.prevent="begin($event, 'NorthEast')" />
    <div class="resize-handle absolute pointer-events-auto touch-none bottom-0 left-0 size-[8px] cursor-nesw-resize" data-resize-edge="SouthWest" @pointerdown.prevent="begin($event, 'SouthWest')" />
    <div class="resize-handle absolute pointer-events-auto touch-none bottom-0 right-0 size-[8px] cursor-nwse-resize" data-resize-edge="SouthEast" @pointerdown.prevent="begin($event, 'SouthEast')" />
  </div>
</template>

