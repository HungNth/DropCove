# 02: About Surface from the notification area

**What to build:** Add one reusable About Surface and expose it from the Windows notification-area menu. A user can open About while the Drop Shelf is visible or hidden, inspect the actual running product identity and Source, and close or reopen the same singleton surface without changing Shelf state or Resident Instance lifetime.

**Blocked by:** None. User confirmed final 0.0.14 human acceptance on 2026-10-16.

**Status:** resolved

**Implementation gate:** Granted by user on 2026-10-16 after accepting 0.0.14. Git commit authorization remains separate.

**Parent specification:** [Drop Shelf Utility Menu and About Surface](../spec.md)

**Testing seam:** Actual Windows Resident Instance with the real notification-area menu, native About Surface, WebView2 content, runtime metadata, opener integration, and multi-monitor placement, supported by the frontend build and native regression suite.

**Demo path:** Start an owned Resident with the Drop Shelf hidden. Open the notification-area menu, select About, inspect logo/name/version/Author/Source/update placeholder, open Source in the default browser, invoke About again to focus the existing surface, close it, then reopen it with the pointer on another monitor.

- [x] The Windows notification-area menu order is exactly `Settings`, `About`, `Exit DropCove`; existing left-click/double-click Show Shelf behavior and Exit DropCove semantics remain unchanged.
- [x] Notification-area About uses one resident-owned opener that can also be reused by the later Drop Shelf Utility Menu; the tray callback does not independently construct or own a window.
- [x] A newly created About Surface is a decorated, fixed `400 × 460` logical-pixel, taskbar-visible, non-modal native window titled `About DropCove` and centered in the pointer monitor work area.
- [x] If About already exists, another request shows, raises, and focuses that window without creating a duplicate or moving the user-positioned surface.
- [x] `X` and `Escape` close and destroy About; a later request recreates and recenters it.
- [x] Opening About while the Drop Shelf is hidden, visible, or EdgeDocked does not show, activate, move, dismiss, or otherwise change any Shelf surface or Shelf state.
- [x] The About Surface uses the canonical bundled DropCove logo as its sole visual emphasis and displays only product name, runtime version, Author, Source, disabled `Check for updates`, and `Coming soon`.
- [x] One read-only resident metadata interface supplies display-ready name, runtime version, author, and source URL from authoritative build metadata. The frontend does not read manifests directly or repeat those literals.
- [x] Cargo package authors remains the canonical `DropCove Contributors` value; Cargo package repository becomes the canonical GitHub Source value.
- [x] Cargo and Tauri homepage remain unset. The repository is labeled `Source`, never `Website`, and no empty or invented Website row appears.
- [x] About displays the actual synchronized runtime version: `0.0.14` before the release bump and `0.0.15` only in a synchronized 0.0.15 build. No milestone version is hard-coded in the frontend.
- [x] Source is keyboard- and pointer-operable, displays the canonical repository location, and opens it through the existing default-browser opener integration.
- [x] An opener failure leaves About open and reports a concise non-destructive error without changing Shelf or resident state.
- [x] `Check for updates` is visibly and programmatically disabled, is associated with `Coming soon`, has no click handler or fake result, performs no network activity, and adds no updater dependency, configuration, capability, endpoint, signing, or background work.
- [x] The About label and capabilities grant only the window, metadata, and opener behavior required by this surface; About cannot mutate Shelf state, start window dragging, access arbitrary files, or invoke unrelated resident operations.
- [x] The accepted compact design is preserved: Segoe UI, existing semantic tokens, centered product identity, restrained metadata rows, one quiet bottom action area, no gradients, decorative card stack, generic SaaS chrome, or unnecessary motion.
- [x] Dark, Light, Windows Contrast, reduced-motion, visible focus, reading order, window title, link semantics, disabled state, and screen-reader output remain usable.
- [x] Supporting permanent regression coverage catches duplicate-window, close/recreate, wrong runtime metadata, missing/incorrect Cargo Source, unauthorized capability, and resident-lifecycle regressions without mock-echo or manifest-source-text assertions.
- [x] The frontend production build, cargo-xwin Windows build, complete produced native test executable, and actual notification-area/About demo path pass. Canonical product and architecture documentation records About, Source metadata ownership, and the notification-area command.
- [x] No Git commit is created without separate explicit user authorization.

## Answer — implementation and exercised evidence, 2026-10-16

- Actual native tray context menu exposed Settings, About, Exit DropCove in that order. Tray About reused the exact native About handle opened from the Drop Shelf. Tray invocation while Drop Shelf was hidden left it hidden.
- About measured a 400×460 logical client area and 64×64 logo with no overflow. Running metadata showed DropCove, 0.0.14 before synchronization, then 0.0.15 in the exact installed candidate; author and Source matched Cargo metadata. Homepage remained absent.
- Actual Source activation opened the canonical GitHub repository in default Chrome, including keyboard activation from the installed About view; only the smoke-created repository tabs were closed. The update action was programmatically disabled with Coming soon.
- Close/Escape destroyed About and later invocation recreated it. Initial placement on both 1920×1080 monitors was measured at scale 1; refocusing a manually positioned About preserved its rectangle.
- Dark and Light screenshots were inspected; actual WebView forced-colors and reduced-motion emulation reported active and rendered correctly. Physical Windows Contrast Theme, physical high/mixed-DPI, screen-reader output, and browser-launch failure injection remain unobserved.
- About rejected unrelated Shelf and resident commands. Native suite passed 111/111; frontend/cargo-xwin production builds and exact installed About smoke passed. Canonical behavior and architecture docs were updated. No Git commit was created.
