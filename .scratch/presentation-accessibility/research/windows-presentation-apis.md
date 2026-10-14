# Windows presentation and accessibility APIs — verified facts and implementation choices

This document provides first-party Windows API facts and conservative technical choices for DropCove 0.0.7 (Windows 11 x64), grounding the presentation and accessibility contracts specified in `docs/ROADMAP.md` (section 0.0.7), `docs/ARCHITECTURE.md` (section 14), and `docs/SPECIFICATION.md` (sections 9, 10, 19).

---

## 1. Windows Shell Item Icons, Thumbnails, and Cache Flags

### 1.1 Verified API Facts

- **`IShellItemImageFactory::GetImage` (`shobjidl_core.h`)**:
  - Retrieves an `HBITMAP` representing a Shell item at a requested `SIZE`. By default, it retrieves an image thumbnail; if no thumbnail exists or extraction fails, it automatically retrieves an `HBITMAP` for the item's icon. [IShellItemImageFactory::GetImage](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitemimagefactory-getimage).
  - Instantiated by querying `IShellItem` created via `SHCreateItemFromParsingName`. [SHCreateItemFromParsingName](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-shcreateitemfromparsingname).
  - Flags (`SIIGBF`):
    - `SIIGBF_RESIZETOFIT` (`0x00`): Shrinks the bitmap to fit the requested size while preserving aspect ratio.
    - `SIIGBF_BIGGERSIZEOK` (`0x01`): Returns a larger image if available, allowing the caller to perform high-quality scaling.
    - `SIIGBF_MEMORYONLY` (`0x02`): Returns the item only if already cached in memory; accesses no disk I/O. The documentation explicitly notes: "*GetImage should not be called from the UI thread without passing SIIGBF_MEMORYONLY*".
    - `SIIGBF_ICONONLY` (`0x04`): Returns only the icon, never a thumbnail.
    - `SIIGBF_THUMBNAILONLY` (`0x08`): Returns only a thumbnail, never an icon; returns an error if no thumbnail handler exists.
    - `SIIGBF_INCACHEONLY` (`0x10`): Allows disk cache retrieval without extracting new thumbnails or icons.
    - `SIIGBF_SCALEUP` (`0x100`): Stretches the bitmap to fit the requested dimensions.
