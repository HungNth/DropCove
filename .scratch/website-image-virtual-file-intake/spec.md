# DropCove 0.0.24 — Website Image Virtual-File Intake

Status: ready-for-agent
Design confirmation: approved by the user through `grill-with-docs` on 2026-10-23
Testing seam: confirmed by the user through `to-spec` on 2026-10-23 — the actual installed Windows Resident Instance receiving real current-stable Edge and Chrome image drags on the Drop Shelf and expanded Edge Rail, then performing native Copy drag-out into a real destination, supported by the existing native `IDataObject` and Core Shelf State SQLite/filesystem regression seams
Release assignment: 0.0.24
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md#0024--website-image-virtual-file-intake)
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 12.2–12.4, 13, 14, and 17
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md), sections 2.3–2.7 and 4–10
Architectural decision: [ADR-0006 — Materialize non-filesystem drops as app-owned Managed Files](../../docs/adr/0006-managed-files-and-durable-cleanup.md)
Research: [Windows non-path drag payloads](../content-intake-multi-batch/research/windows-non-path-drag-payloads.md)
Ticket decomposition: user approved five tracer-bullet tickets with DAG `01 → 02 → {03, 04} → 05` on 2026-10-23: [01 — Single-PNG Website Image Drop Shelf round trip](issues/01-single-png-drop-shelf-round-trip.md), [02 — Bounded multi-format Website Image batch intake](issues/02-bounded-multi-format-image-batch-intake.md), [03 — Attachment security and crash-atomic batch recovery](issues/03-attachment-security-crash-atomic-recovery.md), [04 — Edge Rail, Shake, and image lifecycle parity](issues/04-edge-rail-shake-image-lifecycle-parity.md), and [05 — Release DropCove 0.0.24](issues/05-release-dropcove-0.0.24.md)
Ticket breakdown confirmation: approved by the user through `to-tickets` on 2026-10-23
Implementation approval: not granted by this specification or ticket publication; implementation authorization, release acceptance, and Git commit authorization remain separate gates

## Problem Statement

DropCove can retain external files and folders and can materialize Unicode text as a Managed File, but it cannot accept an image dragged directly from a website when the browser supplies file bytes without a durable filesystem path. Edge and Chrome expose these drags as Website Image Virtual Files through Windows OLE. The current adapter detects that a virtual-file representation exists only to prevent an unsafe fallthrough into companion URL or text data, then rejects it.

Users therefore must first save a website image manually, locate the downloaded file, and drag that file into DropCove. This breaks the direct browser-to-Shelf workflow and discards the browser-provided filename and exact image bytes that are already available in the drag payload.

Accepting these bytes creates trust, resource, and lifecycle obligations. Browser image payloads can contain malformed content, misleading extensions, unsafe filenames, oversized streams, active SVG content, companion URLs or text, and files changed or rejected by antivirus or Windows policy. A multi-image drop must never publish a partial Shelf Batch after an operational failure, leak unowned staging files, bypass Attachment Services, weaken external-source safety, or expose incomplete Managed Files after restart.

The user needs direct website-image intake on both Shelf surfaces with predictable limits, Windows attachment security, exact final-byte preservation, atomic Managed File ownership, truthful aggregate feedback, and the established Temporary/Pinned and Copy-only workflows.

## Solution

Extend the native Windows OLE intake adapter to accept Website Image Virtual Files after valid existing filesystem paths and before Unicode text. Read `CFSTR_FILEDESCRIPTORW` plus indexed `CFSTR_FILECONTENTS` directly from the source data object without issuing an HTTP request. Current-stable Edge and Chrome are the release acceptance matrix; compatible payloads from other applications remain best-effort.

Inspect only the first 20 descriptors in source order. Read candidate contents in bounded chunks, validate supported image formats and resource limits, sanitize names, repair extensions from validated content, and stage accepted bytes under Profile Data. Apply mandatory, non-interactive Windows Attachment Services to every candidate, using only explicit HTTP(S) `CFSTR_INETURLW/A` data as source evidence and Restricted Zone otherwise. Revalidate the post-scan file and treat its final supported bytes as authoritative.

Commit all accepted images from one drag as one ordered Shelf Batch through the existing Managed File `Staged` → `Ready` lifecycle. Publish only after every participating item is durable and Ready. Unsupported, invalid, and over-limit entries may be skipped with one aggregate status; I/O, security, or persistence failure rejects and cleans the complete drop. Reuse the existing Shell Visual, Manage Items, Temporary/Pinned, Copy-only drag-out, removal, cleanup retry, restart, update, uninstall, Recovery Backup, Drop Shelf, Edge Rail, and Shake-to-open behavior.

