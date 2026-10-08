# Global Hotkey Recorder

Status: ready-for-agent

## Problem Statement

DropCove Settings currently requires users to configure the global hotkey by selecting Ctrl, Alt, Shift, and Win independently and then choosing a primary key from a ComboBox. This makes a familiar keyboard action slow to configure, requires several pointer interactions, and forces users to translate the shortcut they want into separate controls.

The current ComboBox also exposes only Space, A–Z, and F1–F12. Common shortcut keys such as digits, navigation keys, editing keys, punctuation, and numpad keys cannot be selected even though the existing global-hotkey model persists a Win32 virtual-key value.

Users need to configure the global hotkey by activating one recorder field, pressing the desired key combination, reviewing the recorded draft, and saving it. The interaction must remain keyboard-accessible, must not accidentally toggle the Drop Shelf while recording, and must preserve the existing registration, persistence, startup, rollback, and conflict behavior.

## Solution

Replace the separate modifier controls and key ComboBox with one button-like global-hotkey recorder in Settings.

The recorder normally displays the current or drafted shortcut in a normalized form such as `Ctrl + Shift + Space`. A pointer click or explicit keyboard activation enters a listening state. Merely moving keyboard focus to the recorder does not start listening. While listening, the recorder displays the modifiers currently held, captures the first supported non-modifier key pressed with at least one modifier, and then returns to its normal draft state. The user must still select Save before the new shortcut is registered or persisted.

The interaction preserves the current `HotKeyDefinition`, application-settings schema, default `Ctrl + Shift + Space` shortcut, Win32 `RegisterHotKey` registration, save transaction, rollback behavior, startup registration, and resident-process behavior. It introduces no low-level keyboard hook, no live registration probe, and no new disabled-hotkey state.

## User Stories

