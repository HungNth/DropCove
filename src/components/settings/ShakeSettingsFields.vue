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
  <section class="section" aria-labelledby="shake-section-title">
    <h2 id="shake-section-title" class="title">Shake-to-open</h2>
    <label class="checkbox-field">
      <input name="shake-enabled" type="checkbox" :checked="enabled" :disabled="disabled" @change="onEnabled" />
      <span>Enable shake gesture</span>
    </label>
    <label class="field">
      <span>Shake sensitivity</span>
      <select name="shake-sensitivity" :value="sensitivity" :disabled="disabled || !enabled" @change="onSensitivity">
        <option value="Low">Low</option>
        <option value="Normal">Normal</option>
        <option value="High">High</option>
      </select>
    </label>
    <p class="help">Move the pointer back and forth while holding the left mouse button.</p>
  </section>
</template>

<style scoped>
.section { display: flex; flex-direction: column; gap: 8px; }
.title { margin: 0; font-size: 12px; font-weight: 600; color: #cbd5e1; text-transform: uppercase; letter-spacing: .05em; }
.checkbox-field { display: flex; align-items: center; gap: 8px; font-size: 12px; color: #cbd5e1; }
.checkbox-field input { accent-color: #6366f1; }
.field { display: flex; flex-direction: column; gap: 6px; font-size: 12px; color: #cbd5e1; }
.field select { padding: 6px; border: 1px solid #46536a; border-radius: 5px; background: #1b2438; color: #edf2fc; font: inherit; }
.field select:focus-visible { outline: 2px solid #a5b4fc; outline-offset: 1px; }
.field select:disabled { opacity: .45; }
.help { margin: 0; font-size: 11px; color: #94a3b8; }
</style>
