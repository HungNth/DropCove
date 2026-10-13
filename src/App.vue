<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import DropShelf from "./components/shelf/DropShelf.vue";
import ManageItems from "./components/shelf/ManageItems.vue";
import SettingsView from "./components/settings/SettingsView.vue";

const currentLabel = getCurrentWindow().label;
const settingsSurface = currentLabel === "settings";
const managementSurface = currentLabel === "manage-items";
</script>

<template>
  <SettingsView v-if="settingsSurface" />
  <ManageItems v-else-if="managementSurface" />
  <DropShelf v-else />
</template>
