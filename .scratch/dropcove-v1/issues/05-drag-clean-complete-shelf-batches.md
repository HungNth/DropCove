# 05: Drag and clean complete Shelf Batches

**What to build:** Let users drag an entire Shelf Batch and manage bulk cleanup while handling mixed pinned, temporary, Missing, and Unavailable Shelf Items without risking source files.

**Blocked by:** 04: Drag one Shelf Item out safely.

**Status:** ready-for-agent

- [ ] Dragging a Shelf Batch starts one native multi-file Copy operation containing its valid available paths.
- [ ] A successful mixed batch drag consumes participating Temporary Items, retains participating Pinned Items, and preserves the reduced Shelf Batch when items remain.
- [ ] Users can Remove Batch and Clear Temporary Items; bulk actions require confirmation and remove references only.
- [ ] Paths confirmed absent are classified as Missing; paths inaccessible because of volume, network, cloud, or permission state are classified as Unavailable.
- [ ] Availability is validated when the shelf is shown and immediately before drag-out without a realtime filesystem watcher.
- [ ] Starting a drag removes confirmed-Missing references before native drag begins, removes any resulting empty Shelf Batch, and does not restore those references if the later drag is canceled.
- [ ] Unavailable references are retained; when valid items also exist, the user must confirm before dragging only the available portion.
- [ ] Failed same-integrity and cross-integrity attempts retain valid participating temporary references.
- [ ] Application-seam tests cover mixed lifecycle, bulk cleanup, Missing cleanup, Unavailable partial drag, cancellation, and filesystem non-modification.
- [ ] Packaged smoke scenarios exercise whole-batch drag-out, folder drag-out to Explorer, partial availability, and elevated-target failure behavior.