# WinUI 3 Tri-State ToggleButton & CheckBox Research Note

**Date**: 2026-10-06  
**Context**: DropCove Batch Pinning Tri-State Control Investigation  
**Destination**: `.scratch/bulk-pinning/winui-tristate-research.md`  

---

## 1. Project Package References & WinUI Version

Inspection of [`src/DropCove.App/DropCove.csproj`](../../src/DropCove.App/DropCove.csproj) identifies the exact Windows App SDK and WinUI package versions in use:

* **Microsoft.WindowsAppSDK**: `2.5.1`
* **Microsoft.Windows.SDK.BuildTools**: `10.0.28000.2705`
* **Target Framework**: `net10.0-windows10.0.22000.0`
* **Transitive WinUI package**: `Microsoft.WindowsAppSDK.WinUI` version `2.3.9` (declared in `~/.nuget/packages/microsoft.windowsappsdk/2.5.1/microsoft.windowsappsdk.nuspec`).

### Primary Sources
* [`Microsoft.WindowsAppSDK NuGet Package`](https://www.nuget.org/packages/Microsoft.WindowsAppSDK/2.5.1)
* [Windows App SDK Release Channels](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/)

---

## 2. ToggleButton Capabilities & UI Automation Semantics

### 2.1 API Capabilities
* **Type**: `Microsoft.UI.Xaml.Controls.Primitives.ToggleButton` (inherits `ButtonBase` : `ContentControl`).
* **`IsThreeState` Property**: Present (`bool`). Identifies whether the control supports three states.
* **`IsChecked` Property**: Nullable boolean (`bool?` / `IReference<bool>`).
  * `true`: Checked
  * `false`: Unchecked
  * `null`: Indeterminate / Mixed
* **Events**: `Checked`, `Unchecked`, `Indeterminate`, and `Click`.

### 2.2 Visual State Implementation in WinUI 3
In `generic.xaml` for `DefaultToggleButtonStyle` (`Microsoft.WindowsAppSDK.WinUI` 2.3.9):
* Visual states defined: `Normal`, `PointerOver`, `Pressed`, `Disabled`, `Checked`, `CheckedPointerOver`, `CheckedPressed`, `CheckedDisabled`, **`Indeterminate`**, **`IndeterminatePointerOver`**, **`IndeterminatePressed`**, **`IndeterminateDisabled`**.
* However, in the default theme resource dictionary:
  * `ToggleButtonBackground` = `ControlFillColorDefaultBrush`
  * `ToggleButtonBackgroundIndeterminate` = `ControlFillColorDefaultBrush`
  * `ToggleButtonForeground` = `TextFillColorPrimaryBrush`
  * `ToggleButtonForegroundIndeterminate` = `TextFillColorPrimaryBrush`
  * `ToggleButtonBorderBrush` = `ControlElevationBorderBrush`
  * `ToggleButtonBorderBrushIndeterminate` = `ControlElevationBorderBrush`
* **Verified Fact**: Default `ToggleButton` visually renders the `Indeterminate` state using the exact same colors and appearance as the `Unchecked` (`Normal`) state unless custom styles or child template content provide a visual distinction. Microsoft Learn explicitly confirms: *"ToggleButton has the same visual state for the indeterminate and unchecked states. Derived controls, like CheckBox, may define different visual states for each state."*

### 2.3 UI Automation & Accessibility
* **Automation Peer**: `Microsoft.UI.Xaml.Automation.Peers.ToggleButtonAutomationPeer` (derives from `ButtonBaseAutomationPeer`, implements `IToggleProvider`).
* **Control Type**: `AutomationControlType.Button` (`UIA_ButtonControlTypeId`).
* **Patterns Supported**: Supports `PatternInterface.Toggle` (`IToggleProvider`).
* **ToggleState Property**: Directly exposes `Microsoft.UI.Xaml.Automation.ToggleState`:
  * `ToggleState.On` when `IsChecked == true`
  * `ToggleState.Off` when `IsChecked == false`
  * `ToggleState.Indeterminate` when `IsChecked == null`
* **Narrator Announcement**: Because its control type is `Button`, screen readers announce it as a button with its toggle state (or invoke pattern). In Narrator, it announces its automation name and toggle state (e.g., "Toggle button, mixed" / "Indeterminate" or "on" / "off").

### Primary Sources
* [ToggleButton Class (Microsoft.UI.Xaml.Controls.Primitives) - Windows App SDK](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.primitives.togglebutton?view=windows-app-sdk-2.0)
* [ToggleButtonAutomationPeer Class (Microsoft.UI.Xaml.Automation.Peers)](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.automation.peers.togglebuttonautomationpeer?view=windows-app-sdk-2.0)
* [ToggleButtonAutomationPeer.ToggleState Property](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.automation.peers.togglebuttonautomationpeer.togglestate?view=windows-app-sdk-2.0)
* [ToggleState Enum (Microsoft.UI.Xaml.Automation)](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.automation.togglestate?view=windows-app-sdk-2.0)
* [Button Control Type - Win32 apps (UI Automation)](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-supportbuttoncontroltype)
* WinUI 3 `generic.xaml` (`Microsoft.WindowsAppSDK.WinUI` 2.3.9, lines 27811–28013).

---

## 3. CheckBox Capabilities & UI Automation Semantics

### 3.1 API Capabilities & User-Input Transitions
* **Type**: `Microsoft.UI.Xaml.Controls.CheckBox` (inherits directly from `ToggleButton`).
* **`IsThreeState`**: Inherited from `ToggleButton`.
* **`IsChecked`**: Inherited from `ToggleButton` (`bool?`).
* **Native User Click/Toggle Cycling**:
  * In standard WinUI / XAML three-state toggle controls, user clicks cycle through:
    $$\text{Unchecked (false)} \longrightarrow \text{Checked (true)} \longrightarrow \text{Indeterminate (null)} \longrightarrow \text{Unchecked (false)}$$
  * **Critical UX & Design Guideline Conflict**: Microsoft's official CheckBox design guidelines state:
    > *"Don't allow users to set an indeterminate state directly... The indeterminate state should only be set programmatically, not by the user."*
    When a user clicks a batch / "Select All" checkbox in the indeterminate state, user expectation is to transition directly to `Checked` (pin all) or `Unchecked` (unpin all), not to leave or cycle into indeterminate. If `IsThreeState="True"` is left to native click handling without custom interception, a click on `Checked` transitions to `Indeterminate` (which makes no sense when clicking to unpin all).
  * Consequently, any tri-state batch selector (whether ToggleButton or CheckBox) **requires explicit click event handling** to control the forward transition sequence (typically: clicking `Indeterminate` $\rightarrow$ `Checked`, clicking `Checked` $\rightarrow$ `Unchecked`, clicking `Unchecked` $\rightarrow$ `Checked`).

### 3.2 Visual State Implementation
* `DefaultCheckBoxStyle` in `generic.xaml` implements distinct glyphs and visuals:
  * Checked glyph: `&#xE73E;` (checkmark)
  * Indeterminate glyph: `&#xE9AE;` (dash / horizontal bar)
  * Visual states: `CheckedNormal`, `IndeterminateNormal`, `UncheckedNormal`, along with pointer-over and pressed variants.

### 3.3 UI Automation & Accessibility
* **Automation Peer**: `Microsoft.UI.Xaml.Automation.Peers.CheckBoxAutomationPeer` (inherits `ToggleButtonAutomationPeer`).
* **Control Type**: `AutomationControlType.CheckBox` (`UIA_CheckBoxControlTypeId`).
* **Patterns Supported**: Supports `PatternInterface.Toggle` (`IToggleProvider`).
* **ToggleState Property**: Inherited from `ToggleButtonAutomationPeer`. Exposes `ToggleState.On`, `ToggleState.Off`, and `ToggleState.Indeterminate`.
* **Narrator Announcement**: Announced as "Check box", with states "Checked", "Unchecked", or "Mixed" (or "Indeterminate").

### 3.4 Keyboard Interaction
* **Space key**: Standard activation for both `ToggleButton` and `CheckBox`. Toggles the state.
* **Enter key**:
  * In WinUI 3, `ButtonBase.OnKeyDown` handles `Space` and `Enter` to route to `Click` for controls that present as buttons.
  * Standard CheckBox controls respond reliably to `Space`. According to Windows keyboard interaction guidelines, `Enter` activates command buttons, whereas `Space` toggles buttons and checkboxes. In DropCove's verified keyboard evidence, both `Enter` and `Space` are used for button invocation.

### Primary Sources
* [CheckBox Class (Microsoft.UI.Xaml.Controls) - Windows App SDK](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.checkbox?view=windows-app-sdk-2.0)
* [CheckBoxAutomationPeer Class (Microsoft.UI.Xaml.Automation.Peers)](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.automation.peers.checkboxautomationpeer?view=windows-app-sdk-2.0)
* [Check boxes design guidelines - Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/checkbox)
* [CheckBox Control Type - Win32 apps (UI Automation)](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-supportcheckboxcontroltype)
* [Keyboard interactions - Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions)

---

## 4. Assessment of Options for DropCove Batch Pinning

DropCove currently displays a compact 24x24 pin button with a Segoe Fluent Icon glyph (`&#xE718;`). For multi-item batches, some items may be pinned and some temporary (a "mixed" batch). The control must preserve this compact look while faithfully reporting state to UI Automation and Narrator.

### Verified Facts vs. Design Recommendations

#### Verified Facts
1. **`ToggleButton` natively supports `IsThreeState="True"` and `IsChecked=null`**:
   `ToggleButton` already possesses the property `IsThreeState`, accepts `bool?` for `IsChecked`, and raises `Indeterminate`.
2. **`ToggleButtonAutomationPeer` natively exposes `ToggleState.Indeterminate`**:
   UI Automation clients (including Narrator and Inspect) querying `IToggleProvider.ToggleState` on a `ToggleButton` with `IsChecked == null` receive `ToggleState.Indeterminate` (value `2`).
3. **`ToggleButton` default visual states do not distinguish `Indeterminate` from `Unchecked`**:
   The built-in `DefaultToggleButtonStyle` in WinUI 3 assigns identical brushes to `Normal` and `Indeterminate`. Therefore, if using `ToggleButton`, custom glyph binding, custom visual states, or a custom style is strictly required to show the mixed visual state.
4. **`CheckBox` natively has distinct visual glyphs for `Indeterminate` (`&#xE9AE;`), but renders as a square checkbox**:
   A raw `CheckBox` uses the classic checkbox box-and-mark chrome, not a pin icon. Retaining the pin-icon appearance on a `CheckBox` requires restyling or replacing the `ControlTemplate`.
5. **Native cyclic click behavior includes `null`**:
   When `IsThreeState="True"`, standard clicking by a user cycles $False \rightarrow True \rightarrow Null \rightarrow False$. Because an indeterminate batch state is a derived aggregation of child items, a user click should never cycle *into* indeterminate; it should toggle between pinning all items (`true`) and unpinning all items (`false`).

---

### Option Evaluation Matrix

| Criterion | Option A: `ToggleButton` (Retained Control) + Glyph & Click Logic | Option B: Re-templated `CheckBox` | Option C: Custom Control / Custom Automation Peer |
| :--- | :--- | :--- | :--- |
| **Control Type** | `ToggleButton` (`ControlType.Button`) | `CheckBox` (`ControlType.CheckBox`) | Custom control or subclassed peer |
| **UIA Toggle Pattern** | `IToggleProvider` built-in; returns `ToggleState.Indeterminate` | `IToggleProvider` built-in; returns `ToggleState.Indeterminate` | Custom implementation required |
| **Narrator Announcement** | *"Pin batch, toggle button, mixed / indeterminate"* | *"Pin batch, check box, mixed"* | Custom |
| **Visual Appearance** | Compact 24x24 icon button; changes glyph/opacity based on state | Requires full template override to strip square border and insert FontIcon | Full template or custom rendering |
| **XAML Complexity** | **Minimal**: Keep existing `<ToggleButton>` in XAML; update bindings for `Glyph`, `AutomationProperties.Name`, and `IsChecked` | **High**: Must maintain custom CheckBox template in `generic.xaml` or page resources | **High**: New class definitions and overrides |
| **Input Interception** | Handle `Click` to enforce binary batch toggle when clicked | Handle `Click` to prevent native cycle to `null` | Must override `OnClick`/`OnToggle` |

---

## 5. Answers to Acceptance Questions

1. **Can the current `ToggleButton` natively satisfy tri-state semantics?**  
   **Yes, at the API and UI Automation level; No, at the default visual level.**  
   * API & UIA: `ToggleButton` natively supports `IsThreeState="True"`, `IsChecked = null`, and its default peer `ToggleButtonAutomationPeer` reports `ToggleState.Indeterminate`.  
   * Visuals: WinUI 3's default template for `ToggleButton` maps `Indeterminate` to the same background/foreground/border brushes as `Unchecked`. Visual differentiation requires updating the button's content (e.g. binding `Glyph` or icon opacity/style) or providing custom VisualStates.

2. **What is the simplest built-in accessible alternative?**  
   * **Retaining `ToggleButton` with `IsThreeState="True"` and dynamic glyph/name binding** is the simplest and cleanest architecture for DropCove. It keeps the existing compact button visual footprint, avoids maintaining a bulky custom `ControlTemplate` for `CheckBox`, and natively satisfies `IToggleProvider` / `ToggleState.Indeterminate` without any custom automation peer code.

3. **What behavior requires custom click handling or styling?**  
   * **Click Transition Handling**: Native `IsThreeState` cycles $False \rightarrow True \rightarrow Null$. In DropCove, a batch pin button must never be placed into indeterminate by user click; clicking an indeterminate button must pin all items (`IsChecked = true`), clicking a fully pinned button must unpin all (`IsChecked = false`), and clicking an unpinned button must pin all (`IsChecked = true`). Thus, `OnPinClicked` must explicitly calculate and apply the target boolean state rather than allowing default tri-state cycle progression.  
   * **Glyph & Automation Name Styling**: The button's visual glyph (e.g., filled pin, outline pin, or badge/slash/intermediate glyph) and accessible name (`Pin [Batch]`, `Unpin [Batch]`, or `Partially pinned [Batch]`) must be bound to view-model properties reflecting the three states.