## User Stories

1. As a Drop Shelf user, I want to drag an image directly from Edge into DropCove, so that I do not need to save it manually first.
2. As a Drop Shelf user, I want to drag an image directly from Chrome into DropCove, so that browser-image intake works in the supported Chromium browsers.
3. As an Edge Rail user, I want to drop a website image onto the expanded Rail, so that direct image intake is available from either Shelf surface.
4. As a user, I want source application identity to be irrelevant when the payload satisfies the Website Image Virtual File contract, so that compatible sources can work without brittle process-name detection.
5. As a user, I want Edge and Chrome to be explicitly release-tested, so that advertised browser support is evidence-backed.
6. As a user of another browser or application, I want compatible payloads handled best-effort without an unsupported compatibility promise, so that the release contract remains truthful.
7. As a user dropping one image, I want one Shelf Batch with one Shelf Item, so that website images follow the established grouping model.
8. As a user dropping multiple virtual images, I want all accepted images grouped in one Shelf Batch, so that one drag remains one user action.
9. As a user dropping multiple images, I want accepted items kept in descriptor order, so that DropCove preserves source ordering.
10. As a user, I want a website image to become a normal Managed File-backed Shelf Item, so that every established file workflow remains available.
11. As a user dragging PNG content, I want exact final PNG bytes retained, so that image fidelity is preserved.
12. As a user dragging JPEG content, I want exact final JPEG bytes and metadata retained, so that image quality and EXIF data are not silently changed.
13. As a user dragging GIF content, I want animation preserved, so that DropCove does not flatten the image.
14. As a user dragging WebP content, I want animation and metadata preserved, so that the destination receives the final accepted file unchanged.
15. As a user dragging AVIF content, I want the supported file retained without re-encoding, so that modern image fidelity is preserved.
16. As a user dragging BMP content, I want the validated bitmap accepted as a normal file, so that browser-provided bitmap files remain usable.
17. As a user dragging ICO content, I want the validated icon accepted as a normal file, so that icon assets can be staged directly.
18. As a user dragging SVG content, I want the raw well-formed SVG bytes retained, so that DropCove does not break scripts, external references, or document structure.
19. As a security-conscious user, I want DropCove to avoid rendering SVG inside a WebView, so that active SVG content is never executed by the application UI.
20. As a user viewing an SVG Shelf Item, I want a Shell Visual or generic icon fallback, so that unsupported thumbnail rendering does not block intake.
21. As a user, I want image content validated rather than trusted from its extension, so that misleading filenames cannot bypass format rules.
22. As a user dragging an image with the wrong extension, I want the extension repaired to match the final validated content, so that the destination receives a truthful filename.
23. As a user dragging an image with a safe basename, I want that basename preserved, so that the file remains recognizable.
24. As a user dragging an image with path components or unsafe Windows filename characters, I want a safe basename produced, so that the Managed File can be stored and copied reliably.
25. As a user dragging an image with no usable basename, I want a timestamped `Dropped image` fallback name, so that valid bytes are not discarded because of missing metadata.
26. As a user creating multiple fallback names in the same second, I want numeric suffixes, so that generated names are deterministic and distinct.
27. As a user dropping multiple source images with the same filename, I want each source name preserved, so that DropCove does not rename valid browser metadata unnecessarily.
28. As a user, I want same-name images isolated by Shelf Item identity, so that their backing files cannot overwrite each other.
29. As a user dropping exactly 20 image descriptors, I want the supported entries evaluated, so that the documented count boundary is usable.
30. As a user dropping more than 20 descriptors, I want content after the first 20 left unopened and counted as skipped, so that intake work remains bounded and deterministic.
31. As a user dropping an image exactly 100 MiB in encoded bytes, I want it eligible for acceptance, so that the per-image boundary is precise.
32. As a user dropping an image over 100 MiB, I want that entry skipped with aggregate feedback, so that one stream cannot consume unbounded storage or time.
33. As a user dropping images whose accepted bytes total exactly 500 MiB, I want the boundary handled predictably, so that the per-drop limit is precise.
34. As a user dropping an entry that would exceed the 500 MiB accepted total, I want that entry skipped while a later smaller entry may still be accepted, so that fit-in-order behavior is useful and deterministic.
35. As a user, I want raster validation bounded to 512 MiB of decoded allocation per entry, so that compressed images cannot cause unbounded memory growth.
36. As a user dropping malformed image bytes, I want the entry skipped rather than published, so that corrupt Managed Files do not enter the Shelf.
37. As a user dropping a format whose validator cannot honor the resource bound, I want the entry skipped, so that safety limits remain real rather than advisory.
38. As a user dropping a mix of valid and unsupported images, I want the valid entries accepted in order, so that one unsupported neighbor does not discard useful content.
39. As a user receiving partial success, I want one `Added N images; skipped M.` status, so that I understand the aggregate result without notification spam.
40. As a user whose first 20 descriptors contain no acceptable image, I want no Shelf Batch created and one categorized warning, so that rejection is visible without mutating Shelf state.
41. As a user, I want invalid or over-limit rejection distinguished from security, I/O, and persistence failure categories, so that the feedback is actionable without exposing low-level implementation details.
42. As a user, I want image intake to avoid a separate progress or cancellation window, so that the native drag workflow remains compact and does not add another modal lifecycle.
43. As a user performing a large allowed drop, I want extraction bounded and one final aggregate result, so that the absence of progress UI cannot create unbounded work.
44. As a user dragging a website image, I want Windows Attachment Services applied every time, so that internet-origin content follows Windows trust policy.
45. As a security-conscious user, I want attachment security to have no setting, domain allowlist, retry-without-scan action, or keep-anyway bypass, so that the policy remains mandatory.
46. As a user dragging an image with an explicit HTTP(S) `CFSTR_INETURLW/A` source, I want that URL used as security-zone evidence, so that Windows can apply the appropriate policy.
47. As a user dragging an image without an accepted source URL, I want the Restricted Zone default, so that missing provenance fails toward the safer Windows policy.
48. As a user dragging companion plain text or HTML, I want DropCove to ignore it as provenance, so that captions and markup cannot be mistaken for security evidence.
49. As a user, I want DropCove to avoid fetching the image URL, so that intake does not depend on cookies, authentication, redirects, network availability, or changed remote content.
50. As a user, I want Attachment Services to run without opening a security prompt, so that the Drop Shelf and no-activate Edge Rail do not gain unpredictable focus or modal behavior.
51. As a user whose trust service requires interaction or rejects the file, I want the complete drop rejected, so that DropCove fails closed.
52. As a user whose antivirus deletes or invalidates a staged image, I want the complete drop rejected and staging cleaned, so that pre-scan bytes cannot bypass the security result.
53. As a user whose trust service changes an image into another supported type, I want the final valid bytes accepted with a repaired extension, so that the authoritative security result is preserved.
54. As a user, I want final post-scan animation and metadata preserved, so that DropCove does not perform a second transformation after Windows security processing.
55. As a user, I want successful Attachment Services policy honored even when Windows policy or the profile filesystem omits `Zone.Identifier`, so that valid enterprise or filesystem policy does not disable the feature.
56. As a user, I want zone metadata preserved whenever the filesystem supplies it, so that Managed File storage and later Copy retain available security evidence.
57. As a user, I want one operational, security, or persistence failure to reject the entire image drop, so that a Shelf Batch never represents an incomplete successful operation.
58. As a user, I want every failed image drop to remove all uncommitted staging and final remnants, so that Profile Data does not accumulate orphaned files.
59. As a user, I want a crash during multi-image intake to restore either the complete Ready batch or no visible batch, so that restart cannot expose a partial operation.
60. As a user, I want Staged image items hidden from the Drop Shelf, Edge Rail, Manage Items, sizing, visuals, and drag-out, so that incomplete ownership is never presented as accepted content.
61. As a user restarting DropCove, I want accepted Temporary and Pinned image Managed Files restored with exact names, bytes, order, and Pin state, so that restart does not lose website images.
62. As a user updating DropCove, I want retained image Managed Files and pending cleanup preserved as Profile Data, so that an update does not destroy content or responsibility.
63. As a user uninstalling and reinstalling DropCove, I want retained image Managed Files preserved under the existing profile contract, so that reinstall can restore them.
64. As a user whose Shelf database requires Recovery Backup, I want image Managed Files, staging evidence, database files, and sidecars preserved together, so that recovery evidence remains internally meaningful.
65. As a user dragging an accepted image into another application, I want native Windows Copy behavior with the final filename and exact final bytes, so that it behaves like a normal file.
66. As a user completing Copy for a Temporary image, I want its Shelf Item consumed and its Managed File permanently cleaned, so that used content does not accumulate.
67. As a user completing Copy for a Pinned image, I want the Shelf Item and Managed File retained, so that I can reuse it.
68. As a user canceling or receiving a rejected drag-out, I want the image Shelf Item and Managed File retained, so that failed use cannot lose content.
69. As a user removing an image item or batch, I want only DropCove-owned Managed Files permanently deleted, so that external sources remain safe.
70. As a user clearing Temporary Items, I want Temporary image Managed Files deleted while Pinned Items remain, so that image cleanup follows the existing lifecycle contract.
71. As a user whose image Managed File is locked, I want the Shelf mutation committed and durable pending deletion retried later, so that Windows sharing behavior does not block Shelf use.
72. As a user dropping the same image bytes twice, I want two intentional batches, so that DropCove does not hide repeated actions through hashing or shared backing.
73. As a user, I want ordinary file/folder drops to remain external path references, so that website-image support does not change deletion authority or duplicate rules.
74. As a user, I want valid filesystem paths to retain precedence over Website Image Virtual Files, so that existing storage drops remain unchanged.
75. As a user, I want an invalid higher-priority path or virtual-image family rejected rather than falling through to companion text or URL, so that one drag cannot be reinterpreted unexpectedly.
76. As a user, I want URL-only, HTML-only, rich-text, bitmap-only, and generic virtual-file payloads rejected, so that 0.0.24 remains limited to approved representations.
77. As a user, I want plain-text intake to continue unchanged after image support is added, so that 0.0.23 behavior does not regress.
78. As a Shake-to-open user, I want an accepted image batch to keep the transient Shelf visible and a rejected image drop to restore the prior display state, so that image intake follows the established Shake contract.
79. As an Edge Rail user, I want image intake and status feedback to preserve no-activate behavior, so that foreground focus remains with my current application.
80. As a user, I want accepted raster images to use the established Shell Visual path, so that thumbnails and icon fallback remain consistent with ordinary Shelf Items.
81. As a user, I want Manage Items, accessibility names, availability, sizing, scrolling, appearance, and reduced-motion behavior preserved, so that image support composes with existing surfaces.
82. As a maintainer, I want `IDataObject` and borrowed COM media confined to the registered window thread, so that COM lifetime and apartment rules remain explicit.
83. As a maintainer, I want stream contents copied directly into bounded owned staging files rather than one large memory buffer, so that the encoded-byte limits also bound allocation and copying.
84. As a maintainer, I want one ordered multi-item Managed File mutation behind Core Shelf State, so that presentation, SQLite, storage, and cleanup cannot publish divergent batches.
85. As a maintainer, I want batch-level startup reconciliation for interrupted image intake, so that one failed participant cannot leave a partial Ready batch.
86. As a maintainer, I want existing Managed File ownership records to remain the only deletion authority, so that filename, extension, path location, source URL, or browser identity can never grant deletion rights.
87. As a maintainer, I want image validation and Attachment Services behind deep native modules, so that OLE parsing, trust policy, and format details do not spread through presentation or lifecycle code.
88. As a release reviewer, I want real current-stable Edge and Chrome drags on both surfaces, so that source-code compatibility is not mistaken for runtime acceptance.
89. As a release reviewer, I want installed Copy drag-out prove exact final bytes, names, animation, metadata, and zone behavior, so that unit tests are not mistaken for the shipped product.
90. As a release owner, I want version metadata to remain `0.0.23` during behavior work and become `0.0.24` only at the approved release gate, so that release sequencing stays auditable.
91. As a project owner, I want specification, ticket approval, implementation authorization, candidate acceptance, and Git commit authorization to remain separate decisions, so that planning cannot imply deployment or commit approval.

