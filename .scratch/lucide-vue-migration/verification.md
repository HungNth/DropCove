# Implementation and verification — 2026-10-15

Current acceptance requires both channel and visible-thumb balance: Drop Shelf 7/7 pixels and expanded Edge Rail 6/6 pixels, with a rounded 4-pixel painted thumb aligned to the content edge of its 8-pixel hit channel. Earlier asymmetric measurements and channel-only checks below remain historical. The latest painted-thumb correction, screenshot-pixel assertions, exercised hash, and cleanup evidence are recorded at the end.

## Earlier implemented scope

- Replaced the deprecated Lucide Vue package with `@lucide/vue` at `^1.57.0`, locked to 1.57.0. The existing role adapter, icon mapping, sizes, and consumers remain intact; no unrelated dependency update.
- Drop Shelf: collection left/right insets 7/10 px; external fixed top boundary 8 px; bottom scroll clearance 8 px. Existing grid, card minimum, gap, and virtualization retained.
- Expanded Edge Rail: left/right insets 6/10 px; external top/bottom collection clearance 8/8 px; row height 48 px and gap 4 px retained. Native target height is `max(280, min(642, 52*n+70))`.
- User-approved runtime amendment: Shelf minimum/default/empty-reset width 191 px, height 180 px unchanged. Validation, restored-width bounding, native resize clamp, window configuration, empty resets, behavioral tests, automatic-growth column budget, and living Product Specification synchronized.
- Historical 0.0.12 specifications, tickets, roadmap evidence, measurements, and artifact hashes were not rewritten. No release version bump, installer work, or Git commit.

## Build and regression evidence

- `pnpm install --frozen-lockfile --strict-peer-dependencies`: passed.
- `pnpm list @lucide/vue lucide-vue-next --depth 0`: only `@lucide/vue@1.57.0` present.
- `pnpm build`: complete Vue SFC typechecking and production Vite bundling passed after both surface edits.
- `cargo xwin build --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc --release --locked`: passed, including the final 191-pixel width amendment.
- Native tests compiled through cargo-xwin and executed directly on Windows. Focused size-validation, restored-width/grid-boundary, and Rail timer/height tests passed.
- Test-first evidence: the updated minimum-width validation test failed before the validation change because 190.9 px was accepted; it passed after the change.
- Final complete native regression executable: **102 passed, 0 failed**. The recovery notification failure message is expected test-path output, not a failed test.
- External SDK LNK4099 missing-PDB linker warnings remain; compilation succeeded.

## Real Windows WebView2 evidence

An owned profile and 65 owned filesystem fixtures were used against the newly built native executable, not a mocked browser application. All fixture sources were checked unchanged before cleanup.

### Drop Shelf

- Baseline measured left/right 7/7 px, top 4 px, bottom 10 px: did not match the approved contract.
- At the old 180-pixel width with a scrollbar, client width was 170 px and scroll width 181 px; a 164-pixel card produced an actual right inset of -1 px. The user approved 191 px rather than reducing card size or gutter.
- Final minimum-width smoke measured window width 191 px, card width 164 px, left/right insets 7/10 px, top/bottom 8/8 px, and identical horizontal client/scroll widths: no horizontal overflow.
- An actual native `WM_SIZING` message requesting width 180 px was clamped to 191 px while height remained 180 px.
- Ordinary and virtualized grids exercised at widths 191, 359, and 527 px with one, two, and three columns. Virtualized card widths stayed at least 164 px; the final row's rightmost card retained the 10-pixel inset.
- A 65-batch collection exercised at start, middle, and maximum scroll. Mounted entries remained bounded (8–30 in the measured cases), the final batch index 64 was reachable, and maximum-scroll bottom clearance was 8 px.
- At an intermediate scroll position, hit-testing inside the fixed top boundary did not hit a Shelf Batch card.
- Tab traversed to virtual batch index 12 and revealed its focused control fully within the viewport; Shift+Tab returned to index 10.

### Expanded Edge Rail

- Actual Right Rail counts 1–4 stayed at 280 px; counts 5–11 used 330, 382, 434, 486, 538, 590, and 642 px with no vertical overflow.
- Eleven batches: window height 642 px, scroll/client height 576/576 px, no scrollbar, final clearance 8 px.
- Twelve batches: window height 642 px, scroll/client height 628/576 px, scrollbar present, final clearance 8 px.
- Right and Left 65-batch Rails: height 642 px, scroll/client height 3384/576 px, maximum scrollTop 2808, final index 64 reachable with 8-pixel bottom clearance, 10-pixel right inset, 48-pixel rows, and no horizontal overflow. Measured mounted rows stayed bounded (13–16).
- Left docking rendered the inward ArrowRight, 24-pixel controls, and no-activate state (`document.hasFocus()` false). Open Shelf and native expansion/collapse were exercised. Source preserves the mirrored ArrowLeft branch for Right docking.
- Short collections stayed top-aligned; unused height below them was not mistaken for a failure of bottom scroll padding.

