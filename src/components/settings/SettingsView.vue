<script setup lang="ts">
import { nextTick, onMounted, useTemplateRef } from "vue";
import { useSettings, type RailEdge } from "../../composables/useSettings";
import RailPlacementFields from "./RailPlacementFields.vue";

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
  border: 1px solid #46536a;
  border-radius: 9px;
  color: #edf2fc;
  background: #141c2b;
  font-family: "Segoe UI", sans-serif;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px 10px;
  border-bottom: 1px solid #232d42;
}

.title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #edf2fc;
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
  color: #cbd5e1;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.description {
  margin: 0;
  font-size: 12px;
  color: #94a3b8;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 4px;
}

.field-label {
  font-size: 12px;
  color: #cbd5e1;
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
  border: 1px solid #46536a;
  border-radius: 5px;
  background: #1b2438;
  color: #edf2fc;
  font-family: inherit;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: border-color 0.15s, background-color 0.15s;
}

.recorder-button:hover:not(:disabled) {
  border-color: #6366f1;
  background: #202b44;
}

.recorder-button.listening {
  border-color: #818cf8;
  background: #283556;
  color: #a5b4fc;
  box-shadow: 0 0 0 1px #818cf8;
}

.recorder-button:focus-visible {
  outline: 2px solid #a5b4fc;
  outline-offset: 1px;
}

.recorder-button:disabled {
  opacity: 0.45;
  cursor: default;
}

.recorder-prompt {
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.field-help {
  margin: 0;
  font-size: 11px;
  color: #8292ab;
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
  accent-color: #6366f1;
  cursor: pointer;
}

.checkbox-input:disabled {
  cursor: default;
  opacity: 0.45;
}

.checkbox-row:has(.checkbox-input:disabled) {
  cursor: default;
  opacity: 0.45;
}

.checkbox-label {
  font-size: 13px;
  font-weight: 500;
  color: #edf2fc;
}


.status {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  margin: 0 14px 10px;
  padding: 8px 10px;
  border: 1px solid #efad70;
  border-radius: 6px;
  background: rgba(239, 173, 112, 0.1);
  color: #fbbf24;
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
  outline: 2px solid #a5b4fc;
}

.footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 10px 14px;
  border-top: 1px solid #232d42;
  background: #101724;
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
  outline: 2px solid #a5b4fc;
  outline-offset: 1px;
}

.action-button:disabled {
  opacity: 0.45;
  cursor: default;
}

.cancel {
  border: 1px solid #46536a;
  background: transparent;
  color: #cbd5e1;
}

.cancel:hover:not(:disabled) {
  background: #1b2438;
  border-color: #64748b;
}

.save {
  border: 1px solid #6366f1;
  background: #4f46e5;
  color: #ffffff;
}

.save:hover:not(:disabled) {
  background: #4338ca;
  border-color: #818cf8;
}
</style>
