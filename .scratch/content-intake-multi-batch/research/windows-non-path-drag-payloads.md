# Windows non-path drag payloads

This note combines primary-source Windows/Chromium research, current repository facts, and the approved product decisions that depend on them. Canonical behavior remains in `GLOSSARY.md`, ADR-0006, `docs/SPECIFICATION.md`, and `docs/ARCHITECTURE.md`.

## Current DropCove boundary

`src-tauri/src/windows_drop.rs` registers a native OLE `IDropTarget` on the Drop Shelf and Edge Rail WebView windows.

Its `storage_paths` path:

1. calls `SHCreateShellItemArrayFromDataObject`;
2. asks each Shell item for `SIGDN_FILESYSPATH`;
3. falls back to `CF_HDROP` path extraction;
4. accepts the drag only when at least one non-empty filesystem path exists.

The synchronous `IDataObject` remains on the window thread. Only extracted owned path strings cross into the blocking persistence task. Therefore the current adapter rejects plain text and browser virtual files that do not expose filesystem paths.

## Existing files versus virtual files

Microsoft distinguishes two relevant representations:

- `CF_HDROP`: paths of existing filesystem files.
- `CFSTR_FILEDESCRIPTOR` plus indexed `CFSTR_FILECONTENTS`: file-like data whose bytes are supplied by the drag source without an existing filesystem path. `CFSTR_FILECONTENTS` may use `TYMED_ISTREAM`, `TYMED_HGLOBAL`, or `TYMED_ISTORAGE`; Microsoft recommends streams for ordinary file-like transfers.

Microsoft advises general file-accepting drop targets to support both existing filesystem paths and the virtual-file pair, and to enumerate the offered `FORMATETC` values rather than assume one format.

Sources:

- [Shell Clipboard Formats](https://learn.microsoft.com/en-us/windows/win32/shell/clipboard)
- [Handling Shell Data Transfer Scenarios](https://learn.microsoft.com/en-us/windows/win32/shell/datascenarios)
- [Shell Data Object](https://learn.microsoft.com/en-us/windows/win32/shell/dataobject)

## Chromium website-image drag

Current Chromium source prepares in-memory image contents with `PrepareDragForFileContents`, obtains a safe filename, and calls `OSExchangeDataProvider::SetFileContents`. On Windows, `OSExchangeDataProviderWin::SetFileContents` exposes the virtual file through `CFSTR_FILEDESCRIPTORW` and indexed `CFSTR_FILECONTENTS`. File contents are added before URL data so actual image bytes take precedence over a synthesized Internet Shortcut.

This gives DropCove a no-network intake seam for Chromium image drags: materialize the bytes offered by the source instead of independently fetching the image URL. Runtime acceptance still needs real Edge and Chrome coverage; source behavior alone is not acceptance evidence.

Sources:

- [Chromium `web_contents_view_aura.cc`](https://chromium.googlesource.com/chromium/src/+/master/content/browser/web_contents/web_contents_view_aura.cc)
- [Chromium `os_exchange_data_provider_win.cc`](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/ui/base/dragdrop/os_exchange_data_provider_win.cc)

## Plain-text drag

Windows defines `CF_UNICODETEXT` as null-terminated Unicode text. Chromium's Windows exchange provider publishes dragged plain text in Unicode and an additional narrow-text representation. DropCove can therefore consume `CF_UNICODETEXT` directly without depending on HTML or browser-private formats.

Sources:

- [Standard Clipboard Formats](https://learn.microsoft.com/en-us/windows/win32/dataxchg/standard-clipboard-formats)
- [Chromium `OSExchangeDataProviderWin::SetString`](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/ui/base/dragdrop/os_exchange_data_provider_win.cc)

## Confirmed product direction

The user selected these boundaries during grilling:

- Preserve `Shelf Item` as the domain entity and introduce `Managed File` only for DropCove-owned filesystem backing.
- Never delete external source files or folders.
- Accept website images from virtual-file bytes only in the first version; no HTTP/URL fallback.
- Accept non-empty plain text as a UTF-8 `.txt` Managed File.
- Use Explorer-style multi-batch selection semantics.
- Support selection on both Drop Shelf and expanded Edge Rail, scoped to the current surface session.
- Deduplicate case-insensitive transfer paths while consuming every selected Temporary occurrence represented by a successfully copied path; retain Pinned occurrences.
- Prepare selected batches as one operation: remove Missing references, retain Unavailable references, show one aggregate confirmation, and drag Available participants.
- Remove Managed File references atomically with a durable pending-deletion record, attempt permanent deletion immediately, and retry failed deletions.
- Accept only validated image virtual-file payloads rather than generic virtual files.
- Support PNG, JPEG, GIF, WebP, AVIF, SVG, BMP, and ICO Managed Files after content validation.
- Preserve a sanitized browser image filename; generate timestamped text filenames.
- Limit intake to 100 MiB per image, 20 images and 500 MiB per image drop, and 10 MiB per text drop.
- Treat retained and pending Managed Files as Profile Data across restart, update, uninstall, and Recovery Backup preservation.
- Warn when permanent deletion remains pending and retry without another blocking modal.
- Expose keyboard multi-selection on the Drop Shelf while the no-activate Edge Rail remains pointer-only.
- Select one representation family per drop: filesystem paths, otherwise virtual images, otherwise Unicode text.
- Skip unsupported image entries but reject the complete drop on operational, attachment-security, or persistence failure.
- Do not hash or reuse Managed File content; each accepted text/image drop is a new user action.
- Pass website Managed Files through Windows Attachment Services and retain zone evidence.
- Preserve raw SVG bytes with attachment-zone evidence and never render SVG inside a DropCove WebView; Shell Visual may fall back to a generic icon.
- Revalidate website images after Attachment Services; accept the final post-scan file only if it still exists and satisfies image and size rules.
- Final delivery split: `0.0.23` Managed File/text/cleanup, `0.0.24` website-image intake, then `0.0.25` multi-batch selection and drag-out.

## Windows attachment security

Microsoft's `IAttachmentExecute` is the Windows attachment-safety interface. For a file already copied to a non-temporary local path, `SetLocalPath` followed by `Save` can invoke antivirus or trust services and attach security evidence in an NTFS alternate data stream. `SetSource` supplies a URL as the primary zone determinant; if no source is supplied, Attachment Services defaults to the Restricted Zone.

Sources:

- [IAttachmentExecute](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-iattachmentexecute)
- [IAttachmentExecute::SetSource](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-iattachmentexecute-setsource)
- [IAttachmentExecute::Save](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-iattachmentexecute-save)

## Documentation outcome

The approved decisions are recorded in `GLOSSARY.md`, ADR-0006, `docs/SPECIFICATION.md`, `docs/ARCHITECTURE.md`, and roadmap versions `0.0.23` through `0.0.25`.
