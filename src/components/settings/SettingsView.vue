<script setup lang="ts">
import { nextTick, onMounted, useTemplateRef } from "vue";
import { useSettings, type RailEdge, type ShakeSensitivity } from "../../composables/useSettings";
import RailPlacementFields from "./RailPlacementFields.vue";
import ShakeSettingsFields from "./ShakeSettingsFields.vue";

const {
  snapshot,
  status,
  busy,
  startRecording,
  cancelRecording,
  recordKey,
  setStartWithWindows,
  setRailPlacement,
  setShowRailOverFullscreen,
  setShake,
  monitors,
  save,
  cancel,
  dismissStatus,
} = useSettings();
onMounted(() => { document.title = "DropCove Settings"; });

const recorder = useTemplateRef<HTMLButtonElement>("recorder");
async function beginRecording() {
  await startRecording();
  await nextTick();
  if (snapshot.value?.listening) recorder.value?.focus();
}

function onRecorderKeydown(event: KeyboardEvent) {
  if (!snapshot.value?.listening) return;
  const modified = event.ctrlKey || event.altKey || event.shiftKey || event.metaKey;
  if (event.key === "Escape" || (event.key === "Tab" && !modified)) {
    if (event.key === "Escape") event.preventDefault();
    void cancelRecording();
    return;
  }
  if (["Control", "Alt", "AltGraph", "Shift", "Meta"].includes(event.key)) return;
  event.preventDefault();
  if (event.repeat || event.isComposing) return;
  // Windows WebView supplies numeric VK codes; Rust validates them before changing the draft.
  void recordKey({ Control: event.ctrlKey, Alt: event.altKey, Shift: event.shiftKey,
    Windows: event.metaKey, VirtualKey: event.keyCode });
}

function onRecorderBlur() {
  if (snapshot.value?.listening) void cancelRecording();
}

async function onStartupChange(event: Event) {
  const input = event.currentTarget;
  if (!(input instanceof HTMLInputElement)) return;
  await setStartWithWindows(input.checked);
  // Restore the authoritative projection even when the draft operation was rejected.
  input.checked = Boolean(snapshot.value?.startWithWindows);
}

async function onPlacementChange(edge: RailEdge, monitorId: string, event: Event) {
  const input = event.currentTarget;
  await setRailPlacement(edge, monitorId);
  if (input instanceof HTMLSelectElement) {
    input.value = input.name === "rail-edge" ? snapshot.value?.railEdge ?? "Right" : snapshot.value?.railMonitorId ?? "";
  }
}

async function onFullscreenChange(enabled: boolean, event: Event) {
  const input = event.currentTarget;
  await setShowRailOverFullscreen(enabled);
  if (input instanceof HTMLInputElement) {
    input.checked = Boolean(snapshot.value?.showRailOverFullscreen);
  }
}

async function onShakeChange(enabled: boolean, sensitivity: ShakeSensitivity, event: Event) {
  const input = event.currentTarget;
  await setShake(enabled, sensitivity);
  if (input instanceof HTMLInputElement) input.checked = Boolean(snapshot.value?.shakeEnabled);
  if (input instanceof HTMLSelectElement) input.value = snapshot.value?.shakeSensitivity ?? "Normal";
}
</script>

