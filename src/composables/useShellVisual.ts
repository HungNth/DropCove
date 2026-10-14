import { computed, onMounted, onUnmounted, shallowRef, toValue, watch, type MaybeRefOrGetter } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
export type SurfaceId = "shelf" | "manageItems" | "edgeRail";
interface GenerationChangedPayload { surface: SurfaceId; generation: number }
const generations = shallowRef<Record<SurfaceId, number>>({ shelf: 0, manageItems: 0, edgeRail: 0 });
let listening: Promise<void> | undefined;
let unlisten: UnlistenFn | undefined;
let users = 0;
const label = getCurrentWindow().label;
const callerSurface: SurfaceId = label === "edge-rail" ? "edgeRail" : label === "manage-items" ? "manageItems" : "shelf";
async function initialize(surface: SurfaceId) {
  if (!listening) listening = (async () => {
    unlisten = await getCurrentWindow().listen<GenerationChangedPayload>("visual-generation-changed", ({ payload }) => {
      generations.value = { ...generations.value, [payload.surface]: payload.generation };
    });
  })();
  await listening;
  const generation = await invoke<number>("get_surface_generation");
  generations.value = { ...generations.value, [surface]: Math.max(generations.value[surface], generation) };
}
export async function invalidateSurface(surface: SurfaceId = callerSurface) {
  const generation = await invoke<number>("invalidate_surface_visuals");
  generations.value = { ...generations.value, [surface]: generation };
}
export function useShellVisual(
  itemId: MaybeRefOrGetter<string>,
  _isFolder: MaybeRefOrGetter<boolean>,
  size: MaybeRefOrGetter<number> = 32,
  requestedSurface?: SurfaceId,
) {
  const surface = requestedSurface ?? callerSurface;
  const visualUrl = shallowRef<string | null>(null);
  const loading = shallowRef(false);
  let disposed = false;
  let ready = false;
  let sequence = 0;
  let active: { itemId: string; generation: number; size: number; scaleFactor: number } | undefined;
  function clear() {
    ++sequence;
    if (visualUrl.value) URL.revokeObjectURL(visualUrl.value);
    visualUrl.value = null;
    if (active) {
      void invoke("cancel_shelf_visual", active).catch(() => {});
      active = undefined;
    }
  }
  async function load() {
    if (!ready || disposed) return;
    clear();
    const request = ++sequence;
    const args = { itemId: toValue(itemId), generation: generations.value[surface], size: toValue(size), scaleFactor: window.devicePixelRatio };
    active = args;
    loading.value = true;
    try {
      const bytes = await invoke<ArrayBuffer>("get_shelf_visual", args);
      if (disposed || request !== sequence || args.generation !== generations.value[surface]) return;
      if (bytes.byteLength) visualUrl.value = URL.createObjectURL(new Blob([bytes], { type: "image/bmp" }));
    } catch {
      // Shell visuals are optional; the existing generic icon remains visible.
    } finally {
      if (request === sequence) { loading.value = false; active = undefined; }
    }
  }
  watch(() => [toValue(itemId), toValue(size), generations.value[surface]], () => { void load(); });
  onMounted(async () => {
    ++users;
    try { await initialize(surface); if (!disposed) { ready = true; await load(); } }
    catch { loading.value = false; }
  });
  onUnmounted(() => {
    disposed = true;
    clear();
    if (--users === 0) {
      const previous = listening;
      void previous?.then(() => { if (users === 0) { unlisten?.(); unlisten = undefined; listening = undefined; } }).catch(() => { listening = undefined; });
    }
  });
  return { visualUrl, loading, hasFallback: computed(() => !visualUrl.value) };
}
