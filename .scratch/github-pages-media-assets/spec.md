Status: ready-for-agent

# Specification: GitHub Pages Media Showcase & Automated Capture

## Problem Statement

Prospective users visiting the DropCove GitHub Pages site cannot see what DropCove looks like or how it functions before downloading and installing it. The current landing site contains text descriptions and download links, but lacks visual media. Creating screenshots and screen recordings manually is time-consuming, prone to personal workstation clutter leaks, and difficult to keep consistent across releases.

## Solution

An automated capture pipeline using headless browser automation renders the Drop Shelf surface with realistic mock Shelf Batches in Dark Appearance Mode, producing both high-resolution static images and a lightweight video workflow demonstration. The GitHub Pages landing site integrates this media into the hero presentation with full accessibility and reduced-motion support.

## User Stories

1. As a prospective user visiting the website, I want to see a visual preview of the Drop Shelf in action, so that I can immediately understand what the product looks like without installing it.
2. As a prospective user with limited bandwidth, I want visual media on the landing page to load quickly without slowing down page navigation, so that I have a smooth browsing experience.
3. As a user with motion sensitivity, I want animations and video playback to respect my system reduced-motion preference, so that browsing the landing page does not cause discomfort.
4. As a visitor on a high-density display, I want the Drop Shelf screenshot to render crisp and sharp without pixelation, so that the UI details remain legible.
5. As a maintainer, I want a single command to generate updated screenshots and video assets, so that I do not need to manually record screen captures or coordinate multi-terminal dev setups.
6. As a maintainer, I want the automated capture process to mock all resident Tauri IPC commands, so that the presentation surfaces render deterministically without requiring a running Windows resident instance or commercial development tools.
7. As a maintainer, I want media assets committed directly to the static site repository, so that landing page builds on continuous integration do not incur heavy browser download overhead.
8. As a visitor browsing the features, I want the demo video to show realistic Shelf Batches—including file references, image thumbnails, and text snippets—so that I understand the multi-format capability of the Shelf.
9. As a visitor reading the hero section, I want the demo video to show item hover inspection and selection actions, so that I can observe the core interaction model in transit.
10. As a visitor whose browser blocks autoplaying media, I want a clean static poster image displayed in place of the video, so that the hero section remains complete and visually informative.

## Implementation Decisions

- Media Showcase Placement: The landing page hero section incorporates a media showcase container beneath the primary download action. The container hosts an HTML5 video element with muted, looped, inline, and autoplay attributes, paired with a high-resolution poster image.
- Accessibility & Motion Reduction: Media presentation honors system motion preferences via CSS media queries. When reduced motion is preferred, video animation is paused or hidden in favor of the static preview.
- Surface Scope: Capture scope is strictly bounded to the floating Drop Shelf surface. Edge Rail, Settings, and Clear Confirmation surfaces are omitted from the showcase media to keep attention focused on core transit capabilities.
- Mock Data Diversity: The mock presentation layer supplies three distinct Shelf Batches: an external image file reference with visual thumbnail, an archive file reference, and a text snippet.
- Headless Automation Seam: Automation drives browser rendering through Playwright against the local Vite application server. An initialization script intercepts window Tauri internals and supplies deterministic mock responses for appearance and shelf queries.
- Standalone Execution Flow: The capture runner orchestrates the entire workflow within a single process lifecycle: spawning an ephemeral local server on an available port, executing headless capture of static and animated sequences, formatting outputs, and shutting down the server cleanly upon completion.
- Asset Specifications: Static preview output is formatted as high-density lossless PNG. Motion showcase output is formatted as optimized WebM with a target total footprint under two megabytes.

## Testing Decisions

- A high-level integration test will verify the presence and non-zero byte size of the generated image and video assets within the website directory.
- The static site builder test suite will be extended to assert that the generated markup includes the video showcase element, proper accessibility attributes, valid poster reference, and reduced-motion styling.
- Good tests will inspect external file outputs and resulting HTML structure rather than testing the internal Playwright runner hooks.
- Prior art: Existing automated site builder tests in the project verification suite.

## Out of Scope

- Capturing secondary surfaces such as Settings, About, or Edge Rail.
- Automating native Windows desktop screen capture via commercial OS recording tools.
- Integrating asset generation into pull-request or branch continuous-integration builds.
- Interactive in-browser client-side drag-and-drop simulations.

## Further Notes

The resulting assets will be checked into source control under the website static asset directory so that standard website deployment scripts can package them directly.
