SHELL := powershell.exe
.SHELLFLAGS := -NoProfile -ExecutionPolicy Bypass -Command

.PHONY: all sync-version build dev clean help

all: build

help:
	@Write-Host "DropCove Build Targets:"
	@Write-Host "  make sync-version  - Synchronize VERSION into package.json, Cargo.toml, tauri.conf.json"
	@Write-Host "  make dev           - Run Tauri development environment"
	@Write-Host "  make build         - Build frontend, compile with cargo-xwin, and output installer to build/"
	@Write-Host "  make clean         - Clean dist/, build/, and cargo target directories"

sync-version:
	@node scripts/sync-version.cjs

dev: sync-version
	@pnpm tauri dev

build: sync-version
	@pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc
	@node scripts/copy-installer.cjs
clean:
	@if (Test-Path build) { Remove-Item -Recurse -Force build }
	@if (Test-Path dist) { Remove-Item -Recurse -Force dist }
	@if (Test-Path src-tauri/target) { Remove-Item -Recurse -Force src-tauri/target }
	@Write-Host "Cleaned build artifacts."
