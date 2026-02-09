# Changelog

All notable changes to snapshot.nvim will be documented in this file.

## [0.1.1] - 2026-02-09

### Fixed
- Fixed "E474: Attempt to decode a blank string" error when running `:Snapshot` without arguments
- Fixed visual selection detection by using visual marks instead of mode checking
- Fixed Treesitter row offset when capturing visual selections
- Fixed font loading by using correct FiraCode download URL
- Fixed font API compatibility by switching from rusttype to ab_glyph

### Improved
- Better error handling for invalid JSON arguments
- More robust visual selection capture
- Automatic fallback to full buffer if no visual selection

## [0.1.0] - 2026-02-09

### Added
- Initial release of snapshot.nvim
- Polacode-style code snapshot generation
- Rust-based image generator for high-quality output
- Syntax highlighting using Treesitter
- Configurable styling options:
  - Custom output paths
  - Adjustable padding, line height, and font size
  - Background color customization
  - Optional line numbers
  - Shadow effects (experimental)
- Visual mode support for capturing selections
- Full buffer capture in normal mode
- FiraCode font integration
- Comprehensive documentation:
  - README with full usage guide
  - Installation guide (INSTALL.md)
  - Quick reference (QUICKREF.md)
  - Examples (EXAMPLES.md)
- Makefile for easy building and installation
- Commands:
  - `:Snapshot` - Main snapshot command
  - `:SnapshotVisual` - Visual selection capture
- Lua API for programmatic usage

### Technical Details
- Neovim plugin written in Lua
- Image generation backend written in Rust
- Uses `ab_glyph` for text rendering
- Uses `imageproc` and `image` crates for PNG generation
- JSON communication between Lua and Rust
- Treesitter integration for syntax highlighting

### Dependencies
- Neovim >= 0.9.0
- Rust and Cargo
- Treesitter (recommended for syntax highlighting)

## Future Plans

### Potential Features
- [ ] More font options
- [ ] Gradient backgrounds
- [ ] Border styles
- [ ] Watermarks
- [ ] Window decorations (like VS Code window chrome)
- [ ] Export to multiple formats (SVG, JPEG, WebP)
- [ ] Custom themes/presets
- [ ] Copy to clipboard support
- [ ] Auto-upload to image hosting services
- [ ] Animated GIFs for code tutorials
- [ ] Diff highlighting