## Implementation Decisions

- Scope this specification to roadmap version `0.0.24`: Website Image Virtual File intake, image validation, mandatory Attachment Services, aggregate feedback, and reuse of the Managed File lifecycle delivered in `0.0.23`. Multi-batch selection remains assigned to `0.0.25`.
- Preserve ordinary file and folder intake as external filesystem references. Existing path extraction, ordering, repeated-path filtering, retained-batch duplicate suppression, source safety, and Copy-only drag-out behavior remain unchanged.
- Keep the established representation priority: valid filesystem paths first, otherwise Website Image Virtual Files, otherwise non-empty Unicode text. Never combine families or fall through from an invalid higher-priority family to companion URL or text.
- Extend the Windows OLE Adapter to recognize `CFSTR_FILEDESCRIPTORW` with indexed `CFSTR_FILECONTENTS`. Source application identity is not part of eligibility. Current-stable Edge and Chrome are the release acceptance matrix; other compatible sources are best-effort.
- Keep the live `IDataObject`, descriptor memory, `STGMEDIUM`, and borrowed COM interfaces on the registered receiving window thread. Copy accepted file contents in bounded chunks into owned staging files before returning ownership to background trust, validation, and persistence work.
- Support the `IStream` and global-memory content media used by ordinary virtual-file transfers. Reject unsupported media rather than materializing generic virtual files. Storage-object media are outside `0.0.24` unless required by the approved Edge/Chrome acceptance matrix.
- Validate the descriptor group allocation, declared count, filename termination, index access, and arithmetic before reading entries. Examine only the first 20 descriptors; do not request content for later descriptors.
- Enforce 100 MiB against actual encoded bytes read for each candidate, not only descriptor metadata. Stop and skip an entry as soon as it exceeds the limit.
- Enforce 500 MiB against accepted encoded bytes in descriptor order. Skip an entry whose inclusion would exceed the total, then continue evaluating later descriptors among the first 20 when they can fit.
- Add one deep image-validation module that identifies content independently of extension and validates PNG, JPEG, GIF, WebP, AVIF, SVG, BMP, and ICO with bounded parsers. The module returns the validated type and canonical extension or a categorized unsupported/invalid/resource-limit result.
- Bound raster validation to 512 MiB of decoded allocation per entry. A decoder that cannot enforce the limit is not eligible for acceptance. Validation must not rasterize merely to generate presentation thumbnails.
- Validate SVG as well-formed XML with DTD and entity expansion disabled. Preserve raw bytes, scripts, event handlers, and external references; do not sanitize, execute, or render SVG in any DropCove WebView.
- Derive a safe filename from the descriptor basename only. Remove path components, invalid/control characters, trailing dots/spaces, and Windows reserved device-name hazards. Preserve a usable basename, replace the extension with one matching final validated content, and generate a local-time `Dropped image YYYY-MM-DD HH-mm-ss[-N].ext` fallback when no safe basename remains.
- Preserve duplicate source filenames. Per-item Managed File directories provide collision isolation; only generated fallback names receive deterministic numeric suffixes.
- Add one attachment-security adapter around non-UI `IAttachmentExecute::Save`. Every candidate that passes pre-scan validation must use this adapter; there is no setting, allowlist, bypass, interactive prompt, or retry-without-scan path.
- Obtain optional provenance only from `CFSTR_INETURLW` or `CFSTR_INETURLA`. Accept only `http` and `https` schemes. Do not parse provenance from Unicode text, HTML, descriptor names, or file contents. Use Attachment Services' Restricted Zone default when no accepted source exists.
- Treat successful Attachment Services policy as authoritative even when Windows policy or the profile filesystem does not create a `Zone.Identifier` stream. Preserve security metadata when the filesystem supplies it; never synthesize a substitute zone record.
- Revalidate each file after Attachment Services. If the final file is missing, invalid, unsupported, or outside any limit, fail the complete drop. If it remains a supported image but its bytes or type changed, preserve the exact final bytes, animation, and metadata and repair the final extension.
- Perform pre-scan staging, Attachment Services, and post-scan validation before creating durable Shelf ownership. Any failure before the Core Shelf State mutation removes all candidate staging files and creates no database rows.
- Add one Core Shelf State mutation that accepts an ordered collection of fully prepared Managed File candidates as one Shelf Batch. Reuse the existing `managed_files` relation and per-item storage; no second ownership model or browser-specific schema is introduced.
- Use one grouped crash-recoverable sequence: commit the batch, all items, and all ownership rows as `Staged` in one SQLite transaction; promote every staged file to its final per-item path; mark every ownership row `Ready` in one SQLite transaction; then publish the accepted snapshot and status.
- If any promotion or `Ready` commit fails, remove the complete unprojected batch, record cleanup responsibility for every owned staging/final path, restore empty-Shelf sizing when applicable, attempt cleanup, and return an operational failure. Never publish successful siblings from the failed batch.
- Reconcile interrupted image intake by Shelf Batch before the first projection. A staged image batch becomes visible only when every participant can be reconciled to one valid final file and all rows become Ready atomically; otherwise remove the entire unprojected batch and retain durable cleanup responsibility.
- Generalize Managed File path validation from text-only filenames to the approved image extensions while retaining per-item identity, relative-path validation, Windows reserved-name rejection, reparse-point protection, and ownership-derived deletion authority.
- Keep final Managed Files under Profile Data and reuse the existing Temporary/Pinned, Successful Drag-Out, Remove Item, Remove Batch, Clear Temporary Items, confirmed-Missing, Pending Managed File Deletion, retry, update, uninstall, and Recovery Backup behavior.
- Preserve each final Managed File exactly after post-scan validation. Do not re-encode, strip metadata, flatten animation, hash content, deduplicate by bytes, share backing files, or retain pre-scan copies.
- Return one ordered result containing accepted count, skipped count, and optional categorized failure. Partial success publishes one batch and one non-modal `Added N images; skipped M.` status. A fully invalid/over-limit family creates no batch and reports one non-modal validation warning.
- Treat descriptor/content I/O failure, Attachment Services failure, required security interaction, staging failure, SQLite failure, promotion failure, or Ready failure as complete-drop operational failure. Clean all candidates and report one categorized non-modal warning.
- Reuse existing Shelf synchronization, warnings, and `shelf-error` presentation paths on both Drop Shelf and Edge Rail. Do not add frontend-owned intake state, a new window, a modal, a progress surface, a cancel action, a setting, or a second status channel.
- Preserve Drop Placeholder, automatic/manual sizing, newest-batch insertion, scrolling, Edge Rail no-activate behavior, Clear-confirmation blocking, application-owned drag rejection, Shake promotion/restoration, fullscreen handling, appearance, accessibility, and reduced-motion contracts.
- Preserve existing Shell Visual extraction for supported raster images. SVG never enters WebView rendering and may use a generic Shell icon fallback.
- Keep package, crate, lockfile, Tauri, PE, About, installer, and uninstall metadata at `0.0.23` during behavior implementation. Synchronize to `0.0.24` only in an approved release ticket after behavior acceptance.
- Do not create a Git commit without separate explicit user authorization.

