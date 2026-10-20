# 02: Cargo-owned About author and description

**What to build:** Make Cargo package metadata accurately identify DropCove and project that authoritative identity into About Surface. About shows author `HungNth` and the exact package description immediately below the synchronized runtime version without duplicating metadata in the frontend or changing established About lifecycle and Source behavior.

**Parent specification:** [DropCove 0.0.17 — Settings Window and About Metadata Polish](../spec.md)

**Testing seam:** Running About Surface in the actual Windows Resident Instance with real WebView2 and compiled package metadata, supported by the existing read-only About metadata projection and production build checks.

**Demo path:** After the 0.0.16 human gate closes, open About Surface from the Drop Shelf and notification area, verify runtime version, author, description, Source, fixed layout, keyboard close, taskbar behavior, and Dark/Light/Windows Contrast presentation from the compiled application.

**Blocked by:** [DropCove 0.0.16 Ticket 04 — Release DropCove 0.0.16](../../drag-placement-shelf-fit/issues/04-release-dropcove-0.0.16.md), specifically its final human acceptance gate.

**Status:** resolved

- [x] The blocking 0.0.16 final human acceptance gate is closed before source implementation begins.
- [x] Cargo package authors are exactly `HungNth` and the package description is exactly `A resident Windows utility providing a temporary drag-and-drop workspace for file and folder references.`
- [x] Product name and synchronized runtime version remain sourced from the running Tauri package; author, description, and Source repository remain sourced from compiled Cargo package metadata.
- [x] The existing read-only About metadata interface gains one display-ready description value; no second IPC command, manifest reader, or parallel metadata owner is introduced.
- [x] About Surface displays the exact Cargo description immediately below the runtime version in the product-identity reading order.
- [x] The description uses restrained centered wrapping and remains fully readable within the established fixed `400 × 460` logical surface without clipping or a scrollbar.
- [x] The Author row displays `HungNth` through the metadata projection. Source text, default-browser opening, opener failure handling, and the absence of a Website field remain unchanged.
- [x] Product name, runtime version, description, Author, Source, disabled Check for updates, and Coming soon retain a coherent visual hierarchy and logical accessibility reading order.
- [x] About singleton reuse, taskbar presence, invocation-aware placement, close action, `Escape`, non-modal behavior, and notification-area behavior remain unchanged.
- [x] No author or description literal is duplicated in Vue, Tauri configuration, or another package manifest; the frontend only renders metadata returned by the authoritative projection.
- [x] No new capability, dependency, filesystem access, network access, updater behavior, settings field, database field, or migration is added.
- [x] Actual IPC metadata coverage validates the compiled display values through the existing projection seam rather than asserting Cargo source text or command forwarding.
- [x] Actual WebView2 smoke verifies the exact author and description, complete wrapping, absence of clipping/scrolling, focus and keyboard behavior, and unchanged Source/update placeholder behavior.
- [x] Dark, Light, Windows Contrast where available, reduced motion, and available accessibility tooling preserve description legibility, disabled state, focus, and reading order; unavailable physical coverage is recorded without inference.
- [x] Relevant native regressions, frontend production build, and cargo-xwin Windows build pass while all canonical version sources remain at `0.0.16`.
- [x] No `0.0.17` version bump, About redesign, native size change, extra metadata content, updater implementation, or Git commit is included in this ticket.

## Answer — 2026-10-20

- Test-first actual About IPC showed no `description` field and the old author. The implemented candidate returns author `HungNth` and the exact approved description through `get_about_metadata`, while runtime version remains `0.0.16` before the release bump.
- Real embedded WebView2 at `400 × 460` shows version followed by the complete two-line description, Author, Source, and disabled Check for updates with Coming soon. The description occupies y=163–202; main scroll height equals client height 460, so nothing clips or introduces scrolling. Screenshot inspected; Escape closes normally.
- Frontend vue-tsc/Vite production build and cargo-xwin embedded release build passed. Actual IPC/assert-based smoke covers the trivial metadata projection instead of a permanent copy/forwarding-only unit test.
- Exact 0.0.17 installed assets, themes, complete native regressions, and release lifecycle are verified in Ticket 03. No dependency, capability, schema change, version bump, or commit was introduced by this slice.
