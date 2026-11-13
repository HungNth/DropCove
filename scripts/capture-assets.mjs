import { spawn } from "node:child_process";
import { once } from "node:events";
import fs from "node:fs";
import path from "node:path";
import { createServer } from "vite";
import { chromium } from "playwright";

const VIEWPORT = { width: 440, height: 220 };
const DEVICE_SCALE_FACTOR = 3;
const VIDEO_FPS = 24;

/**
 * Encodes a 32-bit BGRA BMP byte array (BITMAPV4HEADER) supporting alpha transparency.
 */
function createBmpVisual(size, drawPixel) {
  const headerSize = 122; // 14-byte file header + 108-byte BITMAPV4HEADER
  const pixelArraySize = size * size * 4;
  const fileSize = headerSize + pixelArraySize;
  const buffer = new Uint8Array(fileSize);
  const view = new DataView(buffer.buffer);

  buffer[0] = 0x42; // 'B'
  buffer[1] = 0x4d; // 'M'
  view.setUint32(2, fileSize, true);
  view.setUint32(10, headerSize, true);

  view.setUint32(14, 108, true);
  view.setInt32(18, size, true);
  view.setInt32(22, size, true);
  view.setUint16(26, 1, true);
  view.setUint16(28, 32, true);
  view.setUint32(30, 3, true); // BI_BITFIELDS
  view.setUint32(34, pixelArraySize, true);
  view.setUint32(54, 0x00ff0000, true); // R
  view.setUint32(58, 0x0000ff00, true); // G
  view.setUint32(62, 0x000000ff, true); // B
  view.setUint32(66, 0xff000000, true); // A
  view.setUint32(70, 0x73524742, true); // 'sRGB'

  let offset = headerSize;
  for (let y = size - 1; y >= 0; y--) {
    for (let x = 0; x < size; x++) {
      const [r, g, b, a = 255] = drawPixel(x, y, size);
      buffer[offset++] = b;
      buffer[offset++] = g;
      buffer[offset++] = r;
      buffer[offset++] = a;
    }
  }

  return Array.from(buffer);
}

const imageThumbnailBytes = createBmpVisual(96, (x, y, s) => {
  const nx = x / s;
  const ny = y / s;
  // Crisp outer frame
  if (x < 2 || x >= s - 2 || y < 2 || y >= s - 2) return [67, 72, 112, 255];
  // Glowing sun disc
  const distSun = Math.hypot(nx - 0.72, ny - 0.3);
  if (distSun < 0.12) return [251, 191, 36, 255];
  // Layered foreground ridge
  if (ny > 0.66 - 0.14 * Math.sin(nx * 4.2)) return [49, 46, 129, 255];
  // Background mountain ridge
  if (ny > 0.56 - 0.16 * Math.cos(nx * 3.4 - 0.5)) return [79, 70, 229, 255];
  // Dusk sky gradient
  return [
    Math.round(30 + 68 * ny),
    Math.round(41 + 42 * ny),
    Math.round(98 + 110 * (1 - ny)),
    255,
  ];
});

const archiveIconBytes = createBmpVisual(96, (x, y, s) => {
  const nx = x / s;
  const ny = y / s;
  const inBox = nx >= 0.14 && nx <= 0.86 && ny >= 0.16 && ny <= 0.86;
  const inTab = nx >= 0.14 && nx <= 0.46 && ny >= 0.1 && ny < 0.18;
  if (!inBox && !inTab) return [0, 0, 0, 0];
  if (inTab) return [217, 119, 6, 255];
  if (nx < 0.17 || nx > 0.83 || ny < 0.19 || ny > 0.83) return [245, 158, 11, 255];
  if (Math.abs(nx - 0.5) < 0.045) {
    const tooth = Math.floor(y / 6) % 2 === 0;
    return tooth ? [39, 39, 42, 255] : [253, 224, 71, 255];
  }
  return [234, 179, 8, 255];
});
function findPlaywrightFfmpeg() {
  const browserExe = chromium.executablePath();
  const msPlaywrightDir = path.resolve(path.dirname(browserExe), "..", "..");
  const ffmpegDir = fs.readdirSync(msPlaywrightDir).find((entry) => entry.startsWith("ffmpeg-"));
  if (!ffmpegDir) {
    throw new Error(`Playwright ffmpeg binary not found under ${msPlaywrightDir}`);
  }
  const binaryName = process.platform === "win32" ? "ffmpeg-win64.exe" : "ffmpeg-linux";
  const ffmpegPath = path.join(msPlaywrightDir, ffmpegDir, binaryName);
  if (!fs.existsSync(ffmpegPath)) {
    throw new Error(`Playwright ffmpeg binary missing at ${ffmpegPath}`);
  }
  return ffmpegPath;
}

