# Technical Specification

## 1. Project Overview

**SmartIME** is a macOS menu bar application built on Tauri, designed to automatically switch input methods (Chinese/English) based on the current active window. This project leverages Rust's high-performance system call capabilities to monitor application focus changes, combined with a frontend built on Next.js to provide an intuitive configuration management interface.

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

This project follows the flat directory structure of `nomandhoni-cs/tauri-nextjs-shadcn-boilerplate`:

```
SmartIME/
├── app/                    # Next.js App Router Pages (Frontend)
│   ├── globals.css         # Global styles entry
│   ├── layout.tsx          # Root layout
│   └── page.tsx            # Home page
├── src-tauri/              # Rust Backend Code (Tauri Core)
│   ├── src/
│   │   ├── main.rs         # Entry file, registers commands and system tray
│   │   ├── command.rs      # Defines Tauri Commands for frontend calls
│   │   ├── input_source.rs # macOS input method switching core logic (FFI)
│   │   ├── observer.rs     # Monitors NSWorkspace active app changes
│   │   ├── system_apps.rs  # (New) System app scanning and parsing (Walkdir + Plist)
│   │   └── llm.rs          # (New) LLM API calls and configuration management
│   ├── tauri.conf.json     # Tauri configuration file
│   └── Cargo.toml          # Rust dependency management
├── components/             # Shadcn UI components and business components
│   ├── ui/                 # Basic UI components (Button, Input, etc.)
│   └── ...                 # Business components (Sidebar, Header, etc.)
├── lib/                    # Utility functions (utils)
│   └── utils.ts            # Contains generic utility functions like cn()
├── hooks/                  # Custom React Hooks
├── public/                 # Static resources (Icons, Images)
├── styles/                 # Style files (Tailwind CSS)
├── types/                  # TypeScript type definitions
├── .github/                # GitHub configuration
│   └── workflows/          # CI/CD workflows (Build & Release)
├── components.json         # shadcn/ui configuration file
├── next.config.mjs         # Next.js configuration file
├── tailwind.config.ts      # Tailwind CSS configuration file
├── package.json            # Project dependency management
└── tray-icon.svg           # macOS menu bar icon (Template Icon)
```

### 3.2 Core Module Dependencies

1.  **Input Source Module (Rust)**:
    *   Depends on `core-foundation` and `core-graphics` crates.
    *   Switches input methods via `TISSelectInputSource` API.
    *   **New**: Provides `get_system_input_sources` function, calling `TISCreateInputSourceList` to get the list of currently available system input methods.
2.  **App Observer (Rust)**:
    *   Uses `cocoa` or `objc` crate to listen for `NSWorkspaceDidActivateApplicationNotification`.
    *   When the application switches, it sends an event to the frontend via `app_handle.emit`, or queries the configuration and switches directly in the backend.
3.  **LLM Module (Rust)**:
    *   Depends on `reqwest` (feature: `json`, `rustls-tls`).
    *   Responsible for storing and reading LLM configuration (API Key recommended to use `tauri-plugin-store` or encrypted storage).
    *   Responsible for sending prediction requests to OpenAI-compatible interfaces.
4.  **System Apps Module (Rust)**:
    *   Depends on `walkdir` (directory traversal) and `plist` (parsing Info.plist).
    *   Provides `get_installed_apps` function, scanning `/Applications` and `~/Applications`.
    *   Extracts application name and Bundle ID for building prediction context.
5.  **Config Manager (Hybrid)**:
    *   Frontend is responsible for visual configuration (App List -> Input Method Mapping).
    *   Configuration is stored in a local JSON file (using `tauri-plugin-store` or `fs` module).

### 3.3 LLM Environment Variable Configuration Standard

To uniformly manage LLM configuration across development and deployment, SmartIME adopts an independent environment variable file, avoiding writing API Keys into code or configuration files.

**File Convention**
*   **Filename**: `.env.llm` (Actual use) and `.env.llm.example` (Template)
*   **Format**: Standard `KEY=VALUE`, supporting comments (starting with `#`)
*   **Version Control**: `.env.llm` must be ignored, `.env.llm.example` can be committed for reference

