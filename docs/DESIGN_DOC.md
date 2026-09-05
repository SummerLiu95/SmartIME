# Design Document

## 1. UI/UX Design

### 1.1 Design Principles
Follow **Shadcn/ui** design aesthetics: **Clean, Modern, Distraction-free**.
*   **Color Scheme**: Automatically adapt to macOS system Dark/Light Mode.
*   **Typography**: Use system default font (San Francisco) to ensure native integration feel.
*   **Icons**: Use **Lucide React** icon library to maintain unified visual style and clean lines.
*   **Interaction**: Reduce hierarchy, core functions (modifying rules) should be completed within one step operation.
*   **Motion**: Introduce **Framer Motion** to enhance user experience, using smooth, natural micro-interaction animations to convey state changes, avoiding rigid transitions.

### 1.2 Wireframes

#### A. First Launch Wizard (Onboarding Wizard)
*   **Step 1: Permission Grant**: Guide user to enable accessibility permissions.
*   **Step 2: LLM Settings (New)**:
    *   **Form**:
        *   `API Key *`: Password input box (with show/hide toggle), Info Icon on the right.
        *   `Model *`: Text input box (Input), default placeholder "gpt-4o".
        *   `Base URL`: Text input box, default placeholder "https://api.openai.com/v1".
    *   **Action**: "Test Connection" button (Loading state feedback) -> Show "Start Scanning" button upon success.
*   **Step 3: Scan & Generate**: Progress display should reflect real phases such as app discovery, input source loading, AI rule generation, and saving. Avoid a fixed fake percentage that can appear stuck while waiting for long-running LLM work. Very fast phases may use a short minimum display duration so users can perceive the transition without delaying long-running work.

#### B. Main Settings Panel (Main Window)
*   **Tabs**:
    *   **Rules**: Rule management list.
    *   **General**: Auto-start at login, Dock visibility, etc.
*   **Left Sidebar**:
    *   App icon + name at the top.
    *   Navigation list (Rules / General).
*   **App List (Rules Tab)**:
    *   Use `Table` or `Card` list to display all configured apps.
    *   **Columns**: App Icon | App Name | Preferred Input Method (Dropdown: Only display system enabled input methods) | Action (Delete).
    *   **App names**: Display the localized app name that macOS shows for the installed app when available, for example Chinese systems should prefer names such as "微信" or "阿里云盘" over bundle fallback names such as "WeChat" or "aDrive".
    *   **App icons**: Display the real macOS app icon for each rule when available, matching the icon shown by Finder, Launchpad, and System Settings. If the icon cannot be resolved after lookup, keep the existing rounded initial-letter fallback so the row remains visually stable.
    *   **Icon loading behavior**: While an icon is still loading, use a neutral loading slot/skeleton rather than the initial-letter fallback. The letter fallback is reserved for confirmed lookup failure, so rows do not flash from a placeholder letter to a real icon.
    *   **First-screen priority**: On first entry to Rules and immediately after scan/rescan redirects, request and settle icon loading for the first visible rows first. Remaining rows may continue loading in background once the first screen is visually stable.
    *   **Input method labels**: Dropdown labels should use macOS system-localized input source names, so Chinese systems show names such as "简体拼音" instead of English fallback labels like "Pinyin - Simplified" when the OS provides them.
    *   **System app scope**: Include a curated set of common input-capable macOS system apps, not every internal/background system bundle discovered under system directories. Prefer localized names for those system apps when available.
    *   **Top Bar**:
        *   **Search Bar** (placeholder: "搜索应用...") to quickly filter apps.
        *   **Rescan Button** on the right ("重新扫描") to trigger re-scan + AI prediction.
        *   **Rescan Loading State**: Button shows spinner icon, reduced opacity, and disabled while scanning. Status text should make clear whether SmartIME is discovering apps, generating rules, or syncing cached rules. Returning to Rules during or right after rescan should not cause first-screen rows to flash fallback initials before real icons arrive.
    *   *Animation*: Addition/Deletion of list items should trigger **Layout Animation** (like `layout` prop), making surrounding elements rearrange smoothly instead of instant jumping.
*   **Footer**: Status bar, displaying "AI Prediction Enabled" or "Rules Synced".
*   **General Settings Tab**:
    *   Title "常规设置" with subtitle "管理应用的基础运行行为。".
    *   Two **setting cards** (72px height) with toggle switches:
        1. **登录时自动启动** (desc: 在您进入系统时开启 SmartIME) — default ON.
        2. **隐藏 Dock 图标** (desc: 仅在菜单栏运行（推荐）) — default OFF.
    *   Toggle styling: blue pill when ON, gray pill when OFF.

#### C. Custom Input Method Indicator
*   SmartIME intentionally does not provide a custom current input method indicator surface.
*   The app relies on macOS's native input source prompt where the system already shows one, and automatic switching remains a silent background behavior.
*   The UI must not expose a "显示当前输入法提示" toggle, overlay preview, or prompt customization controls.

### 1.3 Responsive Design
As a macOS desktop application, window size should **adapt to page content** rather than fixed dimensions. Layouts should be flexible and resilient to content growth, while keeping a compact, native feel.
