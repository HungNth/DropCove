import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppearanceMode } from "./composables/useSettings";

interface AppearanceSnapshot {
  mode: AppearanceMode;
  theme: "dark" | "light";
  highContrast: boolean;
  revision: number;
}

export async function initializeAppearance() {
  let revision = -1;
  function apply(snapshot: AppearanceSnapshot) {
    if (snapshot.revision < revision) return;
    revision = snapshot.revision;
    document.documentElement.dataset.appearance = snapshot.theme;
  }
  const unlisten = await listen<AppearanceSnapshot>("appearance-changed", (event) => apply(event.payload));
  try {
    apply(await invoke<AppearanceSnapshot>("get_appearance"));
  } catch (error) {
    unlisten();
    throw error;
  }
  globalThis.addEventListener("pagehide", unlisten, { once: true });
}