## Testing Decisions

- Primary seam: the actual installed Windows Resident Instance with the real native OLE drop target, current-stable Edge and Chrome, real Drop Shelf and expanded Edge Rail WebView2 surfaces, owned test Profile Data, Windows Attachment Services, and a real native Copy destination. This highest seam proves browser payloads, COM extraction, security processing, Core Shelf State, SQLite, filesystem ownership, presentation, drag-out, and cleanup compose correctly.
- Supporting deterministic coverage reuses the existing native payload-evaluation seam and Core Shelf State temporary-profile SQLite/filesystem seam; these do not become alternate product acceptance definitions.
- Use the existing native payload-evaluation seam for deterministic permanent coverage of descriptor parsing, representation precedence, content indexing, URL extraction, medium handling, and drop eligibility. Tests should construct controlled `IDataObject` behavior and assert externally meaningful eligibility/results rather than private COM helper call counts.
- Use the existing Core Shelf State integration seam with real temporary SQLite databases and real temporary directories for deterministic permanent coverage of ordered multi-item Managed File acceptance, `Staged` → `Ready`, lifecycle mutations, rollback, cleanup, restart, and Recovery Backup.
- Reuse the existing Managed File round-trip, exact-byte, accepted-Copy consumption, Pinned retention, sharing-violation retry, mixed external/managed safety, recovery preservation, startup reconciliation, failed-Ready compensation, Remove, and Clear regressions as prior art.
- Add deterministic descriptor coverage for zero entries, malformed group sizes, unterminated names, declared-count/allocation mismatch, invalid indices, arithmetic overflow, first-20 handling, and unopened descriptors after index 19.
- Add content-medium coverage for accepted stream and global-memory forms, short reads, zero-progress reads, declared-size mismatch, source failure during read, and unsupported media rejection. Assert staging cleanup and no Shelf mutation after failure.
- Add representation-precedence coverage proving valid filesystem paths beat virtual images; invalid offered filesystem data does not fall through; virtual images beat companion URL/text; invalid virtual images do not fall through; plain text remains accepted only when no higher family is offered.
- Add format behavior coverage for PNG, JPEG, GIF, WebP, AVIF, SVG, BMP, and ICO using real valid fixtures. Assert final type, exact bytes, extension, name, item order, one batch, and Managed File ownership.
- Use minimal, purpose-built fixtures that exercise public behavior. Do not add large binary fixture collections when generated or tiny canonical files can prove the same boundary.
- Add malformed and extension/content mismatch coverage for every supported family. Assert content-derived type and extension repair rather than trusting the descriptor suffix.
- Add raster resource coverage for the 512 MiB decoded-allocation boundary and decoder paths that cannot guarantee the bound. Tests must avoid allocating the full limit when a controlled decoder seam can prove enforcement.
- Add SVG coverage for well-formed raw preservation, DTD rejection/disablement, entity-expansion rejection, retained script/external-reference text, `.svg` extension repair, and absence of any WebView SVG rendering path.
- Add filename coverage for path components, invalid/control characters, trailing spaces/dots, reserved device names, missing names, Unicode basenames, long names, extension replacement, fallback timestamp collisions, and duplicate source filenames in one batch.
- Add exact encoded-byte coverage for 100 MiB accepted and one byte over skipped without requiring permanent giant fixtures; use bounded generated streams. Add 500 MiB accepted-total and fit-in-order skip coverage through controlled stream lengths.
- Add first-20 behavior coverage proving descriptor 21 content is never requested and contributes to the skipped count.
- Add partial-success coverage where supported entries surround unsupported, malformed, per-image-over-limit, and total-over-limit entries. Assert one ordered batch, exact accepted/skipped counts, one aggregate status, and no extra warnings.
- Add complete-failure coverage for content I/O, staging write/flush, Attachment Services, required interaction, post-scan deletion, post-scan invalidation, SQLite Staged commit, promotion, and Ready commit. Assert no visible batch, no partial Ready rows, no unowned files, and one categorized warning.
- Add Attachment Services coverage using an injectable native adapter for deterministic success, rejection, file deletion, byte mutation, type mutation, absent ADS under successful policy, and security metadata preservation where available. The permanent tests assert final behavior; installed smoke supplies real Windows-policy evidence.
- Add provenance coverage for HTTP and HTTPS `CFSTR_INETURLW/A`, unsupported schemes, malformed URLs, absent URLs, and companion text/HTML that resembles a URL. Assert explicit formats only and Restricted Zone fallback.
- Add grouped crash-window coverage before durable ownership, after the grouped Staged transaction, during multi-file promotion, after all promotions, and before the grouped Ready transaction. Startup must yield either the complete ordered Ready batch or no visible batch with cleanup responsibility.
- Add lifecycle coverage for Temporary accepted Copy cleanup, Pinned Copy retention, cancellation/rejection retention, explicit item and batch removal, Clear Temporary Items, confirmed-Missing cleanup, sharing-violation retry, restart, update/reinstall/uninstall profile preservation, and Recovery Backup with image Managed Files.
- Assert external source safety with mixed external and image Managed File batches. Ordinary source files and folders must remain byte-for-byte and structurally unchanged after every image lifecycle action and failure path.
- Exercise actual current-stable Edge and Chrome image drags on both surfaces for every supported format that the browser can originate, plus multiple descriptors, duplicate names, missing/unsafe names, corrupt data, URL/text companions, and large bounded payloads. Record browser/version/source-page evidence.
- Exercise installed drag-out into at least one real native filesystem destination and one accepting application where practical. Verify exact final filename, bytes, animation, metadata, Copy result, Temporary consumption, and Pinned retention.
- Observe real Attachment Services behavior for explicit HTTP(S) source and Restricted Zone, antivirus/trust rejection where the workstation can produce it, and `Zone.Identifier` presence or policy-authoritative absence. Report unexercised policy conditions rather than inferring them.
- Measure and record worst observed accepted large-drop latency for Edge and Chrome. Verify no progress/cancel window appears, both surfaces remain in their established lifecycle, and one aggregate result is eventually presented.
- Exercise Shake-to-open accepted, partially accepted, unsupported, security-failed, and operational-failed image drops. Only accepted batches promote the transient Shelf; all other outcomes restore the prior Hidden or EdgeDocked state.
- Exercise Dark, Light, and Windows Contrast presentation for raster Shell Visuals, SVG fallback, partial-success status, full-failure warning, Manage Items, and both Shelf surfaces without changing existing geometry or focus behavior.
- Good permanent tests catch consumer-visible failures: wrong representation chosen, wrong file bytes/type/name/order, unsafe SVG execution path, bypassed security, missing zone handling, excessive resource use, partial batch publication, leaked staging, lost retained image, deleted external source, broken lifecycle cleanup, duplicate warning, changed focus, or incorrect Shake restoration.
- Do not add tests for SQL text, helper call order, copied command forwarding, private parser branches, exact dependency types, source-code strings, CSS classes, mocked byte echoes, non-empty collections, or bare not-throw behavior. Assert authoritative Shelf state, real file/database outcomes, and observable native/UI results.
- Run the existing frontend Node regressions, frontend typecheck/production build, native Rust regressions, cargo-xwin Windows release/test compilation, and the freshly produced native Windows regression executable directly.
- After behavior and ticket acceptance, synchronize version `0.0.24`, build one exact current-user Windows x64 NSIS candidate with cargo-xwin, run it from installed embedded assets with the development server unavailable, and exercise startup, secondary activation, tray/hotkey access, Cooperative Shutdown, reinstall, uninstall, and Profile Data preservation.
- Record physical or policy-dependent limits—browser source behavior, antivirus products, enterprise Attachment Services policy, ADS support, cross-integrity drag, actual Windows Contrast Theme, high/mixed DPI, and large-stream timing—as observed or unobserved. Never infer runtime acceptance from source code or deterministic tests alone.

