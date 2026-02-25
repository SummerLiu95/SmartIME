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

### 2.2 Project Initialization

Project initialization flow based on the template (excluding the template's `LICENSE`, `.gitignore`, `.git`):

```bash
# 1. Clone template repository to a temporary directory
git clone https://github.com/nomandhoni-cs/tauri-nextjs-shadcn-boilerplate temp_boilerplate

# 2. Copy template files to the project directory
# Use --ignore-existing argument to ensure all existing files and configurations in the project root are preserved
# (including .figma, .idea, .trae, LICENSE, .gitignore, tray-icon.svg, etc.)
# Only exclude the template's .git directory to avoid breaking the current repository's version control
rsync -av --progress temp_boilerplate/ . --exclude .git --ignore-existing

# 3. Clean up temporary directory
rm -rf temp_boilerplate

# 4. Install dependencies
bun install

# 5. Update project metadata
# - package.json: name ("smartime"), version, description, author
# - src-tauri/tauri.conf.json: productName ("SmartIME"), identifier ("com.smartime.app"), version
# - src-tauri/Cargo.toml: name ("smartime"), version, authors

# 6. Install additional UI dependencies (Icons & Animations)
bun add lucide-react framer-motion clsx tailwind-merge

# 7. Start development server
bun tauri dev
```

### 2.3 Development & Debugging Flow

1.  **Start Development Environment**:
    ```bash
    bun tauri dev
    ```
    This command starts both the Next.js frontend hot-reload server (localhost:3000) and the Tauri application window.

2.  **Frontend Debugging**:
    *   **UI Inspection**: Right-click in the Tauri window -> "Inspect Element" to open the Web Inspector (Safari style).
    *   **Console**: Use `console.log` to output logs, which can be viewed in the Web Inspector's Console panel.

3.  **Backend Debugging (Rust)**:
    *   **Log Output**: Use `println!` or `eprintln!` macros to print logs; output content will be displayed directly in the terminal window running `bun tauri dev`.
    *   **Code Modification**: After modifying Rust code under `src-tauri/src`, Tauri will automatically recompile and restart the application.

4.  **Common Troubleshooting**:
    *   If IPC communication fails, please check if the command name in the frontend `invoke` exactly matches the function name defined by the `#[tauri::command]` macro in the backend.

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
│   │   ├── llm.rs                    # LLM client + persistence
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
│       └── release-dmg.yml           # Tag-triggered universal DMG release workflow
├── REQUIREMENTS.md                   # Product requirements
├── TECHNICAL_SPEC.md                 # Technical specification
├── AGENTS.md                         # Agent rulebook
└── CLAUDE.md                         # Agent guidance
```

### 3.2 Backend Module Responsibilities

| Module | Responsibility | Key Dependencies |
| :--- | :--- | :--- |
| `main.rs` | Tauri app bootstrap, global state registration, command binding, startup integration, close/reopen lifecycle. | `tauri`, `tauri-plugin-log`, `tauri-plugin-store` |
| `command.rs` | IPC command layer for input sources, config, LLM operations, scanning, rescan lifecycle, and permissions. | `tauri::command`, `AppState` |
| `config.rs` | Core config data models and JSON persistence (`config.json`) plus in-memory rule cache (`HashMap`). | `serde`, `serde_json`, `dirs`, `std::fs` |
| `llm.rs` | LLM config/model client, config persistence (`llm_config.json`), connectivity checks, per-app prediction calls. | `reqwest`, `dotenvy`, `serde` |
| `input_source.rs` | macOS TIS input source discovery/filtering and switching (`TISSelectInputSource`). | `core-foundation`, Carbon FFI, `defaults export` parsing |
| `system_apps.rs` | App bundle scanning in `/Applications` and `~/Applications`, Info.plist parsing and de-dup by bundle ID. | `walkdir`, `plist` |
| `observer.rs` | NSWorkspace active-app notifications, emits `app_focused`, applies rules on main thread. | `cocoa`, `objc`, `once_cell` |
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
    *   `dirs::config_dir()/smartime/llm_config.json` stores `LLMConfig`.
    *   If file does not exist, startup fallback reads `.env.llm`.
3.  **Global runtime state (`AppState`)**
    *   `config: Mutex<ConfigManager>`
    *   `llm: Mutex<LLMClient>`
    *   `is_rescanning: AtomicBool`

### 3.5 Configuration and Identity Conventions

1.  **App identity**
    *   Cargo package name: `smartime`
    *   Bundle identifier: `com.smartime.app`
    *   Product name: `SmartIME`
2.  **LLM environment file convention**
    *   Runtime file: `.env.llm`
    *   Template: `.env.llm.example`
    *   Keys: `LLM_API_KEY`, `LLM_MODEL`, `LLM_BASE_URL`
3.  **Source of truth priority**
    *   LLM config loading order: persisted file -> `.env.llm` -> default config.

## 4. System Design

### 4.1 Frontend-Backend Communication

SmartIME uses Tauri's **IPC (Inter-Process Communication)** mechanism.

#### Commands - Frontend calls Backend
| Command Name | Payload | Return Value | Description |
| :--- | :--- | :--- | :--- |
| `cmd_get_system_input_sources` | None | `Result<Vec<InputSource>, AppError>` | Fetch currently enabled/selectable system input sources. |
| `cmd_select_input_source` | `id: String` | `Result<(), AppError>` | Switch to a specific input source ID. |
| `cmd_get_installed_apps` | None | `Result<Vec<SystemApp>, AppError>` | Scan installed apps under `/Applications` and `~/Applications`. |
| `cmd_get_config` | None | `Result<AppConfig, AppError>` | Load app config from state/persistence. |
| `cmd_has_config` | None | `Result<bool, AppError>` | Whether `config.json` exists. |
| `cmd_save_config` | `config: AppConfig` | `Result<(), AppError>` | Save full config; apply general settings delta when changed. |
| `cmd_save_rules` | `rules: Vec<AppRule>` | `Result<(), AppError>` | Save only rules without overriding other config fields. |
| `cmd_get_llm_config` | None | `Result<LLMConfig, AppError>` | Load LLM config (API key masked as `******` when present). |
| `cmd_save_llm_config` | `config: LLMConfig` | `Result<(), AppError>` | Save LLM config to persistent file. |
| `cmd_check_llm_connection` | `config: LLMConfig` | `Result<bool, AppError>` | Validate LLM endpoint/auth by test request. |
| `cmd_scan_and_predict` | `input_sources: Vec<InputSource>` | `Result<Vec<AppRule>, AppError>` | Predict rules for target apps using provided input source list. |
| `cmd_rescan_and_save_rules` | None | `Result<Vec<AppRule>, AppError>` | Background-safe rescan + align + persist (single in-flight). |
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
    *   Read existing config via `cmd_get_llm_config` (masked API key handling).
    *   Validate with `cmd_check_llm_connection`.
    *   Persist via `cmd_save_llm_config` and continue to scan step.

4.  **Initial scan bootstrap (`/onboarding/scan`)**
    *   Fetch input sources via `cmd_get_system_input_sources`.
    *   Call `cmd_scan_and_predict(input_sources)`.
    *   Build initial config (`global_switch=true`, `general.auto_start=false`, `general.hide_dock_icon=false`) and persist via `cmd_save_config`.
    *   Redirect to startup gate (`/`) then to `/settings/rules`.

5.  **Rules page lifecycle**
    *   Initial load calls `cmd_get_config`, `cmd_get_system_input_sources`, and `cmd_is_rescanning` concurrently.
    *   Manual rule edits persist with `cmd_save_rules`.
    *   Rescan calls `cmd_rescan_and_save_rules` and polls `cmd_is_rescanning` until false, then reloads config + input sources.

6.  **Foreground app switching**
    *   `observer.rs` receives `NSWorkspaceDidActivateApplicationNotification`.
    *   Emits `app_focused` and resolves rule from in-memory cache.
    *   Applies input source switch on the main thread with de-dup against last selected input.

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

type AppRule = {
  bundle_id: string
  app_name: string
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

type LLMConfig = {
  api_key: string
  model: string
  base_url: string
}
```

### 4.4 AI Prediction Module (Logic Design)

1.  **Target app set**
    *   Scan installed apps.
    *   Filter out bundle IDs starting with `com.apple.` for prediction targets.
2.  **Per-app prediction**
    *   Run one LLM call per app using current `LLMConfig`.
    *   Prompt includes available input source IDs/names and strict response format: output only one ID.
3.  **Validation**
    *   If returned ID is not in `input_sources`, treat as invalid prediction (error).
    *   Per-app prediction failures are logged and skipped; pipeline continues for other apps.
4.  **Rule alignment**
    *   Preserve manual rules (`is_ai_generated == false`) first.
    *   Apply generated AI rules next.
    *   Reuse existing rules where still relevant.
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
    *   Observer-triggered input switching also runs on main thread with timeout and de-dup.

## 5. Build & Deployment

### 5.1 Environment Requirements
*   **OS**: macOS (macOS only, as it depends on specific input method APIs)
*   **Runtime**: Bun v1.0+
*   **Rust**: 1.70+ (Recommended to install via `rustup`)
*   **XCode Command Line Tools**: Must be installed to support macOS system library compilation.

### 5.2 Build Process

1.  **Production Build**:
    ```bash
    bun tauri build --target universal-apple-darwin
    ```
    This command builds the Next.js frontend (`next build` + static export), compiles Rust code for both Apple Silicon and Intel targets, and packages a universal DMG.

2.  **Artifact Location**:
    Universal DMG artifact is generated at:
    `src-tauri/target/universal-apple-darwin/release/bundle/dmg/SmartIME_<version>_universal.dmg`

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
    *   Install Bun dependencies and Rust targets (`aarch64` + `x86_64`).
    *   Run `bun tauri build --target universal-apple-darwin`.
    *   Generate SHA256 checksum for DMG.
    *   Publish DMG and checksum file to GitHub Release.

4.  **Define Cask (`Casks/smartime.rb`)**:
    The cask in Tap repository should point to universal DMG:

    ```ruby
    cask "smartime" do
      version "0.1.0"
      sha256 "<CHECKSUM_FROM_RELEASE_SHA256_FILE>"

      url "https://github.com/<USERNAME>/SmartIME/releases/download/v#{version}/SmartIME_#{version}_universal.dmg"
      name "SmartIME"
      desc "Automatic input method switcher based on active app"
      homepage "https://github.com/<USERNAME>/SmartIME"

      app "SmartIME.app"
    end
    ```

5.  **Signing and Notarization Scope**:
    Current release pipeline intentionally skips signing and notarization, and only provides unsigned DMG artifacts.

## 6. Post-Release Engineering Knowledge Base (2026-02)

This section captures production-oriented lessons from onboarding tests and regression tests, including bug records, fixes, UI interaction optimizations, and engineering guardrails for future Tauri work.

### 6.1 Incident Catalog (Issue -> Fix -> Lesson)

| Incident ID | Area | What Happened | Implemented Fix | Engineering Lesson |
| :--- | :--- | :--- | :--- | :--- |
| INC-001 | Accessibility authorization | In dev/debug scenarios, users could not reliably add the app from Accessibility settings or could not locate the expected app identity. | Standardized app identity usage across `Cargo.toml`, `tauri.conf.json` (`productName`, `identifier`), and packaged app metadata. Added bundle-first validation for permission behavior. | macOS TCC behavior is identity-sensitive; name/identifier drift causes permission confusion. |
| INC-002 | Permission onboarding UX | Permission guide action triggered both native prompt and system settings navigation, while retry/check also triggered prompt. This created duplicated/conflicting behavior. | Split actions by intent: guide card triggers authorization request only; retry button checks permission status only. Removed implicit cross-triggering. | Request and verification flows must be decoupled, especially for OS-level permission UX. |
| INC-003 | Rescan stability | Clicking rescan could crash app (`EXC_BREAKPOINT` / `SIGTRAP`, tokio worker). | Treated rescan as a single in-flight operation, blocked duplicate triggers, replaced panic-prone runtime paths with explicit error handling, and guarded merge/persist sequence. | Async runtime paths in desktop apps must be panic-free and idempotent. |
| INC-004 | Onboarding -> rules loading state | After onboarding scan success and redirect, rules panel could remain in a perpetual loading state. | Corrected completion-state propagation and ensured scan completion reliably clears loading state after config save. | Cross-page async completion must have one authoritative state transition. |
| INC-005 | Input method list drift | Input method options showed entries that did not match current system-enabled input methods; removed system methods could remain in app options. | Rebuilt filtering logic to use enabled system input sources, removed helper/non-selectable items, deduplicated overlapping entries, and pruned stale options during scan/rescan sync. | Treat system input sources as source of truth on every scan, not as append-only state. |
| INC-006 | Dock/tray/autostart lifecycle | Hide Dock mode, login-item flow, and relaunch/reactivation had edge cases: duplicate icons, wrong reopen behavior, or premature window close. | Separated `autoStart` and `hideDockIcon` concerns, preserved single-instance reactivation behavior, and aligned close/reopen semantics between Dock and tray entry points. | App lifecycle settings must be independent, and all entry points must resolve to one running instance/window state. |

### 6.2 Interaction and UI Optimizations from Testing

#### 6.2.1 Onboarding Permission Page

1.  Changed guide card interaction to trigger native Accessibility authorization request only.
2.  Changed retry/check button to permission check only (`check_permissions`), with no authorization side effect.
3.  Removed automatic dual-action behavior (prompt + settings navigation in one click).
4.  Clarified fallback behavior: manual system settings navigation is explicit, not implicit.

#### 6.2.2 Rules Management Page

1.  Kept rescan loading indicator accurate and persistent until task completion.
2.  Ensured rescan continues and commits even when user navigates to another settings panel.
3.  Corrected loading-state reset after onboarding redirects into rules view.
4.  Enforced application list and input source list full re-sync with system state on every scan/rescan.

#### 6.2.3 General Settings Page

1.  Updated `Hide Dock Icon` behavior so toggling does not close the main window immediately.
2.  Preserved background-running behavior after closing window in hide-Dock mode.
3.  Ensured Dock/tray reactivation opens the existing settings window in a single running process.
4.  Kept `Start at Login` behavior independent from `Hide Dock Icon` state.

### 6.3 Tauri/macOS Testing Methodology Lessons

1.  **Bundle-first principle for system integration**
    *   Permission registration (TCC), login-item behavior, Dock/tray activation policy, and app identity should be validated with a bundled `.app`/`.dmg`.
    *   `bun tauri dev` is useful for fast iteration but not sufficient for release acceptance of OS-integrated behaviors.

2.  **Recommended debug bundle workflow for permission tests**
    *   Build/install bundle to `/Applications`.
    *   Reset permission state when needed: `tccutil reset Accessibility com.smartime.app`.
    *   Re-run onboarding permission flow and validate both authorization and check paths independently.

3.  **Crash analysis baseline**
    *   Always capture full crash report and preserve:
        *   exception type/code
        *   crashing thread name
        *   process identifier and bundle identifier
    *   Map crash location to async task lifecycle (scan, merge, persist, UI state propagation).

### 6.4 Engineering Guardrails for Future Development

1.  Never use `unwrap`/`expect` in runtime paths for scan/rescan, permission flow, or app lifecycle transitions.
2.  Keep permission request actions and permission verification actions separate at command and UI layers.
3.  Keep app identity metadata consistent across Rust package metadata, Tauri config, and release bundle metadata.
4.  Treat scan results as replacement-style synchronization with system state, not incremental append-only data.
5.  Preserve single-instance semantics for all app entry points (launch, Dock, tray, login-item).

### 6.5 Regression Validation Matrix (Release Candidate)

Run this matrix on a bundled app before release:

1.  Onboarding permission flow: request-only and check-only actions are independent.
2.  First scan output: app list and input method options match current system state.
3.  Rules rescan: no crash, duplicate triggers blocked, loading lifecycle correct across panel switches.
4.  Dock/tray behavior: hide/show Dock transitions and window reactivation behavior are stable.
5.  Login-item behavior: startup works without duplicate process/icon side effects.
