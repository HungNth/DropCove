# 03: Qualify and document the installed Hotkey Recorder

**What to build:** Deliver a fully qualified installed Global Hotkey Recorder experience and update user-facing documentation to match it. Exercise the complete approved interaction matrix on one identified self-contained Release payload, correct any behavior that fails the specification, and leave evidence that the recorder is accessible, durable, conflict-safe, rollback-safe, startup-safe, and honest about Windows-reserved shortcuts.

**Blocked by:** 02: Complete stable-key capture and layout-aware labels.

**Status:** needs-info

**Parent specification:** [Global Hotkey Recorder](../spec.md)

- [x] A clean self-contained Release build and publish completes without new warnings or errors, and the exact executable and application payload identity is recorded for all qualification evidence.
- [x] The complete existing automated suite passes and remains regression authority for settings serialization, native window behavior, Drop Shelf lifecycle, persistence, shake input, and established modules.
- [x] All feature qualification uses the same installed Release payload with an isolated profile; evidence from source inspection, another build, direct state assignment, private-handler invocation, or settings-file editing cannot substitute.
- [x] UI Automation verifies the recorder's accessible name, current shortcut value, help text, enabled and focusable state, visible focus, listening state, validation announcement, and Save enabled/disabled state.
- [x] Pointer qualification proves recorder activation, replacement capture, focus-loss cancellation, full-window Cancel, window-close discard, successful Save, and reopening with the persisted value.
- [x] Keyboard qualification proves Tab focuses without listening; Enter and Space activate; the activation input is excluded; Escape cancels only capture; bare Tab cancels through focus loss; and modifier-plus-Tab, modifier-plus-Enter, and modifier-plus-Space are recorded without unintended navigation or button activation.
- [x] Capture qualification proves live modifier display, modifier-only continuation, modifier-release updates, first-primary-key completion, fixed display order, unsupported-input recovery, no nested session, no timeout over a representative wait, and no focus trap.
- [x] The installed supported-key matrix includes a letter, top-row digit, Space, Enter, Tab, Backspace or Delete, navigation key, function key above F12, numpad digit, numpad arithmetic key, and punctuation/OEM key.
- [x] Punctuation qualification records the active Windows keyboard layout, shows a friendly current-layout label, saves through Settings, and proves the settings JSON contains the stable numeric virtual-key value rather than the rendered symbol.
- [x] Left/right modifier variants produce the same aggregate display and persisted flags where the environment permits. AltGr is exercised on a suitable layout when safely available or explicitly recorded as unobserved.
- [x] Active-hotkey qualification records the currently saved shortcut while listening and proves that the recorder receives the chord while the Drop Shelf does not toggle, including the capture-completion input event.
- [x] Draft qualification records a different shortcut without saving and proves the previously saved shortcut resumes toggling the Drop Shelf while the draft shortcut remains inactive.
- [x] Successful-Save qualification proves the draft becomes the active registration only after Save, persists to the isolated profile, closes Settings, toggles the Drop Shelf, and restores after a clean restart.
- [x] Real registration-rejection qualification reserves a safe test combination or otherwise produces a genuine native rejection and proves the accurate unavailable/reserved warning, retained draft, open Settings window, unchanged durable settings, and continued operation of the previous shortcut.
- [x] Deterministic isolated-profile persistence failure proves that a newly registered shortcut is rolled back, the previous shortcut resumes, durable settings remain unchanged, the failure remains visible, and unrelated settings rollback behavior is preserved without a product test hook.
- [x] Startup qualification proves a persisted usable shortcut registers after restart. A separate startup conflict proves DropCove remains resident, tray access remains available, and the warning directs the user to Settings.
- [x] At least one non-reserved Win combination reaches the focused recorder, saves, persists, restores, and activates through the existing registration path without a low-level keyboard hook.
- [x] A safe, reversible Windows-reserved combination records whether Windows consumes the input, whether Settings loses focus, and whether capture restores the prior draft. Qualification makes no claim that all Win or shell-owned combinations are capturable.
- [x] If no suitable non-reserved Win combination reaches the recorder in the supported environment, this ticket remains incomplete; it does not add an unapproved keyboard hook or silently weaken the requirement.
- [x] Light and Dark evidence shows idle, focused, listening, draft, invalid, disabled-Save, and registration-error states remain readable and distinguishable without color-only meaning.
- [x] High Contrast is exercised only in an isolated or proven reversible environment. If safe execution is unavailable, it is recorded as unobserved rather than inferred.
- [x] Supported text-scaling evidence shows the shortcut value, listening guidance, and inline error remain readable and do not obscure Save or Cancel.
- [ ] Qualification observes no new idle timer, polling, permanent hook, sustained background work, or regression against inherited release performance gates. It does not lower, replace, or waive those gates.
- [x] User-facing documentation explains press-to-record activation, explicit Save behavior, the supported key categories, Escape and focus-loss cancellation, conflict recovery, current-layout punctuation labels, and the best-effort limitation for Windows-reserved combinations.
- [x] Documentation continues using the canonical Drop Shelf terminology and does not introduce recorder implementation details into the domain glossary.
- [x] No new UI testing framework, screenshot-baseline framework, telemetry, runtime dependency, product test hook, compatibility path, settings migration, or Git commit is introduced.
- [x] Qualification records exact payload identity, Windows build, active keyboard layout, theme, contrast state, DPI, isolated profile, tested combinations, observed outcomes, unobserved configurations, and cleanup.
- [x] Reserved hotkeys, helper processes, operating-system UI opened during safe probes, isolated profiles, and temporary files are cleaned up without changing normal user settings or host configuration.
- [ ] This ticket is not complete with a known specification failure. Any failure is corrected within the approved scope and the affected installed scenarios are rerun on a newly identified Release payload.

