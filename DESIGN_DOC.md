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
*   **Step 3: Scan & Generate**: Progress bar displays application scanning and AI analysis progress.

#### B. Menu Bar Popup (Tray Window)
Small window pops up when clicking the menu bar icon.
*   **Header**: App Name (SmartIME), right side includes "Settings" gear icon and "Pause" switch.
*   **Current App**: Displays current foreground app's icon, name, and currently effective input method status.
    *   *Animation*: When app switches, app icon and name should execute slight **Fade In / Fade Out** or **Slide Y** animation for smooth transition.

#### C. Main Settings Panel (Main Window)
*   **Tabs**:
    *   **Rules**: Rule management list.
    *   **General**: Auto-start at login, tray icon style, etc.
*   **Left Sidebar**:
    *   App icon + name at the top.
    *   Navigation list (Rules / General).
    *   **Status Card** at bottom with label "STATUS" and text "AI 预测已启用".
*   **App List (Rules Tab)**:
    *   Use `Table` or `Card` list to display all configured apps.
    *   **Columns**: App Icon | App Name | Preferred Input Method (Dropdown: Only display system enabled input methods) | Action (Delete).
    *   **Top Bar**:
        *   **Search Bar** (placeholder: "搜索应用...") to quickly filter apps.
        *   **Rescan Button** on the right ("重新扫描") to trigger re-scan + AI prediction.
        *   **Rescan Loading State**: Button shows spinner icon, reduced opacity, and disabled while scanning.
    *   *Animation*: Addition/Deletion of list items should trigger **Layout Animation** (like `layout` prop), making surrounding elements rearrange smoothly instead of instant jumping.
*   **Footer**: Status bar, displaying "AI Prediction Enabled" or "Rules Synced".
*   **General Settings Tab**:
    *   Title "常规设置" with subtitle "管理应用的基础运行行为。".
    *   Three **setting cards** (72px height) with toggle switches:
        1. **登录时自动启动** (desc: 在您进入系统时开启 SmartIME) — default ON.
        2. **在菜单栏显示状态** (desc: 显示当前 IME 状态图标) — default ON.
        3. **隐藏 Dock 图标** (desc: 仅在菜单栏运行（推荐）) — default OFF.
    *   Toggle styling: blue pill when ON, gray pill when OFF.

### 1.3 Responsive Design
As a macOS desktop application, window size should **adapt to page content** rather than fixed dimensions. Layouts should be flexible and resilient to content growth, while keeping a compact, native feel.
