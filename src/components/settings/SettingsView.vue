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
    class="settings"
    :aria-busy="busy"
    aria-label="Settings"
  >
    <header class="header">
      <h1 class="title">Settings</h1>
    </header>

    <div class="content">
      <section class="section" aria-labelledby="hotkey-section-title">
        <h2 id="hotkey-section-title" class="section-title">Global Hotkey</h2>
        <p class="description">
          Toggle Drop Shelf visibility from anywhere.
        </p>

        <div class="field">
          <label id="hotkey-label" class="field-label">Shortcut</label>
          <div class="recorder-row">
            <button
              type="button"
              ref="recorder"
              class="recorder-button"
              :class="{ listening: snapshot?.listening }"
              :disabled="busy && !snapshot?.listening"
              aria-labelledby="hotkey-label"
              :aria-pressed="Boolean(snapshot?.listening)"
              @click="beginRecording"
              @keydown="onRecorderKeydown"
              @blur="onRecorderBlur"
            >
              <span v-if="snapshot?.listening" class="recorder-prompt">Press shortcut...</span>
              <span v-else class="recorder-value">{{ snapshot?.label || "None" }}</span>
            </button>
          </div>
          <p class="field-help">
            <span v-if="snapshot?.listening">
              Press key combination with modifier (Ctrl, Alt, Shift, or Win). Bare Tab or Esc cancels.
            </span>
            <span v-else>
              Click or press Enter / Space to record a new shortcut.
            </span>
          </p>
        </div>
      </section>
      <section class="section" aria-labelledby="startup-section-title">
        <h2 id="startup-section-title" class="section-title">System Startup</h2>
        <label class="checkbox-row">
          <input
            type="checkbox"
            class="checkbox-input"
            :checked="Boolean(snapshot?.startWithWindows)"
            :disabled="busy || !snapshot"
            @change="onStartupChange"
          />
          <span class="checkbox-label">Start with Windows</span>
        </label>
        <p class="description">
          Automatically launch DropCove in the background when signing in.
        </p>
      </section>
      <RailPlacementFields v-if="snapshot" :edge="snapshot.railEdge" :monitor-id="snapshot.railMonitorId"
        :show-over-fullscreen="Boolean(snapshot.showRailOverFullscreen)"
        :monitors="monitors" :disabled="busy" @change="onPlacementChange" @fullscreen-change="onFullscreenChange" />
      <ShakeSettingsFields v-if="snapshot" :enabled="snapshot.shakeEnabled"
        :sensitivity="snapshot.shakeSensitivity" :disabled="busy" @change="onShakeChange" />
    </div>

    <div v-if="status" class="status" role="alert" aria-live="assertive">
      <span>{{ status }}</span>
      <button class="dismiss" type="button" aria-label="Dismiss status" @click="dismissStatus">×</button>
    </div>

    <footer class="footer">
      <button
        type="button"
        class="action-button cancel"
        :disabled="busy"
        aria-label="Cancel and discard changes"
        @click="cancel"
      >
        Cancel
      </button>
      <button
        type="button"
        class="action-button save"
        :disabled="busy || Boolean(snapshot?.listening)"
        aria-label="Save settings"
        @click="save"
      >
        Save
      </button>
    </footer>
  </main>
</template>

<style scoped>
.settings {
  display: flex;
  flex-direction: column;
  height: 100%;
  box-sizing: border-box;
  border: 1px solid var(--dc-border-default);
  border-radius: 9px;
  color: var(--dc-text-primary);
  background: var(--dc-surface);
  font-family: "Segoe UI", sans-serif;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px 10px;
  border-bottom: 1px solid var(--dc-border-subtle);
}

.title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--dc-text-primary);
}

