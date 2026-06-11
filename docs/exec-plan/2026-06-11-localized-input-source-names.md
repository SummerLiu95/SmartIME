# Localized Input Source Names Execution

## Context and Goal

SmartIME should display input method names the same way macOS presents them locally. For example, a Chinese macOS system should show a Simplified Pinyin label such as `简体拼音` instead of an English fallback like `Pinyin - Simplified` when the system provides a localized label.

## Implementation Summary

- Added AppKit `NSTextInputContext.localizedNameForInputSource:` lookup in `src-tauri/src/input_source.rs`.
- Added a locale-gated Apple built-in fallback for Simplified Chinese Pinyin because AppKit can still return `Pinyin - Simplified` on a Chinese system.
- Kept `kTISPropertyLocalizedName` as the final fallback when AppKit and the built-in fallback do not return a better label.
- Kept rule IDs and IPC data shape unchanged: only the `InputSource.name` display value changes.
- Updated frontend browser-preview mock data to use the localized Simplified Pinyin label.
- Updated requirements, design, technical spec, and Rulebook with the display-name rule and regression lesson.

## Validation Performed

- `cargo fmt`
  - Result: passed.
- `cargo test input_source::tests::test_get_system_input_sources -- --nocapture`
  - Result: passed; current macOS output now shows `com.apple.inputmethod.SCIM.ITABC` as `简体拼音`.
- `cargo test`
  - Result: passed, 18 Rust tests.
- `bun run build`
  - Result: passed.
- `bun tauri build --bundles app`
  - Result: passed, produced `src-tauri/target/release/bundle/macos/SmartIME.app`.

## Follow-up Notes

- Manual validation should inspect the Rules input-method dropdown on a Chinese macOS system and confirm Simplified Pinyin follows the system-localized label.
