# 01: Single-item Pinning and durable Copy retention

Status: resolved
Type: task
Version: 0.0.3
Parent: [Specification](../spec.md)
Blocked by: none
Implementation approval: approved by the user on 2026-10-13

## What to build

A user can Pin or Unpin a single-item card, see its pin state, restart the app, and drag it out with the correct Pinned-versus-Temporary lifecycle. Extend the existing core and native drag path; do not add a second owner or reimplement Copy handling.

## Acceptance criteria

- [x] Newly accepted items remain Temporary by default; Pin and Unpin persist before changed state is presented.
- [x] A Pinned Item exposes a visible indicator and accessible Pin/Unpin action.
- [x] Accepted Copy retains a Pinned Item; after Unpin, a later accepted Copy consumes it.
- [x] Canceled, rejected, or failed drag-out retains references regardless of pin state.
- [x] Restart restores the item, its identifier/path, and pin state.
- [x] Pinning changes no source object, batch/item order, availability, or sizing intent.
- [x] Persistence failure preserves prior visible and durable lifecycle state and reports the failure.
- [x] Existing Remove Batch continues to remove references, including pinned ones, without deleting source objects.

## Testing seam

Use the agreed Core Shelf State interface with isolated real SQLite storage, restart/reopen, and controlled drag outcomes. Check lifecycle transitions and rollback through returned/restored state, not SQL wiring or component text.

## Demo path

Drop one file, Pin it, restart, complete native Copy and observe retention, Unpin, then complete Copy and observe consumption. Repeat cancellation and failed persistence without losing the reference.

## Non-goals

Bulk Pinning, Manage Items, Clear Temporary Items, and full manual resizing belong to their later tickets.

## Comments

- Implemented the serialized transactional `ShelfState::set_item_pinned` mutation, async IPC command, composable action, and single-item card controls. The card exposes accessible names/pressed state and a visible Pinned indicator; it changes only after the durable command succeeds.
- Added three real-SQLite regressions covering Pin/Unpin restore and Copy lifecycle, failed Pin/Unpin rollback, and unknown-item rejection plus explicit pinned batch removal. Existing Temporary consumption is reused rather than replaced.
- `pnpm build` passed (`vue-tsc` and Vite). The Windows-targeted suite compiled through cargo-xwin; its configured Wine runner could not execute on this host. Direct execution of `src-tauri/target/x86_64-pc-windows-msvc/debug/deps/dropcove_lib-46795291c2b9da11.exe` then reported **14 passed, 0 failed**.
- Built the native Windows x64 executable using `cargo xwin build --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc --features tauri/custom-protocol --release --locked`. The explicit feature embeds the frontend; a direct build without it attempted the dev URL and was not counted as surface proof. The linker reported external CRT PDB warnings (LNK4099), not build failures.
- Exercised the actual embedded-frontend app with an owned profile: pointer Pin, accessible pressed state/visible indicator, process restart with identical pinned snapshot, native OLE Copy to an owned Windows Forms target with matching file contents and retained Pinned reference, keyboard Unpin, restart with Temporary state, then native Copy consuming the reference.
- Native Escape cancellation and a rejected drop back onto the Shelf retained both Pinned and Temporary references. Removing the owned source before drag produced the real storage-path error while retaining the reference; the fixture was restored afterward.
- Injected a real SQLite update-failure trigger in the owned smoke profile: failed Unpin displayed the persistence error while preserving visible and durable pinned state. Core regressions separately exercised failed Pin and failed Unpin across reopen.
- Native Remove Batch removed a Pinned reference without deleting its source. Smoke intake used the real `accept_drop` IPC; inbound native drag is unchanged and was not re-tested by this ticket.
- Released the managed CDP tab, terminated only the two known owned app/target processes, released mouse/Escape state and SQLite handles, and removed the owned smoke directory including fixture, copies, Shelf profile, and WebView data. Final existence check was false. Removed the throwaway `smoke-copy-target.ps1`.
- Version metadata stays at 0.0.2 until ticket 06; this verifies the first development slice, not an installed 0.0.3 release. No Git commit created.
