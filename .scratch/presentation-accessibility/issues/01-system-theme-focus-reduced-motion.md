# 01: System Theme, Focus, and Reduced-Motion Resources

**What to build:** Make every existing DropCove surface follow Windows Light, Dark, and Contrast Themes while preserving the compact 0.0.6 layout. Users must receive legible semantic colors, visible keyboard focus, a native frame matching the active application theme, and immediate response to live theme changes. Windows reduced-motion preference must suppress the existing hotkey-recorder pulse without changing recorder behavior.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.7 — Presentation and Accessibility Conformance](../spec.md)

## Acceptance criteria

- [ ] Drop Shelf, Manage Items, Settings, collapsed Edge Rail, and expanded Edge Rail are legible in Windows Light and Dark themes.
- [ ] Changing the Windows application theme while a surface is open updates its web content and native frame without restarting DropCove.
- [ ] Windows Contrast Themes use system colors for canvas, text, buttons, borders, selection, errors, warnings, and focus instead of an application-defined contrast palette.
- [ ] Every actionable control exposes a clearly visible focus indicator in Light, Dark, and Contrast Themes.
- [ ] Windows reduced-motion preference suppresses the hotkey-recorder pulse while recording, cancellation, Save, and focus behavior remain unchanged.
- [ ] No DropCove theme setting, persistent field, compatibility path, new dependency, layout change, window-size change, or wording redesign is introduced.
- [ ] Existing drag, resize, pin, remove, settings, Rail, and Shake-to-open behavior remains unchanged.

## Testing seam

Exercise the running Resident Instance and each real presentation surface. Observe computed theme/focus behavior and native-frame state while changing Windows application theme, Contrast Theme, and Animation Effects. Use UI Automation to confirm focus remains visible and actionable rather than testing stylesheet source text.

## Demo path

Open each surface, move keyboard focus through its controls, switch Windows between Light and Dark, enable a Contrast Theme, then disable Animation Effects while the hotkey recorder is listening. The surfaces update live, retain their geometry, and remain operable.

## Non-goals

Shell Visual extraction, new collection animations, virtualization, visual redesign, and an application-owned theme preference.

## Answer

Delivered in 0.0.7:
- Semantic CSS tokens in `src/style.css` for Dark, Light (`prefers-color-scheme: light`), and Windows Contrast Themes (`forced-colors: active`).
- DWM immersive dark mode live synchronization via native subclass in `src-tauri/src/windows_theme.rs` responding to `WM_SETTINGCHANGE` and `WM_THEMECHANGED`.
- 2px focus-visible indicator on all actionable elements.
- Reduced motion rule zeroing out animation/transition durations under `prefers-reduced-motion: reduce`.
- Verified live in running test instance with CDP media emulation.
