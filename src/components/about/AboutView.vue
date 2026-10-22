<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { getCurrentWindow } from "@tauri-apps/api/window";
import logoUrl from "../../../app-icon.svg";
import { checkForUpdates, type UpdateCheckResult } from "../../updateCheck";

interface AboutMetadata {
  productName: string;
  version: string;
  description: string;
  author: string;
  sourceUrl: string;
}

const metadata = ref<AboutMetadata | null>(null);
const errorMessage = ref<string>("");

async function loadMetadata() {
  try {
    metadata.value = await invoke<AboutMetadata>("get_about_metadata");
  } catch (error) {
    errorMessage.value = `Failed to load product metadata: ${error}`;
  }
}

async function openSource(event: Event) {
  event.preventDefault();
  if (!metadata.value?.sourceUrl) return;
  errorMessage.value = "";
  try {
    await openUrl(metadata.value.sourceUrl);
  } catch (error) {
    errorMessage.value = `Could not open browser for Source: ${error}`;
  }
}

const checking = ref(false);
const updateResult = ref<UpdateCheckResult | null>(null);

const updateStatus = computed(() => {
  switch (updateResult.value?.kind) {
    case "up-to-date": return "You're up to date.";
    case "available": return `Version ${updateResult.value.version} is available.`;
    case "no-release": return "No releases published yet.";
    case "rate-limited": return "Update check limit reached. Try again later.";
    case "failed": return "Couldn't check for updates. Try again later.";
    default: return "";
  }
});

const updateAvailable = computed(() => updateResult.value?.kind === "available");

async function runUpdateAction() {
  if (!metadata.value || checking.value) return;
  const result = updateResult.value;
  if (result?.kind === "available") {
    errorMessage.value = "";
    try {
      await openUrl(result.url);
    } catch (error) {
      errorMessage.value = `Could not open browser for Download: ${error}`;
    }
    return;
  }
  checking.value = true;
  updateResult.value = null;
  try {
    updateResult.value = await checkForUpdates(metadata.value.version, metadata.value.sourceUrl);
  } finally {
    checking.value = false;
  }
}

async function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    event.stopPropagation();
    try {
      await getCurrentWindow().close();
    } catch (error) {
      errorMessage.value = `Could not close About: ${error}`;
    }
  }
}

onMounted(() => {
  document.title = "About DropCove";
  void loadMetadata();
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <main
    class="about flex flex-col items-center justify-between w-full h-full box-border p-6 text-dc-text-primary bg-dc-surface font-['Segoe_UI',sans-serif] select-none"
    aria-label="About DropCove"
  >
    <!-- Top Section: Brand Identity -->
    <div class="flex flex-col items-center text-center mt-2">
      <!-- Canonical Bundled Logo Asset -->
      <img
        :src="logoUrl"
        alt="DropCove logo"
        class="size-[64px] mb-3 object-contain select-none pointer-events-none"
      />

      <h1 class="m-0 text-[18px] font-semibold text-dc-text-primary tracking-tight">
        {{ metadata?.productName }}
      </h1>
      <p class="m-0 mt-0.5 text-[12px] text-dc-text-muted">
        <template v-if="metadata?.version">
          Version {{ metadata.version }}
        </template>
      </p>
      <p class="m-0 mt-2 max-w-[320px] text-[12px] leading-relaxed text-dc-text-secondary">
        {{ metadata?.description }}
      </p>
    </div>

    <!-- Middle Section: Metadata Rows -->
    <div class="flex flex-col w-full max-w-[320px] gap-2 py-3 border-y border-dc-border-subtle text-[12px]">
      <div class="flex items-center justify-between">
        <span class="text-dc-text-muted">Author</span>
        <span class="font-medium text-dc-text-primary">{{ metadata?.author }}</span>
      </div>

      <div class="flex items-center justify-between">
        <span class="text-dc-text-muted">Source</span>
        <a
          v-if="metadata?.sourceUrl"
          :href="metadata.sourceUrl"
          class="text-dc-accent hover:underline cursor-pointer bg-transparent border-0 p-0 text-[12px] font-medium focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-dc-focus rounded-xs max-w-[240px] break-all text-right"
          :title="metadata.sourceUrl"
          @click="openSource"
        >
          {{ metadata.sourceUrl }}
        </a>
      </div>
    </div>

    <!-- Error message display if any -->
    <div
      v-if="errorMessage"
      class="text-[11px] text-dc-danger bg-dc-danger-surface border border-dc-danger-border p-2 rounded-md w-full max-w-[320px] text-center"
      role="alert"
    >
      {{ errorMessage }}
    </div>

    <!-- Bottom Section: Update Check -->
    <div class="flex flex-col items-center gap-1.5 mb-2">
      <button
        type="button"
        :aria-disabled="!metadata || checking"
        :class="[
          'px-4 py-1.5 rounded-md border text-[12px] font-medium select-none cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-dc-focus aria-disabled:opacity-[var(--dc-disabled-opacity)] aria-disabled:cursor-not-allowed',
          updateAvailable
            ? 'border-transparent bg-dc-action text-dc-accent-text hover:bg-dc-accent-hover forced-colors:border-dc-border-default'
            : 'border-dc-border-default bg-dc-surface-elevated text-dc-text-primary hover:bg-dc-surface-hover',
        ]"
        aria-describedby="update-status"
        @click="runUpdateAction"
      >
        {{ checking ? "Checking…" : updateResult?.kind === "available" ? `Download ${updateResult.version}` : "Check for updates" }}
      </button>
      <span
        id="update-status"
        aria-live="polite"
        :class="[
          'flex items-center gap-1.5 min-h-[16px]',
          updateAvailable
            ? 'px-3 py-1.5 rounded-md border border-dc-accent bg-dc-accent-subtle text-dc-text-primary text-[13px] font-semibold forced-colors:bg-transparent'
            : 'text-[11px] text-dc-text-muted',
        ]"
      >
        <svg
          v-if="updateAvailable"
          class="size-4 shrink-0 text-dc-accent"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <circle cx="8" cy="8" r="6.25" />
          <path d="M8 4.75v5.5M5.5 8l2.5 2.5L10.5 8" />
        </svg>
        {{ updateStatus }}
      </span>
    </div>
  </main>
</template>
