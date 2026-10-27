import type { ShelfBatch, ShelfItem } from "./useShelf";

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
export type BatchContextActionId = "ShowInExplorer" | "CopyPath" | "CopyText" | "pin" | "remove";

export interface BatchContextAction {
  id: BatchContextActionId;
  label: string;
  danger?: boolean;
}

function singleItemContextActions(item: ShelfItem): BatchContextAction[] {
  if (item.ownership === "External") {
    return [
      { id: "ShowInExplorer", label: "Show in File Explorer" },
      { id: "CopyPath", label: "Copy Path" },
    ];
  }
  if (item.ownership === "Managed" && item.managedContentKind === "Text") return [{ id: "CopyText", label: "Copy Text" }];
  return [];
}

export function batchContextActions(batches: readonly Pick<ShelfBatch, "items">[]): BatchContextAction[] {
  if (batches.length > 1) {
    const pin = batchSelectionPinAction(batches);
    return [
      { id: "pin", label: `${pin.pinned ? "Pin" : "Unpin"} All Selected` },
      { id: "remove", label: "Remove All Selected", danger: true },
    ];
  }

  const batch = batches[0];
  return batch?.items.length === 1 ? singleItemContextActions(batch.items[0]) : [];
}

export function itemContextActions(items: readonly ShelfItem[]): BatchContextAction[] {
  if (items.length > 1) {
    const pin = itemSelectionPinAction(items);
    return [
      { id: "pin", label: `${pin.pinned ? "Pin" : "Unpin"} All Selected` },
      { id: "remove", label: "Remove All Selected", danger: true },
    ];
  }
  return items.length === 1 ? singleItemContextActions(items[0]) : [];
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