1. As a DropCove user, I want one global-hotkey recorder instead of separate modifier controls and a key list, so that changing the shortcut takes fewer actions.
2. As a DropCove user, I want the recorder to show my saved shortcut when Settings opens, so that I can see the current configuration immediately.
3. As a DropCove user, I want a recorded but unsaved shortcut shown as a draft, so that I can review it before applying it.
4. As a pointer user, I want to click the recorder to start listening, so that configuration feels like pressing the shortcut itself.
5. As a keyboard user, I want to focus the recorder and activate it with Enter or Space, so that pointer input is not required.
6. As a keyboard user, I do not want Tab focus alone to start listening, so that ordinary Settings navigation remains predictable.
7. As a keyboard user, I want the Enter or Space keypress that activates the recorder consumed as an activation command, so that it is not accidentally recorded as the shortcut.
8. As a keyboard user, I want capture to begin only with input after the activation keypress, so that Enter and Space can remain valid primary keys when deliberately pressed during listening.
9. As a user, I want a clear listening indicator and instruction, so that I know the next key combination will replace the draft.
10. As a user, I want held modifiers displayed while I compose a shortcut, so that I can see what DropCove has recognized.
11. As a user, I want a modifier-only press to keep the recorder listening, so that releasing or changing modifiers does not create an incomplete shortcut.
12. As a user, I want the first supported non-modifier key pressed with at least one modifier to complete capture, so that no separate Done action is required.
13. As a user, I want capture to complete on the primary-key press rather than after every key is released, so that feedback is immediate.
14. As a user, I want at least one of Ctrl, Alt, Shift, or Win required, so that a bare key cannot become a disruptive global hotkey.
15. As a user, I want an unsupported or incomplete combination to leave the recorder listening, so that one mistaken keypress does not discard the attempt.
16. As a user, I want an inline explanation when a combination is incomplete or unsupported, so that I know how to correct it.
17. As a screen-reader user, I want the invalid-input message announced when it appears, so that validation is not visual-only.
18. As a user, I want the inline invalid-input message cleared when I try another key, cancel capture, or complete capture, so that stale guidance does not remain.
19. As a user, I want Escape during listening to cancel only the current capture attempt, so that I can recover without closing Settings.
20. As a user, I want canceled capture to restore the draft that existed before listening began, so that an accidental attempt loses no prior work.
21. As a user, I want clicking or tabbing to another Settings control during listening to cancel the capture attempt, so that focus and recorder state cannot diverge.
22. As a user, I want switching to another window during listening to cancel the capture attempt, so that DropCove does not remain invisibly armed.
23. As a user, I do not want a recording timeout, so that I can pause while deciding on a shortcut without losing the draft.
24. As a user, I want Save disabled while the recorder is listening, so that an incomplete shortcut cannot be submitted.
25. As a user, I want activating an already-listening recorder to leave the same capture session active, so that repeated activation cannot create nested or ambiguous sessions.
26. As a user, I want to click the recorder again after completing capture, so that I can replace the draft before saving.
27. As a user, I want letters A–Z available as primary keys, so that common mnemonic shortcuts remain supported.
28. As a user, I want top-row digits 0–9 available as primary keys, so that numbered shortcuts are configurable.
29. As a user, I want Space available as a primary key, so that the default `Ctrl + Shift + Space` remains recordable.
30. As a user, I want Enter available as a primary key, so that modifier-plus-Enter shortcuts are configurable.
31. As a user, I want Tab available as a primary key, so that modifier-plus-Tab shortcuts are configurable when Windows delivers them to DropCove.
32. As a user, I want Backspace and Delete available with a modifier, so that editing-key shortcuts are not mistaken for a clear command.
33. As a user, I want Insert, Home, End, Page Up, Page Down, and the arrow keys available, so that navigation-key shortcuts are configurable.
34. As a user, I want F1–F24 available, so that the recorder is not limited to the first twelve function keys.
35. As a user, I want numpad digits and numpad arithmetic keys available, so that numpad-based workflows are supported.
36. As a user, I want standard punctuation positions available, so that shortcuts using semicolon, equals, comma, minus, period, slash, backtick, brackets, backslash, or quote positions are configurable.
37. As a user, I want Escape reserved for canceling capture, so that I always have a predictable way out of listening mode.
38. As a user, I do not want Caps Lock, Num Lock, Scroll Lock, Print Screen, Pause, media, volume, browser, application-launch, IME, packet/process, mouse-button, or other system-special keys accepted, so that the supported set remains stable and explainable.
39. As a user, I want bare Tab during listening to leave the recorder and cancel capture through normal focus navigation, so that keyboard navigation remains available.
40. As a user, I want modifier-plus-Tab intercepted during listening before focus navigation, so that supported Tab shortcuts can be recorded.
41. As a user, I want modifier-plus-Enter intercepted during listening before default-button behavior, so that supported Enter shortcuts can be recorded.
42. As a user, I want modifier-plus-Space intercepted during listening before button activation, so that supported Space shortcuts can be recorded.
43. As a user, I want Enter or Space without a modifier during listening treated as incomplete input rather than a second activation, so that recorder state remains unambiguous.
44. As a user, I want left and right Ctrl treated as Ctrl, so that physical key side does not change the saved shortcut.
45. As a user, I want left and right Alt treated as Alt, so that physical key side does not change the saved shortcut.
46. As a user, I want left and right Shift treated as Shift, so that physical key side does not change the saved shortcut.
47. As a user, I want left and right Win treated as Win, so that physical key side does not change the saved shortcut.
48. As a user on a keyboard layout with AltGr, I want the existing Windows Ctrl-plus-Alt representation retained, so that no new physical-key model is introduced.
49. As a user, I want shortcut labels displayed in the fixed modifier order Ctrl, Alt, Shift, Win, then primary key, so that recorded shortcuts are easy to scan.
50. As a user, I want friendly key labels rather than Win32 or WinUI enum names, so that the recorder reads like a Windows setting rather than a diagnostic tool.
51. As a user, I want punctuation labels derived from my current Windows keyboard layout, so that the displayed symbol matches the keyboard context I am using.
52. As a user, I want keyboard-layout-derived punctuation to affect display only, so that changing layouts does not rewrite or migrate the persisted virtual-key value.
53. As a user, I accept that changing keyboard layouts may change the displayed punctuation label for the same persisted virtual-key position, so that storage remains compatible with the current model.
54. As a user, I want Win available as a modifier without a permanent keyboard hook, so that existing Win-hotkey support remains available without expanding background input monitoring.
55. As a user, I accept that Windows-reserved combinations may execute an operating-system action or move focus before DropCove can capture them, so that DropCove does not claim control over reserved shortcuts.
56. As a user, I want a Windows-reserved action that removes focus from Settings to cancel capture and restore the prior draft, so that the recorder cannot remain armed after the system takes over.
57. As a user, I want the currently saved global hotkey prevented from toggling the Drop Shelf while the recorder is listening, so that recording the current shortcut does not disrupt Settings.
58. As a user, I want suppression to cover the input event that completes capture, so that the completing shortcut cannot also toggle the Drop Shelf.
59. As a user, I want the saved global hotkey to resume normal operation after capture completes, is canceled, or loses focus, so that listening does not disable invocation longer than necessary.
60. As a user, I want the saved global hotkey to remain active while a different draft waits for Save, so that an unsaved edit does not change application behavior.
61. As a user, I want recording to create only a draft, so that the global registration changes only when I explicitly select Save.
62. As a user, I want Windows registration checked when I select Save, so that the recorder does not repeatedly replace the active registration while I experiment.
63. As a user, I want a registration failure to retain my draft, so that I can immediately record or choose another combination.
64. As a user, I want the previously saved hotkey to remain registered after a registration failure, so that DropCove stays summonable.
65. As a user, I want registration failure described as unavailable or Windows-reserved rather than always blaming another application, so that the message does not claim an unverified cause.
66. As a user, I want a successful Save to register and persist the draft and close Settings, so that the normal Settings workflow remains unchanged.
67. As a user, I want a settings-persistence failure after registration to restore the prior hotkey and prior settings, so that runtime and durable state remain consistent.
68. As a user, I want unrelated Settings rollback behavior preserved, so that changing the recorder cannot weaken the existing all-settings save transaction.
69. As a returning user, I want the saved global hotkey restored after restarting DropCove, so that recorder configuration remains durable.
70. As a Windows user, I want startup registration failure to warn me while leaving the resident process running, so that I can repair the shortcut through Settings.
71. As a new user, I want `Ctrl + Shift + Space` to remain the default, so that this UX change does not alter established invocation behavior.
72. As a user, I do not want the recorder to add a clear, disable, or reset state, so that the feature remains a direct replacement workflow rather than a new hotkey policy.
73. As a user, I want Cancel or the window close button to close Settings and discard every unsaved change even while listening, so that window-level cancellation remains decisive.
74. As a user, I want Escape during listening distinct from Cancel, so that one key cancels only the capture attempt while the explicit window actions discard the full Settings draft.
75. As a keyboard-only user, I want to enter, use, cancel, re-enter, save, and leave the recorder without a focus trap, so that the complete workflow is accessible.
76. As a screen-reader user, I want the recorder to expose its name, current shortcut, help text, listening state, validation state, and enabled state, so that the workflow is understandable without sight.
77. As a user, I want visible focus and listening indicators that remain legible in Light, Dark, and High Contrast modes, so that state is not conveyed by color alone.
78. As a user with larger text, I want recorder labels and guidance to tolerate text scaling and string growth, so that the Settings layout remains usable.
79. As a performance-conscious user, I want no timer, polling loop, permanent hook, background worker, or repeated registration probe added, so that idle behavior remains unchanged.
80. As a privacy-conscious user, I want key capture limited to the focused Settings recorder, so that DropCove does not become a broad keyboard monitor.
81. As a maintainer, I want the existing hotkey model and JSON schema preserved, so that no migration or compatibility layer is required.
82. As a maintainer, I want existing registration, startup, and rollback modules retained, so that the feature changes input UX rather than rebuilding native hotkey ownership.
83. As a maintainer, I want built-in WinUI controls and accessibility behavior preferred, so that the recorder does not require a custom control framework.
84. As a maintainer, I do not want a new view-model architecture, public interface, adapter hierarchy, or runtime dependency introduced only for this recorder, so that the implementation remains local and direct.
85. As a maintainer, I want the installed self-contained Release to be the acceptance authority, so that source inspection cannot falsely prove native input routing or rendered behavior.
86. As a maintainer, I want UI Automation and real keyboard input to verify the complete workflow through one existing seam, so that tests exercise the same interface as users.
87. As a maintainer, I want the existing automated suite to remain green as regression coverage, so that recorder work does not break settings persistence, window interop, Drop Shelf lifecycle, or other established behavior.
88. As a maintainer, I want unavailable layouts, reserved-key paths, contrast modes, or input configurations reported as unobserved rather than inferred, so that qualification remains evidence-based.

