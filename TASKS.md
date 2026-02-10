# Development Tasks

This document is generated based on `REQUIREMENTS.md` (Requirements Document), `DESIGN_DOC.md` (Design Document), and `TECHNICAL_SPEC.md` (Technical Specification), aiming to guide the development process of the SmartIME project.

## 1. Project Initialization & Infrastructure

| Task ID | Task Title | Dependencies | Description | Acceptance Criteria |
|:-----------| :--- | :--- | :--- | :--- |
| **INF-01** | **Project Scaffolding Setup** | - | Initialize project using `nomandhoni-cs/tauri-nextjs-shadcn-boilerplate`. **Must use** `rsync --ignore-existing` strategy to ensure all existing files in the project (such as `.figma`, `.idea`, `.trae`, `LICENSE`, `.gitignore`, `tray-icon.svg`, etc.) are preserved and not overwritten. Only exclude the template's `.git` directory. Update metadata information. | 1. Project successfully runs `bun tauri dev`.<br>2. Directory structure complies with TECHNICAL_SPEC definition.<br>3. **All existing files are preserved intact** (especially IDE configurations and design resources).<br>4. Metadata such as project name, Bundle ID, etc., are updated. |
| **INF-02** | **Frontend Basic Dependencies Installation** | INF-01 | Install UI dependencies such as `lucide-react`, `framer-motion`, `clsx`, `tailwind-merge`. | `package.json` contains specified dependencies, and frontend can import and use them normally. |
| **INF-03** | **Rust Dependencies Configuration** | INF-01 | Add dependencies like `reqwest`, `tauri-plugin-store` (or similar persistence library), `cocoa`, `objc` in `Cargo.toml`. | `cargo build` compiles successfully. |

## 2. Core Components & Shared Modules

| Task ID | Task Title | Dependencies | Description | Acceptance Criteria |
| :--- | :--- | :--- | :--- | :--- |
| **UI-01** | **Basic UI Component Library** | INF-03 | Encapsulate or directly use components like Button, Input, Select, Dialog, Card, Table based on Shadcn/ui. | Component styles comply with Figma design specifications (border radius, shadows, color scheme). |
| **UI-02** | **Layout Component Development** | UI-01 | Develop common layout components like `Sidebar`, `Header`, `OnboardingLayout`. | Layout behaves normally under different window sizes. |
| **UI-03** | **Animation Component Encapsulation** | INF-03 | Encapsulate common `FadeIn`, `SlideUp` animation wrappers using Framer Motion. | Page transitions and element displays have smooth transition effects. |

## 3. Backend Core Logic (Rust Backend)

| Task ID | Task Title | Dependencies | Description | Acceptance Criteria |
| :--- | :--- | :--- | :--- | :--- |
| **BE-01** | **Input Method Management Module** | INF-03 | Implement `input_source.rs`: Get system input method list (`TISCreateInputSourceList`) and switch input method (`TISSelectInputSource`). | 1. Can correctly list all currently enabled system input method IDs.<br>2. Can successfully switch input method via ID. |
| **BE-02** | **App Observer Module** | INF-03 | Implement `observer.rs`: Listen for `NSWorkspaceDidActivateApplicationNotification`. | When switching foreground apps, console can print the new app's Bundle ID in real-time. |
| **BE-03** | **LLM Client Module** | INF-03 | Implement `llm.rs`: Encapsulate Reqwest requests, support OpenAI format Chat Completion API. | Can send test requests and correctly parse returned JSON. |
| **BE-04** | **System App Scanning Module** | INF-03 | Implement `system_apps.rs`: Scan system apps using `walkdir` and `plist`. | Can correctly traverse `/Applications` and parse out app Bundle IDs and names. |
| **BE-05** | **Configuration Persistence Module** | INF-03 | Implement configuration read/write logic (LLM config, App rule table), ensuring data is stored securely. | After restarting the app, configuration data is not lost; API Key is not shown in plain text. |
| **BE-06** | **Tauri Command Registration** | BE-01~05 | Register IPC commands like `get_installed_apps`, `save_llm_config`, `scan_and_predict`. | Frontend can successfully call these commands and get expected return values. |
| **BE-07** | **General Settings System Integration** | BE-05 | Implement OS-level integrations for auto-start and Dock icon hiding; persist in config. | Toggling settings updates system behavior and persists across restarts. |

## 4. Frontend Features Development