- **`IThumbnailCache::GetThumbnail` (`thumbcache.h`)**:
  - Provides direct access to the system thumbnail cache with `WTS_FLAGS` (`WTS_EXTRACT`, `WTS_INCACHEONLY`, `WTS_FASTEXTRACT`, `WTS_FORCEEXTRACTION`, `WTS_EXTRACTDONOTCACHE`). [IThumbnailCache::GetThumbnail](https://learn.microsoft.com/en-us/windows/win32/api/thumbcache/nf-thumbcache-ithumbnailcache-getthumbnail), [WTS_FLAGS](https://learn.microsoft.com/en-us/windows/win32/api/thumbcache/ne-thumbcache-wts_flags).
  - Microsoft documentation notes that `IShellItemImageFactory::GetImage` uses this same cache internally and provides an easier mechanism because it includes automatic icon fallback.
- **`SHGetFileInfoW` (`shellapi.h`)**:
  - Retrieves an `HICON` via `SHGFI_ICON | SHGFI_SMALLICON` (16x16) or `SHGFI_LARGEICON` (32x32). [SHGetFileInfoW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shgetfileinfow).
  - Supports `SHGFI_USEFILEATTRIBUTES`: When passed with `FILE_ATTRIBUTE_NORMAL` or `FILE_ATTRIBUTE_DIRECTORY`, retrieves standard file type or folder icons without disk access or verifying path existence. [File attribute flags](https://learn.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants).

### 1.2 Relation to Requirements

- `docs/ROADMAP.md` 0.0.7: "*Add asynchronous shell file/folder icons and image thumbnails with fallbacks, bounded previews...*"
- `docs/ARCHITECTURE.md` section 14: "*Icon and thumbnail loading is asynchronous. File/folder icons provide fallback when image thumbnails are unavailable. Compact cards keep bounded preview data; a batch requests visuals for at most its first three Shelf Items.*"
- `docs/SPECIFICATION.md` sections 9.1, 9.2, 10.2: Single cards show an icon or thumbnail; multi-item cards show a stacked preview of at most the first three items; Manage Items rows display item icons or thumbnails alongside names and paths.

---

## 2. Ownership, Release, Threading, and Cancellation Options

### 2.1 Verified API Facts

- **Resource Ownership and Cleanup**:
  - `HBITMAP` returned by `IShellItemImageFactory::GetImage`: The caller takes ownership and **must** release it with `DeleteObject(hbm)`. [DeleteObject](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-deleteobject).
  - `HICON` returned by `SHGetFileInfoW`: The caller takes ownership and **must** release it with `DestroyIcon(hIcon)`. [DestroyIcon](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroyicon).
  - COM Interfaces (`IShellItem`, `IShellItemImageFactory`): Must be released via `IUnknown::Release`. [IUnknown::Release](https://learn.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-release).
- **Threading and COM Apartment**:
  - Shell visual extraction can trigger disk I/O, thumbnail provider DLL execution, or network delays. It must run on off-UI background threads. [IShellItemImageFactory::GetImage Remarks](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitemimagefactory-getimage), [SHGetFileInfoW Remarks](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shgetfileinfow).
  - Background threads calling Shell COM APIs must initialize COM via `CoInitializeEx(None, COINIT_APARTMENTTHREADED)`. Single-Threaded Apartment (STA) is required by many Shell extension extractors. [CoInitializeEx](https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-coinitializeex).
- **Cancellation Constraints**:
  - Neither `IShellItemImageFactory::GetImage` nor `SHGetFileInfoW` accepts an abort handle or cancellation callback. An individual extraction call is a synchronous blocking operation.
  - Cancellation must occur at task boundaries:
    1. Pre-execution check: Before invoking `GetImage`, check an atomic cancellation flag or generation token.
    2. Post-execution discard: If cancellation was signaled while `GetImage` was executing, the background worker immediately calls `DeleteObject(hbm)` and drops the result rather than encoding or dispatching it to the UI.
    3. Queue pruning: When a surface closes or a batch is removed, all pending extraction jobs associated with that surface/batch token are dropped from the worker queue.

### 2.2 Relation to Requirements

- `docs/ARCHITECTURE.md` section 14: "*Each surface owns cancellation and release of its active image resources even when visual loading is coordinated centrally. Full item projections are created when management UI opens and cleared when it closes or its owner disappears.*"
- `docs/ROADMAP.md` 0.0.7: "*...cancellation, and resource release tied to surface lifetime.*"

---

## 3. System Light/Dark and High Contrast Signals

### 3.1 Verified API Facts

- **Light and Dark Themes**:
  - Native window frame / title bar: Win32 applications configure dark title bar rendering by calling `DwmSetWindowAttribute` with `DWMWA_USE_IMMERSIVE_DARK_MODE` (`20`). [DwmSetWindowAttribute](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmsetwindowattribute), [DWMWINDOWATTRIBUTE](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute), [Support Dark and Light themes in Win32 apps](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/ui/apply-windows-themes).
  - System setting signals:
    - WinRT `Windows.UI.ViewManagement.UISettings`: `GetColorValue(UIColorType::Foreground)` indicates whether foreground is light or dark, and `ColorValuesChanged` fires when the user toggles color mode. [UISettings](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.uisettings).
    - Registry value: `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\AppsUseLightTheme` (`DWORD`: `0` = Dark, `1` = Light). [Set dark mode](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/set-dark-mode).
  - WebView2 integration:
    - `CoreWebView2Profile.PreferredColorScheme`: Defaults to `COREWEBVIEW2_PREFERRED_COLOR_SCHEME_AUTO`, which automatically syncs WebView2's rendering with the Windows application theme. [CoreWebView2PreferredColorScheme](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2preferredcolorscheme).
    - Web content responds natively via standard CSS `@media (prefers-color-scheme: dark)` and `@media (prefers-color-scheme: light)`. [Support Dark and Light themes](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/ui/apply-windows-themes).
- **High Contrast (Contrast Themes)**:
  - System setting query: `SystemParametersInfoW(SPI_GETHIGHCONTRAST, sizeof(HIGHCONTRAST), &hc, 0)`. High contrast is active when `hc.dwFlags & HCF_HIGHCONTRASTON` is nonzero. [High contrast parameter](https://learn.microsoft.com/en-us/windows/win32/winauto/high-contrast-parameter), [HIGHCONTRASTW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-highcontrastw).
  - Change notifications: Windows broadcasts `WM_THEMECHANGED`, `WM_SYSCOLORCHANGE`, and `WM_SETTINGCHANGE` (with `SPI_SETHIGHCONTRAST` or `lParam` pointing to `"HighContrast"`). [WM_SETTINGCHANGE](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-settingchange).
  - WebView2 / CSS integration:
    - Microsoft Edge and WebView2 automatically detect Windows High Contrast mode and map it to CSS `@media (forced-colors: active)`. [High-contrast mode](https://learn.microsoft.com/en-us/windows/win32/w8cookbook/high-contrast-mode).
    - Under forced colors, standard CSS system colors (`Canvas`, `CanvasText`, `Highlight`, `HighlightText`, `ButtonFace`, `ButtonText`) adapt to the active Windows contrast scheme, while background images and arbitrary tints are overridden unless styled with `forced-color-adjust: none`.

### 3.2 Relation to Requirements

- `docs/SPECIFICATION.md` section 19: "*Theme-aware resources support Light, Dark, and High Contrast presentation.*"
- `docs/ROADMAP.md` 0.0.7: "*Complete Light, Dark, and High Contrast resources, visible focus behavior...*"

---

## 4. Client-Area Animation Policy

### 4.1 Verified API Facts

- **System Parameter**:
  - `SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, &fAnimation, 0)`: Retrieves a `BOOL` indicating whether animations in UI elements should be enabled. If `FALSE`, the user has disabled animations in Windows settings (Settings > Accessibility > Visual effects > "Animation effects"). [Client area animation parameter](https://learn.microsoft.com/en-us/windows/win32/winauto/client-area-animation), [SystemParametersInfoW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow).
  - Change notifications: Broadcast via `WM_SETTINGCHANGE` with `wParam = SPI_SETCLIENTAREAANIMATION`.
- **WebView2 / Web Mapping**:
  - Microsoft Edge and Chromium reflect the Windows "Animation effects" state (`SPI_GETCLIENTAREAANIMATION`) directly into the CSS `@media (prefers-reduced-motion: reduce)` media query. [Simulate reduced motion](https://learn.microsoft.com/en-us/microsoft-edge/devtools/accessibility/reduced-motion-simulation).
  - When animations are turned off in Windows, web content matching `@media (prefers-reduced-motion: reduce)` must suppress non-essential animations.

### 4.2 Relation to Requirements

- `docs/SPECIFICATION.md` section 19: "*Entrance, insertion, removal, and rail transitions are short opacity/scale effects. Non-essential motion is skipped when Windows disables client-area animations. Drag-and-drop and state changes remain functional when animations are disabled.*"
- `docs/ARCHITECTURE.md` section 14: "*Animations are short state-change effects. Non-essential transitions are skipped when Windows disables client-area animations.*"
- `docs/ROADMAP.md` 0.0.7: "*Add short state-change animations and disable non-essential motion when Windows client-area animations are disabled. Validate that earlier workflows remain fully operable by keyboard and without thumbnails or animation.*"

---

## 5. Accessibility: Live Notifications, Names, and States

### 5.1 Verified API Facts

- **Windows UI Automation (UIA) and Live Regions**:
  - In Windows UI Automation, live announcements are governed by `LiveSetting` (`Off = 0`, `Polite = 1`, `Assertive = 2`) via `UIA_LiveSettingPropertyId` (`30135`) and `UIA_LiveRegionChangedEventId` (`20024`). [LiveSetting](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationcore/ne-uiautomationcore-livesetting), [Automation Element Property Identifiers](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-automation-element-propids).
  - Chromium/WebView2 maps W3C ARIA live attributes directly to Windows UIA:
    - `aria-live="polite"` (or `role="status"`): Maps to `LiveSetting_Polite`. When content updates, WebView2 raises `UIA_LiveRegionChangedEventId`, prompting screen readers (Windows Narrator, NVDA) to announce the change when idle. [Announce dynamic changes with live regions](https://learn.microsoft.com/en-us/power-apps/maker/canvas-apps/accessible-apps-live-regions).
    - `aria-live="assertive"` (or `role="alert"`): Maps to `LiveSetting_Assertive`, prompting screen readers to interrupt immediately for urgent errors.
    - `aria-atomic="true"`: Ensures the entire region's updated text is announced rather than only the text delta.
- **Accessible Names and Truncation**:
  - Interactive elements must expose an accessible name computed via content, `aria-label`, or `aria-labelledby`. [MS-ARIA Standards Support](https://learn.microsoft.com/en-us/openspecs/ie_standards/ms-aria/67bd02b3-3cda-4648-9d19-03cddb7162d5).
  - When text is visually truncated with CSS `text-overflow: ellipsis`, the full path remains accessible by providing the full path via `title` attribute (native tooltip) and `aria-label` or accessible text description on the row.
- **Tri-State and Disclosure States**:
  - Tri-state Bulk Pinning requires `aria-checked="true"`, `aria-checked="false"`, or `aria-checked="mixed"` (Indeterminate) on the bulk control. [SPECIFICATION.md section 19](docs/SPECIFICATION.md:733).
  - Popup disclosures (Manage Items trigger, Rail Handle) require `aria-expanded="true" | "false"` and `aria-haspopup="dialog"`.
  - Focus restoration: When a popup or modal surface closes, keyboard focus must programmatically return to the triggering control.

### 5.2 Relation to Requirements

- `docs/SPECIFICATION.md` section 19: "*Action controls expose accessible names. Keyboard users can open and close Manage Items, traverse actions, and restore focus to the originating control. Bulk Pinning exposes Off, On, and Indeterminate states. Status and recorder guidance use live accessibility announcements where required. Full paths remain available through tooltips and accessibility text when visual text is shortened.*"
- `docs/ROADMAP.md` 0.0.7: "*Complete Light, Dark, and High Contrast resources, visible focus behavior, accessibility names/states, live announcements, and full-path accessibility text.*"

---

## 6. Practical Virtualized-List Accessibility Constraints in WebView2/Vue

### 6.1 Verified Facts and Constraints

- **DOM Virtualization Mechanics**:
  - Virtual scrolling recycles DOM nodes, rendering only visible rows plus an overscan buffer.
- **Accessibility Tree Impact**:
  - Screen readers query the browser's accessibility tree (bridged to Windows UIA). Elements not in the DOM do not exist in the accessibility tree.
  - As a result, screen readers cannot determine the total collection size or the relative position of an item unless explicit ARIA collection metadata is provided:
    - The container element must declare `role="list"` (or `role="group"`).
    - Rendered item rows must declare `role="listitem"`, `aria-setsize="<totalItemCount>"`, and `aria-posinset="<oneBasedIndex>"`.
- **Keyboard Focus Loss Hazards**:
  - If a keyboard-focused item is scrolled out of the viewport buffer and unmounted, focus drops to `document.body`, breaking user orientation and keyboard traversal.
  - Mitigation options:
    1. Ensure the currently focused item is preserved (pinned) in the DOM even if its scroll position exits the visible buffer until focus moves elsewhere.
    2. Alternatively, manage container focus with `aria-activedescendant` pointing to the virtual active item ID while keeping DOM focus on the virtual container.
  - Traversal with Arrow keys: Programmatic navigation must scroll the target item into view *and* ensure it is mounted in the DOM before invoking `.focus()`.
- **Scope and Lifetime in DropCove**:
  - Drop Shelf cards: Bounded preview data (at most 3 visuals per batch). The number of active batches is typically small to moderate.
  - Manage Items: Can contain hundreds or thousands of files dropped in a single batch. Virtualization is essential here to prevent DOM explosion and unbounded image extraction.
  - Projection clearance: Item projections and image resources allocated for Manage Items must be cleared when the surface closes or unmounts, satisfying Architecture section 14.

---

## 7. Conservative Recommendations

1. **Shell Visual Extraction**:
   - Use `IShellItemImageFactory::GetImage` in an off-UI worker thread pool initialized as COM STA (`COINIT_APARTMENTTHREADED`).
   - Request `SIIGBF_RESIZETOFIT` with a standard bounding box (e.g., 32x32 logical pixels scaled to monitor DPI).
   - Use `SHGetFileInfoW` with `SHGFI_USEFILEATTRIBUTES` to provide immediate synchronous or cached generic icons when thumbnail extraction is pending or unsupported.
2. **Handle Cleanup and Cancellation**:
   - Wrap `HBITMAP` in an RAII guard calling `DeleteObject` immediately upon failure, cancellation, or conversion to memory bytes.
   - Attach an atomic epoch or cancellation token to each surface request. If the surface closes, mark the token cancelled. Off-thread workers check the token before calling `GetImage` and immediately free the resulting `HBITMAP` if cancelled.
   - When Manage Items closes, emit an explicit clear event that drops pending requests and clears frontend image projections.
3. **Theming and High Contrast**:
   - Rely primarily on native WebView2 CSS media queries: `@media (prefers-color-scheme: dark)` and `@media (forced-colors: active)`.
   - Call `DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &enabled, sizeof(enabled))` on the native window HWND when the OS theme toggles.
   - Use standard CSS system colors (`Canvas`, `CanvasText`, `Highlight`, `HighlightText`, `ButtonFace`, `ButtonText`) and explicit borders (`1px solid transparent` or themed) to ensure legibility in High Contrast.
4. **Motion Policy**:
   - Implement animations as short CSS transitions (opacity and transform).
   - Enforce `@media (prefers-reduced-motion: reduce)` in global styles, overriding non-essential transitions and animations with `transition-duration: 0.001ms !important; animation-duration: 0.001ms !important;`.
5. **Accessibility**:
   - Provide a dedicated, visually hidden `aria-live="polite"` element with `role="status"` and `aria-atomic="true"` for shelf and hotkey recorder status announcements.
   - Maintain `aria-checked="mixed"` on the bulk pinning button when batch items have mixed pin states.
   - Manage focus trap and restoration cleanly on Manage Items open/close.
6. **Virtualization**:
   - Implement fixed-row-height virtualization in `ManageItems.vue` with `role="list"`, `role="listitem"`, `aria-setsize`, and `aria-posinset`.
   - Ensure the keyboard navigation handler scrolls the target row into view before focusing to prevent focus detachment.

---

## 8. Unresolved User Decisions

1. **Drop Shelf Batch Card Virtualization Scope**:
   - *Question*: `docs/ARCHITECTURE.md` section 14 states "*Batch collections and Manage Items lists use virtualized presentation controls.*" Does the outer Drop Shelf batch card list require full virtual scrolling in 0.0.7, or is virtualization only required for the inner Manage Items row list where file counts can be unbounded (e.g. 500+ items)?
2. **Thumbnail Memory Cache Longevity**:
   - *Question*: Architecture 14 specifies that each surface owns cancellation and release of active resources, and full projections are cleared when management UI closes. Should extracted thumbnail byte buffers be purged completely on surface close/hide (ephemeral), or should a bounded LRU memory cache (e.g. 64 items) retain thumbnails across shelf reopenings within the resident session?
3. **Target Preview Icon Dimensions**:
   - *Question*: What exact logical pixel dimensions are preferred for compact card stacked previews (e.g. 20x20 vs 24x24 vs 32x32) and Manage Items rows (e.g. 24x24 vs 32x32)?