## Implementation Decisions

- This specification changes only the global-hotkey input experience in Settings. Existing Drop Shelf toggle semantics, tray access, resident lifetime, startup behavior, Edge Rail behavior, shake-to-open, monitor selection, and every unrelated Settings field remain governed by their current requirements.
- The separate Ctrl, Alt, Shift, and Win controls and primary-key ComboBox are replaced by one button-like recorder surface built from existing WinUI controls and theme resources. It is not a free-form TextBox and does not open a separate modal dialog.
- The recorder presents an idle/draft state and a listening state. Entering listening snapshots the current draft. Completing capture replaces that draft. Canceling capture restores the snapshot.
- A pointer click starts listening. Keyboard focus alone does not. When focused but idle, Enter or Space activates the recorder through normal button semantics.
- An Enter or Space keypress used to activate the recorder is consumed as a command. Listening begins only after that activation input has completed, preventing key repeat or the activation key itself from becoming the recorded primary key.
- While listening, keyboard handlers run before normal control activation and focus navigation for supported combinations. Modifier-plus-Tab, modifier-plus-Enter, and modifier-plus-Space are captured rather than moving focus or activating the button. Bare Tab retains ordinary focus navigation and therefore cancels capture when the recorder loses focus.
- While listening, modifier key-down and key-up update the visible in-progress chord. Releasing all modifiers does not cancel listening. The first supported primary-key key-down completes capture only when at least one supported modifier is active.
- Escape is reserved as the capture-cancel command and cannot be recorded as a primary key. Backspace and Delete are recordable primary keys when a modifier is held and never mean clear.
- Supported primary keys are A–Z; top-row 0–9; Space, Tab, and Enter; Backspace, Delete, and Insert; Left, Right, Up, Down, Home, End, Page Up, and Page Down; F1–F24; numpad digits and arithmetic keys; and standard printable punctuation/OEM positions that Windows can map to a friendly label.
- Standard punctuation positions include semicolon/colon, equals/plus, comma/less-than, minus/underscore, period/greater-than, slash/question-mark, backtick/tilde, bracket/brace, backslash/pipe, and quote/double-quote positions, including the additional ISO punctuation position when Windows supplies a usable label.
- Unsupported primary keys include lock keys, Print Screen, Pause, media and volume keys, browser keys, application-launch keys, IME/process/packet keys, mouse buttons, and system-special keys outside the approved stable set. Unknown virtual keys are rejected rather than displayed as numeric codes.
- Left and right variants of Ctrl, Alt, Shift, and Win collapse to the existing four aggregate modifier flags. AltGr retains Windows' existing Ctrl-plus-Alt representation. Scan codes and physical left/right identity are not persisted.
- Display order is always Ctrl, Alt, Shift, Win, then the primary key. Key labels are user-facing names rather than enum identifiers.
- Punctuation/OEM labels are derived from the active Windows keyboard layout for display only. The persisted `VirtualKey` remains the stable numeric value in the existing `HotKeyDefinition`. A keyboard-layout change may alter the displayed symbol without changing or migrating settings.
- The inline listening instruction, in-progress chord, invalid-input guidance, and recorded draft are separate states. Invalid input does not overwrite the pre-listening draft and does not exit listening.
- The canonical incomplete/unsupported guidance communicates that at least one modifier and one supported key are required. It appears directly beneath the recorder, is announced through automation, and clears on the next attempt, cancellation, or completion.
- The recorder has no timeout. Losing recorder focus for any reason cancels listening and restores the pre-listening draft. This includes pointer or keyboard navigation to another Settings control and deactivation of the Settings window.
- Save is disabled while listening. Other Settings controls remain available; moving to one cancels listening before that control receives normal interaction.
- Cancel and the window close action retain their current window-level meaning: close Settings and discard all unsaved settings. Escape while listening is narrower and cancels only the current recorder session.
- The application suppresses handling of its registered global-hotkey message while the recorder is listening. Suppression includes the complete input event that finishes capture, preventing the completing shortcut from also toggling the Drop Shelf.
- Suppression is implemented at the existing global-hotkey message-dispatch path. The registered hotkey is not unregistered merely to enter listening, avoiding a second registration lifecycle and restoration race.
- Suppression ends after capture completion, capture cancellation, or focus loss. The previously saved shortcut remains the active registration while a completed draft waits for Save.
- Win remains an allowed modifier, but the recorder does not add a low-level keyboard hook or promise interception of Windows-reserved combinations. If Windows consumes a combination or moves focus, Windows behavior wins and focus-loss cancellation restores the prior draft.
- Capturing a shortcut does not register it. Save remains the only point at which the drafted hotkey is submitted through the existing application settings transaction.
- Registration is attempted before persistence, as it is today. A registration rejection leaves the previous registration active, retains the new draft in Settings, and displays an accurate generic warning that the shortcut may be unavailable or reserved by Windows.
- User-facing registration-failure text must not state that another application is the confirmed cause because the existing native result does not prove that classification.
- Successful registration proceeds through the existing settings application flow, including shake configuration, startup registration, settings persistence, in-memory state replacement, and current rail-placement updates.
- If any later part of the settings transaction fails after registration, the existing rollback restores the previous hotkey, shake state, startup registration, persisted settings, and in-memory settings where currently possible. Existing special handling for rollback failure remains unchanged.
- Startup continues loading the persisted settings and attempting to register the saved hotkey. Registration failure warns the user but leaves DropCove resident and accessible through the tray.
- `HotKeyDefinition`, `AppSettings`, the JSON shape, aggregate modifier booleans, virtual-key storage, default `Ctrl + Shift + Space`, `GlobalHotKey` registration slot, `NoRepeat`, application-local hotkey identifier, and native registration ownership remain unchanged.
- No schema migration is required. Every shortcut that the existing Settings UI could create is contained within the new supported set.
- The recorder does not add clear, reset-to-default, disable-hotkey, multiple-hotkey, sequence, or chord-list state.
- The Settings presentation continues owning this interaction. Native registration remains owned by the resident application and native hotkey module. No new public module, replaceable adapter, view-model migration, command bus, compatibility wrapper, or interface hierarchy is introduced solely for the recorder.
- Built-in WinUI focus, automation, theme, and control-state behavior is reused. No new NuGet package, custom-control framework, global input package, or localization framework is introduced.
- New user-facing text is structured so it can be localized later and so text scaling does not obscure the recorder, but this feature does not introduce a localization system.
- The recorder exposes an accessible name, current value, help text, enabled state, listening state, and validation message. Listening and invalid states remain distinguishable in Light, Dark, and High Contrast without relying on color alone.
- No timer, polling loop, background worker, permanent keyboard hook, telemetry, cache, or periodic registration check is introduced.
- User-facing product documentation is updated after implementation to describe press-to-record configuration, the supported key set, explicit Save behavior, Windows-reserved shortcut limitations, and conflict recovery.

