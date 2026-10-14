# 07: Canonical State-Change Motion

**What to build:** Add only the approved short effects that help users understand presentation changes: Shelf entrance, Shelf Item or Shelf Batch insertion/removal, Edge Rail expansion/collapse, and the existing hotkey-recorder listening pulse. Windows Animation Effects must suppress every nonessential effect while preserving identical state, focus, drag, virtualization, and resource-lifetime outcomes.

**Blocked by:** 01 — System Theme, Focus, and Reduced-Motion Resources; 04 — Accessible Manage Items Virtualization; 05 — Accessible Shelf and Rail Batch Virtualization.

**Status:** resolved

**Parent specification:** [DropCove 0.0.7 — Presentation and Accessibility Conformance](../spec.md)

## Acceptance criteria

- [ ] Shelf entrance, collection insertion/removal, and Edge Rail expansion/collapse use short opacity/scale effects between 120 and 160 milliseconds.
- [ ] No thumbnail cross-fade, decorative loop, layout animation, drag animation, or other unapproved motion is added.
- [ ] Effects do not delay authoritative state transitions, persistence, focus movement, drag completion, dismissal, or resource cancellation.
- [ ] Windows Animation Effects disabled suppresses all nonessential transitions and animations, including the hotkey-recorder pulse.
- [ ] Toggling Windows Animation Effects while a surface is open changes subsequent presentation without restarting DropCove.
- [ ] Enabled and disabled motion paths produce identical final display state, Core Shelf State, focus target, virtual range, Shell Visual ownership, and error behavior.
- [ ] Insertion/removal effects cannot resurrect a virtualized, removed, hidden, collapsed, or stale element.

## Testing seam

Exercise actual presentation transitions through the Resident Instance with Windows Animation Effects enabled and disabled. Observe computed motion and final behavior on the real surfaces; assert final state and ownership rather than implementation-specific CSS class names.

## Demo path

Open the Shelf, add and remove content, expand/collapse the Edge Rail, and use the hotkey recorder with Animation Effects enabled. Disable Animation Effects while DropCove remains open and repeat; all outcomes remain identical without nonessential motion.

## Non-goals

Visual redesign, spring physics, configurable durations, thumbnail cross-fades, native window choreography, or animation telemetry.

## Answer

Delivered in 0.0.7:
- 140ms opacity and transform state-change transitions on `.shelf`, `.management`, `.rail-surface`, `.card`, and `.item-row`.
- Global `@media (prefers-reduced-motion: reduce)` overrides transition-duration and animation-duration to 0.01ms.
- Disables hotkey recorder listening pulse when reduced motion is requested.
