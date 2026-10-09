# 02: Qualify compact Manage Items controls for release

**What to build:** Produce installed-Release evidence that the completed Drop Shelf and Edge Rail Manage Items surfaces satisfy the approved compact width, tail-preserving path, header, Close, accessibility, keyboard, dismissal, lifecycle, theme, scrolling, and performance contracts. Update user-facing documentation and retain only reproducible qualification evidence.

**Blocked by:** Inherited Native Residency release gate. Ticket 01 is implemented and verified.

**Status:** needs-info

**Parent specification:** [Compact Manage Items Controls Specification](../spec.md)

- [x] A clean self-contained `win-x64` Release publish succeeds without new warnings or errors, the NSIS installer is produced and silently installed under the normal per-user installation directory, and the installed executable SHA-256 matches the publish output.
- [x] The exact executable and application payload fingerprints are recorded for every qualification artifact, and the installed application smoke-launches successfully.
- [x] The complete existing MSTest suite passes without weakening lifecycle, persistence, sizing, placement, drag, visual-resource, settings, or native-interop coverage.
- [ ] One isolated test profile contains a representative mixed-lifecycle Shelf Batch with a short local path, deeply nested long file path, deeply nested folder path, long item name, unavailable or UNC-style Path Reference, and enough Shelf Items to require scrolling.
- [ ] The same fixture is exercised through Manage Items on the Drop Shelf and Left and Right Edge Rails from the same installed payload and profile.
- [x] UI Automation bounds and the exercised DPI prove that each Manage Items surface is 280 logical pixels wide within normal physical-pixel rounding tolerance and that path length does not change the width.
- [ ] Visual and automation evidence proves short paths remain complete, long files display `…\parent\file.ext`, long folders display `…\parent\folder`, pathological tails remain bounded, and horizontal scrolling is unavailable.
- [ ] Tooltip and accessibility evidence proves the full canonical Path Reference remains available when visible text is compacted.
- [x] Header evidence proves the order `[Bulk Pinning] [X/N pinned] [Close]` and verifies all-Temporary, mixed, and all-Pinned counts on both surfaces.
- [x] UI Automation verifies Bulk Pinning Off/On/Indeterminate state, its count-aware accessible action name, and a Close button named `Close Manage Items` with the established compact interaction target.
- [x] Keyboard evidence proves initial Bulk Pinning focus for keyboard opening; forward and reverse traversal through Close and every virtualized row action; correct wrapping; `Enter` and `Space` activation; and no focus trap.
- [x] Dismissal evidence independently exercises Close, `Escape`, outside click or native light dismissal, and Manage Items toggle-close, with originating-control focus restoration whenever the owner still exists.
- [ ] Guard evidence proves Close cannot bypass active-drag or pending-mutation protection and does not leave stale projections, stale owner-open state, or premature Edge Rail collapse.
- [ ] Edge Rail evidence proves the Flyout holds the rail expanded, opens toward the desktop on both edges, preserves foreground focus under the no-activate policy, and resumes normal collapse timing after closure.
- [ ] Drop Shelf evidence proves the Popup remains anchored, one-at-a-time, bounded in height, vertically scrollable, and independent of Drop Shelf window size.
- [ ] Lifecycle evidence proves individual Pin/Unpin, Bulk Pinning, Remove Item, accepted and canceled drag-out, Temporary consumption, Pinned retention, Missing cleanup, Unavailable retention, and source-filesystem safety remain unchanged.
- [ ] Light- and Dark-mode evidence comes from the same payload. High Contrast and non-default text scaling are exercised only in an isolated or safely reversible environment; unavailable scenarios are recorded as unobserved rather than inferred.
- [x] Long-list evidence proves the header remains fixed, scrolling reaches the final row and returns, row virtualization remains coherent, and closing releases deferred Shelf Item projections.
- [x] Screenshots and captured bounds remain human-review evidence only; no screenshot-baseline framework, permanent UI automation project, or source-text presentation test is added.
- [ ] The established 100 Shelf Batch / 1,000 Shelf Item performance protocol remains responsive and records idle CPU, shelf-show latency p95, visible working set, and post-dismissal working set without lowering or waiving existing gates.
- [x] UI Automation and performance measurement run separately against the same payload identity so automation-peer realization cannot contaminate residency evidence.
- [x] User-facing documentation describes the 280-logical-pixel Manage Items width, tail-preserving Path Reference display, left-aligned Bulk Pinning summary, explicit Close action, and preserved dismissal and lifecycle behavior.
- [x] Qualification records the Windows build, theme, contrast and text-scaling state, DPI, Edge Rail side, isolated profile and fixture identity, observed outcomes, failures, environmental limits, and every unobserved configuration without inferring a pass.
- [x] Disposable profiles, temporary fixtures, helper processes, screenshots not retained as evidence, and temporary qualification scripts are removed or shut down; no test scaffold or generated artifact remains in production code.
- [x] All repository changes remain uncommitted until the user explicitly authorizes a Git commit.

