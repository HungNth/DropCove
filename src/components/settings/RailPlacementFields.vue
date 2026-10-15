<script setup lang="ts">
import type { MonitorChoice, RailEdge } from "../../composables/useSettings";
const props = defineProps<{
  edge: RailEdge;
  monitorId: string;
  showOverFullscreen: boolean;
  monitors: MonitorChoice[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  change: [edge: RailEdge, monitorId: string, event: Event];
  fullscreenChange: [enabled: boolean, event: Event];
}>();
function onEdge(event: Event) {
  const input = event.currentTarget;
  if (input instanceof HTMLSelectElement && (input.value === "Left" || input.value === "Right")) {
    emit("change", input.value, props.monitorId, event);
  }
}
function onMonitor(event: Event) {
  const input = event.currentTarget;
  if (input instanceof HTMLSelectElement) emit("change", props.edge, input.value, event);
}
function onFullscreen(event: Event) {
  const input = event.currentTarget;
  if (input instanceof HTMLInputElement) emit("fullscreenChange", input.checked, event);
}
</script>

<template>
  <section class="flex flex-col gap-[8px]" aria-labelledby="rail-section-title">
    <h2 id="rail-section-title" class="m-0 text-[12px] font-semibold text-dc-text-secondary uppercase tracking-[0.05em]">Edge Rail</h2>
    <label class="flex flex-col gap-[6px] text-[12px] text-dc-text-secondary">
      <span>Docking edge</span>
      <select
        name="rail-edge"
        class="w-full min-w-0 p-[6px] border border-dc-border-default rounded-[5px] bg-dc-surface-elevated text-dc-text-primary [font-family:inherit] focus-visible:outline-2 focus-visible:outline-dc-focus focus-visible:outline-offset-[1px] disabled:opacity-[var(--dc-disabled-opacity)] forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:focus-visible:outline-[2px] forced-colors:focus-visible:outline-[Highlight]!"
        :value="edge"
        :disabled="disabled"
        @change="onEdge"
      >
        <option value="Left">Left</option>
        <option value="Right">Right</option>
      </select>
    </label>
    <label class="flex flex-col gap-[6px] text-[12px] text-dc-text-secondary">
      <span>Target monitor</span>
      <select
        name="rail-monitor"
        class="w-full min-w-0 p-[6px] border border-dc-border-default rounded-[5px] bg-dc-surface-elevated text-dc-text-primary [font-family:inherit] focus-visible:outline-2 focus-visible:outline-dc-focus focus-visible:outline-offset-[1px] disabled:opacity-[var(--dc-disabled-opacity)] forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:focus-visible:outline-[2px] forced-colors:focus-visible:outline-[Highlight]!"
        :value="monitorId"
        :disabled="disabled"
        @change="onMonitor"
      >
        <option value="">Use remembered monitor</option>
        <option v-for="monitor in monitors" :key="monitor.id" :value="monitor.id">
          {{ monitor.label }}{{ monitor.connected ? '' : ' (disconnected)' }}
        </option>
      </select>
    </label>
    <label class="flex items-center gap-[8px] text-[12px] text-dc-text-secondary mt-[4px] cursor-pointer">
      <input
        type="checkbox"
        name="rail-fullscreen"
        class="w-[14px] h-[14px] accent-dc-accent"
        :checked="showOverFullscreen"
        :disabled="disabled"
        @change="onFullscreen"
      />
      <span>Show over fullscreen applications</span>
    </label>
  </section>
</template>
