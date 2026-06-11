# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-06-11

### Added
- **System Apps Support**: Extended scanning, rule generation, rule management, and automatic input method switching to eligible macOS system applications.
- **Localized Input Source Names**: Display input method names using macOS-localized labels, including a Simplified Chinese Pinyin fallback when the system label remains English.

### Changed
- **System App Discovery**: Limited system application scanning to a curated allowlist of common input-capable Apple apps.
- **Input Source Runtime**: Marshaled current input-source reads onto the main thread to avoid HIToolbox/TIS thread-context crashes in bundled app builds.

### Removed
- **Custom Input Method Indicator**: Removed the experimental Tauri overlay indicator, its settings toggle, IPC surface, and related macOS private API feature usage.

## [1.0.1] - 2026-02-25

### Changed
- **Build Target**: Switched from Universal Binary (Intel + Apple Silicon) to Apple Silicon (aarch64) only.
- **Release Workflow**: Updated GitHub Actions to generate `_aarch64.dmg` artifacts instead of `_universal.dmg`.
- **Distribution**: Updated Homebrew Cask to depend on `arch: :arm64` and point to the new artifact naming convention.

## [1.0.0] - 2026-02-25

### Added
- Initial release of SmartIME.
- Zero-configuration intelligent input method switching based on active application.
- LLM-based rule prediction for automatic configuration.
- Manual rule management interface.
- macOS Menu Bar support for background operation.
- Support for macOS 12 (Monterey) and above.
