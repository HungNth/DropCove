<script setup lang="ts">
import {
  Pin,
  Minus,
  List,
  CircleMinus,
  X,
  Trash2,
  ArrowLeft,
  ArrowRight,
  ArrowDown,
  GripHorizontal,
  Ellipsis
} from "@lucide/vue";

defineProps<{
  name: "pin" | "list" | "remove" | "close" | "trash" | "left" | "right" | "down" | "grip" | "more";
  size?: 10 | 14;
  filled?: boolean;
  mixed?: boolean;
}>();
</script>

<template>
  <span v-if="name === 'pin' && mixed" class="relative inline-flex items-center justify-center size-[14px] flex-none pointer-events-none" aria-hidden="true">
    <Pin :size="14" class="block size-[14px] flex-none" />
    <Minus :size="10" :stroke-width="2.5" class="absolute block size-[10px] top-[2px] pointer-events-none" />
  </span>
  <Pin
    v-else-if="name === 'pin'"
    :size="14"
    :fill="filled ? 'currentColor' : 'none'"
    class="block size-[14px] flex-none"
  />
  <List v-else-if="name === 'list'" :size="14" class="block size-[14px] flex-none" />
  <CircleMinus v-else-if="name === 'remove'" :size="14" class="block size-[14px] flex-none" />
  <X v-else-if="name === 'close'" :size="14" class="block size-[14px] flex-none" />
  <Trash2 v-else-if="name === 'trash'" :size="14" class="block size-[14px] flex-none" />
  <ArrowLeft v-else-if="name === 'left'" :size="size ?? 14" class="block flex-none" :class="size === 10 ? 'size-[10px]' : 'size-[14px]'" />
  <ArrowRight v-else-if="name === 'right'" :size="size ?? 14" class="block flex-none" :class="size === 10 ? 'size-[10px]' : 'size-[14px]'" />
  <ArrowDown v-else-if="name === 'down'" :size="size ?? 14" class="block flex-none" :class="size === 10 ? 'size-[10px]' : 'size-[14px]'" />
  <GripHorizontal v-else-if="name === 'grip'" :size="size ?? 14" class="block flex-none" :class="size === 10 ? 'size-[10px]' : 'size-[14px]'" />
  <Ellipsis v-else :size="size ?? 14" class="block flex-none" :class="size === 10 ? 'size-[10px]' : 'size-[14px]'" />
</template>