### Icons, interactions, and themes

- Empty state: 28-by-28-pixel ArrowDown, `aria-hidden=true`, `Drop here`, reset window 191 by 180 px.
- Observed 14-pixel action glyphs, 24-pixel targets, List, CircleMinus, Trash, management Close X, filled Pin, and mixed Pin/Minus with `aria-checked=mixed` retained by the button. The composed Minus remains 10 px, as before.
- Real intake, Pin, Remove Batch, Manage Items, management close, Clear confirmation, pinned-reference retention, native expansion/collapse, and Open Shelf exercised. All 65 owned source files remained unchanged.
- Light and Dark rendered in the real WebView2 through media emulation; screenshots inspected.
- Actual Windows High Contrast was enabled through the native system setting, confirmed by `SPI_GETHIGHCONTRAST` and `matchMedia('(forced-colors: active)')`. Rail screenshot inspected; the Shelf retained its 10-pixel inset under actual forced colors. This observation is distinct from the earlier supporting forced-colors emulation.

## Review

Two independent axes reviewed the scoped implementation: Standards and Spec. The initial Spec review identified the minimum-width conflict; the approved 191-pixel amendment fixes it. Separate Standards and Spec reviews of the amendment found no remaining source defect. The reviewed delta did not include unrelated older uncommitted repository work.

## Exercised executable

- Native release executable: `src-tauri/target/x86_64-pc-windows-msvc/release/dropcove.exe`.
- Size: 13,386,752 bytes.
- SHA-256: `25bf7e67c19a4d96bf0750d67580bf3d86a70b59fc01d51c8bd99afc740b94d6`.
- This is an unbundled implementation smoke artifact, not a newly packaged or accepted release candidate.

## Cleanup and limits

- Owned application instances shut down cooperatively with exit 0. Browser attachments released; the owned temporary tree, profile, settings fixture, and 65 source fixtures were removed. No claim is made about clearing shared WebView2 caches outside that tree.
- Running an isolated profile applied the application's global HKCU Run startup registration. The pre-smoke Run bytes were not captured. Cleanup reconciled the value to the verified installed DropCove executable and existing default profile whose persisted StartWithWindows preference is true. Exact pre-smoke registry-byte restoration is not claimed.
- Actual High Contrast was turned off again and original flags 126 restored. Captured scheme preference was empty; the stored `High Contrast Scheme` value was restored to empty. Windows' SPI cache still reports the normalized scheme name `High Contrast Black`, and its previous-scheme metadata may have been populated by Windows. Exact pre-smoke theme-registry/cache restoration is not claimed. No further theme toggles were attempted after three restoration approaches.
- Physical Explorer/OLE Copy and cancel, exhaustive hover/disabled/tooltip interaction, fractional/high-DPI scaling, fullscreen interactions, and monitor-disconnection combinations were not re-exercised. Existing native regression coverage and unchanged ownership paths do not constitute fresh physical interaction evidence.
- Tickets remain ready-for-human for final interactive acceptance and acknowledgement of the recorded environmental restoration limits. No source implementation is deferred.

## Symmetric gutter correction after visual feedback — 2026-10-15

The earlier fixed-right-inset checks proved the old asymmetric contract, not horizontal visual balance. The user supplied screenshots and explicitly selected equality to the scrollbar channel rather than the painted thumb. The current contract is symmetric collection padding; the thumb retains its 2-pixel transparent border. No custom scrollbar, overflow observer, or virtualization offset is needed.

The same real-WebView2 probe measured card bounds, inner surface borders, client width, scrollbar channel width, and computed padding before and after the correction:

| Surface / candidate | Left clearance | Right clearance to channel | Equality assertion |
| --- | ---: | ---: | --- |
| Drop Shelf before | 7 px | 10 px | FAIL |
| Drop Shelf corrected | 7 px | 7 px | PASS |
| Expanded Edge Rail before | 6 px | 10 px | FAIL |
| Expanded Edge Rail corrected | 6 px | 6 px | PASS |

