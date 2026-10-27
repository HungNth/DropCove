import type { ShelfBatch } from "./useShelf";

export interface PinAction {
  label: string;
  pinned: boolean;
}

// Cardinality follows the selection unit (batches), not the affected item count.
export function batchSelectionPinAction(batches: readonly Pick<ShelfBatch, "items">[]): PinAction {
  const items = batches.flatMap(batch => batch.items);
  const pinned = !(items.length > 0 && items.every(item => item.isPinned));
  const verb = pinned ? "Pin" : "Unpin";
  if (batches.length > 1) return { label: `${verb} Selected`, pinned };
  return { label: `${verb} ${items.length > 1 ? "All Items" : "Item"}`, pinned };
}

// Item Selection cardinality is the number of selected rows.
export function itemSelectionPinAction(items: readonly { isPinned: boolean }[]): PinAction {
  const pinned = !(items.length > 0 && items.every(item => item.isPinned));
  const verb = pinned ? "Pin" : "Unpin";
  return { label: `${verb} ${items.length > 1 ? "Selected" : "Item"}`, pinned };
}

export interface RemovalPreview {
  batches: number;
  items: number;
  managedFiles: number;
  token: string;
}

const files = (count: number) => `${count} DropCove-created ${count === 1 ? "file" : "files"}`;

// Copy follows the authoritative ownership counts; the caller never supplies them.
export function removalConfirmationCopy(
  preview: Pick<RemovalPreview, "batches" | "items" | "managedFiles">,
  domain: "batches" | "items",
): { scope: string; body: string } {
  const scope = domain === "batches" ? `${preview.batches} batches · ${preview.items} items` : `${preview.items} items`;
  if (preview.managedFiles === 0) return { scope, body: "Source files and folders will not be deleted." };
  const deleted = `${files(preview.managedFiles)} will be permanently deleted.`;
  const external = preview.managedFiles < preview.items;
  return { scope, body: external ? `External sources stay. ${deleted}` : deleted };
}
