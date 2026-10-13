const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const version = fs.readFileSync(path.join(rootDir, 'VERSION'), 'utf8').trim();
const sourceFile = path.join(rootDir, 'src-tauri', 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle', 'nsis', `DropCove_${version}_x64-setup.exe`);
const buildDir = path.join(rootDir, 'build');
const targetFile = path.join(buildDir, `DropCove-Setup-${version}.exe`);

if (!fs.existsSync(sourceFile)) {
  console.error(`Error: Current Windows x64 installer not found: ${sourceFile}`);
  process.exit(1);
}

fs.mkdirSync(buildDir, { recursive: true });
fs.copyFileSync(sourceFile, targetFile);
console.log(`Successfully packaged installer: ${targetFile}`);