**Configuration Items**
| Key | Type | Required | Description | Example |
| :--- | :--- | :--- | :--- | :--- |
| `LLM_API_KEY` | string | Yes | LLM Service API Key | `sk-xxxx` |
| `LLM_MODEL` | string | Yes | LLM Model Name | `gpt-4o-mini` |
| `LLM_BASE_URL` | string | Yes | LLM Service Base URL | `https://api.openai.com/v1` |

**Template Example**
```env
# LLM configuration
LLM_API_KEY=sk-your-key-here
LLM_MODEL=gpt-4o-mini
LLM_BASE_URL=https://api.openai.com/v1
```

**Loading Method (Rust Side Convention)**
*   Read `.env.llm` at application startup, parse and inject into `LLMConfig` (structure same as frontend `LLMConfig`).
*   If configuration is missing or read fails, a clear error message should be returned to guide the user to complete the configuration.
*   Reading method options:
    *   Directly use `std::env::var` to read injected environment variables
    *   Use libraries like `dotenvy` to load `.env.llm` and then read (if dependency is introduced later)

**Testing Best Practices**
*   Inject test variables via `std::env::set_var` in unit tests, and clean up after testing.
*   Avoid reading the real `.env.llm` to ensure tests are reproducible and parallelizable.
*   Use virtual Keys and local fake addresses only in tests.

## 4. System Design

### 4.1 Frontend-Backend Communication

SmartIME uses Tauri's **IPC (Inter-Process Communication)** mechanism.

#### Commands - Frontend calls Backend
| Command Name | Payload | Return Value | Description |
| :--- | :--- | :--- | :--- |
| `save_llm_config` | `config: LLMConfig` | `Result<bool, String>` | Save and validate LLM configuration (returns connectivity result) |
| `get_llm_config` | None | `LLMConfig` | Get current LLM configuration (API Key needs to be masked) |
| `get_system_input_sources` | None | `Vec<InputSource>` | Get the list of currently enabled system input methods |
| `scan_and_predict` | None | `Vec<AppRule>` | Execute scan + AI prediction flow (requires LLM configuration first) |
| `save_config` | `config: AppConfig` | `Result<(), String>` | Save user-modified rules |
| `get_installed_apps` | None | `Vec<AppInfo>` | Scan and return system application list |
| `get_current_input_source` | None | `String` | Get current system input method ID |
| `check_permissions` | None | `bool` | Check if accessibility permissions are granted |
| `open_system_settings` | None | None | Open macOS system settings page |

#### Events - Backend pushes to Frontend
| Event Name | Payload | Description |
| :--- | :--- | :--- |
| `app_focused` | `{ bundle_id: String, app_name: String }` | Triggered when user switches foreground application, frontend uses this to update UI display |
| `input_switched` | `{ source_id: String }` | Triggered when input method changes |

### 4.2 Data Flow

1.  **Initialization**:
    *   Frontend checks if `get_llm_config` is empty.
    *   If empty -> Route to Onboarding page.
    *   If not empty -> Open Main Settings window directly.
    *   LLM configuration is only handled in onboarding; the main settings panel has no LLM settings tab.

2.  **AI Prediction Flow**:
    *   Frontend calls `scan_and_predict`.
    *   Rust side gets app list + system input method list (`TISCreateInputSourceList`).
    *   Rust side reads LLM configuration, constructs Prompt (including system input method ID list as Enum constraint).
    *   Rust side sends HTTP request to LLM Provider.
    *   Parses JSON response, generates `AppRule` list and returns to frontend.

3.  **Manual Rescan (Main Settings > Rules)**:
    *   User clicks "重新扫描" in the top-right of the Rules view.
    *   Frontend sets `isRescanning = true` (button disabled, spinner shown).
    *   Calls `scan_and_predict`, then merges/overwrites rules per product decision.
    *   Persist updated `AppConfig` via `save_config`, then clears loading state.

4.  **App Monitoring Loop (Rust Side)**:
    *   `Observer Thread`: Uses `NSWorkspace` to listen for notifications.
    *   **Trigger**: `NSWorkspaceDidActivateApplicationNotification`.
    *   **Action**:
        1. Get new App's `bundle_id`.
        2. Read `Config HashMap` from memory.
        3. Match rule -> Call `TISSelectInputSource`.
        4. Emit `app_focused` event to frontend (if frontend window is open).

