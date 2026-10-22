# 04: Apply the approved semantic palette

**Parent specification:** [DropCove 0.0.19 — Appearance modes and palette refresh](../spec.md)

**What to build:** Replace the accepted navy/indigo presentation with the approved neutral Dark and Light palette on every synchronized DropCove surface. Preserve one semantic-token source, introduce a dedicated Shelf Batch card role, coordinate controls, borders, focus, icons, feedback, shadows, drag states, and scrollbars, and retain Windows system colors in Contrast Themes without changing any workflow or geometry.

**Blocked by:** 03: Synchronize Appearance across every DropCove surface.

**Status:** resolved

- [x] Dark application backgrounds render as `#000000`; Dark Shelf Batch cards render as `#101010`.
- [x] Light application backgrounds render as `#F3F3F3`; Light Shelf Batch cards render as `#F8F8F8`.
- [x] Raised controls or menus use Dark `#181818` and Light `#FFFFFF`; hover surfaces use Dark `#232323` and Light `#EAEAEA`.
- [x] Primary text uses Dark `#F5F5F5` and Light `#111111`; secondary text uses Dark `#C7C7C7` and Light `#3F3F3F`; muted text uses Dark `#929292` and Light `#686868`.
- [x] Default borders use Dark `#383838` and Light `#D0D0D0`; interactive borders use Dark `#5A5A5A` and Light `#858585` only for resting/hover boundaries, not as the focus indicator.
- [x] Primary actions use `#2B63EB` with white text; Dark accent text/icons use `#7AA2FF`; Light accent text/icons use `#2B63EB`.
- [x] Focus remains a dedicated visible semantic role meeting the applicable non-text contrast requirement in Dark, Light, and Windows Contrast Theme.
- [x] One semantic-token source remains authoritative and its Tailwind exposure does not duplicate values; component-local palette copies are removed rather than retained as fallbacks.
- [x] Shelf Batch cards use a dedicated semantic role instead of sharing every elevated control/menu color.
- [x] Dark Shelf Batch cards have no decorative shadow; Light cards use only slight one-pixel separation; stronger shadows are reserved for menus, dialogs, and floating surfaces.
- [x] Ordinary icons inherit secondary text, selected/action icons use the applicable cobalt, destructive icons use danger, and warnings retain a distinct contrast-checked amber role.
- [x] Drop Shelf, Edge Rail, Manage Items, Settings, About, confirmations, menus, status/error states, drag placeholders, item rows, empty states, scrollbars, selected/pinned states, and disabled controls use the approved hierarchy.
- [x] Windows Contrast Theme ignores the application palette and continues to use Windows system colors, explicit visible borders, system focus, and readable controls.
- [x] Contractual computed colors and contrast are verified in representative roles; tests do not pin antialiasing, incidental DOM structure, or non-contractual derived pixels.
- [x] Actual Windows Resident/WebView2 inspection covers every surface in Dark, Light, and Contrast, including hover, active, focus, disabled, warning, danger, drag, scrolling, and floating elevation states.
- [x] Reduced motion, keyboard operation, accessible names/states, Shelf geometry, window geometry, virtualization, drag/drop, source-file safety, and all existing workflows remain unchanged.
- [x] No runtime dependency, component framework, visual-regression framework, screenshot framework, theme library, or new icon dependency is added.
- [x] Version metadata remains at 0.0.18 and no Git commit is created without separate explicit authorization.

## Comments

- 2026-10-22: Actual Windows/WebView2 computed styles measured Dark surface/card `rgb(0,0,0)` / `rgb(16,16,16)` and Light `rgb(243,243,243)` / `rgb(248,248,248)` in both Drop Shelf and expanded Rail. Dark card shadow was `none`; Light was a one-pixel `0.035`-alpha separation shadow. Real menu background was `rgb(24,24,24)` with the stronger floating shadow and interactive border.
- Actual Settings Save measured `rgb(43,99,235)` with white text. Native Clear dialog danger action measured `rgb(198,40,40)` with white text in both appearances. Eleven action/text/focus/feedback contrast checks passed; primary action ratio was `5.138:1`, danger action `5.622:1`, Light accent on the background `4.631:1`, and Dark accent `8.440:1`.
- Inspected real `420 × 520` Light and Dark Settings screenshots. Dedicated preview miniatures use the same scoped semantic palettes, while keyboard focus remains in the outer active application palette. Real radio ArrowLeft retained focus and checked state; Cancel restored committed Light.
- Actual Windows Contrast was enabled again and System, Dark, and Light selections each retained Windows rendering authority. The root and both miniature scopes resolved surface tokens to `Canvas`; exact override copy remained visible. Captured native flags, personalization/Contrast registry values, colors, cursor, and Run value were restored.
- Reduced-motion emulation on the actual WebView yielded the existing `1e-05s` suppressed card transition. No new animation, dependency, geometry change, or test framework was introduced. Final installed-candidate review and full interaction acceptance remain Ticket 05.