## Testing Decisions

- Good tests and qualification evidence assert user-observable behavior: visible recorder state, keyboard and pointer interaction, focus movement, accessible state, active global-hotkey behavior, registration outcome, persisted settings, restart restoration, and rollback. They do not assert XAML source text, private field names, event-handler names, control-tree implementation, enum-switch structure, or forwarding callbacks.
- The user confirmed one authoritative feature seam: the installed self-contained Release running with an isolated test profile. This is the highest existing seam that can prove WinUI input routing, native hotkey messages, focus, rendered state, UI Automation, persistence, restart behavior, and Windows-reserved-key interaction together.
- The seam reuses the repository's existing PowerShell, UI Automation, native keyboard-input, isolated-profile, settings-file, and installed-process conventions. No new UI testing framework, mock native registration layer, screenshot-baseline framework, or product test hook is introduced.
- A single same-build qualification run records the executable and application payload identity, Windows build, active keyboard layout, theme, contrast mode, DPI, profile location, exercised shortcut combinations, observed outcomes, cleanup, and unobserved configurations.
- UI Automation verifies the recorder's accessible name, current shortcut value, help text, enabled/focusable state, visible focus, listening state, validation announcement, and Save enabled/disabled state.
- Pointer smoke verifies that clicking the idle recorder enters listening, clicking another Settings control cancels listening and restores the prior draft, clicking the recorder after a completed capture starts a replacement attempt, and Cancel or window close discards the full Settings draft.
- Keyboard smoke verifies that Tab focuses the idle recorder without starting capture; Enter and Space each activate it; the activating keypress is not recorded; and capture begins only from subsequent input.
- Listening-mode keyboard smoke verifies Ctrl+Tab, Ctrl+Enter, and Ctrl+Space are intercepted before focus navigation or button activation and become drafts. Bare Tab leaves the recorder, cancels capture, and continues logical Settings traversal.
- Capture smoke verifies modifier-only input remains listening; releasing modifiers updates the in-progress display; the first supported primary-key press completes capture; repeated keydown cannot create a second draft; and display order remains Ctrl, Alt, Shift, Win, primary key.
- Representative supported-key smoke covers at least a letter, top-row digit, Space, Enter, Tab, Backspace or Delete, navigation key, function key above F12, numpad key, numpad arithmetic key, and punctuation/OEM key.
- Punctuation smoke records the active Windows layout, proves a friendly current-layout label is displayed, saves the shortcut, and verifies that JSON retains the numeric virtual-key value rather than the rendered symbol.
- Modifier smoke exercises left and right variants where the environment permits and proves they produce the same aggregate display and persisted flags. AltGr behavior is exercised on a suitable layout when safely available; otherwise it is recorded as unobserved rather than inferred.
- Invalid-input smoke verifies a bare supported primary key, modifier-only sequence, and unsupported key keep listening active, preserve the prior draft, and show the inline guidance. Escape then cancels and clears the guidance.
- Focus-loss smoke verifies pointer navigation, Tab navigation, and Settings-window deactivation each cancel listening, restore the prior draft, and end global-hotkey suppression. The no-timeout rule is observed over a representative wait without adding a test-only clock.
- Active-hotkey smoke starts listening and enters the currently registered shortcut. The recorder must receive and display the chord while the same input does not toggle the Drop Shelf. Suppression must include the completion event.
- Draft smoke records a different valid shortcut without saving, then verifies the previously saved shortcut has resumed toggling the Drop Shelf and the draft shortcut is not active.
- Save smoke verifies the drafted shortcut is registered only after Save, persists to the isolated settings JSON, closes Settings on success, toggles the Drop Shelf, and is restored after a clean application restart.
- Registration-failure smoke reserves a safe test combination from another process or otherwise creates a real native rejection, attempts Save, and verifies the generic unavailable/reserved warning, retained draft, open Settings window, unchanged settings file, and continued operation of the previous hotkey.
- Persistence-failure smoke creates a deterministic isolated-profile write failure without a product test hook. It verifies that a newly registered shortcut is rolled back, the previous shortcut resumes, durable settings remain unchanged, Settings reports the save failure, and unrelated existing rollback behavior remains intact.
- Startup smoke launches with a persisted usable shortcut and verifies registration. A separate real conflict at startup verifies the resident process continues, the tray remains available, and the existing startup warning directs the user to Settings.
- Win-modifier smoke proves at least one non-reserved Win combination reaches the recorder, saves, persists, and activates through the existing registration path without a keyboard hook. If the environment cannot deliver any suitable Win combination, acceptance fails rather than silently adding a hook or weakening the requirement.
- A safe Windows-reserved combination is exercised only in an isolated, reversible scenario. Evidence records whether Windows consumed it, whether Settings lost focus, and whether the recorder restored the prior draft. No claim is made that every Win combination is capturable.
- Light and Dark verification confirm readable idle, focused, listening, draft, invalid, disabled-Save, and registration-error states. High Contrast is exercised only in a safely reversible environment; otherwise it remains explicitly unobserved.
- Text-scaling verification confirms the shortcut value, instruction, and inline error remain readable and do not obscure Save or Cancel at the exercised supported scale.
- The complete existing automated suite remains green and remains regression authority for settings serialization, application behavior, native window calculations, shelf lifecycle, persistence, shake input, and other established modules. It is not expanded with source-text or private-control tests merely to duplicate the installed seam.
- No permanent unit test is added merely to prove a key-name mapping table, a XAML property assignment, or handler forwarding. The installed behavior must demonstrate the mapping and interaction through the user interface and persisted outcome.
- The installed smoke must use actual keyboard input for input-routing claims. Directly assigning recorder state, invoking private handlers, editing settings JSON instead of Save, or posting only a synthetic hotkey message cannot substitute for capture evidence.
- Screenshots may support human review of visual states but are not permanent pixel baselines. UI Automation values, focus, enabled state, real input outcomes, active registration, and persisted settings are the behavioral evidence.
- Qualification cleans up the isolated profile, reserved test hotkeys, helper processes, Windows-reserved UI opened during safe probes, and any temporary files. Normal user settings and host configuration remain untouched.
- Existing release performance gates remain inherited. Qualification checks that listening adds no idle timer, polling, permanent hook, or sustained CPU activity; this feature does not lower, replace, or waive any established gate.