.content {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 14px;
  gap: 16px;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  color: var(--dc-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.description {
  margin: 0;
  font-size: 12px;
  color: var(--dc-text-muted);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 4px;
}

.field-label {
  font-size: 12px;
  color: var(--dc-text-secondary);
}

.recorder-row {
  display: flex;
  align-items: center;
}

.recorder-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 140px;
  padding: 6px 12px;
  border: 1px solid var(--dc-border-default);
  border-radius: 5px;
  background: var(--dc-surface-elevated);
  color: var(--dc-text-primary);
  font-family: inherit;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: border-color 0.15s, background-color 0.15s;
}

.recorder-button:hover:not(:disabled) {
  border-color: var(--dc-accent);
  background: var(--dc-surface-hover);
}

.recorder-button.listening {
  border-color: var(--dc-accent);
  background: var(--dc-accent-subtle);
  color: var(--dc-accent);
  box-shadow: 0 0 0 1px var(--dc-accent);
}

.recorder-button:focus-visible {
  outline: 2px solid var(--dc-focus);
  outline-offset: 1px;
}

.recorder-button:disabled {
  opacity: var(--dc-disabled-opacity);
  cursor: default;
}

.recorder-prompt {
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

@media (prefers-reduced-motion: reduce) {
  .recorder-prompt {
    animation: none;
  }
}

.field-help {
  margin: 0;
  font-size: 11px;
  color: var(--dc-text-muted);
  line-height: 1.4;
}

.checkbox-row {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  user-select: none;
  width: fit-content;
}

.checkbox-input {
  margin: 0;
  width: 15px;
  height: 15px;
  accent-color: var(--dc-accent);
  cursor: pointer;
}

.checkbox-input:disabled {
  cursor: default;
  opacity: var(--dc-disabled-opacity);
}

.checkbox-row:has(.checkbox-input:disabled) {
  cursor: default;
  opacity: var(--dc-disabled-opacity);
}

.checkbox-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--dc-text-primary);
}

.status {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  margin: 0 14px 10px;
  padding: 8px 10px;
  border: 1px solid var(--dc-warning-border);
  border-radius: 6px;
  background: var(--dc-warning-surface);
  color: var(--dc-warning-text);
  font-size: 11px;
  overflow-wrap: anywhere;
}

.dismiss {
  margin-left: auto;
  border: 0;
  color: inherit;
  background: transparent;
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  padding: 0 2px;
}

.dismiss:focus-visible {
  outline: 2px solid var(--dc-focus);
}

.footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 10px 14px;
  border-top: 1px solid var(--dc-border-subtle);
  background: var(--dc-surface-subtle);
  border-bottom-left-radius: 8px;
  border-bottom-right-radius: 8px;
}

.action-button {
  padding: 5px 12px;
  border-radius: 4px;
  font-family: inherit;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.15s, border-color 0.15s, opacity 0.15s;
}

.action-button:focus-visible {
  outline: 2px solid var(--dc-focus);
  outline-offset: 1px;
}

.action-button:disabled {
  opacity: var(--dc-disabled-opacity);
  cursor: default;
}

.cancel {
  border: 1px solid var(--dc-border-default);
  background: transparent;
  color: var(--dc-text-secondary);
}

.cancel:hover:not(:disabled) {
  background: var(--dc-surface-elevated);
  border-color: var(--dc-border-hover);
}

.save {
  border: 1px solid var(--dc-accent);
  background: var(--dc-accent);
  color: var(--dc-accent-text);
}

.save:hover:not(:disabled) {
  background: var(--dc-accent-hover);
  border-color: var(--dc-accent-hover);
}

@media (forced-colors: active) {
  .settings {
    border-color: ButtonText;
  }
  .header, .footer {
    border-color: ButtonText;
    background: Canvas;
  }
  .recorder-button, .action-button, .cancel {
    border-color: ButtonText;
    background: ButtonFace;
    color: ButtonText;
  }
  .recorder-button.listening {
    border-color: Highlight;
    color: Highlight;
  }
  .save {
    border-color: Highlight;
    background: Highlight;
    color: HighlightText;
  }
  .recorder-button:focus-visible, .action-button:focus-visible, .dismiss:focus-visible {
    outline: 2px solid Highlight !important;
  }
}
</style>
