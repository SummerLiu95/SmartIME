# System Apps Support Execution

## Scope Note

This record now covers System Apps Support only. The custom current input method indicator was intentionally removed on 2026-06-09; see `docs/exec-plan/2026-06-09-remove-input-indicator.md` for that removal record.

## Context and Goal

This record covers implementation of the confirmed System Apps Support plan: include eligible macOS system apps in scan, prediction, rule management, and automatic switching.

Source requirements and design references:

- `docs/REQUIREMENTS.md` FR-14
- `docs/DESIGN_DOC.md` system app scope behavior
- `docs/TASKS.md` sections 4.5 through 4.6

## Implementation Summary

- Expanded app scanning roots to include `/System/Applications`, `/System/Applications/Utilities`, and `/System/Library/CoreServices` in addition to user app locations.
- Removed blanket Apple bundle filtering from target-app rule alignment and LLM prediction entry points; only empty app identities are skipped.
- Follow-up refinement: system-app scan output is now limited to a curated allowlist of common input-capable Apple apps instead of every bundle under system directories.
- Follow-up refinement: current-input-source reads used by automatic switching are marshaled to the main thread after a bundled-app crash showed HIToolbox queue assertions in background-thread work.

## Affected Files and Modules

- Backend:
  - `src-tauri/src/system_apps.rs`
  - `src-tauri/src/command.rs`
  - `src-tauri/src/config.rs`
  - `src-tauri/src/input_source.rs`
  - `src-tauri/src/observer.rs`
  - `src-tauri/src/main.rs`
- Frontend:
  - `lib/api.ts`
  - `app/settings/general/page.tsx`
  - `app/settings/rules/page.tsx`
  - `app/onboarding/scan/page.tsx`
- Documentation:
  - `docs/TECHNICAL_SPEC.md`

## Validation Performed

- `cargo fmt`
- `cargo test`
  - Result: passed, 17 tests after scan filtering coverage was added.
- `bun run build`
  - Result: passed after network access was allowed for Google Fonts fetch.
- `bun tauri build --bundles app`
  - Result: passed, produced `src-tauri/target/release/bundle/macos/SmartIME.app`.
- `bun tauri build`
  - Result: release binary and `.app` bundle were produced, but final DMG bundling failed inside Tauri's generated `bundle_dmg.sh`.

## Follow-up Notes

- Full DMG packaging still needs follow-up because Tauri's generated `bundle_dmg.sh` failed without useful stderr output, even after a non-sandbox rerun. This did not block the release `.app` bundle.
- Tauri emits an existing warning that bundle identifier `com.smartime.app` ends with `.app`; this is not caused by this feature work but should be addressed before release hardening.
- Manual macOS validation is still required for system-app discovery and automatic input-source switching in real apps with Accessibility permission granted.
