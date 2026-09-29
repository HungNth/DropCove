# 09: Dock a non-empty shelf as the Edge Rail

**What to build:** Keep held content reachable after dismissal by transitioning a non-empty Drop Shelf into a narrow overlay Edge Rail on the correct monitor and edge.

**Blocked by:** 08: Prove the Stage 1 packaged vertical slice.

**Status:** ready-for-human

- [x] After Edge Rail is available, dismissing an empty shelf enters Hidden and dismissing a non-empty shelf enters EdgeDocked.
- [x] The rail overlays the desktop without using Windows AppBar/work-area reservation.
- [x] The initial rail docks to the Right edge of the monitor where the shelf was dismissed.
- [x] The selected monitor and edge are remembered across shelf invocations and process restart.
- [x] The collapsed rail presents Shelf Batches compactly and remains identifiable without opening the full shelf.
- [x] Activating Open Shelf from the rail opens the existing unified Drop Shelf and activates DropCove.
- [x] Emptying the last Shelf Batch transitions an EdgeDocked manager to Hidden automatically.
- [x] Application-seam tests cover Hidden/EdgeDocked transitions and restored placement.
- [ ] Packaged smoke covers overlay placement across multiple monitors and DPI settings.

## Comments

- 2026-09-29: Added the `ShelfDisplayState` lifecycle, remembered `ShelfRailPlacement`, and SQLite `shelf_ui_state` persistence. Non-empty dismissal uses the first dismissed monitor with the Right edge; later dismissals reuse the remembered placement.
- 2026-09-29: Added the collapsed Edge Rail overlay as a borderless topmost WinUI window. It spans the selected monitor bounds, uses monitor DPI for its 64-logical-pixel width, shows compact batch summaries, and opens the existing unified shelf through the Open Shelf button.
- 2026-09-29: Application-seam coverage now verifies empty/non-empty dismissal, automatic Hidden transition after the last item is removed, unified-shelf reopening, and rail placement restoration through a real temporary SQLite database.
- 2026-09-29: Debug-build packaged-style smoke exercised non-empty dismissal, real pointer Open Shelf activation, persisted `\\.\DISPLAY1|Right` placement, graceful process restart, and the same restored rail placement. Multi-monitor and changed-per-monitor-DPI smoke remains open.
- 2026-09-29: Corrected Open Shelf from the rail to restore the unified shelf on the remembered rail monitor instead of the cursor monitor; the hotkey path remains cursor-based. Removed an unused cursor-monitor interop API and a duplicate rail item-type formatter.