5.  **Configuration Sync**:
    *   Configuration file stored in `$APP_DATA/config.json`.
    *   At application startup, Rust backend reads JSON and loads it into `Mutex<Config>` in memory to ensure read speed.
    *   Frontend modifies configuration -> Calls `save_config` -> Rust updates memory & asynchronously writes to disk.

### 4.3 Data Structure Definitions

#### LLMConfig
```typescript
interface LLMConfig {
  apiKey: string;
  model: string;
  baseUrl: string;
}
```

#### GeneralSettings
```typescript
interface GeneralSettings {
  autoStart: boolean; // Start on login
  hideDockIcon: boolean; // Run without Dock icon
}
```

#### AppConfig (TypeScript Interface)
```typescript
interface AppConfig {
  version: number;
  globalSwitch: boolean; // Master switch
  defaultInput: "en" | "zh" | "keep"; // Default policy
  general: GeneralSettings;
  rules: AppRule[];
}

interface AppRule {
  bundleId: string;   // e.g., "com.google.Chrome"
  appName: string;    // e.g., "Google Chrome"
  preferredInput: string; // Must be a system existing InputSourceID
  isAiGenerated: boolean; // Flag for AI prediction, allowing user override
}
```

### 4.4 AI Prediction Module (Logic Design)

*   **Input**:
    *   App List (`appName`, `appCategory`)
    *   **System Input Method List** (e.g., `['com.apple.keylayout.ABC', 'com.apple.inputmethod.SCIM.ITABC']`)
*   **Logic**:
    *   **Pure LLM Inference**: System has no built-in static rules. Completely relies on Prompt to match app characteristics (name, category) with input method features.
*   **Prompt Strategy**:
    *   Instruct AI: "For the following applications, please select the most appropriate one from the given input method ID list based on how people *commonly use* it in daily life like If it's a code editor, select the English ID; if it's a chat software, select the Chinese ID."
*   **Output**: Mapping table `Map<BundleID, InputSourceID>`.

### 4.5 Directory & State Management

*   **State Management**: Use `Zustand` to manage frontend state (current app list, search keywords, rescan loading state, general settings toggles).
*   **Persistence**: Core configuration persistence is handled by Rust backend, frontend is only responsible for rendering and sending modification commands.

### 4.6 General Settings Integration (macOS)

*   **Auto-start**: Use macOS login item mechanism (or a Tauri plugin if adopted) to register/unregister SmartIME at login.
*   **Hide Dock Icon**: Switch app activation policy to accessory-only; change may require app relaunch depending on implementation.

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
    This command will first build the Next.js frontend (`next build` + `next export`), then compile Rust code, and finally package it into a `.dmg` or `.app` file.

3.  **Artifact Location**:
    Generated executable files are located in the `src-tauri/target/release/bundle/macos/` directory.

### 5.3 Release & Distribution (Homebrew Cask)

To support `brew install --cask` installation, a Homebrew Tap repository needs to be maintained and a Cask defined.

1.  **Create Tap Repository**:
    Create a public repository named `homebrew-smartime` or `homebrew-tap` on GitHub.

2.  **Define Cask (`Casks/smartime.rb`)**:
    Create a Ruby script in the Tap repository pointing to the GitHub Release download link:

    ```ruby
    cask "smartime" do
      version "0.1.0"
      sha256 "<CHECKSUM_OF_DMG>"

      url "https://github.com/<USERNAME>/SmartIME/releases/download/v#{version}/SmartIME_#{version}_aarch64.dmg"
      name "SmartIME"
      desc "Automatic input method switcher based on active app"
      homepage "https://github.com/<USERNAME>/SmartIME"

      app "SmartIME.app"

      zap trash: [
        "~/Library/Application Support/SmartIME",
        "~/Library/Preferences/com.smartime.app.plist",
      ]
    end
    ```

3.  **Automate Release Process**:
    Configure GitHub Actions to automatically update the version number and SHA256 checksum in the Tap repository when a new Release is published.
