# 01: Render Stacked Multi-Item Shelf Batch Previews

**What to build:** Replace the current horizontal overlap for multi-item Shelf Batch previews with the approved compact diagonal stack on both the Drop Shelf and expanded Edge Rail. The first Shelf Item remains the front layer at the lower-left; up to two later Shelf Items appear behind it toward the upper-right. Preserve every established batch interaction, count, action, accessibility contract, and card dimension.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [Stacked Shelf Batch Preview Refinement](../spec.md)

## Acceptance criteria

- [x] Every multi-item Shelf Batch uses the same stacked-preview geometry on the Drop Shelf and expanded Edge Rail.
- [x] Preview selection remains limited to the first three Shelf Items in source order.
- [x] Each preview layer uses a `24 × 24` logical-pixel Shell Visual box.
- [x] The first Shelf Item is the front layer at the lower-left; the second is three logical pixels right and up behind it; the third is another three logical pixels right and up behind the second.
- [x] Explicit layer ownership prevents later source items from painting over the approved first-item front layer.
- [x] Each Shell Visual preserves natural transparency without an opaque square backing; later layers or the card surface remain visible through transparent artwork and aspect-fit margins.
- [x] A two-item Shelf Batch shows two layers, a three-item batch shows three layers, and a larger batch still shows only three layers.
- [x] The three-layer preview remains bounded to approximately `30 × 30` logical pixels inside the existing 48-logical-pixel card without clipping the title or trailing actions.
- [x] Real Shell Visuals, fallback visuals, aspect-ratio behavior, and asynchronous visual replacement retain their assigned source order and layer position.
- [x] The existing `N items` title remains the sole total-count indicator; no `+N` badge, filename label, extra layer, border, shadow, hover expansion, or animation is added.
- [x] The preview remains decorative and non-interactive. Pointer drag beginning over the preview continues to use the established whole-batch drag behavior and movement threshold.
- [x] Bulk Pinning, Manage Items, and Remove Batch retain their existing hit areas, busy/disabled behavior, tooltips, accessible names/states, and pointer activation.
- [x] Keyboard order, Enter/Space activation, focus indicators, accessible batch drag instructions, and screen-reader presentation remain unchanged; preview layers add no focus target or announcement.
- [x] Single-item Shelf Batch presentation and direct item drag behavior remain unchanged.
- [x] Card height, collection sizing, virtualization measurements, ordering, scrolling, Drop Shelf minimum-width behavior, and expanded Edge Rail layout remain unchanged.
- [x] Core Shelf State, persistence, schema, IPC, settings, native-window behavior, Windows Shell extraction, dependencies, glossary, ADRs, roadmap, release metadata, and installer behavior remain unchanged.
- [x] The canonical multi-item Shelf Batch presentation contract documents the approved front-layer order, three-pixel diagonal offset, bounded geometry, and preserved interactions.
- [x] The existing frontend typecheck and production build complete successfully.
- [x] Actual Windows Resident/WebView2 verification proves the geometry and preserved behavior on both presentation surfaces for two-item, three-item, and more-than-three-item Shelf Batches.
- [x] Dark, Light, and actual Windows Contrast presentation are inspected where available; any unavailable environment check is reported as unobserved rather than accepted.

## Testing seam

Use the user-approved running Windows Resident Instance with the actual WebView2 Drop Shelf and expanded Edge Rail. Observe user-visible geometry, layer order, asynchronous Shell Visual replacement, pointer drag, actions, keyboard behavior, accessibility, and narrow layouts. Use the existing frontend typecheck/production build as supporting verification only. Do not add a frontend component-test framework or source-text/CSS-class assertions.

## Demo path

1. Create visually distinct two-item, three-item, and more-than-three-item Shelf Batches containing a mix of files, folders, thumbnails, and fallback visuals.
2. Open the Drop Shelf and verify the approved front-first diagonal stack, bounded footprint, item counts, actions, and whole-batch drag from the preview area.
3. Dismiss to and expand the Edge Rail; verify the same Shelf Batches use identical stack geometry and preserved interactions.
4. Allow asynchronous Shell Visuals to replace fallbacks and verify that source order and layer ownership do not change.
5. Exercise Bulk Pinning, Manage Items, Remove Batch, keyboard traversal, focus, tooltips, and accessible presentation at narrow supported widths.
6. Inspect Dark, Light, and available Windows Contrast presentation, then run the existing frontend build contract.

## Non-goals

Single-item redesign, more than three previews, additional count indicators, item-level preview actions, preview expansion or animation, new test infrastructure, state or API changes, window or virtualization redesign, release packaging, version assignment, or Git commit creation.

## Answer

