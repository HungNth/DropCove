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

## Firefox website-image drag

Firefox's Windows `nsDataObj` supports `CFSTR_FILEDESCRIPTORW` plus `CFSTR_FILECONTENTS` for file-promise and file flavors. The descriptor contains one sanitized filename and `CFSTR_FILECONTENTS` returns an `IStream`; this path can therefore use the same bounded virtual-file seam as Chromium when Firefox offers it.

Firefox also has a different native-image fallback. When `CF_HDROP` is negotiated for `kNativeImageMime`, Firefox re-encodes the image as PNG or BMP, writes a random file under the Windows temporary directory, and returns that temporary path. Under DropCove's current paths-first classification, this is indistinguishable from an ordinary external path unless companion OLE formats are used as evidence. Retaining that browser-owned temporary path would not satisfy Managed File ownership or durability.

Holdem handles received paths under the process temporary directory by copying them into `%TEMP%\\holdem_drops` before adding them to its shelf. That explains one practical Firefox compatibility route, but its daily temporary-folder lifecycle, lack of DropCove's durable ownership relation, and absence of the approved Attachment Services boundary make it an implementation reference rather than a reusable product contract.

Sources:

- [Firefox `nsDataObj.cpp`](https://github.com/mozilla/gecko-dev/blob/master/widget/windows/nsDataObj.cpp)
- [Firefox `nsDataObj.h`](https://github.com/mozilla/gecko-dev/blob/master/widget/windows/nsDataObj.h)
- [Holdem `file_drop.rs`](https://github.com/iamzubin/holdem/blob/master/app/src-tauri/src/file_drop.rs)
- [Holdem MIT license](https://github.com/iamzubin/holdem/blob/master/LICENSE)

## Plain-text drag

Windows defines `CF_UNICODETEXT` as null-terminated Unicode text. Chromium's Windows exchange provider publishes dragged plain text in Unicode and an additional narrow-text representation. DropCove can therefore consume `CF_UNICODETEXT` directly without depending on HTML or browser-private formats.

Sources:

- [Standard Clipboard Formats](https://learn.microsoft.com/en-us/windows/win32/dataxchg/standard-clipboard-formats)
- [Chromium `OSExchangeDataProviderWin::SetString`](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/ui/base/dragdrop/os_exchange_data_provider_win.cc)

## Confirmed product direction

The user selected these boundaries during grilling:

- Preserve `Shelf Item` as the domain entity and introduce `Managed File` only for DropCove-owned filesystem backing.
- Never delete external source files or folders.
- Accept Website Image Virtual Files by validated byte capability; source application identity and source URL are optional evidence rather than identity.
- Keep website-image intake to virtual-file bytes only; no HTTP/URL fallback or generic virtual-file intake.
- Limit the `0.0.24` release acceptance matrix to current stable Edge and Chrome. Compatible sources are best-effort; Firefox acceptance is deferred because its native-image fallback exposes a browser-owned `%TEMP%` path through `CF_HDROP` and conflicts with external-path semantics.
- Accept non-empty plain text as a UTF-8 `.txt` Managed File.
- Use Explorer-style multi-batch selection semantics.
- Support selection on both Drop Shelf and expanded Edge Rail, scoped to the current surface session.
- Deduplicate case-insensitive transfer paths while consuming every selected Temporary occurrence represented by a successfully copied path; retain Pinned occurrences.
- Prepare selected batches as one operation: remove Missing references, retain Unavailable references, show one aggregate confirmation, and drag Available participants.
- Remove Managed File references atomically with a durable pending-deletion record, attempt permanent deletion immediately, and retry failed deletions.
- Support PNG, JPEG, GIF, WebP, AVIF, SVG, BMP, and ICO after bounded content validation.
- Preserve a sanitized basename, repair the extension to match validated content, generate timestamped numeric-suffix fallback names, and retain duplicate source names in isolated per-item storage.
- Examine only the first 20 descriptors; enforce 100 MiB encoded bytes per image, 500 MiB accepted encoded bytes per image drop, 512 MiB maximum decoded allocation per raster validation, and 10 MiB per text drop.
- Preserve raw well-formed SVG with DTD/entity expansion disabled, retain scripts and external references, apply attachment security, and never render SVG inside a DropCove WebView; Shell Visual may fall back to a generic icon.
- Skip unsupported, invalid, and over-limit entries in fit-in-order descriptor order and report one aggregate non-modal result. Reject the complete drop on I/O, attachment-security, or persistence failure.
- Require non-UI `IAttachmentExecute::Save` for every Website Image Virtual File with no bypass. Use only HTTP(S) `CFSTR_INETURLW/A` as source evidence and Restricted Zone otherwise.
- Treat successful Attachment Services policy as authoritative even when the filesystem or Windows policy omits `Zone.Identifier`; preserve zone metadata whenever present.
- Revalidate after Attachment Services and preserve exact final supported bytes, animation, and metadata, repairing the extension if the final type changed. Never retain pre-scan bytes.
- Add no progress/cancel UI; bound native extraction by descriptor and byte limits and report one aggregate result when processing finishes.
- Treat retained and pending Managed Files as Profile Data across restart, update, uninstall, and Recovery Backup preservation.
- Warn when permanent deletion remains pending and retry without another blocking modal.
- Expose keyboard multi-selection on the Drop Shelf while the no-activate Edge Rail remains pointer-only.
- Select one representation family per drop: filesystem paths, otherwise Website Image Virtual Files, otherwise Unicode text.
- Do not hash or reuse Managed File content; each accepted text/image drop is a new user action.
- Final delivery split: `0.0.23` Managed File/text/cleanup, `0.0.24` website-image intake, then `0.0.25` multi-batch selection and drag-out.

## Windows attachment security

Microsoft's `IAttachmentExecute` is the Windows attachment-safety interface. For a file already copied to a non-temporary local path, `SetLocalPath` followed by `Save` can invoke antivirus or trust services and attach security evidence in an NTFS alternate data stream. `SetSource` supplies a URL as the primary zone determinant; if no source is supplied, Attachment Services defaults to the Restricted Zone.

Sources:

- [IAttachmentExecute](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-iattachmentexecute)
- [IAttachmentExecute::SetSource](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-iattachmentexecute-setsource)
- [IAttachmentExecute::Save](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-iattachmentexecute-save)

## Documentation outcome

The approved decisions are recorded in `GLOSSARY.md`, ADR-0006, `docs/SPECIFICATION.md`, `docs/ARCHITECTURE.md`, and roadmap versions `0.0.23` through `0.0.25`. The user confirmed the final `0.0.24` design tree through `grill-with-docs` on 2026-10-23.
