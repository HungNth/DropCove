# 01: Managed-text Drop Shelf round trip

**What to build:** Make one complete plain-text workflow work through the Drop Shelf: receive real Windows Unicode text, materialize one crash-safe Managed File, restore it after restart, copy it out as a normal `.txt` file, preserve it when Pinned or when drag-out is canceled/rejected, and permanently clean it after Temporary consumption or explicit batch removal. This first slice owns the complete Managed File lifecycle—staging, durable ownership, promotion, startup reconciliation, pending deletion, retry, and external-source safety—so no later ticket has to finish a non-running `Staged → Ready` foundation.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.23 — Managed File Lifecycle and Plain-Text Intake](../spec.md)

**Testing seam:** Actual Windows Resident Instance with a real OLE Unicode-text source, real Drop Shelf/WebView2 surface, owned Profile Data, real native Copy destination, and supporting Core Shelf State SQLite/filesystem regressions.

**Demo path:** Drag multiline Unicode text into the Drop Shelf, restart, copy the generated `.txt` to a native destination, repeat while Pinned, then hold the Managed File open to observe pending deletion and successful retry after releasing the handle.

- [x] Existing file/folder intake remains path-reference-only and retains current ordering, duplicate, Copy cursor, placeholder, modal-blocking, and application-owned-drag rejection behavior.
- [x] For `0.0.23`, the native OLE adapter chooses valid filesystem paths before `CF_UNICODETEXT`; invalid or unsupported higher-priority file-like payloads do not fall through into companion URL/text content.
- [x] Non-empty `CF_UNICODETEXT` is copied into owned memory on the registered window thread; borrowed COM media never crosses into persistence work.
- [x] Accepted text preserves Unicode code points, whitespace, multiline content, URL-shaped text, and line endings; output is UTF-8 without a BOM and without trimming or normalization.
- [x] Empty text is rejected, whitespace-only text is accepted, exactly `10 MiB` of UTF-8 is accepted, and one byte over is rejected without Shelf, sizing, scroll, staging, or Shake state change.
- [x] Each accepted text drop creates one Temporary Shelf Item in one new Shelf Batch with `Dropped text YYYY-MM-DD HH-mm-ss.txt` naming and deterministic numeric collision suffixes.
- [x] Managed ownership is represented only by the authoritative optional ownership relation; a path location by itself never grants deletion authority.
- [x] Managed files use isolated per-item final storage and a dedicated same-volume staging area without content hashing, shared backing, or byte-level duplicate suppression.
- [x] Intake executes the complete durable sequence: flushed staging, `Staged` ownership commit, atomic promotion, `Ready` commit, then Shelf publication.
- [x] `Staged` items are absent from every Shelf projection, sizing calculation, visual request, duplicate comparison, drag participant set, and user-facing event.
- [x] Startup reconciliation handles unowned staging, `Staged` with staging only, `Staged` with final only, and invalid/missing staging+final states before restoring the Shelf.
- [x] Runtime failure after the `Staged` commit compensates by removing the unprojected item/batch and all staging/final remnants; interrupted compensation is completed on restart.
- [x] Restart restores complete `Ready` text items with exact identity, filename, bytes, order, Pin state, sizing, and availability.
- [x] Shell Visual and existing single-item card actions work without a text-specific presentation path.
- [x] Native item/batch drag-out transfers exact UTF-8 bytes and filename to a real Copy destination.
- [x] Accepted Copy consumes a Temporary text item and atomically records Pending Managed File Deletion before permanent deletion; Pinned Copy retains reference and bytes.
- [x] Canceled, rejected, and pre-acceptance-failed drag-out retains both the Shelf Item and Managed File.
- [x] Remove Batch on the one-item text batch removes the item and records owned cleanup; ordinary external batch removal never deletes source objects.
- [x] Missing owned targets count as already deleted; successful cleanup removes the pending record and empty item directory.
- [x] A real Windows sharing violation leaves the Shelf mutation committed, retains one durable pending record, reports the exact singular/plural pending count, and deletes on a later cleanup event or startup after the handle closes.
- [x] No timer, polling loop, filesystem watcher, manual cleanup command, Settings field, frontend-owned file state, or second lifecycle owner is introduced.
- [x] Shake-to-open keeps the Shelf visible only after a newly accepted text batch; empty, over-limit, unsupported, or failed text intake restores the prior display state.
- [x] Existing external source bytes and directories remain unchanged through intake, Copy consumption, batch removal, pending retry, and restart.
- [x] Permanent tests cover authoritative Shelf/database/filesystem outcomes and every intake crash window; they do not pin helper call order, SQL text, source strings, or mock echoes.
- [x] The actual Windows demo, frontend production build, cargo-xwin Windows build, and freshly compiled native regression executable pass while all release metadata remains `0.0.22`.
- [x] No Git commit is created without separate explicit user authorization.

## Comments

- Implementation authorized by the user with “hãy triển khai các ticket”. Core Shelf State/Managed File work and the native OLE adapter are separate edit owners; the integration owner runs TDD checks, Windows smoke, and the final release gate. No Git commit is authorized.

## Answer

Delivered through the vertical slice in `src-tauri/src/managed_files.rs`, `src-tauri/src/shelf.rs`, `src-tauri/src/windows_drop.rs`, `src-tauri/src/windows_drag.rs`, and `src/components/`. Real Windows smoke proved OLE Unicode intake, 71-byte UTF-8 materialization, process restart, native destination Copy, Temporary consumption, Pinned Copy retention, Windows sharing violation queuing, and startup cleanup retry. All 140 native regressions passed. Release metadata remains `0.0.22`. No commit created.