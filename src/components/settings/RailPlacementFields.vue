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
  <section class="section" aria-labelledby="rail-section-title">
    <h2 id="rail-section-title" class="title">Edge Rail</h2>
    <label class="field">
      <span>Docking edge</span>
      <select name="rail-edge" :value="edge" :disabled="disabled" @change="onEdge">
        <option value="Left">Left</option>
        <option value="Right">Right</option>
      </select>
    </label>
    <label class="field">
      <span>Target monitor</span>
      <select name="rail-monitor" :value="monitorId" :disabled="disabled" @change="onMonitor">
        <option value="">Use remembered monitor</option>
        <option v-for="monitor in monitors" :key="monitor.id" :value="monitor.id">
          {{ monitor.label }}{{ monitor.connected ? '' : ' (disconnected)' }}
        </option>
      </select>
    </label>
    <label class="checkbox-field">
      <input
        type="checkbox"
        name="rail-fullscreen"
        class="checkbox-input"
        :checked="showOverFullscreen"
        :disabled="disabled"
        @change="onFullscreen"
      />
      <span>Show over fullscreen applications</span>
    </label>
  </section>
</template>

<style scoped>
.section { display: flex; flex-direction: column; gap: 8px; }
.title { margin: 0; font-size: 12px; font-weight: 600; color: #cbd5e1; text-transform: uppercase; letter-spacing: .05em; }
.field { display: flex; flex-direction: column; gap: 6px; font-size: 12px; color: #cbd5e1; }
.field select { width: 100%; min-width: 0; padding: 6px; border: 1px solid #46536a; border-radius: 5px; background: #1b2438; color: #edf2fc; font: inherit; }
.field select:focus-visible { outline: 2px solid #a5b4fc; outline-offset: 1px; }
.field select:disabled { opacity: .45; }
.checkbox-field { display: flex; align-items: center; gap: 8px; font-size: 12px; color: #cbd5e1; margin-top: 4px; cursor: pointer; }
.checkbox-input { width: 14px; height: 14px; accent-color: #6366f1; }
</style>
