# 03: Migrate Settings and Confirmation Surfaces to Tailwind

**What to build:** Migrate Settings, Edge Rail placement fields, Shake-to-open settings, Batch Drag confirmation, and Clear Temporary Items confirmation to Tailwind utilities while preserving every accepted form-control state, status presentation, animation, reduced-motion behavior, backdrop, dialog bound, focus path, and theme treatment.

**Blocked by:** 01 — Establish Tailwind Theme Foundation and Shared Action Controls.

**Status:** resolved

**Parent specification:** [DropCove 0.0.11 — Tailwind Component Styling Migration](../spec.md)

## Acceptance criteria

- [x] Ordinary layout, spacing, dimensions, borders, radii, overflow, typography, form-control, button, status, and interaction presentation on Settings and both confirmation surfaces use Tailwind utilities.
- [x] Superseded component CSS is removed as each surface migrates; no compatibility class, dormant fallback, or parallel handwritten presentation path remains.
- [x] Settings surface geometry, section spacing, headings, labels, descriptions, checkboxes, selects, hotkey recorder, status banner, Save, Cancel, and listening state match the captured baseline.
- [x] Edge Rail placement controls preserve monitor/edge field geometry, labels, enabled/disabled states, focus, Save/Cancel behavior, and placement rollback semantics.
- [x] Shake-to-open controls preserve sensitivity values, recorder prompt, listening pulse, success/error presentation, disabled behavior, and registration semantics.
- [x] The listening animation retains its accepted timing and appearance, and the existing reduced-motion contract disables it without introducing a second motion policy.
- [x] Batch Drag confirmation preserves dialog bounds, backdrop, text hierarchy, choices, primary/secondary actions, keyboard behavior, Escape, focus, drag outcome, and source-file safety.
- [x] Clear Temporary Items confirmation preserves danger border and action presentation, backdrop, content, cancellation, failure retention, mixed Temporary/Pinned behavior, and focus.
- [x] Required `dialog::backdrop`, keyframe, Windows forced-color, focus, and reduced-motion rules remain only within the approved narrow CSS exceptions.
- [x] Conditional utility classes are literal and production-discoverable; no runtime utility-name construction is introduced.
- [x] Dark, Light, and actual Windows Contrast Theme presentation preserve accepted semantic colors, system colors, focus indicators, form controls, warnings, danger states, and dialog legibility.
- [x] Pointer and keyboard checks cover hover, active, focus-visible, disabled, checked, selected, listening, warning, danger, confirm, cancel, and Escape states where applicable.
- [x] Before/after bounding-box and computed-style evidence shows no unexplained difference for the migrated surfaces on identical fixtures.
- [x] Settings persistence, hotkey registration, startup behavior, Shake-to-open behavior, Edge Rail placement ownership, warning behavior, and confirmation commands remain unchanged.
- [x] No new component wrapper, styling prop, form abstraction, animation subsystem, visual-regression dependency, or frontend component-test framework is introduced.
- [x] Compact scrollbar styling, persistent scroll-viewport insets, Lucide icon replacement, and every other 0.0.12 change remain absent.
- [x] Package, crate, Tauri, PE, installer registration, and output filename remain `0.0.10`.

## Testing seam

Use the actual Settings and confirmation windows in the running Windows Resident Instance. Compare them against the ticket 01 baseline with the same machine, DPI, dimensions, themes, and states. Exercise real field changes, Save/Cancel, rollback, hotkey listening, reduced motion, confirmation/cancellation, errors, focus, and actual Windows Contrast Theme. Run the existing frontend production build and relevant native regression paths; do not test CSS source text or utility class names.

## Demo path

Open Settings in Dark, Light, and actual Windows Contrast Theme. Exercise every field, disabled condition, the hotkey recorder, Shake-to-open listening with and without reduced motion, status, Save, and Cancel. Open both confirmation dialogs, use pointer and keyboard to cancel and confirm, and show matching backdrop, focus, danger, geometry, and before/after computed evidence.

## Non-goals

Shelf or Edge Rail surface migration; new Settings fields; changed persistence or registration behavior; confirmation copy redesign; animation redesign; scrollbar styling; persistent viewport inset; Lucide icons; TypeScript 7; version synchronization; packaging; or Git commit creation.

## Answer

Delivered in 0.0.11:
- Migrated SettingsView, RailPlacementFields, ShakeSettingsFields, BatchDragConfirmation, and ClearTemporaryConfirmation to direct Tailwind utilities.
- Retained only scoped dialog backdrop rules and keyframes (`dc-recorder-pulse`).
- 35 states and 2,292 elements verified against 0.0.10 baseline with 0 computed-style/geometry differences.
- User confirmed human acceptance across all interactive on-screen gates: Settings fields, hotkey recorder, Shake-to-open, and confirmation dialogs.
- Package, crate, and Tauri metadata maintained at 0.0.10 until authorized release cutover to 0.0.11.
- No Git commit created.
