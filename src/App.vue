<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import DropShelf from "./components/shelf/DropShelf.vue";
import ManageItems from "./components/shelf/ManageItems.vue";

const managementSurface = getCurrentWindow().label === "manage-items";
</script>

<template>
  <ManageItems v-if="managementSurface" />
  <DropShelf v-else />
</template>
