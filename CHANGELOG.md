# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.2.0] - 2026-02-05

### Changed
- **Process architecture**: Introduced a dedicated ProcessManager to decouple IPC from process lifecycle and make the core logic testable without Tauri
- **Log streaming**: Added log batching (`log-batch`) and throttled IPC forwarding to prevent UI freezes under heavy output
- **Error handling**: Replaced string errors with structured `AppError` variants to enable typed frontend responses

### Improved
- **Terminal performance**: Switched to a fixed-size ring buffer and batched terminal writes to reduce render pressure
- **Filter rewrites**: Chunked terminal rewrites on filter changes to avoid blocking the UI thread
- **Config safety**: Config saves now use atomic temp-write + rename

### Fixed
- **State consistency**: `set_active_project` now validates, persists, then commits in-memory to avoid divergence

## [2.1.0] - 2026-01-21

### Fixed
- **Terminal persistence**: Fixed terminal content being cleared when switching between project tabs. Each project now maintains its own terminal state in memory
- **TypeScript build errors**: Resolved unused variable warnings in `ConfigModal.tsx` and `ProjectView.tsx`
- **Rust compiler warnings**: Removed unused imports and dead code in backend codebase

### Improved
- **Search UI**: Replaced text-based arrows (^, v, x) with proper SVG icons in the search bar for a more polished look
- **Tab switching performance**: Optimized rendering by keeping all project views mounted and toggling visibility instead of mounting/unmounting

## [2.0.0] - 2026-01-20

### Changed
- Complete rewrite from egui to Tauri + React + xterm.js
- Modern web-based UI with improved terminal rendering
- Multi-project support with tabbed interface

[2.2.0]: https://github.com/P4ST4S/rust-developer-tool-web/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/P4ST4S/rust-developer-tool-web/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/P4ST4S/rust-developer-tool-web/releases/tag/v2.0.0
