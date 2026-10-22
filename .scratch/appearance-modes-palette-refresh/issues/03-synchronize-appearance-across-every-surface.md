# 03: Synchronize Appearance across every DropCove surface

**Parent specification:** [DropCove 0.0.19 — Appearance modes and palette refresh](../spec.md)

**What to build:** Extend the committed and previewed Appearance Mode from the core path to every DropCove surface and native frame. System follows live Windows Light/Dark changes, explicit Dark and Light remain fixed, Windows Contrast Theme always overrides rendering, and hidden or newly created surfaces appear directly in the current authoritative appearance.

**Blocked by:** 02: Preview and roll back Appearance drafts.

**Status:** resolved

- [x] One Resident-owned resolved appearance is projected to Drop Shelf, collapsed and expanded Edge Rail, Manage Items, Settings, About, native and WebView confirmations, status messages, utility menus, drag feedback, scrollbars, and every owned native frame.
- [x] Individual windows and frontends do not own durable Appearance Mode, recompute preference precedence, or persist browser-local theme state.
- [x] Surfaces already open when preview or committed mode changes update without restart, focus loss, geometry change, or per-window drift.
- [x] Hidden surfaces and auxiliary surfaces created after preview starts resolve the current preview before becoming visible; after rollback they resolve the committed mode.
- [x] System mode observes live Windows application-theme changes and updates every current WebView and native frame without restart.
- [x] Explicit Dark and Light ignore ordinary Windows application-theme changes across every surface.
- [x] Windows Contrast Theme has higher rendering precedence than System, Dark, and Light while leaving the selected and persisted Appearance Mode unchanged.
- [x] Entering Contrast Theme applies Windows system colors and native-frame treatment; leaving it restores the exact selected committed or previewed mode.
- [x] While Contrast Theme is active, the radio group remains operable and exposes the exact associated message: “Windows Contrast Theme currently overrides this appearance.”
- [x] Browser `color-scheme`, form controls, scrollbars, focus, and UA rendering match the resolved Light or Dark appearance without suppressing forced colors.
- [x] Application startup and every later show/create path establish authoritative appearance before a visible stale frame can appear.
- [x] Cancel, native close, failed Save, and incomplete rollback reporting converge every still-live surface and frame on the actual retained state.
- [x] Multi-window regressions cover simultaneous Settings, Drop Shelf, Edge Rail, Manage Items, About, menus, and confirmations through preview, Save, Cancel, OS theme change, Contrast entry/exit, hide/show, and close/reopen.
- [x] Actual running Windows Resident/WebView2 smoke exercises every user-facing surface and real native frames in System, Dark, Light, and an actual Windows Contrast Theme where available; media emulation is not the final acceptance evidence.
- [x] Shelf state, Profile Data, source files, drag/drop, window bounds, placement, fullscreen, topmost, virtualization, reduced motion, and Resident lifecycle remain unchanged.
- [x] No second theme observer, generalized window manager, state library, theme package, visual-regression framework, or screenshot framework is introduced.
- [x] Version metadata remains at 0.0.18 and no Git commit is created without separate explicit authorization.

## Comments

- 2026-10-22: Parent reproduced the pre-fix Rail mismatch: committed Light with no root marker and Dark color-scheme. After fan-out/bootstrap integration, all six WebViews (Drop Shelf, Rail, Settings, About, Manage Items, native Clear confirmation) reported Light, Light color-scheme, and the same revision `10`; all six corresponding native frames reported immersive-dark attribute `0`.
- System preview followed an actual Windows application-theme change across five simultaneously open surfaces and frames. Explicit Dark remained Dark across the next Windows change. Newly created About, Manage Items, and Clear surfaces inherited the current preview; a hidden Drop Shelf later reopened with the authoritative Light mode.
- Actual `SPI_SETHIGHCONTRAST` enabled Windows High Contrast Black: all five open WebViews reported forced-colors active, their surface token was `Canvas`, the selector remained operable, and the exact associated override message was visible. Selecting Dark during Contrast preserved mode `Dark` while Windows retained rendering authority. Leaving Contrast restored Dark.
- Native Rail hover expanded from width `16` to `280`; the real Rail Clear dialog rendered and retained the current semantic appearance. Outside-focus lifecycle behavior remained unchanged. Both native and WebView confirmation paths were exercised without deleting fixture references.
- Captured Windows personalization values, every captured Contrast registry value (including the initially empty High Contrast Scheme), colors, cursor, and startup Run value were restored. Owned Cooperative Shutdown exited `0`. Chromium emitted a class-unregistration `1412` diagnostic during teardown; no DropCove application error was observed in the inspected WebViews.
- Exact installed-candidate, complete interaction-matrix review and human acceptance remain the release ticket. Metadata remains `0.0.18`; no commit was created.
