# Multi-Provider LLM Support

## Summary

Replaced the user-editable Base URL setup with an explicit provider model covering DeepSeek, OpenAI, Anthropic, and Google Gemini. The provider is now part of the persisted config, the Keychain credential binding, the IPC contract, and the onboarding form, instead of being a frontend-only concern.

The LLM backend was rewritten on the `genai` crate: the explicit provider maps to a native `AdapterKind` rather than inferring the protocol from model-name strings or posting OpenAI-shaped payloads to a custom URL. Every provider request shares one policy — a restricted `reqwest` client (HTTPS-only, no redirects, 60-second timeout), temperature 0.1, bounded output tokens (16 for connection tests, 2048 for prediction batches), and JSON mode for batch prediction; DeepSeek classification additionally disables reasoning.

Backward compatibility is preserved. Legacy `llm_config.json` files with `base_url` or plaintext `api_key` migrate on first credential access, inferring the provider once from the model or service address and dropping the legacy field on the next successful save. Legacy Base-URL-bound Keychain credentials remain readable until replaced, and debug-only environment import now uses `LLM_PROVIDER` (legacy `LLM_BASE_URL` only helps infer a provider).

## Affected Areas

- `src-tauri/src/llm.rs`: `LLMProvider` enum with native adapter mapping and legacy inference, `genai` client built with a restricted `reqwest` client and zeroizing auth resolver, shared bounded chat options per provider, debug env import via `LLM_PROVIDER`.
- `src-tauri/src/credentials.rs`: provider-bound Keychain credentials, legacy `base_url` read/migrate path, blank-key reuse limited to the unchanged provider.
- `src-tauri/Cargo.toml` / `Cargo.lock`: added `genai` 0.6.5, bumped `reqwest`.
- `lib/api.ts`: `LLMProvider` type and provider-based `LLMConfig` / `LLMConfigStatus` contracts, updated browser-preview mock.
- `app/onboarding/llm/page.tsx`: provider selector with per-provider recommended model, Base URL field removed, saved-key reuse gated on the unchanged provider.
- `.env.llm.example`, `README.md`: development credential setup now documents `LLM_API_KEY` / `LLM_PROVIDER` / `LLM_MODEL`.
- `docs/REQUIREMENTS.md`, `docs/DESIGN_DOC.md`, `docs/TECHNICAL_SPEC.md`: product behavior, onboarding design, and architecture contracts for the new setup flow.
- `docs/Rulebook.md`: added mistake record 3.15, incident INC-014, updated credential regression coverage, and release regression matrix item 12.
- `CHANGELOG.md`: intentionally left unchanged per project convention; release notes are added only when the feature is officially released and manually instructed.

## Validation

- `cd src-tauri && cargo test`: 54 passed, 1 ignored Keychain integration test. New coverage includes provider-to-adapter mapping without model-string inference, legacy inference used only for migration, provider-bound key reuse, and bounded per-provider JSON batch options.
- `bun run build`: frontend production build passed.
- Manual bundled-app validation remains required with real provider keys: connection test and first scan on the configured provider, legacy Base-URL config migration on first credential access, and key replacement behavior when switching providers.
