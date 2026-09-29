# 03: Drop filesystem paths into Shelf Batches

**What to build:** Let users drag real files and folders from Explorer or Desktop into the unified Drop Shelf and see one ordered Shelf Batch for each accepted drop operation.

**Blocked by:** 02: Summon and manage the resident Drop Shelf.

**Status:** ready-for-agent

- [ ] File and folder paths from Explorer and Desktop are accepted into the Drop Shelf without copying or changing the source filesystem objects.
- [ ] Each accepted drop creates one Shelf Batch containing Shelf Items in source order, and newest Shelf Batches appear first.
- [ ] Repeated paths within one drop are deduplicated while the same path dropped later creates an independent Shelf Item in a new Shelf Batch.
- [ ] Local, removable, UNC/network, and cloud-backed filesystem paths are accepted when a usable path is supplied.
- [ ] Stream-only virtual items are rejected and never materialized into DropCove storage.
- [ ] Mixed payloads accept supported paths and visibly report the number of skipped unsupported items; payloads with no supported path create no Shelf Batch.
- [ ] Every new Shelf Item is temporary by default and displays its name, type, and a usable native file/folder icon.
- [ ] Application-seam tests cover batching, source order, deduplication boundaries, newest-first ordering, and mixed-payload outcomes.
- [ ] A packaged smoke scenario demonstrates Explorer/Desktop drag-in through the visible shelf.