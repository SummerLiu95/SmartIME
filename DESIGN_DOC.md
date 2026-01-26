# 设计文档 (Design Document)

## 1. UI/UX 设计

### 1.1 设计原则
遵循 **Shadcn/ui** 的设计美学：**简洁、现代、无干扰**。
*   **色调**: 自动适配 macOS 系统深色/浅色模式 (Dark/Light Mode)。
*   **排版**: 使用系统默认字体 (San Francisco)，保证原生融合感。
*   **图标**: 使用 **Lucide React** 图标库，保持视觉风格统一、线条简洁。
*   **交互**: 减少层级，核心功能（修改规则）应在一步操作内完成。
*   **动效 (Motion)**: 引入 **Framer Motion** 增强用户体验，使用平滑、自然的微交互动画传达状态变化，避免生硬的突变。

### 1.2 界面原型 (Wireframes)

#### A. 首次启动向导 (Onboarding Wizard)
*   **Step 1: 权限授予**: 引导用户开启辅助功能权限。
*   **Step 2: LLM 设置 (新增)**:
    *   **Form 表单**:
        *   `API Key *`: 密码输入框 (带显示/隐藏切换)，右侧显示 Info Icon。
        *   `Model *`: 下拉选择框 (Select)，预设 "GPT-4o", "GPT-4-Turbo", "Claude-3.5-Sonnet" 等，支持自定义输入。
        *   `Base URL`: 文本输入框，默认 placeholder "https://api.openai.com/v1"。
    *   **Action**: "Test Connection" 按钮 (Loading 状态反馈) -> 成功后显示 "Start Scanning" 按钮。
*   **Step 3: 扫描与生成**: 进度条显示应用扫描和 AI 分析进度。

#### B. 菜单栏弹窗 (Tray Window)
点击菜单栏图标时弹出的小型窗口。
*   **Header**: 应用名称 (SmartIME)，右侧包含“设置”齿轮图标和“暂停”开关。
*   **Current App**: 显示当前前台应用的图标、名称，以及当前生效的输入法状态。
    *   *动画*: 当应用切换时，应用图标和名称应执行轻微的 **Fade In / Fade Out** 或 **Slide Y** 动画，平滑过渡。
*   **Quick Switch**: 一个醒目的 Toggle 或 Segmented Control，允许用户快速修正当前 App 的默认输入法（例如：从“英文”修正为“中文”）。
    *   *动画*: 点击切换时，滑块应有弹簧 (Spring) 阻尼效果。

#### C. 主设置面板 (Main Window)
*   **Tabs**:
    *   **Rules**: 规则管理列表。
    *   **LLM Settings (新增)**: 复用 Onboarding 中的表单，允许用户随时更新 API 配置。
    *   **General**: 开机自启、托盘图标样式等。
*   **App List (Rules Tab)**:
    *   使用 `Table` 或 `Card` 列表展示所有已配置应用。
    *   **列**: 应用图标 | 应用名称 | 偏好输入法 (Dropdown: 仅显示系统已启用输入法) | 操作 (Delete)。
    *   **搜索栏**: 顶部提供搜索框，快速过滤应用。
    *   *动画*: 列表项的添加/删除应触发 **Layout Animation** (如 `layout` prop)，使周围元素平滑重排，而非瞬间跳变。
*   **Footer**: 状态栏，显示“AI 预测已启用”或“规则已同步”。

### 1.3 响应式设计
由于主要作为桌面工具应用，界面设计主要针对固定宽度的窗口（如 800x600 为主设置页，300x400 为托盘页），但组件应支持弹性布局以适应拉伸。