## Out of Scope

- Changing the default global hotkey from `Ctrl + Shift + Space`.
- Changing the global hotkey from a Drop Shelf toggle into an open-only command or any other action.
- Disabling the global hotkey, allowing an empty value, adding Clear, or adding Reset to default.
- Supporting bare primary keys without Ctrl, Alt, Shift, or Win.
- Supporting multiple global hotkeys, multi-step key sequences, simultaneous multiple primary keys, mouse buttons, gestures, or application-specific shortcut profiles.
- Distinguishing left and right modifiers, persisting scan codes, or introducing a physical-key model.
- Supporting lock keys, Print Screen, Pause, media, volume, browser, application-launch, IME, packet/process, or arbitrary unknown virtual keys.
- Guaranteeing capture or suppression of Windows-reserved combinations such as secure attention or shell-owned shortcuts.
- Adding a low-level keyboard hook, elevated helper, `uiAccess`, driver, background keyboard monitor, or global suppression service.
- Temporarily registering every draft, probing registration live during capture, or changing conflict validation from Save time.
- Classifying native registration failures into definitive application-conflict, operating-system-reserved, permission, or other categories beyond the accurate generic warning.
- Changing the settings JSON schema, adding migration, changing load fallback, or replacing virtual-key persistence with text labels.
- Redesigning Settings outside the global-hotkey controls, migrating Settings to MVVM, or introducing a new settings architecture.
- Changing startup registration, tray commands, shake-to-open configuration, Edge Rail settings, monitor selection, fullscreen policy, Drop Shelf lifecycle, or persistence ownership.
- Adding retries, undo, telemetry, analytics, background updates, new runtime dependencies, a new UI testing framework, or a screenshot-regression framework.
- Creating implementation tickets, implementing the feature, creating a Git commit, or publishing a release. Those require the subsequent ticket workflow and explicit user approval.

