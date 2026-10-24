# 01: Single-PNG Website Image Drop Shelf round trip

**What to build:** Make one narrow but complete Website Image Virtual File workflow work through the Drop Shelf: receive one real PNG virtual file from a supported browser payload, copy its bounded bytes into owned staging, apply mandatory Windows Attachment Services, materialize one crash-safe Managed File, restore it after restart, copy it out as a normal PNG, preserve it when Pinned or when drag-out is canceled/rejected, and permanently clean it after Temporary consumption or explicit removal. This slice owns the working end-to-end seam so later tickets extend a product that already runs.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.24 — Website Image Virtual-File Intake](../spec.md)

**Testing seam:** Actual Windows Resident Instance with a real native OLE Website Image Virtual File source, current-stable Edge or Chrome, the real Drop Shelf/WebView2 surface, owned Profile Data, Windows Attachment Services, a real native Copy destination, and supporting native payload/Core Shelf State SQLite/filesystem regressions.

**Demo path:** Drag one PNG from Edge or Chrome onto the Drop Shelf, restart DropCove, copy the generated Managed File to a native destination, repeat while Pinned, then remove or consume the Temporary item and verify permanent owned-file cleanup while the browser source remains untouched.

- [x] Existing file/folder intake remains path-reference-only and retains path precedence, ordering, repeated-path filtering, retained-batch duplicate suppression, Copy cursor, placeholder, modal blocking, application-owned-drag rejection, and external-source safety.
- [x] Plain-text intake remains unchanged and is considered only when neither valid filesystem paths nor a Website Image Virtual File family is offered.
- [x] The native OLE adapter recognizes one `CFSTR_FILEDESCRIPTORW` entry with indexed `CFSTR_FILECONTENTS`; source application identity is not required.
- [x] Stream and global-memory content forms used by the supported browser seam are read on the registered receiving thread; borrowed COM media never crosses into background trust or persistence work.
- [x] PNG bytes are copied in bounded chunks directly into owned staging rather than accumulated as one unbounded memory buffer.
- [x] Exactly `100 MiB` of encoded PNG data is eligible and one byte over is rejected without Shelf, sizing, scroll, staging, final-file, or Shake promotion residue.
- [x] The PNG signature/container is validated independently of the descriptor extension and within the approved raster-allocation bound.
- [x] A usable descriptor basename is sanitized for Windows and its extension is repaired to `.png`; an unusable basename receives the approved timestamped fallback with deterministic numeric collision suffixing.
- [x] The implementation does not issue an HTTP request, parse companion text/HTML as provenance, or accept URL-only, bitmap-only, rich-text, or generic virtual-file data.
- [x] Optional provenance is accepted only from HTTP(S) `CFSTR_INETURLW/A`; absent or unacceptable provenance uses Attachment Services' Restricted Zone default.
- [x] Every accepted candidate passes through non-interactive `IAttachmentExecute::Save`; no setting, prompt, allowlist, retry-without-scan, or keep-anyway bypass exists.
- [x] The post-scan file is revalidated. Missing, invalid, unsupported, or over-limit output rejects the drop; valid final bytes are authoritative and pre-scan bytes are not retained.
- [x] One accepted image creates one new Shelf Batch with one Temporary Managed File-backed Shelf Item; repeated identical bytes create another intentional batch.
- [x] Managed File ownership is established only by the authoritative ownership relation; browser identity, filename, extension, source URL, or Profile Data path never grants deletion authority.
- [x] Intake follows the durable grouped sequence: flushed staging, one `Staged` batch/item/ownership transaction, atomic same-volume promotion, one `Ready` transaction, then Shelf publication.
- [x] `Staged` image items are absent from the Drop Shelf, Edge Rail, Manage Items, sizing, Shell Visuals, duplicate comparisons, drag participants, and user-facing events.
- [x] Runtime failure before publication creates no visible item and removes or durably records cleanup for every staging/final remnant.
- [x] Restart restores a complete Ready PNG with exact identity, filename, bytes, Pin state, availability, and Shelf order.
- [x] The accepted PNG uses the established Shell Visual path without a browser-specific presentation model.
- [x] Native item and batch drag-out transfer the exact final bytes and filename with Windows Copy behavior.
- [x] Accepted Copy consumes a Temporary PNG and permanently cleans its Managed File; Pinned Copy retains both reference and bytes.
- [x] Canceled, rejected, and pre-acceptance-failed drag-out retains the Shelf Item and Managed File.
- [x] Remove Item, Remove Batch, and confirmed-Missing cleanup affect the owned Managed File only; ordinary external sources are never deleted, moved, renamed, or overwritten.
- [x] A one-image Drop Shelf browser smoke, restart, native Copy round trip, Temporary cleanup, Pinned retention, native regressions, frontend build, and cargo-xwin Windows build pass while release metadata remains `0.0.23`.
- [x] No Git commit is created without separate explicit user authorization.

## Comments

- User authorized implementation of all five tickets. Ticket 01 is active; the integrated native intake, validation/security, and Managed File ownership paths are separate edit owners inside this first vertical slice. Later tickets remain gated by demonstrated completion of their blockers. No Git commit is authorized.
- TDD red observed: native virtual-image eligibility regression fails because the 0.0.23 adapter returns Unsupported. Compile with cargo-xwin `--no-run` and execute the Windows test binary directly; cargo-xwin otherwise selects an unavailable Wine runner on this workstation.

## Verification evidence

- Native eligibility regression observed failing before implementation and passing afterward.
- Real current Chrome image drag into the Drop Shelf created a Managed File with exact source PNG bytes (SHA-256 `aa3750d447a3c439a98b265c1bc74ab278a783a68ac6e4d9f0a2184d2d3c4a7e`). Windows Attachment Services wrote `ZoneId=3` with the offered HTTP image source.
- Actual Windows Forms FileDrop destination received exact PNG filename/bytes through native Copy. Temporary Copy consumed the Shelf Item and permanently deleted the Managed File; Pinned Copy retained both after Resident restart. Escape retained content, and explicit Pinned removal deleted the owned file.
- Restricted-Zone/no-source `Save` was observed returning `INET_E_SECURITY_PROBLEM (0x800C000E)` with no accepted batch. This is fail-closed Windows policy, not bypassed.
- Native stream/global-memory extraction, deferred content reads, first-20 handling, exact `100 MiB` acceptance and one-byte-over skip, complete PNG-container rejection, and generated-name collision regressions executed. Truncated-PNG and same-second naming cases were observed red before their respective fixes, then green.
- Frontend typecheck/production build and cargo-xwin Windows build succeeded with release metadata unchanged at `0.0.23`. The Windows SDK emits existing missing-PDB linker diagnostics; no Visual Studio toolchain was installed or used.
- First-ticket gates passed: **154/154 native Windows regressions**, **13/13 frontend regressions**, Vue typecheck/production build, and cargo-xwin compilation. Exact installed `0.0.24` acceptance and final integrated rerun remain owned by Ticket 05; the runtime evidence above is from the owned pre-release tracer-bullet candidate.