### 4.1 First Launch Permission Check Interface (Onboarding Step 1)
*Reference: [Permission Check Interface](https://www.figma.com/design/VRUhsQxvw3cpybCwvbD7Pt/SmartIME?node-id=12-294&m=dev)*

| Task ID | Task Title | Dependencies | Description | Acceptance Criteria |
| :--- | :--- | :--- | :--- | :--- |
| **FE-ONB-01** | **Permission Check UI Implementation** | UI-02 | Implement permission grant guide page, including icons, explanatory text, and "Settings > Privacy..." path guidance. | High interface fidelity, adapts to Light/Dark mode. |
| **FE-ONB-02** | **Permission Detection Logic** | BE-06 | Call backend `check_permissions` command, recheck permission status when clicking "I have enabled". | Prompt retry when permission not enabled; auto jump to next step after enabled. |

### 4.2 LLM Settings Interface (Onboarding Step 2 Only)
*Reference: [LLM Settings Interface](https://www.figma.com/design/VRUhsQxvw3cpybCwvbD7Pt/SmartIME?node-id=47-382&m=dev)*

| Task ID | Task Title | Dependencies | Description | Acceptance Criteria |
| :--- | :--- | :--- | :--- | :--- |
| **FE-LLM-01** | **LLM Form Development** | UI-01 | Implement API Key (with show/hide toggle), Model (text input), Base URL form. | Form validation logic is correct (required fields check). |
| **FE-LLM-02** | **Connection Test Logic** | BE-03 | Click "Test Connection" to call backend interface, handle Loading/Success/Error states. | Show green prompt on connection success; show specific error message on failure. |

### 4.3 First Scan & Rule Generation Interface (Onboarding Step 3)
*Reference: [First Scan & Rule Generation Interface](https://www.figma.com/design/VRUhsQxvw3cpybCwvbD7Pt/SmartIME?node-id=12-46&m=dev)*

| Task ID | Task Title | Dependencies | Description | Acceptance Criteria |
| :--- | :--- | :--- | :--- | :--- |
| **FE-SCAN-01** | **Scan Progress UI** | UI-01 | Implement progress bar animation and status text (Scanning -> Analyzing -> Generated). | Smooth animation, real progress feedback. |
| **FE-SCAN-02** | **Prediction Flow Integration** | BE-06 | Call `scan_and_predict`, get generated rule list and store in local state. | Successfully obtain rule list containing Bundle ID and Input Source ID. |

### 4.4 App Settings Interface
*Reference: [Main Settings Interface](https://www.figma.com/design/VRUhsQxvw3cpybCwvbD7Pt/SmartIME?node-id=73-471&m=dev), [General Settings Panel](https://www.figma.com/design/VRUhsQxvw3cpybCwvbD7Pt/SmartIME?node-id=73-616&m=dev), [Rescan Loading State](https://www.figma.com/design/VRUhsQxvw3cpybCwvbD7Pt/SmartIME?node-id=73-670&m=dev)*

| Task ID | Task Title | Dependencies | Description | Acceptance Criteria |
| :--- | :--- | :--- | :--- | :--- |
| **FE-MAIN-01** | **Sidebar Navigation** | UI-02 | Implement switching logic for "Rule Management", "General Settings". | Clicking nav items correctly switches right-side content area. |
| **FE-MAIN-02** | **Rule List Development** | UI-01 | Implement App list Table, including icon, name, Input Method Pill Badge, delete button. | List rendering performance is good, supports scrolling. |
| **FE-MAIN-03** | **Search & Actions** | FE-MAIN-02 | Implement top bar search input, "重新扫描" action to trigger scan + AI prediction | Search responds quickly; rescan triggers backend and refreshes list. |
| **FE-MAIN-04** | **Rule Modification Logic** | BE-04 | When user modifies input method or deletes rule in list, call `save_config` to sync backend. | Configuration remains effective after restarting app upon modification. |
| **FE-MAIN-05** | **General Settings UI** | UI-02 | Implement General Settings view with two setting cards and toggle switches. | Layout and toggle styles match Figma; default states reflect config. |
| **FE-MAIN-06** | **General Settings Persistence** | BE-07 | Bind toggles to config state and persist changes via `save_config`. | Toggling settings updates config and survives restart. |
| **FE-MAIN-07** | **Rescan Loading State** | FE-MAIN-03 | Add loading/disabled state for rescan button (spinner + opacity). | While scanning, button is disabled and shows loading feedback; re-enabled after completion. |

## 5. Packaging & Distribution

| Task ID | Task Title | Dependencies | Description | Acceptance Criteria |
| :--- | :--- | :--- | :--- | :--- |
| **DIST-01** | **Build Script Configuration** | INF-01 | Optimize `package.json` build script to ensure smooth build flow. | `bun tauri build` can generate final artifact. |
| **DIST-02** | **GitHub Actions** | - | Configure CI/CD flow, automatically build Release version and upload Artifacts. | Automatically trigger build and publish Release after pushing tag. |
| **DIST-03** | **Homebrew Tap** | DIST-02 | Create `homebrew-smartime` repository, write Cask script. | App can be installed via `brew install --cask smartime`. |
