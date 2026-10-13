<script setup lang="ts">
import { useManageItems } from "../../composables/useManageItems";
import ShelfItemRow from "./ShelfItemRow.vue";

const { batch, status, busy, setItemPinned, removeItem, startItemDrag, close, dismissStatus } = useManageItems();
</script>

<template>
  <main class="management" :aria-busy="busy" aria-label="Manage Items">
    <header class="header">
      <h1 class="title">Manage Items<span v-if="batch" class="count"> · {{ batch.items.length }}</span></h1>
      <button class="close" type="button" :disabled="busy" aria-label="Close Manage Items" @click="close">Close</button>
    </header>
    <ul v-if="batch" class="items" aria-label="Batch items">
      <ShelfItemRow
        v-for="item in batch.items"
        :key="item.id"
        :item="item"
        :busy="busy"
        @pin="setItemPinned"
        @remove="removeItem"
        @drag="startItemDrag"
      />
    </ul>
    <div v-if="status" class="status" role="alert">
      <span>{{ status }}</span>
      <button class="dismiss" type="button" aria-label="Dismiss status" @click="dismissStatus">×</button>
    </div>
  </main>
</template>

<style scoped>
.management { display: flex; flex-direction: column; height: 100%; box-sizing: border-box; border: 1px solid #46536a; border-radius: 9px; color: #edf2fc; background: #141c2b; font-family: "Segoe UI", sans-serif; }
.header { display: flex; align-items: center; justify-content: space-between; gap: 6px; padding: 9px; }
.title { margin: 0; font-size: 13px; }
.count { color: #a7b4cb; font-weight: normal; }
.close { border: 1px solid #46536a; border-radius: 4px; padding: 3px 6px; color: inherit; background: transparent; font: inherit; font-size: 11px; cursor: pointer; }
.close:focus-visible, .dismiss:focus-visible { outline: 2px solid #a5b4fc; }
.close:disabled { opacity: .45; cursor: default; }
.items { display: flex; flex-direction: column; gap: 5px; min-height: 0; overflow-y: auto; overflow-x: hidden; margin: 0; padding: 0 7px 7px; list-style: none; }
.status { display: flex; align-items: start; gap: 4px; flex-shrink: 0; margin: auto 7px 7px; padding: 6px; border: 1px solid #efad70; border-radius: 5px; font-size: 11px; overflow-wrap: anywhere; }
.dismiss { margin-left: auto; border: 0; color: inherit; background: transparent; cursor: pointer; }
</style>
