# 02: Website Hero Media Showcase & Build Verification

**What to build:** The GitHub Pages landing site hero section features a media showcase presenting the animated demonstration video with an accessible high-resolution static poster fallback, responsive styling, and reduced-motion consideration, backed by automated site builder test assertions verifying markup validity and accessibility attributes.

**Blocked by:** 01: Automated Drop Shelf Media Capture Tooling & Asset Verification

Status: resolved

- [x] Hero section of the landing page displays the Drop Shelf media showcase container beneath the primary download action.
- [x] Showcase incorporates an HTML5 video element with muted, loop, autoplay, and inline playback attributes, linked to the generated WebM demonstration video.
- [x] Showcase defines the generated static preview image as the poster fallback for devices blocking autoplay or before playback starts.
- [x] Styling pauses, hides, or replaces video animation with the static preview when system reduced-motion preference is active.
- [x] Static site builder tests assert the presence of showcase markup, valid poster and video source references, and required accessibility attributes in the built site output.
