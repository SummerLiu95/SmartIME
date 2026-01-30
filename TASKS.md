# 开发任务分解 (Development Tasks)

本文档基于 `REQUIREMENTS.md` (需求文档)、`DESIGN_DOC.md` (设计文档) 和 `TECHNICAL_SPEC.md` (技术规格说明书) 生成，旨在指导 SmartIME 项目的开发流程。

## 1. 项目初始化与基础设施 (Infrastructure)

| 任务 ID      | 任务标题 | 依赖 | 描述 | 验收标准 |
|:-----------| :--- | :--- | :--- | :--- |
| **INF-01** | **项目脚手架搭建** | - | 使用 `nomandhoni-cs/tauri-nextjs-shadcn-boilerplate` 初始化项目。**必须使用** `rsync --ignore-existing` 策略，以确保保留项目现有的所有文件（如 `.figma`, `.idea`, `.trae`, `LICENSE`, `.gitignore`, `tray-icon.svg` 等）不被覆盖。仅排除模板的 `.git` 目录。更新元数据信息。 | 1. 项目成功运行 `bun tauri dev`。<br>2. 目录结构符合 TECHNICAL_SPEC 定义。<br>3. **所有现有文件完整保留**（特别是 IDE 配置和设计资源）。<br>4. 项目名称、Bundle ID 等元数据已更新。 |
| **INF-02** | **前端基础依赖安装** | INF-01 | 安装 `lucide-react`, `framer-motion`, `clsx`, `tailwind-merge` 等 UI 依赖。 | `package.json` 中包含指定依赖，且前端可正常 import 使用。 |
| **INF-03** | **Rust 依赖配置** | INF-01 | 在 `Cargo.toml` 中添加 `reqwest`, `tauri-plugin-store` (或类似持久化库), `cocoa`, `objc` 等依赖。 | `cargo build` 成功编译。 |

## 2. 核心组件与共享模块 (Shared Components)

| 任务 ID | 任务标题 | 依赖 | 描述 | 验收标准 |
| :--- | :--- | :--- | :--- | :--- |
| **UI-01** | **基础 UI 组件库** | INF-03 | 基于 Shadcn/ui 封装或直接使用 Button, Input, Select, Dialog, Card, Table 等组件。 | 组件样式符合 Figma 设计规范（圆角、阴影、配色）。 |
| **UI-02** | **布局组件开发** | UI-01 | 开发 `Sidebar`, `Header`, `OnboardingLayout` 等通用布局组件。 | 布局在不同窗口尺寸下表现正常。 |
| **UI-03** | **动画组件封装** | INF-03 | 使用 Framer Motion 封装通用的 `FadeIn`, `SlideUp` 等动画 Wrapper。 | 页面切换和元素显示有平滑过渡效果。 |

## 3. 后端核心逻辑 (Rust Backend)

| 任务 ID | 任务标题 | 依赖 | 描述 | 验收标准 |
| :--- | :--- | :--- | :--- | :--- |
| **BE-01** | **输入法管理模块** | INF-04 | 实现 `input_source.rs`：获取系统输入法列表 (`TISCreateInputSourceList`) 和切换输入法 (`TISSelectInputSource`)。 | 1. 能正确列出当前系统所有启用的输入法 ID。<br>2. 能通过 ID 成功切换输入法。 |
| **BE-02** | **应用监听模块** | INF-04 | 实现 `observer.rs`：监听 `NSWorkspaceDidActivateApplicationNotification`。 | 切换前台应用时，控制台能实时打印新应用的 Bundle ID。 |
| **BE-03** | **LLM 客户端模块** | INF-04 | 实现 `llm.rs`：封装 Reqwest 请求，支持 OpenAI 格式的 Chat Completion API。 | 能发送测试请求并正确解析返回的 JSON。 |
| **BE-04** | **系统应用扫描模块** | INF-03 | 实现 `system_apps.rs`：使用 `walkdir` 和 `plist` 扫描系统应用。 | 能正确遍历 `/Applications` 并解析出应用的 Bundle ID 和名称。 |
| **BE-05** | **配置持久化模块** | INF-04 | 实现配置的读写逻辑（LLM 配置、App 规则表），确保数据安全存储。 | 重启应用后，配置数据不丢失；API Key 不明文显示。 |
| **BE-06** | **Tauri 命令注册** | BE-01~05 | 注册 `get_installed_apps`, `save_llm_config`, `scan_and_predict` 等 IPC 命令。 | 前端能成功调用这些命令并获取预期返回值。 |

## 4. 界面模块开发 (Frontend Features)

### 4.1 首次安装权限检查界面 (Onboarding Step 1)
*参考: Screenshot 12_294*

