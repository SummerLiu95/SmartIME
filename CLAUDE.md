# SmartIME Claude Guide

## Project Snapshot
SmartIME is a macOS app with both a main window and a menu bar indicator, built with Tauri v2 (Rust) + Next.js (App Router). It automatically switches input methods based on the active application window, using AI to bootstrap per-app rules.

## Product Requirements (non-negotiable)
- Only use system-enabled input sources; never fabricate input method IDs.
- App scanning must traverse `/Applications` and `~/Applications`.
- Switching should complete within 100ms (target) and under 200ms (max).
- Request only Accessibility/Input Monitoring permissions.
- API keys must be stored securely (Keychain or encrypted store) and masked in UI.
- macOS 12+ only; universal binary (Intel + Apple Silicon).

## Tech Stack
- Frontend: Next.js, shadcn/ui, Tailwind, Radix UI, framer-motion, lucide-react.
- Backend: Rust + Tauri v2. Key modules:
  - `src-tauri/src/input_source.rs`
  - `src-tauri/src/observer.rs`
  - `src-tauri/src/llm.rs`
  - `src-tauri/src/system_apps.rs`
  - `src-tauri/src/config.rs`
- Package manager: bun.

## LLM Config
- Real config lives in `.env.llm` (ignored). Example in `.env.llm.example`.
- Required: `LLM_API_KEY`, `LLM_MODEL`, `LLM_BASE_URL` (default `https://api.openai.com/v1`).
- Provide a “Test Connection” flow before scanning.

## UX/Design
- Clean, minimal shadcn/ui styling; adapt to Light/Dark mode.
- System font (San Francisco); lucide icons.
- Use framer-motion for subtle transitions (fade/slide, layout animations).
- Desktop sizing targets: main window 800x600, tray 300x400.

## Dev Commands
- `bun tauri dev`
- `bun run lint`
- `bun run tauri build`

## Test Workflow (per TASKS.md)
- After finishing code implementation for each task in `TASKS.md`, run unit tests for the affected module(s) to confirm behavior.
- Developers perform functional tests manually after unit tests pass.

## Architecture Notes
- IPC commands must match names exactly between frontend and backend.
- Configs and rules persisted with `tauri-plugin-store` or local JSON (per spec).
- App scanning and input source lists are retrieved from macOS APIs.

## Working Style
- Keep changes small and reviewable.
- Prefer existing patterns and components.
- Avoid introducing new dependencies unless clearly justified.
- Never commit secrets or `.env.llm`.
