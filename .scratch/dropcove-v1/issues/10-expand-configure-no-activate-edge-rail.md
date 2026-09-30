# 10: Expand and configure the no-activate Edge Rail

**What to build:** Make the Edge Rail reveal useful details without stealing focus, jumping monitors, covering fullscreen applications unexpectedly, or flickering during pointer movement.

**Blocked by:** 09: Dock a non-empty shelf as the Edge Rail.

**Interaction geometry contract for Tickets 10-12:**

- Collapsed footprint is `64 × 112` logical pixels at the selected monitor edge.
- Expanded footprint is at most `320 × 640` logical pixels, clamped to the selected monitor work area; excess batches scroll inside the rail.
- The collapsed window bounds are the hover and direct-interaction hit zone. No separate proximity sensor is part of this ticket.
- The rail is vertically centered on the selected monitor work area and expands inward from the selected Left/Right edge.
- Edge-Drop-specific transparency, shadows, beacon/proximity styling, custom assets, and motion polish remain deferred to Ticket 16; this contract covers interaction geometry, not visual parity.

**Status:** ready-for-human

- [x] Pointer hover expands the rail after 200 ms without activating DropCove.
- [x] Pointer leave collapses the rail after 300 ms unless an item or batch flyout remains open.
- [x] Direct rail interaction preserves the foreground application's focus; opening the full shelf activates DropCove.
- [x] Settings allow Left/Right edge and target-monitor selection, and changes apply predictably and survive restart.
- [x] The rail remains on its selected monitor instead of following the cursor.
- [x] The rail hides over a fullscreen foreground application by default using event-driven detection rather than polling.
- [x] A setting can keep the rail visible over fullscreen applications and survives restart.
- [x] Reduced accidental expansion/collapse and no-activate behavior are verified in packaged pointer/focus smoke scenarios.

## Comments

- 2026-09-30: Implemented bounded collapsed/expanded geometry, no-activate native styles, delayed hover state transitions, selected-monitor/edge settings, persisted placement updates, and event-driven foreground-window fullscreen policy. Edge-Drop-specific visual skin remains deferred to Ticket 16.
- 2026-09-30: The application-seam suite passes all 42 tests, including settings round-trip and legacy-settings defaults, and the x64 Release app builds successfully. A release smoke run observed the `64 × 112` collapsed and `320 × 640` expanded bounds, delayed collapse, and `WS_EX_NOACTIVATE`/tool-window styles.
- 2026-09-30: Packaged smoke scenarios verified focus preservation, settings persistence, edge docking, and monitor pinning across restarts.
- 2026-09-30: Diagnosed F11 fullscreen bug: in-place window resize does not change foreground HWND, so `EVENT_SYSTEM_FOREGROUND` was never dispatched. Added `EVENT_OBJECT_LOCATIONCHANGE` (0x800B) hook filtered to `OBJID_WINDOW` and the current foreground window to trigger event-driven visibility updates immediately on F11 toggle without continuous cursor/window polling. Added JsonStringEnumConverter and test coverage (43 tests passing).
- 2026-09-30: User confirmed full interactive acceptance on the live Release package: 200 ms hover expansion, 300 ms collapse, focus preservation on direct rail interaction, settings persistence across restarts, monitor pinning, and real-time in-place F11 fullscreen hiding/restoration without polling. All 8 acceptance criteria met.