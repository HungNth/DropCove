<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import DropShelf from "./components/shelf/DropShelf.vue";
import ManageItems from "./components/shelf/ManageItems.vue";
import SettingsView from "./components/settings/SettingsView.vue";
import RailHandle from "./components/rail/RailHandle.vue";
import AboutView from "./components/about/AboutView.vue";
import ClearConfirmationView from "./components/shelf/ClearConfirmationView.vue";

const currentLabel = getCurrentWindow().label;
const settingsSurface = currentLabel === "settings";
const managementSurface = currentLabel === "manage-items";
</script>

<template>
  <SettingsView v-if="settingsSurface" />
  <AboutView v-else-if="currentLabel === 'about'" />
  <ClearConfirmationView v-else-if="currentLabel === 'clear-temporary'" />
  <ManageItems v-else-if="managementSurface" />
  <RailHandle v-else-if="currentLabel === 'edge-rail'" />
  <DropShelf v-else />
</template>
