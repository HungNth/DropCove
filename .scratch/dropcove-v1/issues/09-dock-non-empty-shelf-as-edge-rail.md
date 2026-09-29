# 09: Dock a non-empty shelf as the Edge Rail

**What to build:** Keep held content reachable after dismissal by transitioning a non-empty Drop Shelf into a narrow overlay Edge Rail on the correct monitor and edge.

**Blocked by:** 08: Prove the Stage 1 packaged vertical slice.

**Status:** ready-for-agent

- [ ] After Edge Rail is available, dismissing an empty shelf enters Hidden and dismissing a non-empty shelf enters EdgeDocked.
- [ ] The rail overlays the desktop and does not reserve or change Windows work area.
- [ ] The initial rail docks to the Right edge of the monitor where the shelf was dismissed.
- [ ] The selected monitor and edge are remembered across shelf invocations and process restart.
- [ ] The collapsed rail presents Shelf Batches compactly and remains identifiable without opening the full shelf.
- [ ] Activating Open Shelf from the rail opens the existing unified Drop Shelf and activates DropCove.
- [ ] Emptying the last Shelf Batch hides the rail automatically.
- [ ] Application-seam tests cover Hidden/EdgeDocked transitions and restored placement; packaged smoke covers overlay placement across monitors and DPI settings.