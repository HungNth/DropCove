import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";

test("DropShelf and ShelfResizeHandles remove border hover highlight completely", () => {
  const dropShelfContent = readFileSync("src/components/shelf/DropShelf.vue", "utf-8");
  const useShelfContent = readFileSync("src/composables/useShelf.ts", "utf-8");
  const resizeHandlesContent = readFileSync("src/components/shelf/ShelfResizeHandles.vue", "utf-8");

  // 1. DropShelf must not dynamically light up border or inset shadow on resizeHover
  const dropShelfHasHoverHighlight = dropShelfContent.includes("border-dc-drag-hover-border") ||
    dropShelfContent.includes("resizeHover");
  assert.equal(dropShelfHasHoverHighlight, false, "DropShelf still toggles border-dc-drag-hover-border or binds resizeHover");
  assert.match(dropShelfContent, /border-dc-border-default/, "DropShelf must retain standard default border");

  // 2. useShelf must not export or retain dead resizeHover state
  const useShelfHasResizeHover = useShelfContent.includes("resizeHover");
  assert.equal(useShelfHasResizeHover, false, "useShelf still contains or exports resizeHover");

  // 3. ShelfResizeHandles must NOT illuminate border / box-shadow on hover
  const handlesHaveHoverShadow = resizeHandlesContent.includes("has-[.resize-handle:hover]");
  assert.equal(handlesHaveHoverShadow, false, "ShelfResizeHandles still has hover highlight");
});
