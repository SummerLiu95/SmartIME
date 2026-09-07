# LLM Batch Timeout Recovery

## Summary

Reworked scan prediction from one whole-app-set request into bounded batches of at most 20 apps with at most 2 concurrent requests. Each batch now contributes independent validated results and emits real settled-app progress. Failed, missing, or invalid predictions remain without a rule instead of being persisted as AI-generated fallback values.

The rescan path also detects the narrowly scoped legacy incident signature—at least 20 rules, all AI-generated, all using the first input source while another source is available—and discards that set before prediction. Any manual rule prevents this recovery from triggering.

Live validation then showed that two 20-app DeepSeek batches still exhausted the 60-second client timeout because V4 thinking was left at its provider default. DeepSeek classification requests now disable thinking, request JSON output, and cap output at 2048 tokens. Failed batches are not retried within the same scan; their apps remain gaps for a later user-triggered rescan.

## Affected Areas

- `src-tauri/src/command.rs`: bounded concurrency, progress events, partial-success handling, no-rule failure semantics, logging, and legacy recovery.
- `src-tauri/src/llm.rs`: macOS preferred-language context, balanced Chinese/English prediction guidance, bounded DeepSeek response controls, and stage-specific safe network errors.
- `src-tauri/src/input_source.rs`: exposes the preferred-language identifier within the backend crate.
- `app/onboarding/scan/page.tsx`: real processed-app count and derived progress.
- `app/settings/rules/page.tsx`: rescan processed-app count.
- `lib/api.ts`: typed scan-stage and progress-event subscription.

## Validation

- `cd src-tauri && cargo test`: 54 passed, 1 ignored Keychain integration test.
- `cd src-tauri && cargo clippy --all-targets`: completed with 10 pre-existing warnings and no warning introduced by this change.
- `bun test lib/api.test.mjs`: 2 passed.
- `bun run build`: passed.
- `bun tauri build --bundles app`: passed; generated `src-tauri/target/release/bundle/macos/SmartIME.app`.
- `git diff --check`: passed.
- Manual bundled-app validation remains required with a configured provider to verify that DeepSeek completes the remaining 40-app gap within the timeout, returns strict JSON, and produces the expected Chinese-app rules without an automatic same-scan retry.
