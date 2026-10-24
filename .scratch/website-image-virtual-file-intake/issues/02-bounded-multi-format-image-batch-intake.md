# 02: Bounded multi-format Website Image batch intake

**What to build:** Extend the working single-PNG Drop Shelf round trip into the complete bounded Website Image Virtual File family: all approved image formats, ordered multi-image batches, safe and truthful names, deterministic count/byte/resource limits, exact final-byte preservation, fit-in-order partial acceptance, and one aggregate result. The user can drop a mixed image payload and receive one coherent Shelf Batch containing every acceptable entry and no unsupported neighbor.

**Blocked by:** 01: Single-PNG Website Image Drop Shelf round trip.

**Status:** resolved

**Parent specification:** [DropCove 0.0.24 — Website Image Virtual-File Intake](../spec.md)

**Testing seam:** Actual Windows Resident Instance on the Drop Shelf with real Edge/Chrome payloads where available, a controlled native multi-descriptor OLE source for deterministic boundaries, owned Profile Data, real Windows Attachment Services, native Copy drag-out, and Core Shelf State SQLite/filesystem regressions.

**Demo path:** Drop an ordered mixed payload containing valid PNG/JPEG/GIF/WebP/AVIF/SVG/BMP/ICO entries, duplicate and unsafe names, unsupported/corrupt neighbors, and an entry that would exceed the accepted total; observe one ordered Shelf Batch, one aggregate skip status, exact final bytes, truthful names, native Copy, and no SVG WebView rendering.

- [x] The single-PNG round trip from Ticket 01 remains fully working, including path/text precedence, mandatory Attachment Services, restart, drag-out, Pinning, and cleanup.
- [x] The adapter validates descriptor-group allocation, count, filename termination, index access, and arithmetic before requesting file contents.
- [x] Only the first `20` descriptors are examined in source order; descriptor 21 and later content is never requested and is counted as skipped.
- [x] PNG, JPEG, GIF, WebP, AVIF, SVG, BMP, and ICO are identified from content rather than extension and accepted through the same Managed File workflow.
- [x] Raster validators enforce the `512 MiB` decoded-allocation bound; malformed input and validators that cannot guarantee the bound produce skipped entries rather than unbounded work.
- [x] SVG accepts raw well-formed XML with DTD and entity expansion disabled, preserves scripts and external references byte-for-byte, receives an `.svg` extension, and is never rendered inside a DropCove WebView.
- [x] Each entry enforces a `100 MiB` actual encoded-byte limit while streaming; exactly the limit is eligible and one byte over is skipped with complete candidate cleanup.
- [x] The drop enforces a `500 MiB` accepted encoded-byte total in descriptor order; an entry that would exceed the total is skipped and a later smaller entry among the first 20 may still be accepted.
- [x] A safe descriptor basename is preserved, path components and unsafe Windows filename forms are removed, and the extension is repaired to match final validated content.
- [x] Missing or unusable basenames receive `Dropped image YYYY-MM-DD HH-mm-ss[-N].ext` names with deterministic collision suffixes.
- [x] Duplicate valid source filenames remain unchanged and coexist through per-item Managed File identity without overwriting each other.
- [x] Animation, metadata, color information, and all other final post-scan bytes remain exact; DropCove does not re-encode, optimize, sanitize, flatten, strip metadata, hash, deduplicate, or share backing files.
- [x] Unsupported, malformed, per-image-over-limit, total-over-limit, and beyond-first-20 entries are skipped in the approved fit-in-order semantics.
- [x] When at least one entry remains, every accepted entry forms one ordered Shelf Batch and the surfaces receive exactly one `Added N images; skipped M.` result.
- [x] When no entry remains after validation/limits, no Shelf Batch, sizing mutation, scroll mutation, staging residue, final file, or Shake promotion occurs, and one categorized validation warning is reported.
- [x] An image family with companion URL, Unicode text, or HTML remains the selected family; representations are never combined and invalid higher-priority image input never falls through.
- [x] URL-only, HTML-only, rich-text, bitmap-only, generic virtual-file, and unsupported storage-object payloads create no Shelf state.
- [x] All accepted candidates pass mandatory non-interactive Attachment Services and post-scan validation before durable Shelf ownership is created.
- [x] One multi-image drop uses one grouped `Staged` transaction, promotes every accepted candidate, uses one grouped `Ready` transaction, and publishes only the complete ordered batch.
- [x] Successful multi-item native Copy transfers exact names and bytes; Temporary participants are consumed and cleaned while Pinned participants remain under established lifecycle rules.
- [x] Shell Visuals use the established raster thumbnail/icon path; SVG and Shell extraction failure use the approved fallback without blocking acceptance.
- [x] Permanent tests use small canonical or generated fixtures and controlled streams rather than a large binary fixture collection or real 100/500/512 MiB allocations.
- [x] Real or controlled Drop Shelf demos prove every supported type, duplicate/fallback naming, first-20 handling, exact limits, fit-in-order skipping, aggregate feedback, exact drag-out bytes, and unchanged external/plain-text workflows.
- [x] Frontend regressions, native regressions, frontend production build, and cargo-xwin Windows build pass while release metadata remains `0.0.23`.
- [x] No Git commit is created without separate explicit user authorization.

## Verification evidence

- All eight formats were dragged from an actual Chrome window into the real Drop Shelf, each producing the original offered filename and exact bytes. Animated GIF and WebP bytes remained unchanged; raw SVG was accepted without WebView rendering.
- A controlled native OLE source offered PNG, animated GIF, and an unsupported neighbor through indexed stream/global-memory media. The actual Shelf showed one ordered two-item batch and `Added 2 images; skipped 1.`. Native Copy into a real Windows Forms destination preserved both files exactly and consumed/deleted both Temporary Managed Files.
- Native tests cover all eight formats in one ordered batch, duplicate source names, fallback collisions, unsupported neighbors, first-20 content requests, exact 100 MiB/one-byte-over behavior, exact 500 MiB accepted total, fit-in-order later-small-entry acceptance, and huge raster dimensions without allocating a huge pixel buffer.
- Secure SVG regression was observed red for malformed attribute entities, CDATA outside the root, duplicate attributes, and invalid numeric references, then green after parser validation. Namespaced raw SVG with predefined entities and script content stays byte-exact. No additional namespace allowlist or SVG sanitization policy was introduced.
- Latest targeted native command passed **13/13** tests (the 11 virtual-file tests plus two existing native-failure regressions). Ticket 01's full native **154/154** and frontend **13/13** baseline gates passed; final full-suite rerun remains at release integration. Cargo-xwin build and production frontend assets succeeded with version `0.0.23`.
