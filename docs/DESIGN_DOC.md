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
        2. **隐藏 Dock 图标** (desc: 仅在菜单栏运行（推荐）) — default OFF.
        3. **显示当前输入法提示** (desc: 切换应用或输入法时短暂显示当前输入法) — default ON.
    *   Toggle styling: blue pill when ON, gray pill when OFF.

#### C. Current Input Method Indicator
*   **Reference behavior**: Inspired by Input Source Pro's ["自动展示当前输入法"](https://inputsource.pro/zh-CN) demo, but SmartIME only uses two switch-completion timings: after the user/system switches input source, and after an app change causes SmartIME to complete an automatic input source switch. The left-mouse-hold interaction is intentionally out of scope.
*   **Display gate**:
    *   The indicator appears only when the current system cursor is in an input/editing style and an editable text field or editor is focused.
    *   App switching is only a candidate trigger. The visual indicator appears after SmartIME completes the resulting automatic input source switch, so the message confirms that automatic switching took effect.
    *   Input-source switching is also shown after the switch has completed, based on the newly current input source.
    *   If the app change preserves the current input source or the switch fails, the indicator is suppressed.
    *   If no editable input context is focused, the indicator is suppressed.
*   **Visual form**:
    *   A compact floating overlay with the current input method name and a concise visual marker.
    *   Uses the app's existing light/dark adaptive styling and avoids heavy decoration.
    *   Should be readable at a glance without looking like an app notification or modal.
*   **Motion**:
    *   Fade/scale in quickly, remain visible briefly, then fade out.
    *   Repeated triggers should update the same overlay instead of stacking multiple indicators.
*   **Placement**:
    *   Prefer the focused input/cursor position when available.
    *   Fall back to a stable location near the active window when precise focus geometry is unavailable.
*   **Interaction constraints**:
    *   The overlay is display-only, non-clickable, and non-activating.
    *   It must not steal keyboard focus or obscure the user's input for more than a brief moment.

### 1.3 Responsive Design
As a macOS desktop application, window size should **adapt to page content** rather than fixed dimensions. Layouts should be flexible and resilient to content growth, while keeping a compact, native feel.
