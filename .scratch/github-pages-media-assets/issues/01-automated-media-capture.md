# 01: Automated Drop Shelf Media Capture Tooling & Asset Verification

**What to build:** An automated runner that renders the Drop Shelf surface with realistic mock Shelf Batches in Dark Appearance Mode, captures a high-resolution static lossless PNG preview image and an optimized WebM demonstration video, and verifies through automated test assertions that the generated media assets exist within size and format constraints.

**Blocked by:** None (can start immediately)

Status: resolved

- [x] Maintainer can execute a single project command to run headless browser automation against the Drop Shelf surface without starting a resident Windows instance.
- [x] Mock data layer supplies three distinct Shelf Batches: an image reference with preview thumbnail, an archive file reference, and a text snippet.
- [x] Runner outputs a high-resolution static preview image in lossless PNG format and an animated workflow demonstration in WebM format under two megabytes.
- [x] Automated test asserts that generated media assets exist with non-zero file sizes and conform to format and size limits.
