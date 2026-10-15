<script setup lang="ts">
import type { ShakeSensitivity } from "../../composables/useSettings";
const props = defineProps<{ enabled: boolean; sensitivity: ShakeSensitivity; disabled: boolean }>();
const emit = defineEmits<{ change: [enabled: boolean, sensitivity: ShakeSensitivity, event: Event] }>();
function onEnabled(event: Event) {
  const input = event.currentTarget;
  if (input instanceof HTMLInputElement) emit("change", input.checked, props.sensitivity, event);
}
function onSensitivity(event: Event) {
  const input = event.currentTarget;
  if (input instanceof HTMLSelectElement && ["Low", "Normal", "High"].includes(input.value)) {
    emit("change", props.enabled, input.value as ShakeSensitivity, event);
  }
}
</script>

<template>
  <section class="flex flex-col gap-[8px]" aria-labelledby="shake-section-title">
    <h2 id="shake-section-title" class="m-0 text-[12px] font-semibold text-dc-text-secondary uppercase tracking-[0.05em]">Shake-to-open</h2>
    <label class="flex items-center gap-[8px] text-[12px] text-dc-text-secondary select-none">
      <input
        name="shake-enabled"
        type="checkbox"
        class="accent-dc-accent"
        :checked="enabled"
        :disabled="disabled"
        @change="onEnabled"
      />
      <span>Enable shake gesture</span>
    </label>
    <label class="flex flex-col gap-[6px] text-[12px] text-dc-text-secondary">
      <span>Shake sensitivity</span>
      <select
        name="shake-sensitivity"
        class="p-[6px] border border-dc-border-default rounded-[5px] bg-dc-surface-elevated text-dc-text-primary [font-family:inherit] focus-visible:outline-2 focus-visible:outline-dc-focus focus-visible:outline-offset-[1px] disabled:opacity-[var(--dc-disabled-opacity)] forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:focus-visible:outline-[2px] forced-colors:focus-visible:outline-[Highlight]!"
        :value="sensitivity"
        :disabled="disabled || !enabled"
        @change="onSensitivity"
      >
        <option value="Low">Low</option>
        <option value="Normal">Normal</option>
        <option value="High">High</option>
      </select>
    </label>
    <p class="m-0 text-[11px] text-dc-text-muted">Move the pointer back and forth while holding the left mouse button.</p>
  </section>
</template>
