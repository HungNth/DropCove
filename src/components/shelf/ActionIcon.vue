<script setup lang="ts">
defineProps<{ name: "pin" | "list" | "remove" | "close" | "trash" | "left" | "right" | "down"; filled?: boolean; mixed?: boolean }>();
</script>

<template>
  <svg class="block size-[16px] flex-none" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
    <template v-if="name === 'pin'">
      <path d="M8 3h8l-1 7 4 4v2H5v-2l4-4z" :fill="filled ? 'currentColor' : 'none'" />
      <path d="M12 16v5" />
      <path v-if="mixed" d="M9 7h6" stroke-width="3" />
    </template>
    <path v-else-if="name === 'list'" d="M9 6h12M9 12h12M9 18h12M3 6h1M3 12h1M3 18h1" />
    <template v-else-if="name === 'remove'"><circle cx="12" cy="12" r="9" /><path d="M8 12h8" /></template>
    <path v-else-if="name === 'close'" d="m6 6 12 12M18 6 6 18" />
    <path v-else-if="name === 'trash'" d="M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7M14 10v7" />
    <path v-else-if="name === 'left'" d="M20 12H4m7-7-7 7 7 7" />
    <path v-else-if="name === 'right'" d="M4 12h16m-7-7 7 7-7 7" />
    <path v-else d="M12 4v16m-7-7 7 7 7-7" />
  </svg>
</template>

