import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

const PREVIEW_IMAGE = path.resolve("website/assets/shelf-preview.png");
const DEMO_VIDEO = path.resolve("website/assets/dropcove-demo.webm");
const TOUR_IMAGES = [
  "tour-shake.webp",
  "tour-edge-rail.webp",
  "tour-manage-items.webp",
  "tour-pin-dragout.webp",
  "tour-settings.webp",
];
const MAX_VIDEO_BYTES = 4 * 1024 * 1024; // 4 MB for full 56s 720p WebM walkthrough
const MIN_HIDPI_WIDTH = 880; // >= 2x CSS width

function readPngDimensions(buffer) {
  const pngSig = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  assert.ok(buffer.subarray(0, 8).equals(pngSig), "Expected preview image to have valid PNG signature");
  assert.equal(buffer.toString("ascii", 12, 16), "IHDR", "Expected PNG IHDR chunk");
  return {
    width: buffer.readUInt32BE(16),
    height: buffer.readUInt32BE(20),
  };
}

function readWebmDimensions(buffer) {
  for (let i = 0; i < buffer.length - 8; i++) {
    if (buffer[i] === 0xb0 && (buffer[i + 1] === 0x81 || buffer[i + 1] === 0x82)) {
      const wLen = buffer[i + 1] & 0x7f;
      const width = wLen === 1 ? buffer[i + 2] : (buffer[i + 2] << 8) | buffer[i + 3];
      const j = i + 2 + wLen;
      if (buffer[j] === 0xba && (buffer[j + 1] === 0x81 || buffer[j + 1] === 0x82)) {
        const hLen = buffer[j + 1] & 0x7f;
        const height = hLen === 1 ? buffer[j + 2] : (buffer[j + 2] << 8) | buffer[j + 3];
        return { width, height };
      }
    }
  }
  return null;
}

test("generated website assets exist and meet HiDPI resolution and size constraints", () => {
  assert.ok(fs.existsSync(PREVIEW_IMAGE), `Expected preview image to exist at ${PREVIEW_IMAGE}`);
  const imgBuf = fs.readFileSync(PREVIEW_IMAGE);
  assert.ok(imgBuf.length > 0, "Expected preview image to have non-zero size");
  const imgSize = readPngDimensions(imgBuf);
  assert.ok(imgSize.width >= MIN_HIDPI_WIDTH, `Expected preview image width >= ${MIN_HIDPI_WIDTH}px, got ${imgSize.width}px`);
  assert.equal(imgSize.width * 9, imgSize.height * 16, `Expected 16:9 preview image, got ${imgSize.width}x${imgSize.height}`);

  assert.ok(fs.existsSync(DEMO_VIDEO), `Expected demo video to exist at ${DEMO_VIDEO}`);
  const videoBuf = fs.readFileSync(DEMO_VIDEO);
  assert.ok(videoBuf.length > 0, "Expected demo video to have non-zero size");
  assert.ok(videoBuf.length <= MAX_VIDEO_BYTES, `Expected demo video to be <= 4MB, but was ${videoBuf.length} bytes`);
  const videoSize = readWebmDimensions(videoBuf);
  assert.ok(videoSize, "Expected demo video to contain valid WebM PixelWidth/PixelHeight metadata");
  assert.ok(videoSize.width >= MIN_HIDPI_WIDTH, `Expected demo video width >= ${MIN_HIDPI_WIDTH}px, got ${videoSize.width}px`);
  assert.equal(videoSize.width * 9, videoSize.height * 16, `Expected 16:9 demo video, got ${videoSize.width}x${videoSize.height}`);

  for (const name of TOUR_IMAGES) {
    const filePath = path.resolve("website/assets", name);
    assert.ok(fs.existsSync(filePath), `Expected tour image to exist at ${filePath}`);
    const buf = fs.readFileSync(filePath);
    assert.ok(buf.length > 0, `Expected tour image ${name} to have non-zero size`);
    assert.equal(buf.toString("ascii", 0, 4), "RIFF", `Expected ${name} to have RIFF header`);
    assert.equal(buf.toString("ascii", 8, 12), "WEBP", `Expected ${name} to have WEBP signature`);
  }
});
