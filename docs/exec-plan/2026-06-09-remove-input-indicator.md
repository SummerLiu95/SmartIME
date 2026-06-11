# Remove Custom Input Method Indicator Execution

## Context and Goal

SmartIME no longer implements the custom current input method indicator. The decision is based on the cost of building reliable native macOS overlay behavior from the Tauri stack and the fact that macOS already provides its own input-source prompt in focused editable contexts for user-triggered changes.

## Implementation Summary

- Removed the frontend `/indicator` route and the General Settings "显示当前输入法提示" toggle.
- Removed `general.show_input_indicator` from frontend and backend config models.
- Removed the indicator Tauri window, indicator event emission, focused editable-context detection module, and related IPC surface.
- Removed the `cmd_get_current_input_source` frontend command because it existed only for the custom indicator path.
- Removed `macOSPrivateApi` and the `macos-private-api` Tauri feature because the custom transparent overlay is no longer used.
- Kept main-thread input-source reads in automatic switching because HIToolbox/TIS current-input APIs are still sensitive to thread context.

## Affected Files and Modules

- Backend:
  - `src-tauri/src/main.rs`
  - `src-tauri/src/command.rs`
  - `src-tauri/src/config.rs`
  - `src-tauri/src/observer.rs`
  - `src-tauri/Cargo.toml`
  - `src-tauri/tauri.conf.json`
  - `src-tauri/capabilities/default.json`
- Removed backend modules:
  - `src-tauri/src/indicator.rs`
  - `src-tauri/src/input_context.rs`
- Frontend:
  - `lib/api.ts`
  - `app/settings/general/page.tsx`
  - `app/settings/rules/page.tsx`
  - `app/onboarding/scan/page.tsx`
- Removed frontend route:
  - `app/indicator/page.tsx`
- Documentation:
  - `README.md`
  - `docs/REQUIREMENTS.md`
  - `docs/DESIGN_DOC.md`
  - `docs/TECHNICAL_SPEC.md`
  - `docs/TASKS.md`
  - `docs/Rulebook.md`

## Validation Performed

- `cargo fmt`
  - Result: passed.
- `cargo test`
  - Result: passed, 14 Rust tests.
- `bun run build`
  - Result: passed; static route output no longer includes `/indicator`.
- `bun tauri build --bundles app`
  - Result: passed, produced `src-tauri/target/release/bundle/macos/SmartIME.app`.

## Follow-up Notes

- Manual validation should confirm General Settings no longer shows the indicator toggle and app-switch automatic input-source switching still works silently.
- If this feature is re-opened later, it should use a deliberate native macOS implementation strategy rather than a Tauri overlay route.
