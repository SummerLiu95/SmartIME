# Scan Performance Optimization Execution

## Context and Goal

First-run scan and manual rescan could take several minutes because SmartIME sent one LLM request per managed app. The onboarding progress UI also used a fake random progress animation capped at 88%, which made long LLM work look stuck.

The goal was to reduce scan latency by batching LLM prediction, make manual rescan incremental by reusing existing valid rules, and make the scan UI communicate real phases instead of simulated progress.

## Implementation Summary

- Added `LLMClient::predict_batch` in `src-tauri/src/llm.rs` to send all target apps for a prediction pass in one OpenAI-compatible chat completion request.
- Added strict batch response parsing for direct bundle-ID maps and structured rule arrays, while rejecting unknown bundle IDs and input source IDs not present in the current system input-source list.
- Updated `src-tauri/src/command.rs` so first onboarding scan batch-predicts the full target app set.
- Updated manual rescan so manual rules are preserved, valid existing AI rules are reused, and only missing/new/invalid AI-rule gaps are predicted.
- Preserved deterministic alignment fallback: partial or failed batch predictions do not persist invalid input IDs and still return a complete aligned rule list.
- Replaced the onboarding fake random 88% progress with phase-based progress for input-source loading, rule generation, config saving, and completion.
- Added a short minimum visible duration for each scan phase so fast local phases are perceptible while the actual backend work still starts immediately.
- Updated the Rules rescan loading copy to reflect that SmartIME is reusing existing rules and generating only missing rules.
- Added Rulebook notes for keeping first-scan and rescan semantics separate and for batch parser response-shape precedence.

## Affected Files and Modules

- `src-tauri/src/llm.rs`
  - Batch prediction request.
  - Batch response extraction, parsing, validation, and unit tests.
- `src-tauri/src/command.rs`
  - Batch prediction orchestration.
  - Rescan gap detection and incremental rule reuse.
  - Unit tests for rescan gap behavior.
- `app/onboarding/scan/page.tsx`
  - Phase-based scan progress and status text.
- `app/settings/rules/page.tsx`
  - Incremental rescan status copy.
- `docs/Rulebook.md`
  - New scan/rescan and batch parser mistake-prevention entry.

## Validation Performed

- `cd src-tauri && cargo fmt`
  - Result: passed.
- `cd src-tauri && cargo test`
  - Result: passed, 25 Rust tests.
- `bun run build`
  - Result: passed.
- `bun tauri build --bundles app`
  - Result: passed, produced `src-tauri/target/release/bundle/macos/SmartIME.app`.
  - Note: Tauri still warns that bundle identifier `com.smartime.app` ends with `.app`; this is pre-existing and unrelated to this change.

## Follow-up Notes

- Manual timing validation should compare first scan and unchanged manual rescan on a config with many managed apps and a valid LLM config.
- Unchanged manual rescan should be near local scan speed because it should skip LLM prediction when all existing AI rules are valid.
- First onboarding scan still depends on provider latency, but it now avoids N serial LLM calls.