## Further Notes

- The existing V1 specification remains authoritative for global-hotkey registration, conflict survival, Drop Shelf toggle behavior, startup registration, resident lifetime, and installed native verification. This specification supersedes only the modifier-checkbox and key-ComboBox configuration experience and expands the user-selectable stable primary-key set.
- `CONTEXT.md` does not require a new entry. Global hotkey and recorder are general Windows and UI concepts rather than DropCove-specific domain terms. This specification continues using the canonical **Drop Shelf** term where invocation behavior is described.
- No ADR is required. The change is a reversible Settings interaction that preserves domain boundaries, persistence ownership, native registration ownership, deployment, and background-input policy.
- The confirmed testing seam is the installed self-contained Release with an isolated profile, UI Automation, and actual keyboard input. No second feature-specific test seam is introduced.
- Win-modifier support is intentionally best-effort within focused WinUI input and the existing native registration model. Acceptance requires proof for a non-reserved Win combination and honest evidence for a safe reserved combination; it does not authorize a hidden low-level-hook fallback.
- Enter and Space have two deliberate roles. When the idle recorder has focus, they activate listening and that activation input is excluded. Once listening is active, a later modifier-plus-Enter or modifier-plus-Space is intercepted and recorded as the primary key.
- OEM/punctuation names are display-only projections of the active keyboard layout. Persisted virtual-key identity remains unchanged even if a later layout renders a different symbol.
- This specification is ready for ticket decomposition but does not itself authorize implementation or a Git commit.