<template>
  <main
    class="settings flex flex-col h-full box-border border border-dc-border-default rounded-[9px] text-dc-text-primary bg-dc-surface font-['Segoe_UI',sans-serif] forced-colors:border-[ButtonText]"
    :aria-busy="busy"
    aria-label="Settings"
  >
    <header class="header flex items-center justify-between pt-[12px] px-[14px] pb-[10px] border-b border-b-dc-border-subtle forced-colors:border-[ButtonText] forced-colors:bg-[Canvas]">
      <h1 class="title m-0 text-[14px] font-semibold text-dc-text-primary">Settings</h1>
    </header>

    <div class="content flex flex-col flex-1 min-h-0 overflow-y-auto p-[14px] gap-[16px]">
      <section class="section flex flex-col gap-[8px]" aria-labelledby="hotkey-section-title">
        <h2 id="hotkey-section-title" class="section-title m-0 text-[12px] font-semibold text-dc-text-secondary uppercase tracking-[0.05em]">Global Hotkey</h2>
        <p class="description m-0 text-[12px] text-dc-text-muted">
          Toggle Drop Shelf visibility from anywhere.
        </p>

        <div class="field flex flex-col gap-[6px] mt-[4px]">
          <label id="hotkey-label" class="field-label text-[12px] text-dc-text-secondary">Shortcut</label>
          <div class="recorder-row flex items-center">
            <button
              type="button"
              ref="recorder"
              class="recorder-button inline-flex items-center justify-center min-w-[140px] py-[6px] px-[12px] border rounded-[5px] text-[12px] font-medium [font-family:inherit] cursor-pointer transition-[border-color,background-color] duration-[150ms,150ms] ease-[ease] [&:hover:not(:disabled)]:border-dc-accent [&:hover:not(:disabled)]:bg-dc-surface-hover focus-visible:outline-2 focus-visible:outline-dc-focus focus-visible:outline-offset-[1px] disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:focus-visible:outline-[2px] forced-colors:focus-visible:outline-[Highlight]!"
              :class="snapshot?.listening ? 'listening border-dc-accent bg-dc-accent-subtle text-dc-accent [box-shadow:0_0_0_1px_var(--dc-accent)] forced-colors:border-[Highlight] forced-colors:bg-[ButtonFace] forced-colors:text-[Highlight]' : 'border-dc-border-default bg-dc-surface-elevated text-dc-text-primary'"
              :disabled="busy && !snapshot?.listening"
              aria-labelledby="hotkey-label"
              :aria-pressed="Boolean(snapshot?.listening)"
              @click="beginRecording"
              @keydown="onRecorderKeydown"
              @blur="onRecorderBlur"
            >
              <span v-if="snapshot?.listening" class="recorder-prompt animate-[dc-recorder-pulse_1.5s_infinite] motion-reduce:animate-none">Press shortcut...</span>
              <span v-else class="recorder-value">{{ snapshot?.label || "None" }}</span>
            </button>
          </div>
          <p class="field-help m-0 text-[11px] text-dc-text-muted leading-[1.4]">
            <span v-if="snapshot?.listening">
              Press key combination with modifier (Ctrl, Alt, Shift, or Win). Bare Tab or Esc cancels.
            </span>
            <span v-else>
              Click or press Enter / Space to record a new shortcut.
            </span>
          </p>
        </div>
      </section>
      <section class="section flex flex-col gap-[8px]" aria-labelledby="startup-section-title">
        <h2 id="startup-section-title" class="section-title m-0 text-[12px] font-semibold text-dc-text-secondary uppercase tracking-[0.05em]">System Startup</h2>
        <label class="checkbox-row inline-flex items-center gap-[8px] cursor-pointer select-none w-fit has-[:disabled]:cursor-default has-[:disabled]:opacity-[var(--dc-disabled-opacity)]">
          <input
            type="checkbox"
            class="checkbox-input m-0 w-[15px] h-[15px] accent-dc-accent cursor-pointer disabled:cursor-default disabled:opacity-[var(--dc-disabled-opacity)]"
            :checked="Boolean(snapshot?.startWithWindows)"
            :disabled="busy || !snapshot"
            @change="onStartupChange"
          />
          <span class="checkbox-label text-[13px] font-medium text-dc-text-primary">Start with Windows</span>
        </label>
        <p class="description m-0 text-[12px] text-dc-text-muted">
          Automatically launch DropCove in the background when signing in.
        </p>
      </section>
      <RailPlacementFields v-if="snapshot" :edge="snapshot.railEdge" :monitor-id="snapshot.railMonitorId"
        :show-over-fullscreen="Boolean(snapshot.showRailOverFullscreen)"
        :monitors="monitors" :disabled="busy" @change="onPlacementChange" @fullscreen-change="onFullscreenChange" />
      <ShakeSettingsFields v-if="snapshot" :enabled="snapshot.shakeEnabled"
        :sensitivity="snapshot.shakeSensitivity" :disabled="busy" @change="onShakeChange" />
    </div>

    <div v-if="status" class="status flex items-start gap-[6px] mx-[14px] mb-[10px] p-[8px_10px] border border-dc-warning-border rounded-[6px] bg-dc-warning-surface text-dc-warning-text text-[11px] [overflow-wrap:anywhere]" role="alert" aria-live="assertive">
      <span>{{ status }}</span>
      <button class="dismiss ml-auto border-0 border-none text-inherit bg-transparent text-[14px] leading-none cursor-pointer py-0 px-[2px] focus-visible:outline-2 focus-visible:outline-dc-focus forced-colors:focus-visible:outline-[2px] forced-colors:focus-visible:outline-[Highlight]!" type="button" aria-label="Dismiss status" @click="dismissStatus">×</button>
    </div>

    <footer class="footer flex items-center justify-end gap-[8px] py-[10px] px-[14px] border-t border-t-dc-border-subtle bg-dc-surface-subtle rounded-b-[8px] forced-colors:border-[ButtonText] forced-colors:bg-[Canvas]">
      <button
        type="button"
        class="action-button cancel py-[5px] px-[12px] rounded-[4px] [font-family:inherit] text-[12px] font-medium cursor-pointer transition-[background-color,border-color,opacity] duration-[150ms,150ms,150ms] ease-[ease] focus-visible:outline-2 focus-visible:outline-dc-focus focus-visible:outline-offset-[1px] disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default border border-dc-border-default bg-transparent text-dc-text-secondary [&:hover:not(:disabled)]:bg-dc-surface-elevated [&:hover:not(:disabled)]:border-dc-border-hover forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText] forced-colors:focus-visible:outline-[2px] forced-colors:focus-visible:outline-[Highlight]!"
        :disabled="busy"
        aria-label="Cancel and discard changes"
        @click="cancel"
      >
        Cancel
      </button>
      <button
        type="button"
        class="action-button save py-[5px] px-[12px] rounded-[4px] [font-family:inherit] text-[12px] font-medium cursor-pointer transition-[background-color,border-color,opacity] duration-[150ms,150ms,150ms] ease-[ease] focus-visible:outline-2 focus-visible:outline-dc-focus focus-visible:outline-offset-[1px] disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default border border-dc-accent bg-dc-accent text-dc-accent-text [&:hover:not(:disabled)]:bg-dc-accent-hover [&:hover:not(:disabled)]:border-dc-accent-hover forced-colors:border-[Highlight] forced-colors:bg-[Highlight] forced-colors:text-[HighlightText] forced-colors:focus-visible:outline-[2px] forced-colors:focus-visible:outline-[Highlight]!"
        :disabled="busy || Boolean(snapshot?.listening)"
        aria-label="Save settings"
        @click="save"
      >
        Save
      </button>
    </footer>
  </main>
</template>

<style>
@keyframes dc-recorder-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
</style>