- Delivered the shared diagonal stack with a fixed 30-pixel preview region, 24-pixel layers, explicit first-item-front ordering, and three-pixel upper-right offsets. The user-approved correction below removes the initial opaque backing so natural transparency is preserved. Single-item visuals and every action/event contract remain unchanged.
- Updated the canonical Shelf Batch presentation contract. No roadmap, version metadata, schema, IPC, dependency, ADR, glossary, installer, or Git commit changes were made.
- Frontend typecheck and production bundling passed through `pnpm build`. Cargo-xwin compiled the Windows regression executables; direct Windows execution passed **119/119** library tests and the zero-test binary target. Vendor CRT missing-PDB LNK4099 messages remain nonfatal.
- Initial delivery: real WebView2 smoke loaded the production frontend in an isolated native development host. Populated Drop Shelf (191 pixels wide) and expanded Edge Rail (280 pixels wide) were observed and captured after switching away from an unstable HMR session. Two-, three-, and four-item batches measured 24-pixel layers, a 30-by-30 wrapper, 48-pixel cards, and relative layer positions `(0,6)`, `(3,3)`, `(6,0)` for three layers. Explicit decreasing layer order and the original nontransparent backing were observed; that backing is superseded by the correction below.
- Pointer Bulk Pinning changed actual retained pin state; pointer and Enter opened Manage Items for the expected three-item batch; Remove Batch removed the owned two-item batch without deleting source fixtures. Preview hit testing resolved to the existing batch drag region, with no new focus targets. A preview-origin drag/cancel smoke returned with all participants retained. A 54-batch collection mounted six 48-pixel cards and retained the single-item 32-pixel visual box.
- Dark and Light rendered surfaces were captured. Actual Windows Contrast Theme, spoken screen-reader output, physical Explorer accepted-Copy, and mixed-DPI hardware were not exercised. Shell extraction returned fallbacks, including for owned PNG fixtures; real thumbnail replacement is **not** claimed as observed. The extraction/composable contracts were unchanged and independently reviewed.
- No valid behavioral RED was recorded: the initial baseline was disrupted by repeated HMR reloads. This is smoke-backed implementation, not a claimed passing TDD red-green cycle. No new test framework or source-text tests were added.
- Independent parallel Standards and Spec reviewers returned no actionable findings. Their review covers implementation conformance, not unobserved physical runtime behavior.
- Owned processes exited cooperatively with code 0, inspection tabs and temporary profile/fixtures were removed, the temporary server was stopped, and the captured HKCU Run value was restored exactly. Ambient Shelf data and the installed application were not used.
- Native development-host rebuild initially failed with Windows access denied because the smoke executable was still running. After Cooperative Shutdown, `cargo xwin build --target x86_64-pc-windows-msvc --no-default-features` passed. The rebuilt host loaded the same production frontend and reproduced the three-layer geometry with owned native-format Windows paths; extraction still used fallbacks. The second owned profile/server/processes were also removed and the original startup value restored.

### Post-resolution correction — natural transparency

- User reported dark square backing around nonsquare artwork and approved fixing it by removing per-layer opaque backing. This supersedes the original full-box occlusion requirement; the product specification and parent specification now explicitly allow later layers/card surface to show through transparent artwork and aspect-fit margins.
- Root cause: the multi-item Shell Visual wrapper applied an opaque semantic card-surface background to the entire 24-by-24 visual box, including empty artwork regions. Removed those background utilities only; offsets, dimensions, z-order, decorative hit routing, image decoding, native extraction, and single-item behavior are unchanged.
- Behavioral RED on an actual Windows WebView2 Drop Shelf: deterministic PNG artwork was inserted into the real rendered Shell Visual roots to isolate compositing from Shell extraction. A transparent area of the front visual should expose the green back layer `(31,234,184)`; before the fix, screenshot pixel `(24,56)` was card backing `(32,42,59)` and the assertion failed. Setting only the wrapper background transparent made the prediction pass.
- After `pnpm build` passed, reloaded the production frontend without the temporary style override. The same screenshot assertion passed with `(31,234,184)` on both Drop Shelf and expanded Edge Rail. Computed wrapper backgrounds were transparent; previews remained 24-by-24 at 3-pixel offsets in a 30-by-30 region, card height remained 48 pixels, and the preview region remained pointer-events-none.
- Scope of proof: real shared-card layout and WebView2 compositing with deterministic fixture artwork, not a claim that native Shell thumbnail extraction or the user's original files were exercised. No permanent source-text test, new framework, native-code change, or Git commit was added. Earlier native-suite results are historical evidence, not a new native-suite run for this CSS-only correction.
- Independent parallel Standards and Spec reviews returned no findings. Actual Windows Contrast Theme and spoken screen-reader output remain unobserved.
- Closed inspection tabs, cooperatively exited the owned host with code 0, stopped the fixture server, removed the owned profile/fixtures/screenshots after WebView released its cache, and restored the captured startup registry value exactly.