- Changing only right padding on the diagnostic DOM made the equality probe pass, isolating the cause. The app was then rebuilt from the corrected source and all acceptance measurements repeated with no inline padding override.
- Source correction: symmetric collection padding on both surfaces and the matching native automatic-growth column budget. The permanent behavioral sizing test now checks 347 pixels as one column and 348 pixels as two. Before the native fix it failed with actual height 236 versus expected 180; after the fix it passed.
- `pnpm build`, cargo-xwin release build, focused native sizing regression, and the complete native regression executable passed: **102 passed, 0 failed**. External SDK missing-PDB linker warnings remain.
- Ordinary Drop Shelf collections passed at widths 191, 356, and 524, with both fitting and overflowing states. A 65-batch virtualized collection passed equality at start, middle, and maximum scroll at all three widths, retaining one/two/three columns, no horizontal overflow, bounded mounting, final batch index 64, and 8-pixel bottom clearance.
- Right and Left expanded Rails passed equality for ordinary eleven-batch/no-scroll and twelve-batch/scroll states. Both sides also passed start/middle/end checks with 65 batches, bounded mounting, no horizontal overflow, reachable final index 64, and 8-pixel maximum-scroll bottom clearance. Native Rail height remains unchanged at the approved 642-pixel maximum.
- Narrow Drop Shelf and expanded Rail screenshots were visually inspected. The approved 191-pixel window floor, 164-pixel minimum card width, vertical boundaries, icon mapping, and native Rail height budget were not changed.
- Corrected executable: 13,386,752 bytes; SHA-256 `7c626e4d067a434fdce640dbccdddc678d7074777e22c1c8ae0a66c54125d15c`. This supersedes the earlier unbundled implementation-smoke hash for the current gutter correction; historical release hashes remain untouched.
- This diagnostic run captured HKCU Run before launching, restored its exact value and type afterward, confirmed all 65 owned source files unchanged, closed all diagnostic processes/browser attachments, and removed the owned profile/fixtures/WebView2 tree. No Windows theme change was made during this correction.
- Reopened the corrected executable for user acceptance with the existing acceptance profile and its Shelf data preserved. Startup registration still matched the pre-diagnostic snapshot after that relaunch. No default Shelf profile was used and no commit or installer was produced.


## Painted-thumb alignment after further visual feedback — 2026-10-15

Channel equality alone did not establish visible equality. The centered thumb retained a 2-pixel transparent left border, adding that distance to the card-to-painted-thumb gap. The minimal Drop Shelf reproduction was a 191-by-180 window: two batches without overflow measured 7/7 pixels, while three batches with overflow measured 7/9. The expanded Right Rail measured 6/8 to the visible thumb. These are failing results for the newly approved visible-paint invariant, not a regression in the earlier channel-only assertion.

The user approved a paint-only correction. The two Shelf scrollbar selectors redistribute transparent border width to the right and use elliptical outer radii to preserve rounded inner paint. The 8-pixel hit channel, 4-pixel paint width, collection padding, card bounds, virtual coordinates, native dimensions, grid thresholds, and existing color/state rules remain unchanged. Manage Items and Settings retain their centered thumbs. No overflow observer or production JavaScript was added.

### Real screenshot-pixel proof

The baseline Shelf screenshot painted columns 184–187 at a straight part of the thumb, although the channel began at 182. A diagnostic CSS probe moved the paint to 182–185 without moving the card. The application was then rebuilt and all checks repeated without the probe style:

- Rebuilt Shelf: card right edge 175; painted columns 182–185; painted width 4; visible gap 7, equal to the left gap 7. No injected prototype style was present.
- Rebuilt Right Rail: card right edge 265; painted columns 271–274; painted width 4; visible gap 6, equal to the left gap 6.
- Computed thumb borders are left 0 and right 4; the native channel still measures 8 pixels. Screenshots were visually inspected rather than accepting only computed padding or channel bounds.
- Fitting Shelf content retained symmetric 7-pixel border clearances without a scrollbar. A 65-batch virtualized Shelf passed the visible-gap assertion at start, middle, and maximum scroll at widths 191, 356, and 524; all measured painted widths remained 4 pixels.
- The Right Rail passed visible-gap assertions at start/middle/end with 65 batches and with ordinary twelve-batch overflow. Eleven batches removed the scrollbar and retained symmetric 6-pixel border clearances. This follow-up did not re-exercise Left docking, fractional DPI, or physical scrollbar hover/drag; previous docking geometry evidence is not fresh painted-pixel evidence for those unmeasured conditions.
- Programmatic pointer interaction did not expose hover/active colors through the pseudo-element computed-style probe; no physical hover/drag acceptance is claimed. The existing hover/active color rules were not changed.

### Build and handoff

- `pnpm build` and cargo-xwin release compilation passed. Rust code was unchanged by this paint-only fix; the existing native regression executable was rerun with **102 passed, 0 failed**. Missing external SDK PDB warnings remain non-fatal.
- No permanent CSS source-text or mocked geometry test was introduced. The regression seam was the disposable real WebView2 bound/pixel assertions above; owned screenshots and the injected diagnostic style disappeared with the isolated session/tree.
- Exercised executable: 13,386,752 bytes; SHA-256 `bf7024f32fd673868e4fc8fc631a25184be7bbfe8261e6d38c6cfc2a7f6bd5e2`. Earlier unbundled hashes remain historical.
- Captured HKCU Run value and type before launch and restored them exactly after shutdown; all 65 owned source fixtures remained unchanged. Removed the owned diagnostic profile, screenshots, WebView2 data tree, and browser attachment. No OS theme changes were made.
- Reopened this rebuilt executable with the existing user-acceptance profile and its Shelf data preserved. The process was observed running and HKCU Run still matched the pre-diagnostic snapshot afterward. No default Shelf profile, installer, or Git commit was used.

