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
  <section class="flex flex-col gap-3 p-3.5 rounded-lg border border-dc-border-subtle bg-dc-surface-subtle/50" aria-labelledby="rail-section-title">
    <div class="flex flex-col gap-0.5">
      <h2 id="rail-section-title" class="m-0 text-[13px] font-medium text-dc-text-primary">Edge Rail</h2>
      <p class="m-0 text-[11.5px] text-dc-text-muted leading-relaxed">Dock to screen edge for immediate drag-and-drop staging</p>
    </div>

    <div class="grid grid-cols-2 gap-2.5 pt-0.5">
      <label class="flex flex-col gap-1.5 text-[12px] text-dc-text-secondary">
        <span class="font-medium text-dc-text-secondary text-[11.5px]">Docking edge</span>
        <div class="relative">
          <select
            name="rail-edge"
            class="w-full appearance-none py-1.5 pl-2.5 pr-7 border border-dc-border-default rounded-md bg-dc-surface-elevated text-dc-text-primary text-[12px] font-medium [font-family:inherit] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-dc-focus forced-colors:focus-visible:outline-[2px]! forced-colors:focus-visible:outline-[Highlight]! disabled:opacity-[var(--dc-disabled-opacity)] cursor-pointer hover:border-dc-border-hover transition-colors forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText]"
            :value="edge"
            :disabled="disabled"
            @change="onEdge"
          >
            <option value="Left">Left</option>
            <option value="Right">Right</option>
          </select>
          <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2 text-dc-text-muted">
            <svg class="size-3.5" viewBox="0 0 16 16" fill="currentColor"><path fill-rule="evenodd" d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z" clip-rule="evenodd"/></svg>
          </div>
        </div>
      </label>

      <label class="flex flex-col gap-1.5 text-[12px] text-dc-text-secondary">
        <span class="font-medium text-dc-text-secondary text-[11.5px]">Target monitor</span>
        <div class="relative">
          <select
            name="rail-monitor"
            class="w-full appearance-none py-1.5 pl-2.5 pr-7 border border-dc-border-default rounded-md bg-dc-surface-elevated text-dc-text-primary text-[12px] font-medium [font-family:inherit] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-dc-focus forced-colors:focus-visible:outline-[2px]! forced-colors:focus-visible:outline-[Highlight]! disabled:opacity-[var(--dc-disabled-opacity)] cursor-pointer hover:border-dc-border-hover transition-colors forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText]"
            :value="monitorId"
            :disabled="disabled"
            @change="onMonitor"
          >
            <option value="">Remembered</option>
            <option v-for="monitor in monitors" :key="monitor.id" :value="monitor.id">
              {{ monitor.label }}{{ monitor.connected ? '' : ' (disconnected)' }}
            </option>
          </select>
          <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2 text-dc-text-muted">
            <svg class="size-3.5" viewBox="0 0 16 16" fill="currentColor"><path fill-rule="evenodd" d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z" clip-rule="evenodd"/></svg>
          </div>
        </div>
      </label>
    </div>

    <div class="flex items-center justify-between pt-2 border-t border-dc-border-subtle/80">
      <div class="flex flex-col gap-0.5">
        <span class="text-[12px] font-medium text-dc-text-primary">Show over fullscreen</span>
        <span class="text-[11px] text-dc-text-muted">Keep Edge Rail visible over fullscreen apps</span>
      </div>
      <label class="relative inline-flex items-center cursor-pointer select-none shrink-0 has-[:disabled]:cursor-default has-[:disabled]:opacity-[var(--dc-disabled-opacity)]">
        <input
          type="checkbox"
          name="rail-fullscreen"
          class="sr-only peer"
          aria-label="Show over fullscreen applications"
          :checked="showOverFullscreen"
          :disabled="disabled"
          @change="onFullscreen"
        />
        <div class="w-8 h-[18px] bg-dc-surface-elevated border border-dc-border-default rounded-full peer-focus-visible:ring-2 peer-focus-visible:ring-dc-focus forced-colors:peer-focus-visible:outline-[2px]! forced-colors:peer-focus-visible:outline-[Highlight]! forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:peer-checked:bg-[Highlight] peer-checked:bg-dc-accent peer-checked:border-dc-accent transition-colors after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-dc-text-muted forced-colors:after:bg-[ButtonText] peer-checked:after:bg-white forced-colors:peer-checked:after:bg-[HighlightText] after:rounded-full after:size-[14px] after:transition-transform peer-checked:after:translate-x-[14px] peer-hover:border-dc-border-hover"></div>
      </label>
    </div>
  </section>
</template>
