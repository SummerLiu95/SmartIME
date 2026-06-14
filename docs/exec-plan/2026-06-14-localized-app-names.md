# Localized App Names Execution

## Context and Goal

The Rules list could show raw bundle fallback names such as `WeChat` and `aDrive` even when macOS and the app bundle provide localized names such as `微信` and `阿里云盘`. The goal was to make the app name shown in SmartIME match the localized app name macOS exposes for both system apps and third-party apps, while keeping `bundle_id` as the stable matching key.

## Implementation Summary

- Kept the existing rule schema unchanged: `AppRule.app_name` remains the persisted display-name snapshot, and `bundle_id` remains the rule matching key.
- Updated `src-tauri/src/system_apps.rs` to resolve app names with this priority:
  1. macOS `NSFileManager.displayNameAtPath` for the installed bundle path.
  2. Localized `InfoPlist.strings` resources for the app bundle.
  3. Curated Simplified Chinese names for supported system apps.
  4. Raw `CFBundleDisplayName`, `CFBundleName`, then `.app` filename fallback.
- Added a guard so a system display name that is only the plain `.app` filename does not block localized `InfoPlist.strings` lookup.
- Added text `.strings` parsing fallback for Apple strings files such as `"CFBundleDisplayName" = "阿里云盘";`, because not all third-party app `InfoPlist.strings` files parse through the plist crate directly.
- Fixed plain filename detection to be case-insensitive so fallback names such as `Doubao` for `doubao.app` and `NetEaseMusic` for `NeteaseMusic.app` do not block localized names.
- Added UTF-16 BOM decoding for localized `InfoPlist.strings`, which is required for apps such as WeChat and NetEaseMusic.
- Preserved the runtime app icon path: icons still resolve from current installed bundle paths through `NSWorkspace.iconForFile` and remain frontend-only cache data.
- Updated requirements, design, technical spec, and Rulebook with the localized app-name behavior and regression guidance.

## Validation Performed

- `cd src-tauri && cargo fmt`
  - Result: passed.
- `cd src-tauri && cargo test system_apps::tests`
  - Result: passed, including localized name precedence and text `InfoPlist.strings` parsing tests.
- `cd src-tauri && cargo test`
  - Result: passed, 30 Rust tests.
- `bun run build`
  - Result: passed.
- `bun tauri build --bundles app`
  - Result: passed, produced `src-tauri/target/release/bundle/macos/SmartIME.app`.
  - Note: Tauri still warns that bundle identifier `com.smartime.app` ends with `.app`; this is pre-existing and unrelated to this change.
- Local bundle resource checks:
  - `/Applications/doubao.app/Contents/Resources/zh_CN.lproj/InfoPlist.strings` contains `豆包`.
  - `/Applications/NeteaseMusic.app/Contents/Resources/zh-Hans.lproj/InfoPlist.strings` contains `网易云音乐`.
  - `/Applications/WeChat.app/Contents/Resources/zh-Hans.lproj/InfoPlist.strings` contains `微信`.
  - `/Applications/aDrive.app/Contents/Resources/zh_CN.lproj/InfoPlist.strings` contains `阿里云盘`.
- Full scan verification:
  - `cargo test system_apps::tests::test_get_installed_apps -- --nocapture` printed `豆包`, `网易云音乐`, and `微信` for the corresponding installed bundle IDs.

## Follow-up Notes

- Manual validation should launch the rebuilt bundled app, run Rules rescan, and confirm Doubao, NetEaseMusic, WeChat, and aDrive display as `豆包`, `网易云音乐`, `微信`, and `阿里云盘` on a Chinese macOS system.
- Existing persisted rules will update their `app_name` snapshots on scan/rescan because alignment rewrites rule names from the latest scanned app metadata.
