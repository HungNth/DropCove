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
    class="settings flex flex-col w-full h-full box-border text-dc-text-primary bg-dc-surface font-['Segoe_UI',sans-serif]"
    :aria-busy="busy"
    aria-label="Settings"
  >
    <header class="header flex items-center justify-between px-4 py-3 border-b border-dc-border-subtle bg-dc-surface forced-colors:border-[ButtonText] forced-colors:bg-[Canvas]">
      <h1 class="title m-0 text-[14px] font-semibold text-dc-text-primary">Settings</h1>
    </header>

    <div class="content flex flex-col flex-1 min-h-0 overflow-y-auto p-3.5 gap-2.5">
      <section class="section flex flex-col gap-2 p-3.5 rounded-lg border border-dc-border-subtle bg-dc-surface-subtle/50" aria-labelledby="hotkey-section-title">
        <div class="flex items-start justify-between gap-3">
          <div class="flex flex-col gap-0.5">
            <h2 id="hotkey-section-title" class="section-title m-0 text-[13px] font-medium text-dc-text-primary">Global shortcut</h2>
            <p class="description m-0 text-[11.5px] text-dc-text-muted leading-relaxed">
              Toggle Drop Shelf visibility from anywhere
            </p>
          </div>
          <button
            type="button"
            ref="recorder"
            class="recorder-button inline-flex items-center justify-center shrink-0 min-w-[130px] py-1.5 px-3 border rounded-md text-[12px] font-mono cursor-pointer transition-[border-color,background-color,box-shadow] duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-dc-focus forced-colors:focus-visible:outline-[2px]! forced-colors:focus-visible:outline-[Highlight]! disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText]"
            :class="snapshot?.listening ? 'listening border-dc-accent bg-dc-accent-subtle text-dc-accent shadow-[0_0_0_1px_var(--dc-accent)] forced-colors:border-[Highlight] forced-colors:bg-[ButtonFace] forced-colors:text-[Highlight]' : 'border-dc-border-default bg-dc-surface-elevated hover:bg-dc-surface-hover hover:border-dc-border-hover text-dc-text-primary shadow-xs'"
            :disabled="busy && !snapshot?.listening"
            aria-labelledby="hotkey-section-title"
            :aria-pressed="Boolean(snapshot?.listening)"
            @click="beginRecording"
            @keydown="onRecorderKeydown"
            @blur="onRecorderBlur"
          >
            <span v-if="snapshot?.listening" class="recorder-prompt inline-flex items-center gap-1.5 animate-[dc-recorder-pulse_1.5s_infinite] motion-reduce:animate-none font-medium text-dc-accent">
              <span class="size-1.5 rounded-full bg-dc-accent"></span>
              Press shortcut...
            </span>
            <span v-else class="recorder-value font-medium tracking-tight">{{ snapshot?.label || "None" }}</span>
          </button>
        </div>
        <p class="field-help m-0 text-[11px] text-dc-text-muted leading-normal">
          <span v-if="snapshot?.listening" class="text-dc-accent">
            Press key combination with modifier (Ctrl, Alt, Shift, or Win). Bare Tab or Esc cancels.
          </span>
          <span v-else>
            Click or press Enter / Space to record a new shortcut.
          </span>
        </p>
      </section>

      <section class="section flex items-center justify-between p-3.5 rounded-lg border border-dc-border-subtle bg-dc-surface-subtle/50" aria-labelledby="startup-section-title">
        <div class="flex flex-col gap-0.5">
          <h2 id="startup-section-title" class="section-title m-0 text-[13px] font-medium text-dc-text-primary">System startup</h2>
          <p class="description m-0 text-[11.5px] text-dc-text-muted leading-relaxed">
            Automatically launch DropCove in the background when signing in
          </p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer select-none shrink-0 has-[:disabled]:cursor-default has-[:disabled]:opacity-[var(--dc-disabled-opacity)]">
          <input
            type="checkbox"
            class="sr-only peer"
            aria-labelledby="startup-section-title"
            :checked="Boolean(snapshot?.startWithWindows)"
            :disabled="busy || !snapshot"
            @change="onStartupChange"
          />
          <div class="w-8 h-[18px] bg-dc-surface-elevated border border-dc-border-default rounded-full peer-focus-visible:ring-2 peer-focus-visible:ring-dc-focus forced-colors:peer-focus-visible:outline-[2px]! forced-colors:peer-focus-visible:outline-[Highlight]! forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:peer-checked:bg-[Highlight] peer-checked:bg-dc-accent peer-checked:border-dc-accent transition-colors after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-dc-text-muted forced-colors:after:bg-[ButtonText] peer-checked:after:bg-white forced-colors:peer-checked:after:bg-[HighlightText] after:rounded-full after:size-[14px] after:transition-transform peer-checked:after:translate-x-[14px] peer-hover:border-dc-border-hover"></div>
        </label>
      </section>

      <RailPlacementFields v-if="snapshot" :edge="snapshot.railEdge" :monitor-id="snapshot.railMonitorId"
        :show-over-fullscreen="Boolean(snapshot.showRailOverFullscreen)"
        :monitors="monitors" :disabled="busy" @change="onPlacementChange" @fullscreen-change="onFullscreenChange" />
      <ShakeSettingsFields v-if="snapshot" :enabled="snapshot.shakeEnabled"
        :sensitivity="snapshot.shakeSensitivity" :disabled="busy" @change="onShakeChange" />
    </div>

    <div v-if="status" class="status mx-3.5 mb-2.5 p-2.5 border border-dc-warning-border rounded-lg bg-dc-warning-surface text-dc-warning-text text-[11.5px] flex items-start gap-2 shadow-xs [overflow-wrap:anywhere]" role="alert" aria-live="assertive">
      <svg class="size-4 shrink-0 mt-0.5 text-dc-warning" viewBox="0 0 16 16" fill="currentColor">
        <path fill-rule="evenodd" d="M6.701 2.25c.577-1 2.02-1 2.598 0l5.196 9a1.5 1.5 0 0 1-1.299 2.25H2.794A1.5 1.5 0 0 1 1.495 11.25l5.206-9ZM8 4c.414 0 .75.336.75.75v3.5a.75.75 0 0 1-1.5 0v-3.5c0-.414.336-.75.75-.75Zm0 7a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Z" clip-rule="evenodd"/>
      </svg>
      <span class="flex-1 leading-snug">{{ status }}</span>
      <button class="dismiss shrink-0 p-0.5 text-dc-warning-text hover:text-white rounded focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-dc-focus forced-colors:focus-visible:outline-[2px]! forced-colors:focus-visible:outline-[Highlight]! transition-colors cursor-pointer" type="button" aria-label="Dismiss status" @click="dismissStatus">
        <svg class="size-3.5" viewBox="0 0 16 16" fill="currentColor"><path d="M5.28 4.22a.75.75 0 0 0-1.06 1.06L6.94 8l-2.72 2.72a.75.75 0 1 0 1.06 1.06L8 9.06l2.72 2.72a.75.75 0 1 0 1.06-1.06L9.06 8l2.72-2.72a.75.75 0 0 0-1.06-1.06L8 6.94 5.28 4.22Z"/></svg>
      </button>
    </div>

    <footer class="footer flex items-center justify-end gap-2.5 py-2.5 px-4 border-t border-dc-border-subtle bg-dc-surface-subtle/80 forced-colors:border-[ButtonText] forced-colors:bg-[Canvas]">
      <button
        type="button"
        class="action-button cancel py-1.5 px-3.5 rounded-md [font-family:inherit] text-[12px] font-medium cursor-pointer transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-dc-focus forced-colors:focus-visible:outline-[2px]! forced-colors:focus-visible:outline-[Highlight]! disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default border border-dc-border-default bg-transparent text-dc-text-secondary hover:bg-dc-surface-elevated hover:text-dc-text-primary forced-colors:border-[ButtonText] forced-colors:bg-[ButtonFace] forced-colors:text-[ButtonText]"
        :disabled="busy"
        aria-label="Cancel and discard changes"
        @click="cancel"
      >
        Cancel
      </button>
      <button
        type="button"
        class="action-button save py-1.5 px-4 rounded-md [font-family:inherit] text-[12px] font-medium cursor-pointer transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-dc-focus forced-colors:focus-visible:outline-[2px]! forced-colors:focus-visible:outline-[Highlight]! disabled:opacity-[var(--dc-disabled-opacity)] disabled:cursor-default border border-dc-accent bg-dc-accent text-dc-accent-text hover:bg-dc-accent-hover shadow-xs forced-colors:border-[Highlight] forced-colors:bg-[Highlight] forced-colors:text-[HighlightText]"
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
