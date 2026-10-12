const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const versionFile = path.join(rootDir, 'VERSION');
const version = fs.readFileSync(versionFile, 'utf8').trim();

const buildDir = path.join(rootDir, 'build');
if (!fs.existsSync(buildDir)) {
  fs.mkdirSync(buildDir, { recursive: true });
}

const nsisDir = path.join(rootDir, 'src-tauri', 'target', 'release', 'bundle', 'nsis');
const xwinNsisDir = path.join(rootDir, 'src-tauri', 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle', 'nsis');

let sourceDir = null;
if (fs.existsSync(nsisDir)) {
  sourceDir = nsisDir;
} else if (fs.existsSync(xwinNsisDir)) {
  sourceDir = xwinNsisDir;
}

const targetFile = path.join(buildDir, `DropCove-Setup-${version}.exe`);

if (!sourceDir) {
  console.error('Error: Could not find Tauri NSIS bundle directory.');
  console.error(`Looked in:\n- ${nsisDir}\n- ${xwinNsisDir}`);
  process.exit(1);
}

const exeFiles = fs.readdirSync(sourceDir).filter(f => f.endsWith('.exe') && !f.endsWith('.pdb'));
if (exeFiles.length === 0) {
  console.error(`Error: No installer .exe found in ${sourceDir}`);
  process.exit(1);
}

// Prefer the file with setup or the newest .exe
const installerName = exeFiles.find(f => f.includes('setup')) || exeFiles[0];
const sourceFile = path.join(sourceDir, installerName);

fs.copyFileSync(sourceFile, targetFile);
console.log(`Successfully packaged installer: ${targetFile}`);
