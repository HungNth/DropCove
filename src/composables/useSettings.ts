import { computed, onMounted, onUnmounted, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";

export interface HotKey {
  Control: boolean;
  Alt: boolean;
  Shift: boolean;
  Windows: boolean;
  VirtualKey: number;
}

export type RailEdge = "Left" | "Right";
export type ShakeSensitivity = "Low" | "Normal" | "High";
export interface MonitorChoice { id: string; label: string; connected: boolean }

export interface SettingsSnapshot {
  hotKey: HotKey;
  label: string;
  listening: boolean;
  startWithWindows: boolean;
  railEdge: RailEdge;
  railMonitorId: string;
  showRailOverFullscreen: boolean;
  shakeEnabled: boolean;
  shakeSensitivity: ShakeSensitivity;
  monitors: MonitorChoice[];
}

export function useSettings() {
  const snapshot = shallowRef<SettingsSnapshot | null>(null);
  const status = shallowRef("");
  const busy = shallowRef(false);
  const window = getCurrentWindow();
  const unlisteners: UnlistenFn[] = [];
  let disposed = false;
  let refreshRequest = 0;
  const monitors = computed(() => {
    const choices = snapshot.value?.monitors ?? [];
    const saved = snapshot.value?.railMonitorId;
    return saved && !choices.some((choice) => choice.id.toLowerCase() === saved.toLowerCase())
      ? [...choices, { id: saved, label: "Unavailable saved display", connected: false }]
      : choices;
  });

  async function refresh() {
    const request = ++refreshRequest;
    const current = await invoke<SettingsSnapshot>("get_settings");
    if (!disposed && request === refreshRequest) {
      snapshot.value = current;
    }
  }

  async function startRecording() {
    if (busy.value || disposed) return;
    busy.value = true;
    try {
      const next = await invoke<SettingsSnapshot>("start_settings_recording");
      if (!disposed) {
        snapshot.value = next;
        status.value = "";
      }
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally {
      if (!disposed) busy.value = false;
    }
  }

  async function cancelRecording() {
    if (disposed) return;
    try {
      const next = await invoke<SettingsSnapshot>("cancel_settings_recording");
      if (!disposed) snapshot.value = next;
    } catch (error) {
      if (!disposed) status.value = String(error);
    }
  }

  async function recordKey(key: HotKey) {
    if (busy.value || disposed || !snapshot.value?.listening) return;
    busy.value = true;
    try {
      const next = await invoke<SettingsSnapshot>("record_settings_key", { key });
      if (!disposed) { snapshot.value = next; status.value = ""; }
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally {
      if (!disposed) busy.value = false;
    }
  }
  async function setStartWithWindows(enabled: boolean) {
    if (busy.value || disposed) return;
    busy.value = true;
    try {
      const next = await invoke<SettingsSnapshot>("set_start_with_windows", { enabled });
      if (!disposed) {
        snapshot.value = next;
        status.value = "";
      }
      return next;
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally {
      if (!disposed) busy.value = false;
    }
  }

  async function setRailPlacement(edge: RailEdge, monitorId: string) {
    if (busy.value || disposed) return;
    busy.value = true;
    try {
      const next = await invoke<SettingsSnapshot>("set_rail_placement", { edge, monitorId });
      if (!disposed) { snapshot.value = next; status.value = ""; }
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally { if (!disposed) busy.value = false; }
  }
  async function setShowRailOverFullscreen(enabled: boolean) {
    if (busy.value || disposed) return;
    busy.value = true;
    try {
      const next = await invoke<SettingsSnapshot>("set_show_rail_over_fullscreen", { enabled });
      if (!disposed) { snapshot.value = next; status.value = ""; }
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally { if (!disposed) busy.value = false; }
  }

  async function setShake(enabled: boolean, sensitivity: ShakeSensitivity) {
    if (busy.value || disposed) return;
    busy.value = true;
    try {
      const next = await invoke<SettingsSnapshot>("set_shake", { enabled, sensitivity });
      if (!disposed) { snapshot.value = next; status.value = ""; }
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally { if (!disposed) busy.value = false; }
  }


  async function save() {
    if (busy.value || disposed) return;
    busy.value = true;
    try {
      await invoke("save_settings");
      if (!disposed) status.value = "";
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally {
      if (!disposed) busy.value = false;
    }
  }

  async function cancel() {
    if (busy.value || disposed) return;
    busy.value = true;
    try {
      await invoke("cancel_settings");
      if (!disposed) status.value = "";
    } catch (error) {
      if (!disposed) status.value = String(error);
    } finally {
      if (!disposed) busy.value = false;
    }
  }

  function handleWindowBlur() {
    if (snapshot.value?.listening) {
      void cancelRecording();
    }
  }

  onMounted(async () => {
    globalThis.addEventListener("blur", handleWindowBlur);

    try {
      const unlisten = await window.listen<SettingsSnapshot>("settings-changed", (event) => {
        if (!disposed && event.payload) {
          snapshot.value = event.payload;
        }
      });

      if (disposed) {
        unlisten();
      } else {
        unlisteners.push(unlisten);
      }

      await refresh();
    } catch (error) {
      if (!disposed) status.value = String(error);
    }
  });

  onUnmounted(() => {
    disposed = true;
    globalThis.removeEventListener("blur", handleWindowBlur);
    snapshot.value = null;
    for (const unlisten of unlisteners) unlisten();
  });

  return {
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
    dismissStatus: () => { status.value = ""; },
  };
}