## Testing seam

Use the installed self-contained Release with an isolated profile as the single authoritative seam for rendered WinUI geometry, path presentation, UI Automation, keyboard and pointer behavior, focus restoration, Edge Rail no-activate behavior, themes, scrolling, lifecycle, and resource cleanup. Use the existing MSTest suite only as regression coverage. Run performance measurement separately from UI Automation against the same payload fingerprint.

## Demo path

Launch the installed Release with the representative fixture. Open Manage Items on the Drop Shelf and both Edge Rail sides, show stable 280-logical-pixel bounds, every path presentation case, `[pin] X/N pinned [close]`, pointer and keyboard dismissal, complete focus traversal, guard behavior, scrolling, lifecycle actions, theme evidence, documentation, and recorded payload identity. Finish by demonstrating cleanup and the inherited performance result without modifying live user data.

## Comments

### 2026-10-09 — UI qualified; release qualification remains blocked

- User explicitly chose **Commit UI, keep qualification blocked** after reviewing the inherited residency result. This does not waive either historical memory threshold and does not authorize a release-ready claim.
- Qualified installed application DLL: `6558106D6D1EE1631757BAE91003398308E52AE51577383EEF19D1EE5D363461`. EXE plus application/Core/Services/Native DLL hashes match the self-contained publish. Release build/publish and NSIS installation succeeded; all 221 MSTest cases pass.
- Same-payload 96-DPI UI evidence passes 466 checks across Shelf, Left Rail and Right Rail in Light and Dark. The logical Popup surfaces measure 280 wide; the native PopupHost's transparent padding is recorded separately and is not the surface width.
- Verified cases include complete short paths, compact long file/folder paths, full-path accessibility metadata, header order, Off/Mixed/On pin counts, all 26 logical focus actions in both directions with wrapping and row realization, fixed header, all four dismissal routes, focus return and pending-mutation Close disabling/re-enabling.
- Paired canonical protocol uses 100 Shelf Batches / 1,000 all-Temporary Shelf Items, ten PNG previews and 30 production show/dismiss cycles. Candidate: visible 158.82 MB, post-dismissal 173.15 MB, idle CPU 0.0%, native-message-to-visible p95 30.56 ms. Pre-change payload (`9A547296882EE06B90092A2D8A9DEC53EBEF29D5D99511190E5FE47BDB731B39`): visible 158.41 MB, post-dismissal 173.86 MB, idle CPU 0.0%, p95 16.41 ms.
- Both payloads fail the 150 MB and 160 MB post-dismissal gates under this protocol. The candidate does not exceed the observed pre-change post-dismissal sample; this records an inherited failing gate, not a performance pass or a claim that all environments are regression-free.
- Native-message-to-visible timing does not certify physical-keyboard or first-render-frame latency. High Contrast, non-default text scaling, pathological-name/UNC tooltip cases and actual drag acceptance/cancellation remain uncertified. A Sandbox probe failed in the temporary guest harness before contrast activation; host theme was not modified and the owned Sandbox session was stopped.
- The UI matrix uses logically identical disposable fixtures rebuilt per surface/theme, not one persisted profile across all six runs. Strict same-profile cross-surface qualification remains open with the other unchecked release criteria.
- Temporary scripts/profile fixtures are removed after evidence capture. [Qualification evidence](../qualification.json) retains exact payload identity, exercised checks, environment, cleanup, comparison results and limits.
- **Resume:** resolve the independent Native Residency gate, then finish the remaining unchecked installed-release qualification scenarios. No memory fix is included in this UI feature commit.