## Out of Scope

- Firefox acceptance in `0.0.24`. Firefox's native-image fallback can expose a browser-owned Windows temporary path through `CF_HDROP`, which conflicts with the approved external-path ownership semantics and requires a separate product decision.
- HTTP downloads, URL fallback, cookies, referrer forwarding, authentication, redirects, remote retries, cache revalidation, or any network request by DropCove.
- Generic virtual-file intake, arbitrary email attachments, office objects, storage-object media not required by Edge/Chrome, URL items, HTML/RTF files, bitmap-only clipboard capture, clipboard history, editable image items, or arbitrary file import.
- Image editing, conversion, optimization, resizing, recompression, metadata stripping, animation flattening, color-profile conversion, SVG sanitization, or active-content rewriting.
- Rendering SVG in a DropCove WebView, adding an SVG previewer, or guaranteeing a thumbnail when Windows Shell cannot produce one.
- More than the first 20 descriptors, images over 100 MiB, accepted totals over 500 MiB, raster validation over 512 MiB decoded allocation, storage quotas, quota-management UI, or user-configurable limits.
- Security prompts, user overrides, trusted-domain lists, retry-without-scan, keep-anyway actions, Attachment Services settings, custom antivirus integration, or synthetic zone metadata.
- Intake progress UI, cancellation UI, a new window, modal confirmation, background network progress, or a second status channel.
- Content hashing, byte-level duplicate suppression, shared backing files, reference counting, source-name suffixing, cross-batch Managed File reuse, or deduplicated storage.
- Multi-batch selection and drag-out, selected-batch preparation, cross-batch outbound path deduplication, or any `0.0.25` capability.
- Changing existing external path intake, Unicode text behavior, Managed File deletion authority, Pin semantics, Clear confirmation, Drop Shelf/Edge Rail geometry, Shake detector thresholds, appearance palette, accessibility model, virtualization, update behavior, or non-Windows support.
- A compatibility shim, deprecated image path, browser-specific ownership field, alternate schema, or second lifecycle implementation.
- Creating implementation tickets, beginning implementation, synchronizing release metadata, packaging a candidate, accepting a release, or creating a Git commit as part of this specification step.

## Further Notes

The feature deliberately consumes bytes already offered by the drag source. It is not a downloader. This keeps browser authentication, cookies, redirects, network availability, and remote mutation outside DropCove while preserving the exact final file that passed Windows attachment policy.

Holdem is useful as a comparator for practical browser-drop behavior, especially its handling of temporary paths, but its temporary-folder lifecycle is not reused. DropCove retains the stricter Managed File ownership, durable cleanup, Recovery Backup, and Attachment Services contracts established in `0.0.23`.

The testing strategy has one primary product seam: the real installed Windows Resident receiving real Edge/Chrome drags and copying accepted files into a real destination. Existing native `IDataObject` and Core Shelf State seams provide deterministic boundary and failure coverage without becoming alternate acceptance definitions.

The central new invariant is grouped visibility: one image drag creates either one complete ordered Ready Shelf Batch or no visible batch. Per-item Managed File rows remain the ownership authority, while Shelf Batch identity supplies the recovery boundary for an interrupted multi-image intake.

This specification is published as `ready-for-agent` after explicit testing-seam confirmation. It does not authorize tickets, implementation, release metadata changes, packaging, acceptance, Git commits, or deployment. The next planning step after specification approval is `to-tickets`.