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
        *   `Model *`: Dropdown select box (Select), preset "GPT-4o", "GPT-4-Turbo", "Claude-3.5-Sonnet", etc., supports custom input.
        *   `Base URL`: Text input box, default placeholder "https://api.openai.com/v1".
    *   **Action**: "Test Connection" button (Loading state feedback) -> Show "Start Scanning" button upon success.
*   **Step 3: Scan & Generate**: Progress bar displays application scanning and AI analysis progress.

#### B. Menu Bar Popup (Tray Window)
Small window pops up when clicking the menu bar icon.
*   **Header**: App Name (SmartIME), right side includes "Settings" gear icon and "Pause" switch.
*   **Current App**: Displays current foreground app's icon, name, and currently effective input method status.
    *   *Animation*: When app switches, app icon and name should execute slight **Fade In / Fade Out** or **Slide Y** animation for smooth transition.
*   **Quick Switch**: A prominent Toggle or Segmented Control, allowing user to quickly correct the current App's default input method (e.g., correct from "English" to "Chinese").
    *   *Animation*: When clicking switch, slider should have Spring damping effect.

#### C. Main Settings Panel (Main Window)
*   **Tabs**:
    *   **Rules**: Rule management list.
    *   **LLM Settings (New)**: Reuse form from Onboarding, allowing user to update API configuration at any time.
    *   **General**: Auto-start at login, tray icon style, etc.
*   **App List (Rules Tab)**:
    *   Use `Table` or `Card` list to display all configured apps.
    *   **Columns**: App Icon | App Name | Preferred Input Method (Dropdown: Only display system enabled input methods) | Action (Delete).
    *   **Search Bar**: Search box at the top to quickly filter apps.
    *   *Animation*: Addition/Deletion of list items should trigger **Layout Animation** (like `layout` prop), making surrounding elements rearrange smoothly instead of instant jumping.
*   **Footer**: Status bar, displaying "AI Prediction Enabled" or "Rules Synced".

### 1.3 Responsive Design
Since it is mainly a desktop tool application, the interface design mainly targets fixed-width windows (e.g., 800x600 for main settings page, 300x400 for tray page), but components should support flexible layout to adapt to stretching.
