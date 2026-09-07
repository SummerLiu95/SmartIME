# Technical Specification

## 1. Project Overview

**SmartIME** is a macOS menu bar application built on Tauri, designed to automatically switch input methods based on the current active window. This project leverages Rust's high-performance system call capabilities to monitor application focus changes, combined with a frontend built on Next.js to provide an intuitive configuration management interface.

## 2. Tech Stack & Architecture

### 2.1 Boilerplate

This project is initialized based on **`nomandhoni-cs/tauri-nextjs-shadcn-boilerplate`**. This template integrates best practices for modern Web development and desktop applications.

*   **Core Framework**: [Tauri v2](https://tauri.app/) (Rust + Webview)
*   **Frontend Framework**: [Next.js](https://nextjs.org/) (React)
*   **UI Component Library**: [shadcn/ui](https://ui.shadcn.com/) (Based on Tailwind CSS and Radix UI)
*   **Icon Library**: [lucide-react](https://lucide.dev/) (For a consistent and aesthetic SVG icon system)
*   **Utility Library**: `cn()` (Class merger based on `clsx` and `tailwind-merge`)
*   **Animation Library**: [framer-motion](https://www.framer.com/motion/) (For declarative animations and complex interactions)
*   **HTTP Client**: [reqwest](https://docs.rs/reqwest/latest/reqwest/) (Rust side, for LLM API calls)
*   **Package Manager**: bun

### 2.2 Architectural Constraints

1.  **Platform boundary**
    *   SmartIME is macOS-only because it depends on macOS input-source APIs, Accessibility permission behavior, LaunchAgent integration, and Dock/tray activation semantics.
    *   Release artifacts target macOS 12+ and should be validated as bundled apps, not only through the dev runtime.
2.  **Input-source authority**
    *   The app must only use system-enabled and selectable input sources returned by macOS APIs.
    *   Rule options and persisted rule values must be re-synced from system truth during onboarding scans and manual rescans.
3.  **Frontend-backend contract**
    *   Frontend IPC calls must match backend `#[tauri::command]` command names exactly.
    *   Frontend code never writes app config or LLM config files directly; writes flow through Tauri commands.
4.  **Performance target**
    *   Foreground-app input switching should complete within 100ms where possible and stay under 200ms in normal operation.
5.  **Secret handling**
    *   `.env.llm` is local-only and ignored by Git.
    *   API keys should be masked in UI and persisted securely.

### 2.3 Scaffold Origin

The project was initialized from `nomandhoni-cs/tauri-nextjs-shadcn-boilerplate` and then adapted for SmartIME's macOS input-method switching domain.

The original scaffold import preserved existing repository files and excluded the template `.git` directory. Development commands, setup steps, debugging workflow, and release procedures are maintained in `README.md`.

## 3. Project Architecture Design

### 3.1 Directory Structure

Current repository structure (build artifacts such as `node_modules`, `.next`, `out`, `src-tauri/target` are omitted):

```text
SmartIME/
├── app/                              # Next.js App Router pages
│   ├── page.tsx                      # Startup router gate
│   ├── onboarding/
│   │   ├── page.tsx                  # Redirect to onboarding/permission
│   │   ├── permission/page.tsx       # Accessibility onboarding
│   │   ├── llm/page.tsx              # LLM config onboarding
│   │   └── scan/page.tsx             # Initial scan + bootstrap config
│   └── settings/
│       ├── rules/page.tsx            # Rule management
│       └── general/page.tsx          # General settings
├── components/
│   ├── layout/                       # Sidebar/layout shell
│   ├── motion/                       # Animation wrappers
│   ├── settings/rules/               # Rules-specific UI components
│   └── ui/                           # shadcn/ui primitives
├── lib/
│   ├── api.ts                        # Frontend IPC wrapper and types
│   ├── utils.ts
│   └── window-size.ts
├── public/                           # Static assets (icons/images)
├── src-tauri/                        # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs                   # Runtime entry, command registration
│   │   ├── command.rs                # Tauri IPC commands
│   │   ├── config.rs                 # Config models + persistence manager
│   │   ├── error.rs                  # Unified app error type
│   │   ├── general_settings.rs       # Dock/tray/auto-start integration
│   │   ├── input_source.rs           # TIS input-source query/switch
│   │   ├── llm.rs                    # LLM client + secure request resolution
│   │   ├── observer.rs               # NSWorkspace focus observer
│   │   ├── single_instance.rs        # Unix-socket single instance
│   │   ├── system_apps.rs            # Installed app scanner
│   │   └── lib.rs                    # Template library entry (not runtime path)
│   ├── capabilities/
│   │   └── default.json              # Tauri capability profile
│   ├── icons/                        # App/tray icons
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
├── .github/
│   └── workflows/
│       └── release-dmg.yml           # Tag-triggered Apple Silicon DMG release workflow
├── docs/
│   ├── DESIGN_DOC.md                 # Product and UX design document
│   ├── Rulebook.md                   # AI mistake-prevention notes and testing lessons
│   ├── REQUIREMENTS.md               # Product requirements
│   ├── TASKS.md                      # User-requested or Plan-mode task planning
│   ├── TECHNICAL_SPEC.md             # Technical specification
│   └── exec-plan/                    # Records after confirmed planned work is executed
├── CHANGELOG.md                      # Release history
├── README.md                         # User-facing overview
└── AGENTS.md                         # AI agent documentation index
```

### 3.2 Backend Module Responsibilities

| Module | Responsibility | Key Dependencies |
| :--- | :--- | :--- |
| `main.rs` | Tauri app bootstrap, global state registration, command binding, startup integration, close/reopen lifecycle. | `tauri`, `tauri-plugin-log`, `tauri-plugin-store` |
| `command.rs` | IPC command layer for input sources, config, LLM operations, scanning, rescan lifecycle, and permissions. | `tauri::command`, `AppState` |
| `config.rs` | Core config data models and JSON persistence (`config.json`) plus in-memory rule cache (`HashMap`) and planned runtime-only app metadata/icon caches used to accelerate Rules rendering. | `serde`, `serde_json`, `dirs`, `std::fs`, `std::collections` |
| `llm.rs` | HTTPS-only requests, secret-free runtime client, serialized credential access, debug-only environment import, and prediction parsing. | `reqwest`, `dotenvy`, `zeroize` |
| `credentials.rs` | Keychain storage, destination binding, verified legacy migration, atomic metadata writes, key replacement/deletion and retryable retired-key cleanup. | `security-framework`, `serde`, `uuid`, `zeroize` |
| `input_source.rs` | macOS input source discovery/filtering, system-localized display-name resolution, current input-source query, and switching (`TISSelectInputSource`). | `core-foundation`, Carbon FFI, AppKit `NSTextInputContext`, `defaults export` parsing |
| `system_apps.rs` | App bundle scanning in user, system, and CoreServices app locations; localized app display-name resolution; Info.plist parsing and de-dup by bundle ID. | `walkdir`, `plist`, `NSFileManager` |
| `app_icon.rs` | Runtime macOS app icon lookup from installed bundle paths and PNG data URL conversion for Rules UI display. | `NSWorkspace`, `NSImage`, `NSBitmapImageRep` |
| `observer.rs` | NSWorkspace active-app notifications, emits `app_focused`, and applies rules on main thread after comparing the current input source with the target rule. | `cocoa`, `objc`, `once_cell` |
| `general_settings.rs` | Applies `auto_start` and `hide_dock_icon` settings, tray icon visibility, macOS LaunchAgent management. | `tauri tray`, `launchctl`, `std::process` |
| `single_instance.rs` | Enforces one app instance via Unix domain socket and focuses existing main window on re-activation. | `std::os::unix::net`, `tauri` |
| `error.rs` | Unified `AppError` model serialized to frontend-friendly strings. | `thiserror`, `serde` |
| `lib.rs` | Template library entry (`greet`) retained from scaffold; runtime path is `main.rs`. | Tauri template glue |

### 3.3 Frontend Runtime Layers

1.  **Routing and flow control (`app/`)**
    *   `app/page.tsx` is a bootstrap router gate.
    *   Routes into onboarding or settings based on config presence + permission status.
2.  **Onboarding sequence**
    *   `/onboarding/permission`: request/check Accessibility permission.
    *   `/onboarding/llm`: configure and validate LLM connection.
    *   `/onboarding/scan`: run initial app/input-source scan and persist initial rules.
3.  **Main settings sequence**
    *   `/settings/rules`: search/edit/delete rules and run background rescan.
    *   `/settings/general`: toggle global switch, auto-start, and Dock/tray behavior.
4.  **Frontend integration layer (`lib/api.ts`)**
    *   Defines shared TypeScript models.
    *   Centralizes all Tauri command invocations.
    *   Provides browser-preview fallback behavior when Tauri runtime is absent.
5.  **UI composition**
    *   `components/layout/*`: sidebar + app shell.
    *   `components/settings/rules/*`: rules interactions (input method selector).
    *   `components/ui/*`: shadcn/ui primitives.

### 3.4 Persistence Layout and Runtime State

1.  **Config persistence**
    *   `dirs::config_dir()/smartime/config.json` stores `AppConfig` (rules + general settings + global switch).
    *   `ConfigManager` keeps in-memory cache (`rule_map`) for fast bundle-ID lookup.
2.  **LLM persistence**
    *   `dirs::config_dir()/smartime/llm_config.json` stores only `model`, `base_url`, `credential_id`, and optional `retired_credentials` cleanup references; writes use a new 0600 temporary file, sync and atomic rename. Missing user directory is an error, never a current-directory fallback.
    *   macOS Keychain service `com.smartime.app.llm` stores a random-ID credential containing the API Key and its bound Base URL. Public metadata edits cannot reroute an existing key. A replacement is written and read-verified before committing metadata; retired keys are deleted afterward, with references retained for cleanup retries.
    *   First credential access migrates legacy `api_key` JSON: Keychain write/read verification must succeed before the plaintext field is removed. Failure preserves the original file and returns an error; corrupt JSON never triggers fallback. No plaintext backup is created.
    *   `LLMClient` retains only the config path. Requests resolve secrets on blocking workers, then release/zeroize owned key buffers; HTTP uses HTTPS only, disables redirects, and returns safe status/error messages without provider response bodies. A shared lock serializes credential operations, including prediction snapshots.
    *   Browser preview clears legacy `smartime_llm`, uses a disabled virtual-key field, and never persists real credentials. Production CSP permits bundled scripts and native IPC, with development-only script/HMR allowances.
3.  **Global runtime state (`AppState`)**
    *   `config: Mutex<ConfigManager>`
    *   `llm: Mutex<LLMClient>`
    *   `is_rescanning: AtomicBool`

### 3.5 Frontend and UX Architecture Notes

1.  **Design system**
    *   UI should stay aligned with shadcn/ui, Tailwind CSS, Radix UI primitives, and lucide-react icons.
    *   Motion should use framer-motion for subtle transitions such as fade, slide, and layout animations.
2.  **Visual environment**
    *   The interface adapts to macOS light/dark mode.
    *   The app uses the system font stack, matching the San Francisco feel on macOS.
3.  **Window sizing targets**
    *   Main settings window target: 800x600.
    *   Menu bar/tray panel target: 300x400.
4.  **State shape**
    *   Current implementation primarily uses page-local React state and Tauri IPC calls.
    *   Avoid introducing a global client state store unless the workflow genuinely needs shared state across pages.

### 3.6 Configuration and Identity Conventions

1.  **App identity**
    *   Cargo package name: `smartime`
    *   Bundle identifier: `com.smartime.app`
    *   Product name: `SmartIME`
2.  **LLM environment file convention**
    *   Debug-only import file: `.env.llm` (not read in release builds)
    *   Template: `.env.llm.example`
    *   Keys: `LLM_API_KEY`, `LLM_MODEL`, `LLM_BASE_URL`
3.  **Source of truth priority**
    *   Existing metadata/legacy file is authoritative. Only debug builds with no file may import `.env.llm` / process environment once into Keychain; otherwise use empty defaults. Keychain and parse errors never trigger environment fallback. Deletion retains empty metadata to prevent re-import.

## 4. System Design

### 4.1 Frontend-Backend Communication

SmartIME uses Tauri's **IPC (Inter-Process Communication)** mechanism.

#### Commands - Frontend calls Backend
| Command Name | Payload | Return Value | Description |
| :--- | :--- | :--- | :--- |
| `cmd_get_system_input_sources` | None | `Result<Vec<InputSource>, AppError>` | Fetch currently enabled/selectable system input sources on the main thread with system-localized display names when available. |
| `cmd_select_input_source` | `id: String` | `Result<(), AppError>` | Switch to a specific input source ID on the main thread. |
| `cmd_get_installed_apps` | None | `Result<Vec<SystemApp>, AppError>` | Scan installed apps under `/Applications`, `~/Applications`, `/System/Applications`, `/System/Cryptexes/App/System/Applications`, and `/System/Library/CoreServices`, then keep only curated input-capable system apps from system roots. |
| `cmd_get_app_icons` | `bundle_ids: Vec<String>` | `Result<HashMap<String, String>, AppError>` | Reuse runtime-installed-app metadata and icon caches where possible, resolve any missing bundle paths by bundle ID, render macOS icons to PNG data URLs on the main thread, and return only successfully resolved icons. |
| `cmd_get_config` | None | `Result<AppConfig, AppError>` | Load app config from state/persistence. |
| `cmd_has_config` | None | `Result<bool, AppError>` | Whether `config.json` exists. |
| `cmd_save_config` | `config: AppConfig` | `Result<(), AppError>` | Save full config; apply general settings delta when changed. |
| `cmd_save_rules` | `rules: Vec<AppRule>` | `Result<(), AppError>` | Save only rules without overriding other config fields. |
| `cmd_get_llm_config` | None | `Result<LLMConfigStatus, AppError>` | Return model, base_url, has_api_key; never return a secret or placeholder. |
| `cmd_save_llm_config` | `config: LLMConfig` | `Result<(), AppError>` | Save metadata and verified Keychain credential; empty api_key reuses the existing key only for the same Base URL. |
| `cmd_delete_llm_key` | None | `Result<(), AppError>` | Delete Keychain credential and clear its reference; no environment re-import on next launch. |
| `cmd_check_llm_connection` | `config: LLMConfig` | `Result<bool, AppError>` | Validate LLM endpoint/auth by test request. |
| `cmd_scan_and_predict` | `input_sources: Vec<InputSource>` | `Result<Vec<AppRule>, AppError>` | Discover target apps and generate initial rules, preferring batch LLM prediction and validating every returned rule against the provided input source list. |
| `cmd_rescan_and_save_rules` | None | `Result<Vec<AppRule>, AppError>` | Background-safe rescan + align + persist (single in-flight), reusing existing valid manual and AI rules and predicting only missing/new/invalid rule gaps. |
| `cmd_is_rescanning` | None | `bool` | Query whether backend rescan task is currently running. |
| `cmd_check_permissions` | None | `bool` | Accessibility permission check only (no prompt). |
| `cmd_request_permissions` | None | `bool` | Trigger native Accessibility authorization prompt. |
| `cmd_open_system_settings` | None | `()` | Open Accessibility settings page (manual fallback path). |

#### Events - Backend pushes to Frontend
| Event Name | Payload | Description |
| :--- | :--- | :--- |
| `app_focused` | `{ bundle_id: String, app_name: String }` | Emitted by observer on foreground app change. Input switching is applied in backend in the same processing loop. |

### 4.2 Data Flow

1.  **Startup routing (`app/page.tsx`)**
    *   Call `cmd_has_config`.
    *   If config missing: check permission.
        *   no permission -> `/onboarding/permission`
        *   permission granted -> `/onboarding/llm`
    *   If config exists: check permission.
        *   no permission -> `/onboarding/permission`
        *   permission granted -> `/settings/rules`

2.  **Permission onboarding**
    *   Guide action calls `cmd_request_permissions` (prompt path).
    *   Retry action calls `cmd_check_permissions` (check-only path).
    *   Optional manual fallback can call `cmd_open_system_settings`.

3.  **LLM onboarding**
    *   Read existing config via `cmd_get_llm_config` (has_api_key only). Leave key blank to reuse it at the same Base URL; a changed URL requires a new key. Expose replacement/deletion and recoverable storage errors. Credential IPC runs Keychain/file work via spawn_blocking.
    *   Validate with `cmd_check_llm_connection`.
    *   Persist via `cmd_save_llm_config` and continue to scan step.

4.  **Initial scan bootstrap (`/onboarding/scan`)**
    *   Fetch input sources via `cmd_get_system_input_sources`.
    *   Call `cmd_scan_and_predict(input_sources)`.
    *   Backend discovers target apps and requests batch LLM prediction for the target set where possible.
    *   Frontend progress should present real phases; the scan UI must not imply exact per-app completion when the backend is waiting for a single batch LLM response.
    *   Build initial config (`global_switch=true`, `general.auto_start=false`, `general.hide_dock_icon=false`) and persist via `cmd_save_config`.
    *   Scan flow may also warm runtime app metadata caches for the Rules page redirect target, but icon payloads remain runtime-only and are never persisted into `config.json`.
    *   Redirect to startup gate (`/`) then to `/settings/rules`.

5.  **Rules page lifecycle**
    *   Initial load calls `cmd_get_config`, `cmd_get_system_input_sources`, and `cmd_is_rescanning` concurrently.
    *   After rules are known, frontend derives a first-screen visible subset of rule bundle IDs and requests those icons through `cmd_get_app_icons` first.
    *   Backend should reuse runtime app metadata and icon caches rather than rescanning the full installed-app set for every icon request. Icons remain UI/runtime cache only and are not persisted to `config.json`.
    *   Once first-screen rows are visually stable, frontend may request remaining icon batches in background for off-screen rows.
    *   Pending icon load and confirmed icon lookup failure are distinct UI states; the initial-letter avatar is reserved for true fallback only.
    *   Manual rule edits persist with `cmd_save_rules`.
    *   Rescan calls `cmd_rescan_and_save_rules` and polls `cmd_is_rescanning` until false, then reloads config + input sources.
    *   Rescan keeps valid existing manual and AI rules, prunes stale apps, normalizes invalid input source IDs, and calls LLM only for apps still missing a valid rule.
    *   Rescan should also refresh the runtime app metadata cache and favor first-screen icon readiness when the user returns to Rules.

6.  **Foreground app switching**
    *   `observer.rs` receives `NSWorkspaceDidActivateApplicationNotification`.
    *   Emits `app_focused` and resolves rule from in-memory cache.
    *   Applies input source switch on the main thread after comparing the current system input source with the target rule.
    *   No custom input-method indicator is emitted; automatic switching remains a silent background behavior.

7.  **General settings application**
    *   Startup applies persisted general settings once in `setup()`.
    *   `cmd_save_config` compares previous vs next `general` values and applies only changed settings (`apply_general_settings_delta`).
    *   `hide_dock_icon` close behavior is enforced in `on_window_event(CloseRequested)`.

8.  **Single-instance activation**
    *   New process attempts Unix socket handshake.
    *   If primary instance exists, send activation signal and exit.
    *   Primary instance focuses the existing main window.

### 4.3 Data Structure Definitions

Rust and TypeScript use the same field names (snake_case) for IPC payload compatibility.

```typescript
type InputSource = {
  id: string
  name: string
  category: string
}

type SystemApp = {
  name: string
  bundle_id: string
  path: string
}

type AppIconMap = Record<string, string> // bundle_id -> PNG data URL, runtime UI cache only

type AppRule = {
  bundle_id: string
  app_name: string // Latest localized display-name snapshot from scan/rescan; bundle_id remains the matching key.
  preferred_input: string
  is_ai_generated: boolean
}

type GeneralSettings = {
  auto_start: boolean
  hide_dock_icon: boolean
}

type AppConfig = {
  version: number
  global_switch: boolean
  default_input: "en" | "zh" | "keep" // currently persisted, not active in switch loop
  general: GeneralSettings
  rules: AppRule[]
}

type LLMConfigStatus = {
  model: string
  base_url: string
  has_api_key: boolean
}

type LLMConfig = { // Write/test input only; never used as a response or disk schema.
  api_key: string
  model: string
  base_url: string
}

```

### 4.4 AI Prediction Module (Logic Design)

1.  **Target app set**
    *   Scan user-installed apps plus macOS system-app roots including Cryptex Safari locations.
    *   Keep all user-installed apps with non-empty name and bundle ID.
    *   From system roots, keep only a curated allowlist of common input-capable Apple apps such as Safari, Terminal, TextEdit, Notes, Reminders, Mail, Messages, Calendar, and Finder.
    *   Prefer the localized app display name that macOS exposes for the installed bundle when available.
    *   If the system display name is only the plain `.app` filename, continue checking localized app resources before falling back to curated Simplified Chinese labels for supported system apps.
2.  **Batch prediction**
    *   Prefer one batch LLM call for a set of target apps using current `LLMConfig`.
    *   Prompt includes available input source IDs/names and target apps (`name`, `bundle_id`).
    *   Response format should be strict JSON mapping bundle IDs to input source IDs.
    *   If a provider response cannot be parsed or validated, the caller may retry in smaller batches or fall back to deterministic fallback rules rather than blocking indefinitely on per-app serial calls.
3.  **Validation**
    *   If returned ID is not in `input_sources`, treat that app prediction as invalid.
    *   Ignore unknown returned bundle IDs and missing entries.
    *   Prediction failures are logged and skipped; pipeline continues for other apps.
4.  **Rule alignment**
    *   Preserve manual rules (`is_ai_generated == false`) first.
    *   Reuse existing valid rules, including AI-generated rules, where still relevant.
    *   Apply newly generated AI rules only for apps with no valid existing rule.
    *   Create fallback AI rule with first available input source if no rule exists.
5.  **Normalization**
    *   Any rule referencing removed/invalid input source ID is rewritten to fallback first input source.

### 4.5 Directory & State Management

1.  **Frontend state**
    *   Managed with local React state per page (`useState`, `useEffect`, polling where needed).
    *   No centralized Zustand store in current implementation.
2.  **Backend state**
    *   Centralized in `AppState` and shared through Tauri `State`.
    *   Access synchronized with `Mutex`; rescan progress guarded by `AtomicBool`.
3.  **Persistence boundary**
    *   Frontend never writes files directly.
    *   All writes flow through IPC commands (`cmd_save_config`, `cmd_save_rules`, `cmd_save_llm_config`).

### 4.6 General Settings Integration (macOS)

1.  **Auto-start (`auto_start`)**
    *   Implemented via LaunchAgent plist in `~/Library/LaunchAgents/<bundle-id>.plist`.
    *   Uses `launchctl bootout/unload` best-effort disable path on toggle off.
2.  **Hide Dock Icon (`hide_dock_icon`)**
    *   When enabled and main window closes, close is intercepted:
        *   prevent close
        *   hide Dock icon
        *   hide main window
    *   Tray icon remains visible for background re-entry.
3.  **Tray icon behavior**
    *   Tray icon created/updated by `sync_tray_icon_visibility`.
    *   Left click shows, unminimizes, and focuses main window.
4.  **Single-instance reactivation**
    *   Dock reopen and socket activation both focus existing main window.
    *   Prevents duplicate running instances for typical launch/reactivation paths.

### 4.7 Error and Concurrency Model

1.  **Unified error surface**
    *   Backend errors are represented by `AppError` (`Io`, `Network`, `Config`, `InputSource`, `Llm`, `Lock`, `Json`).
    *   Serialized into string messages for IPC responses.
2.  **Rescan concurrency guard**
    *   `cmd_rescan_and_save_rules` uses `AtomicBool::compare_exchange` to enforce single in-flight execution.
    *   RAII guard resets `is_rescanning` even on early-return errors.
3.  **Main-thread boundaries**
    *   Input-source retrieval in rescan uses `run_on_main_thread` + timeout guard.
    *   Frontend-exposed input-source commands (`get system`, `select`) also use the same main-thread scheduling helper.
    *   Observer-triggered input switching runs on main thread with timeout and current-system-input comparison.

## 5. Build & Deployment

### 5.1 Environment Requirements
*   **OS**: macOS (macOS only, as it depends on specific input method APIs)
*   **Runtime**: Bun v1.0+
*   **Rust**: 1.70+ (Recommended to install via `rustup`)
*   **XCode Command Line Tools**: Must be installed to support macOS system library compilation.

### 5.2 Build Process

1.  **Production Build**:
    ```bash
    bun tauri build
    ```
    This command builds the Next.js frontend (`next build` + static export), compiles the Rust backend for the current Apple Silicon release target, and packages an Apple Silicon DMG.

2.  **Artifact Location**:
    Apple Silicon DMG artifact is generated at:
    `src-tauri/target/release/bundle/dmg/SmartIME_<version>_aarch64.dmg`

3.  **Universal Build Scope**:
    Universal DMG packaging is not currently supported by the release pipeline. Intel (`x86_64`) support is tracked as future roadmap work rather than current distribution behavior.

### 5.3 Release & Distribution (GitHub Release + Homebrew Cask)

To support `brew install --cask` installation, distribution is based on Git tags, GitHub Releases, and a Homebrew Tap cask.

1.  **Release Trigger**:
    GitHub Actions workflow `.github/workflows/release-dmg.yml` is triggered on `push` tags in format `v<version>` (for example `v0.1.0`).

2.  **Version Consistency Gate**:
    The workflow validates that tag version matches:
    *   `package.json`
    *   `src-tauri/Cargo.toml`
    *   `src-tauri/tauri.conf.json`

3.  **CI Build and Publish Steps**:
    *   Install Bun dependencies and Rust toolchain targets.
    *   Run `bun tauri build`.
    *   Generate SHA256 checksum for DMG.
    *   Publish `SmartIME_<version>_aarch64.dmg` and checksum file to GitHub Release.

4.  **Define Cask (`Casks/smartime.rb`)**:
    The cask in Tap repository should point to the Apple Silicon DMG:

    ```ruby
    cask "smartime" do
      version "0.1.0"
      sha256 "<CHECKSUM_FROM_RELEASE_SHA256_FILE>"

      url "https://github.com/<USERNAME>/SmartIME/releases/download/v#{version}/SmartIME_#{version}_aarch64.dmg"
      name "SmartIME"
      desc "Automatic input method switcher based on active app"
      homepage "https://github.com/<USERNAME>/SmartIME"

      app "SmartIME.app"
    end
    ```

5.  **Signing and Notarization Scope**:
    Current release pipeline intentionally skips signing and notarization, and only provides unsigned DMG artifacts.

## 6. Operational Cross-References

AI-prone mistake patterns, testing methods, project lessons, and release regression matrices live in `docs/Rulebook.md`.

Implementation details and execution histories for confirmed planned work should be stored under `docs/exec-plan/`.
