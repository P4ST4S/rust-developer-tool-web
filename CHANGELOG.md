# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[2.1.0]: https://github.com/P4ST4S/rust-developer-tool-web/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/P4ST4S/rust-developer-tool-web/releases/tag/v2.0.0