export async function captureDropShelfAssets({
  outDir = path.resolve("website/assets"),
  previewName = "shelf-preview.png",
  videoName = "dropcove-demo.webm",
} = {}) {
  fs.mkdirSync(outDir, { recursive: true });
  const ffmpegPath = findPlaywrightFfmpeg();

  // 1. Launch ephemeral local Vite dev server
  const server = await createServer({
    server: { port: 0 },
    logLevel: "error",
  });
  await server.listen();
  const address = server.httpServer?.address();
  const port = typeof address === "object" && address ? address.port : server.config.server.port;

  // 2. Launch HiDPI headless browser (3x deviceScaleFactor -> 1320x660)
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: VIEWPORT,
    deviceScaleFactor: DEVICE_SCALE_FACTOR,
  });
  const page = await context.newPage();

  // 3. Inject mock Tauri IPC environment before page scripts execute
  await page.addInitScript(({ imageBytes, archiveBytes, width, height }) => {
    const callbacks = new Map();
    let nextCallbackId = 1;
    const makeItem = (id, batchId, name, overrides = {}) => ({
      id,
      batchId,
      position: 0,
      path: "",
      name,
      isFolder: false,
      isPinned: false,
      availability: "Available",
      ownership: "External",
      managedContentKind: null,
      ...overrides,
    });

    window.__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: "main" },
        currentWebview: { label: "main" },
      },
      transformCallback: (callback, onceCb) => {
        const id = nextCallbackId++;
        callbacks.set(id, (arg) => {
          if (onceCb) callbacks.delete(id);
          callback(arg);
        });
        return id;
      },
      unregisterCallback: (id) => {
        callbacks.delete(id);
      },
      invoke: async (cmd, args = {}) => {
        if (cmd === "get_appearance") {
          return { mode: "dark", theme: "dark", highContrast: false, revision: 1 };
        }
        if (cmd === "plugin:event|listen") return 1;
        if (cmd === "plugin:event|unlisten") return null;
        if (cmd === "plugin:window|current_monitor") {
          return {
            name: "Primary",
            scaleFactor: 1,
            position: { x: 0, y: 0 },
            size: { width: 1920, height: 1080 },
            workArea: {
              position: { x: 0, y: 0 },
              size: { width: 1920, height: 1080 },
            },
          };
        }
        if (cmd === "plugin:window|scale_factor") return 1;
        if (cmd === "plugin:window|inner_size" || cmd === "plugin:window|outer_size") {
          return { width, height };
        }
        if (cmd === "plugin:window|outer_position" || cmd === "plugin:window|inner_position") {
          return { x: 100, y: 100 };
        }
        if (cmd === "plugin:window|set_size" || cmd === "plugin:window|set_position") return null;
        if (cmd === "get_shelf_resizing") return false;
        if (cmd === "get_drop_shelf_clear_temporary_state") return false;
        if (cmd === "get_surface_generation") return 1;
        if (cmd === "get_shelf") {
          return {
            batches: [
              {
                id: "batch-1",
                createdAt: Date.now() - 60000,
                position: 0,
                items: [
                  makeItem("item-1", "batch-1", "hero-mockup.png", {
                    path: "C:\\Users\\User\\Pictures\\hero-mockup.png",
                  }),
                ],
              },
              {
                id: "batch-2",
                createdAt: Date.now() - 30000,
                position: 1,
                items: [
                  makeItem("item-2", "batch-2", "dropcove-0.1.0.zip", {
                    path: "C:\\Users\\User\\Documents\\dropcove-0.1.0.zip",
                  }),
                ],
              },
              {
                id: "batch-3",
                createdAt: Date.now() - 10000,
                position: 2,
                items: [
                  makeItem("item-3", "batch-3", "theme-tokens.css", {
                    ownership: "Managed",
                    managedContentKind: "Text",
                  }),
                ],
              },
            ],
            preferredWidth: width,
            height,
            warning: null,
          };
        }
        if (cmd === "get_managed_text") {
          return "@theme { --color-dc-accent: #6366f1; }";
        }
        if (cmd === "get_shelf_visual") {
          if (args.itemId === "item-1") return new Uint8Array(imageBytes).buffer;
          if (args.itemId === "item-2") return new Uint8Array(archiveBytes).buffer;
          return null;
        }
        return null;
      },
    };
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
      unregisterListener: () => {},
    };
  }, {
    imageBytes: imageThumbnailBytes,
    archiveBytes: archiveIconBytes,
    width: VIEWPORT.width,
    height: VIEWPORT.height,
  });

  try {
    await page.goto(`http://localhost:${port}`);
    await page.waitForSelector("main.shelf [data-batch-id]");
    await page.waitForTimeout(500);

    // 4. Capture lossless 3x HiDPI PNG snapshot
    const shelf = page.locator("main.shelf");
    const previewPath = path.join(outDir, previewName);
    await shelf.screenshot({
      path: previewPath,
      type: "png",
    });
    console.log(`Generated preview image: ${previewPath} (${fs.statSync(previewPath).size} bytes)`);

    // 5. Capture HiDPI frames into FFmpeg VP9 encoder (crf 14, 3x resolution)
    const videoDestPath = path.join(outDir, videoName);
    const ffmpegArgs = [
      "-y",
      "-f", "image2pipe",
      "-c:v", "mjpeg",
      "-framerate", String(VIDEO_FPS),
      "-i", "pipe:0",
      "-c:v", "libvpx-vp9",
      "-crf", "14",
      "-b:v", "0",
      "-pix_fmt", "yuv420p",
      "-row-mt", "1",
      "-deadline", "good",
      "-cpu-used", "2",
      videoDestPath,
    ];

    const ffmpegProc = spawn(ffmpegPath, ffmpegArgs, { stdio: ["pipe", "ignore", "pipe"] });
    let ffmpegStderr = "";
    ffmpegProc.stderr.on("data", (chunk) => {
      ffmpegStderr += chunk.toString();
    });
    ffmpegProc.stdin.on("error", () => {});
    const captureFrame = () => page.screenshot({ type: "jpeg", quality: 100 });

    const writeFrames = async (frameBuf, count) => {
      for (let i = 0; i < count; i++) {
        if (!ffmpegProc.stdin.write(frameBuf)) {
          await once(ffmpegProc.stdin, "drain");
        }
      }
    };

    const captureBurst = async (durationMs) => {
      const frameInterval = 1000 / VIDEO_FPS;
      const start = Date.now();
      while (Date.now() - start < durationMs) {
        const t0 = Date.now();
        const frame = await captureFrame();
        await writeFrames(frame, 1);
        const remaining = frameInterval - (Date.now() - t0);
        if (remaining > 0) await page.waitForTimeout(remaining);
      }
    };

    // Sequence: idle -> hover batch-1 -> hover batch-2 -> select batch-2 -> Ctrl+select batch-3 -> hold
    await writeFrames(await captureFrame(), 18); // 0.75s idle

    await page.hover('[data-batch-id="batch-1"]');
    await captureBurst(260);
    await writeFrames(await captureFrame(), 20);

    await page.hover('[data-batch-id="batch-2"]');
    await captureBurst(260);
    await writeFrames(await captureFrame(), 20);

    await page.click('[data-batch-id="batch-2"]');
    await captureBurst(260);
    await writeFrames(await captureFrame(), 20);

    await page.keyboard.down("Control");
    await page.click('[data-batch-id="batch-3"]');
    await page.keyboard.up("Control");
    await captureBurst(260);
    await writeFrames(await captureFrame(), 30);

    // Clear selection cleanly before loop restart
    await page.mouse.click(220, 195);
    await captureBurst(240);
    await writeFrames(await captureFrame(), 12);

    ffmpegProc.stdin.end();
    const [exitCode] = await once(ffmpegProc, "close");
    if (exitCode !== 0) {
      throw new Error(`FFmpeg VP9 encoding failed (code ${exitCode}):\n${ffmpegStderr}`);
    }

    console.log(`Generated demo video: ${videoDestPath} (${fs.statSync(videoDestPath).size} bytes)`);
    return { previewPath, videoDestPath };
  } finally {
    await browser.close();
    await server.close();
  }
}

if (process.argv[1] && process.argv[1].endsWith("capture-assets.mjs")) {
  captureDropShelfAssets().catch((error) => {
    console.error("Failed to capture Drop Shelf assets:", error);
    process.exit(1);
  });
}
