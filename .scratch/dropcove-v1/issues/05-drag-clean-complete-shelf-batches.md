# 05: Drag and clean complete Shelf Batches

**What to build:** Let users drag an entire Shelf Batch and manage bulk cleanup while handling mixed pinned, temporary, Missing, and Unavailable Shelf Items without risking source files.

**Blocked by:** 04: Drag one Shelf Item out safely.

**Status:** ready-for-agent

- [x] Dragging a Shelf Batch starts one native multi-file Copy operation containing its valid available paths.
- [x] A successful mixed batch drag consumes participating Temporary Items, retains participating Pinned Items, and preserves the reduced Shelf Batch when items remain.
- [x] Users can Remove Batch directly and Clear Temporary Items; clearing temporary items requires confirmation and actions remove references only.
- [x] Paths confirmed absent are classified as Missing; paths inaccessible because of volume, network, cloud, or permission state are classified as Unavailable.
- [ ] Availability is validated when the shelf is shown and immediately before drag-out without a realtime filesystem watcher.
- [x] Starting a drag removes confirmed-Missing references before native drag begins, removes any resulting empty Shelf Batch, and does not restore those references if the later drag is canceled.
- [ ] Unavailable references are retained; when valid items also exist, the user must confirm before dragging only the available portion.
- [ ] Failed same-integrity and cross-integrity attempts retain valid participating temporary references.
- [ ] Application-seam tests cover mixed lifecycle, bulk cleanup, Missing cleanup, Unavailable partial drag, cancellation, and filesystem non-modification.
- [ ] Packaged smoke scenarios exercise whole-batch drag-out, folder drag-out to Explorer, partial availability, and elevated-target failure behavior.

## Comments

- 2026-09-29: Implemented whole-batch drag-out via `Expander.Header` (`CanDrag="True"`, `DragStarting`, `DropCompleted`), setting default `IsExpanded="False"`.
- 2026-09-29: Added Core seam methods in `DropShelfManager` for `CompleteBatchDrag`, `RemoveBatch`, `ClearTemporaryItems`, and `PrepareBatchForDrag` with 7 new unit tests (16/16 passed).
- 2026-09-29: Published and installed build verified by user: dropping multiple files defaults to collapsed state; dragging the whole batch header to File Explorer copies all files concurrently; dragging a mixed batch consumes temporary files while retaining pinned files in the reduced batch.
- 2026-09-29: Replaced Win32 MessageBox with a dedicated modern WinUI 3 `ConfirmWindow` matching DropCove Fluent styles and colors. Positioned directly centered over the DropCove shelf window (`WindowInterop.CenterOverWindow`), set owner HWND, made topmost, and enforced true modality by disabling input to the shelf while open.
- 2026-09-29: Product decision: Remove Batch operates directly on click (matching single-item removal for fast UX); Clear Temporary Items retains the modern centered modal ConfirmWindow with Yes/No. Packaged build updated and running at PID 4412; runtime verification pending user test.
- 2026-09-29: User confirmed T5.1 through T5.5 at runtime, but source inspection found no availability-validation call when the shelf is shown; T5.5 remains open until that missing path exists and is verified.
- 2026-09-29: User reproduced T5.6 with one Missing item: available files copied successfully but participating Temporary Items remained on the shelf. The cause was rebuilding `BatchList.ItemsSource` during `DragStarting`, which removed the visual source before `DropCompleted`. The fix defers that refresh until completion; 19 Core tests now cover accepted and canceled post-Missing state transitions. Installed drag verification remains pending.
- 2026-09-29: T5.7 remains open because partial availability proceeds without confirmation. T5.8 lacks cross-integrity runtime evidence; T5.9 lacks full adapter/filesystem coverage; T5.10 lacks folder, partial-availability, and elevated-target installed smoke.
- 2026-09-29: User completed the prepared installed-EXE partial-Missing drag and confirmed T5.6 passes: the available file copied successfully and the participating Temporary batch was cleared. Cancellation state remains covered by the combined Core regression test.