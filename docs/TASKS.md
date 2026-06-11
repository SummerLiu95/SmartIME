# Development Tasks

This document is generated based on sibling docs `REQUIREMENTS.md` (Requirements Document), `DESIGN_DOC.md` (Design Document), and `TECHNICAL_SPEC.md` (Technical Specification), aiming to guide the development process of the SmartIME project.

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
| **FE-MAIN-05** | **General Settings UI** | UI-02 | Implement General Settings view with setting cards and toggle switches. | Layout and toggle styles match design docs; default states reflect config. |
| **FE-MAIN-06** | **General Settings Persistence** | BE-07 | Bind toggles to config state and persist changes via `save_config`. | Toggling settings updates config and survives restart. |
| **FE-MAIN-07** | **Rescan Loading State** | FE-MAIN-03 | Add loading/disabled state for rescan button (spinner + opacity). | While scanning, button is disabled and shows loading feedback; re-enabled after completion. |

### 4.5 Milestone 2 & 3 Implementation Plan

This plan covers:

1.  **System Apps Support**: include eligible macOS system apps in scan, prediction, rule management, and automatic switching.
2.  **Removed scope**: the custom current input method indicator is no longer part of this project. SmartIME relies on macOS native input-source prompts where available and keeps automatic switching silent.

Implementation order:

1.  Ship System Apps Support first because it affects the app/rule data pipeline used by automatic switching.
2.  Keep input-source reads and selection on the main thread because HIToolbox/TIS APIs are sensitive to queue context.
3.  Finish with bundled-app manual validation on macOS; dev runtime alone is not enough for Accessibility, app identity, and automatic switching behavior.

### 4.6 System Apps Support

| Task ID | Task Title | Dependencies | Files | Description | Acceptance Criteria |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **BE-SYS-01** | **Extract Testable App Scan Roots** | BE-04 | Modify `src-tauri/src/system_apps.rs` | Refactor scanning so production roots and test roots are separate. Keep `/Applications` and `~/Applications` behavior unchanged before adding new roots. | `cargo test` passes; existing app scan behavior remains sorted and de-duplicated by bundle ID. |
| **BE-SYS-02** | **Add macOS System App Roots** | BE-SYS-01 | Modify `src-tauri/src/system_apps.rs` | Add eligible system roots: `/System/Applications`, `/System/Applications/Utilities`, and selected CoreServices app bundles where observable. Skip unreadable paths without failing the scan. | Safari, Terminal, TextEdit, Mail, Notes, and Finder are discovered when present on the machine; duplicate bundle IDs are still returned once. |
| **BE-SYS-03** | **Remove Blanket Apple Bundle Filtering** | BE-SYS-02 | Modify `src-tauri/src/command.rs` | Replace `!bundle_id.starts_with("com.apple.")` target filtering with capability-oriented filtering. Do not drop `com.apple.*` apps only because they are Apple apps. | Unit test verifies Safari is retained as a prediction/rescan target; unsupported entries can be skipped without aborting scan. |
| **BE-SYS-04** | **Preserve System App Manual Rules** | BE-SYS-03 | Modify `src-tauri/src/command.rs` | Ensure `align_rules_with_apps` treats system app rules exactly like third-party rules: manual rules win, generated rules fill gaps, stale apps are pruned, invalid input source IDs normalize to fallback. | Unit test covers a manual Safari rule surviving rescan alignment and retaining `is_ai_generated=false`. |
| **FE-SYS-01** | **Verify Rules UI Handles System Apps** | BE-SYS-04, FE-MAIN-02 | Check `app/settings/rules/page.tsx`, `lib/api.ts` | Confirm the existing Rules table can search, edit, delete, and display system app rules without special casing. Update mocks only if needed. | Safari/Terminal rules appear in preview data and real Tauri data; search by app name and bundle ID works. |
| **QA-SYS-01** | **System App Regression Pass** | BE-SYS-04, FE-SYS-01 | Manual validation | Run onboarding scan or manual rescan on macOS with Accessibility permission enabled. Validate Safari and Terminal rule creation, manual override, persistence, and app-switch automatic switching. | Manual Safari rule persists after app restart and rescan; switching into Safari applies the configured input source. |

Validation commands:

```bash
cd src-tauri && cargo test
bun run build
```

Manual validation:

1.  Run `bun tauri dev`.
2.  Grant Accessibility permission if needed.
3.  Trigger manual rescan from Rules.
4.  Confirm Safari and Terminal appear.
5.  Set Safari to a different input source.
6.  Switch away and back to Safari.
7.  Confirm the configured input source is applied.

### 4.7 Custom Input Method Indicator Removal

The custom current input method indicator has been removed from the implementation plan. Do not add `show_input_indicator`, an `/indicator` route, a Tauri indicator window, focused editable-context detection, or indicator events unless the requirement is explicitly re-opened with a native macOS implementation strategy.

Validation after removal:

```bash
cd src-tauri && cargo test
bun run build
```

Manual validation:

1.  Open General Settings and confirm there is no "显示当前输入法提示" toggle.
2.  Switch into a managed app and confirm automatic input-source switching still works.
3.  Confirm SmartIME does not open an overlay window or show a custom input method prompt.

## 5. Packaging & Distribution

| Task ID | Task Title | Dependencies | Description | Acceptance Criteria |
| :--- | :--- | :--- | :--- | :--- |
| **DIST-01** | **Build Script Configuration** | INF-01 | Optimize `package.json` build script to ensure smooth build flow. | `bun tauri build` can generate final artifact. |
| **DIST-02** | **GitHub Actions** | - | Configure CI/CD flow, automatically build Release version and upload Artifacts. | Automatically trigger build and publish Release after pushing tag. |
| **DIST-03** | **Homebrew Tap** | DIST-02 | Create `homebrew-smartime` repository, write Cask script. | App can be installed via `brew install --cask smartime`. |