| 任务 ID | 任务标题 | 依赖 | 描述 | 验收标准 |
| :--- | :--- | :--- | :--- | :--- |
| **FE-ONB-01** | **权限检查 UI 实现** | UI-02 | 实现权限授予引导页，包含图标、说明文案和“设置 > 隐私...”路径指引。 | 界面还原度高，适配 Light/Dark 模式。 |
| **FE-ONB-02** | **权限检测逻辑** | BE-06 | 调用后端 `check_permissions` 命令，点击“我已开启”时复查权限状态。 | 权限未开启时提示重试；开启后自动跳转下一步。 |

### 4.2 LLM 设置界面 (Onboarding Step 2 / Settings Tab)
*参考: Figma LLM Settings*

| 任务 ID | 任务标题 | 依赖 | 描述 | 验收标准 |
| :--- | :--- | :--- | :--- | :--- |
| **FE-LLM-01** | **LLM 表单开发** | UI-01 | 实现 API Key (带显隐切换)、Model (下拉选择)、Base URL 表单。 | 表单验证逻辑正确（必填项检查）。 |
| **FE-LLM-02** | **连接测试逻辑** | BE-03 | 点击“测试连接”调用后端接口，处理 Loading/Success/Error 状态。 | 连接成功显示绿色提示；失败显示具体错误信息。 |

### 4.3 首次扫描与规则生成界面 (Onboarding Step 3)
*参考: Screenshot 12_47*

| 任务 ID | 任务标题 | 依赖 | 描述 | 验收标准 |
| :--- | :--- | :--- | :--- | :--- |
| **FE-SCAN-01** | **扫描进度 UI** | UI-01 | 实现进度条动画和状态文字（扫描中 -> 分析中 -> 生成完毕）。 | 动画流畅，进度反馈真实。 |
| **FE-SCAN-02** | **预测流程集成** | BE-06 | 调用 `scan_and_predict`，获取生成的规则列表并存入本地状态。 | 成功获取到包含 Bundle ID 和 Input Source ID 的规则列表。 |

### 4.4 菜单栏应用弹窗界面 (Tray Window)
*参考: Screenshot 12_247*

| 任务 ID | 任务标题 | 依赖 | 描述 | 验收标准 |
| :--- | :--- | :--- | :--- | :--- |
| **FE-TRAY-01** | **托盘窗口 UI** | UI-01 | 实现紧凑的卡片布局，显示当前 App 图标、名称、AI 模式状态。 | 界面尺寸固定，布局紧凑美观。 |
| **FE-TRAY-02** | **实时状态同步** | BE-02 | 监听 `app_focused` 事件，实时更新当前 App 信息和输入法状态。 | 切换 App 时，托盘窗口内容即时刷新。 |
| **FE-TRAY-03** | **快速切换交互** | BE-01 | 实现“中/英”分段控制器，点击后立即调用后端切换输入法并更新规则。 | 点击切换后，系统输入法实际发生改变。 |

### 4.5 应用主设置界面 (Main Settings)
*参考: Screenshot 3_262*

| 任务 ID | 任务标题 | 依赖 | 描述 | 验收标准 |
| :--- | :--- | :--- | :--- | :--- |
| **FE-MAIN-01** | **侧边栏导航** | UI-02 | 实现“规则管理”、“LLM 设置”、“常规设置”的切换逻辑。 | 点击导航项正确切换右侧内容区域。 |
| **FE-MAIN-02** | **规则列表开发** | UI-01 | 实现应用列表 Table，包含图标、名称、输入法 Pill Badge、删除按钮。 | 列表渲染性能良好，支持滚动。 |
| **FE-MAIN-03** | **搜索与添加** | FE-MAIN-02 | 实现列表搜索过滤功能；“添加应用”按钮逻辑（弹窗选择未配置的 App）。 | 搜索响应迅速；能成功添加新规则。 |
| **FE-MAIN-04** | **规则修改逻辑** | BE-04 | 用户在列表中修改输入法或删除规则时，调用 `save_config` 同步后端。 | 修改后重启应用，配置依然生效。 |

## 5. 打包与发布 (Distribution)

| 任务 ID | 任务标题 | 依赖 | 描述 | 验收标准 |
| :--- | :--- | :--- | :--- | :--- |
| **DIST-01** | **构建脚本配置** | INF-01 | 优化 `package.json` 构建脚本，确保构建流程顺畅。 | `bun tauri build` 能生成最终产物。 |
| **DIST-02** | **GitHub Actions** | - | 配置 CI/CD 流程，自动构建 Release 版本并上传 Artifacts。 | Push tag 后自动触发构建并发布 Release。 |
| **DIST-03** | **Homebrew Tap** | DIST-02 | 创建 `homebrew-smartime` 仓库，编写 Cask 脚本。 | 能通过 `brew install --cask smartime` 安装应用。 |