## Testing seam

Use the installed self-contained Release with one isolated profile as the single authoritative feature seam. Drive it through UI Automation and actual keyboard input, observe the active global registration and settings persistence, and retain the complete existing automated suite only as regression coverage. Screenshots may support human review but are not permanent pixel baselines.

## Demo path

Using the qualified Release payload, open Settings and demonstrate pointer and keyboard activation, activation-key exclusion, live recording, invalid guidance, Escape and focus-loss cancellation, special Tab/Enter/Space capture, representative stable keys, current-layout punctuation, and Win best-effort behavior. Show the saved hotkey suppressed only during listening, the old hotkey active while a draft waits, successful Save and restart restoration, real conflict retention, deterministic persistence rollback, and startup conflict survival. Finish by showing the updated user documentation and the recorded qualification evidence and cleanup.

## Comments

- 2026-10-08: **Functional/UI qualification complete; release qualification remains blocked.** The final application payload SHA-256 is `223BF7EB587F18B99B9DC4C42DD5CBEB7EF40764F55BEDA64A7B72E24D859DBB`. The [functional matrix](../../../artifacts/hotkey-recorder-qualification.json) passes 25 scenarios, including all key categories, all left/right modifier groups, real registration rejection, persistence rollback, restart, startup conflict tray Settings recovery, and native modifierless-input rejection while listening. The same real legacy registration still toggles the shelf after recording is canceled. No product hook or settings-load policy change is introduced.
- [Visual/UIA evidence](../../../artifacts/hotkey-recorder-visual.json) records the same final payload in Light, Dark, and actual 150% Windows text scaling, including full registration-error text. The expanded InfoBar is measured before bringing it into view. Theme and text-scale values are restored exactly. High Contrast, non-US AltGr/layouts, non-100% DPI, and actual tray balloon rendering remain unobserved; no pass is inferred for them.
- Full automated suite: 226 passed, zero failed/skipped, 659 ms. Separate Standards and Spec reviews found zero actionable source findings; they explicitly withhold release-gate sign-off.
- The earlier recorder payload `8D1D96A6BFFF20838A53BFD8D713CF4A9D8C5563B8D9357BACCBB0A86CB3EB13` [valid 100-batch/1,000-item, 30-cycle stress run](../../../artifacts/hotkey-recorder-performance.json) recorded peak sampled working set `170.3515625 MB`, initial post-summon CPU sample average `0.10358237918217585%` per logical processor, and show p95 `26.2487 ms`. Working-set and CPU gate fields are false; the CPU window includes post-summon work and is not an idle pass.
- The [original published baseline](../../../artifacts/hotkey-recorder-baseline-performance.json), payload `B5D7A57FFCCCCA772BBAF51EE7B9F93C561D77AD82DD32320F438F157BA9D65C`, also fails that identical stress protocol: peak sampled working set `169.34765625 MB`, CPU sample average `0.1293709757058052%` per logical processor, show p95 `25.4524 ms`. This establishes a pre-existing failing gate under this protocol; it does not prove a root cause or waive a possible regression.
- The [later-payload stress attempt](../../../artifacts/hotkey-recorder-final-performance.json) was interrupted by loss of owned foreground before the first cycle. Its empty latency/working-set-cycle arrays are **not** accepted as completed performance evidence. Final-source full stress qualification is therefore unverified, in addition to the recorded prior failing gates. No threshold, metric, fixture size, forced collection, or working-set trimming is changed.
- Status `needs-info` awaits a planning decision: retain this release qualification blocker while accepting the completed UI implementation, or approve a separate performance investigation. No unrelated memory/CPU implementation work is authorized by this UI ticket. Temporary qualification scripts and profiles are removed; the reusable installed behavior regression remains. No Git commit has been created.
- 2026-10-08: User explicitly selected **Chốt UI và cho phép commit**. The approved delivery scope is completed Tickets 01/02 plus functional/UI qualification and documentation from Ticket 03. Ticket 03 remains `needs-info`; release performance gates are not waived, lowered, or declared passed. The user separately authorizes a Git commit for source, regression script, docs, specification, and tickets on the current branch. A final isolated preview is intentionally left running for inspection; the normal resident executable/profile remains untouched.
- 2026-10-08: **Diagnostic investigation of inherited release performance gate completed** (see [Performance Diagnosis](../performance-diagnosis.md)):
  - **0-Cycle baseline (100×1,000 items, 10 images)**: Fresh visible working set measures **153.68 MB**, strictly passing the `< 160 MB` gate at rest.
  - **30-Cycle canonical stress (100×1,000 items, 10 images)**: Peak working set measures **167.27 MB** (accumulation: **+13.71 MB**). Idle CPU average: **0.0515%** (pass <= 0.1%). Show latency: **5.72 ms** (pass <= 150 ms).
  - **1×10 control run (1 batch, 10 items, 1 image)**: Fresh working set measures **147.00 MB**; 30-cycle peak measures **160.52 MB** (accumulation: **+13.52 MB**).
  - **Attribution Conclusion**: The failure is proven to be **lifecycle churn** (+13.5–13.7 MB accumulation across 30 show/dismiss transitions, independent of batch scale), rather than static collection footprint.
  - Per ticket boundary and engineering principles, no production code or residency fix was modified under this hotkey ticket. Ticket 03 remains `needs-info` until a dedicated Native Residency recovery ticket is authorized.

