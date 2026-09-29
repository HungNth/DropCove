# 10: Expand and configure the no-activate Edge Rail

**What to build:** Make the Edge Rail reveal useful details without stealing focus, jumping monitors, covering fullscreen applications unexpectedly, or flickering during pointer movement.

**Blocked by:** 09: Dock a non-empty shelf as the Edge Rail.

**Status:** ready-for-agent

- [ ] Pointer hover expands the rail after 200 ms without activating DropCove.
- [ ] Pointer leave collapses the rail after 300 ms unless an item or batch flyout remains open.
- [ ] Direct rail interaction preserves the foreground application's focus; opening the full shelf activates DropCove.
- [ ] Settings allow Left/Right edge and target-monitor selection, and changes apply predictably and survive restart.
- [ ] The rail remains on its selected monitor instead of following the cursor.
- [ ] The rail hides over a fullscreen foreground application by default using event-driven detection rather than polling.
- [ ] A setting can keep the rail visible over fullscreen applications and survives restart.
- [ ] Reduced accidental expansion/collapse and no-activate behavior are verified in packaged pointer/focus smoke scenarios.