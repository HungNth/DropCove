const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const versionFile = path.join(rootDir, 'VERSION');

if (!fs.existsSync(versionFile)) {
  console.error('Error: VERSION file not found at ' + versionFile);
  process.exit(1);
}

const version = fs.readFileSync(versionFile, 'utf8').trim();
if (!version) {
  console.error('Error: VERSION file is empty');
  process.exit(1);
}

console.log(`Syncing version ${version}...`);

// 1. Sync package.json
const pkgJsonPath = path.join(rootDir, 'package.json');
if (fs.existsSync(pkgJsonPath)) {
  const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
  pkg.version = version;
  fs.writeFileSync(pkgJsonPath, JSON.stringify(pkg, null, 2) + '\n');
  console.log(`- Updated package.json -> ${version}`);
}

// 2. Sync src-tauri/Cargo.toml
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');
if (fs.existsSync(cargoTomlPath)) {
  let cargoContent = fs.readFileSync(cargoTomlPath, 'utf8');
  cargoContent = cargoContent.replace(/^version = ".*?"/m, `version = "${version}"`);
  fs.writeFileSync(cargoTomlPath, cargoContent);
  console.log(`- Updated src-tauri/Cargo.toml -> ${version}`);
}

// 3. Sync src-tauri/tauri.conf.json
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
if (fs.existsSync(tauriConfPath)) {
  const conf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
  conf.version = version;
  fs.writeFileSync(tauriConfPath, JSON.stringify(conf, null, 2) + '\n');
  console.log(`- Updated src-tauri/tauri.conf.json -> ${version}`);
}

console.log('Version sync complete.');
