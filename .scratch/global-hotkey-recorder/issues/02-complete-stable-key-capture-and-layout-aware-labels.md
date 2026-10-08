# 02: Complete stable-key capture and layout-aware labels

**What to build:** Extend the working recorder across the full approved stable primary-key set and make every supported combination capture, display, save, persist, restore, and activate correctly. Complete the special WinUI input-routing, keyboard-layout labeling, modifier normalization, unsupported-key, and Win-modifier behavior without changing the existing hotkey schema or adding global keyboard interception.

**Blocked by:** 01: Replace hotkey selectors with a working recorder.

**Status:** ready-for-agent

**Parent specification:** [Global Hotkey Recorder](../spec.md)

- [x] The recorder supports A–Z, top-row 0–9, Space, Tab, Enter, Backspace, Delete, Insert, Left, Right, Up, Down, Home, End, Page Up, Page Down, F1–F24, numpad digits, numpad arithmetic keys, and the approved standard punctuation/OEM positions.
- [x] Standard punctuation coverage includes semicolon/colon, equals/plus, comma/less-than, minus/underscore, period/greater-than, slash/question-mark, backtick/tilde, bracket/brace, backslash/pipe, quote/double-quote, and the additional ISO punctuation position when Windows supplies a usable label.
- [x] Escape remains the capture-cancel command and cannot be recorded as a primary key.
- [x] Backspace and Delete are recorded when a modifier is held and never clear or disable the shortcut.
- [x] Lock keys, Print Screen, Pause, media and volume keys, browser keys, application-launch keys, IME/process/packet keys, mouse buttons, unknown virtual keys, and other unapproved system-special keys remain unsupported.
- [x] Modifier-plus-Tab is intercepted while listening before WinUI focus navigation and becomes a draft. Bare Tab retains normal focus navigation, cancels listening through focus loss, and continues logical Settings traversal.
- [x] Modifier-plus-Enter is intercepted while listening before default-button behavior and becomes a draft. Bare Enter while listening is incomplete input rather than a second recorder activation.
- [x] Modifier-plus-Space is intercepted while listening before button activation and becomes a draft. Bare Space while listening is incomplete input rather than a second recorder activation.
- [x] The Enter or Space keypress that activates the idle recorder remains excluded from capture; only later input can produce an Enter- or Space-based draft.
- [x] Left and right Ctrl, Alt, Shift, and Win variants collapse to the existing aggregate modifiers and produce the same display and persisted flags.
- [x] AltGr retains the Windows Ctrl-plus-Alt representation; no physical-key identity, scan code, or additional persisted modifier state is introduced.
- [x] Shortcut display order remains Ctrl, Alt, Shift, Win, then primary key for every supported combination.
- [x] Every supported key uses a friendly user-facing label rather than a Win32 or WinUI enum identifier or numeric virtual-key value.
- [x] Punctuation/OEM labels are derived from the active Windows keyboard layout for display only.
- [x] Saving a punctuation/OEM shortcut persists the stable numeric virtual-key value in the existing settings shape rather than the rendered symbol or layout-specific text.
- [x] Changing keyboard layout may change the displayed punctuation symbol for the persisted virtual-key position but does not rewrite, migrate, or invalidate settings.
- [x] If Windows cannot map an OEM or unknown key to an approved friendly label, the recorder rejects it and remains listening instead of exposing a technical name.
- [x] Win is available as an aggregate modifier without adding a low-level keyboard hook, elevated helper, `uiAccess`, driver, or global suppression module.
- [x] A non-reserved Win combination that Windows delivers to the focused Settings window can be captured, displayed, saved, persisted, restored, and used to toggle the Drop Shelf.
- [x] If Windows consumes a reserved combination or removes focus from Settings, Windows behavior wins and the recorder cancels the attempt and restores the pre-listening draft.
- [x] The recorder does not claim that every Win, Alt+Tab, secure-attention, shell-owned, or otherwise operating-system-reserved combination is capturable.
- [x] Unsupported or incomplete input keeps listening active, preserves the pre-listening draft, refreshes the inline accessible guidance, and permits an immediate corrected attempt.
- [x] The complete supported-key behavior works through the same Save-time registration, accurate rejection message, rollback, persistence, restart, and startup contracts established by Ticket 01.
- [x] The recorder remains fully usable by keyboard, exposes current-layout labels and validation through UI Automation, preserves visible focus, and does not create a focus trap.
- [x] Recorder text and layout tolerate the longest approved normalized shortcut, friendly key labels, inline validation, and supported text scaling without obscuring Save or Cancel.
- [x] No settings schema change, migration, second key-name persistence format, compatibility wrapper, custom-control framework, new runtime dependency, timer, polling loop, or permanent hook is introduced.
- [x] The complete existing automated suite passes without lower-level tests that duplicate the installed input seam or pin the key-label implementation.
- [x] A targeted installed self-contained Release smoke run with an isolated profile and actual keyboard input proves representative capture, display, Save, JSON persistence, restart restoration, and Drop Shelf activation for every approved key category before this ticket is complete.

## Testing seam

Use the installed self-contained Release with an isolated profile, UI Automation, actual keyboard input, and persisted settings observation. The representative matrix must include a letter, top-row digit, Space, Enter, Tab, Backspace or Delete, navigation key, function key above F12, numpad digit, numpad arithmetic key, and punctuation/OEM key. Exercise left/right modifiers and AltGr when the environment supports them; record unavailable layout-specific scenarios as unobserved rather than inferred.

## Demo path

From the recorder, capture and save representative shortcuts from each approved category, including modifier-plus-Tab, modifier-plus-Enter, modifier-plus-Space, F13 or above, a numpad key, and current-layout punctuation. Reopen Settings and restart DropCove to show normalized labels, stable persisted virtual keys, and working Drop Shelf invocation. Demonstrate that Escape cancels, Backspace or Delete records with a modifier, an unsupported media or lock key remains rejected, a non-reserved Win combination works, and a safely exercised Windows-reserved combination restores the prior draft when the operating system takes focus.

## Comments

- 2026-10-08: Implementation complete. [Installed functional qualification](../../../artifacts/hotkey-recorder-qualification.json) proves capture, Save, numeric JSON persistence, clean restart, and activation for letters, digits, Tab, Enter, Space, Backspace, Page Down, F14, numpad digit/arithmetic, current-layout punctuation, and `Shift + Win + F13`. All four modifier groups are exercised with left/right variants.
- `Win + R` is observed opening Run, taking focus, canceling capture, and restoring the prior draft; the probe closes Run. No keyboard hook is added. AltGr on a non-US layout, another layout's OEM glyphs, and non-100% DPI are unobserved rather than inferred.
- [Visual/accessibility qualification](../../../artifacts/hotkey-recorder-visual.json) covers Light, Dark, and actual 150% text scaling. Bringing the whole recorder group and native error InfoBar into view prevents guidance/warning clipping while preserving standard ScrollViewer ownership and keyboard traversal. Theme and text-scale settings are restored exactly.
- Full automated suite: 226 passed, zero failed/skipped. Standards and Spec source reviews report zero actionable findings. Completion of this implementation slice does not waive Ticket 03's release gates.

