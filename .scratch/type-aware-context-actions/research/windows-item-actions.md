# Windows Item Actions — Primary-source constraints

Checked against Microsoft Learn during 0.0.27 implementation and integration on 2026-10-26.

## Clipboard ownership

- `OpenClipboard` must be balanced by `CloseClipboard`. Opening with a null HWND and then calling `EmptyClipboard` sets the clipboard owner to null, which makes `SetClipboardData` fail. The adapter therefore passes the invoking live application window HWND, not null. [OpenClipboard](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-openclipboard)
- Memory passed to `SetClipboardData` must use `GMEM_MOVEABLE`. Ownership transfers to Windows only on successful publication; the caller must not free it afterward. Failure leaves cleanup with the caller. [SetClipboardData](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setclipboarddata)
- Actual Windows clipboard readback verified exact Unicode external paths and full 1,385-character multilingual Managed Text, including retained CRLF/LF content. A second process holding the real clipboard produced access denied without Shelf mutation. These are runtime observations, not claims that documentation guarantees all destination applications preserve formatting.

## Explorer selection

- `SHOpenFolderAndSelectItems` opens Explorer with selected items. With zero child entries, the absolute PIDL identifies the single object; Explorer opens its parent and selects that object. COM must be initialized first. [SHOpenFolderAndSelectItems](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shopenfolderandselectitems)
- Roots use the separate Shell open operation because they have no selectable parent. Native smoke observed file selection, folder selection in its parent, and `file:///C:/` root opening.

## COM lifetime

- Every successful `CoInitializeEx`, including `S_FALSE`, must be balanced by `CoUninitialize`. An incompatible existing apartment returns `RPC_E_CHANGED_MODE` and must not be balanced as a successful initialization. The scoped adapter propagates failure and balances only success. [CoInitializeEx](https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-coinitializeex)

## Verification boundaries

- Documentation constrains native resource ownership; it does not prove frontend target selection, focus, no-activation, menu dismissal, or installation lifecycle.
- Those behaviors were checked separately through component regressions, the actual Windows surfaces, clipboard readback, Explorer observation, accessibility-tree inspection, and the release ticket's installed-candidate evidence.
