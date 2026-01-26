# 技术规格说明书 (Technical Specification)

## 1. 项目概述

**SmartIME** 是一个基于 Tauri 构建的 macOS 菜单栏应用程序，旨在根据当前活动窗口自动切换输入法（中文/英文）。本项目利用 Rust 的高性能系统调用能力监控应用焦点变化，结合前端 Next.js 提供直观的配置管理界面。

## 2. 技术栈与架构

### 2.1 基础模板 (Boilerplate)

本项目基于 **`nomandhoni-cs/tauri-nextjs-shadcn-boilerplate`** 初始化。该模板集成了现代 Web 开发与桌面应用的最佳实践。

*   **核心框架**: [Tauri v2](https://tauri.app/) (Rust + Webview)
*   **前端框架**: [Next.js](https://nextjs.org/) (React)
*   **UI 组件库**: [shadcn/ui](https://ui.shadcn.com/) (基于 Tailwind CSS 和 Radix UI)
*   **图标库**: [lucide-react](https://lucide.dev/) (用于一致且美观的 SVG 图标系统)
*   **工具库**: `cn()` (基于 `clsx` 和 `tailwind-merge` 的 Class 合并工具)
*   **动画库**: [framer-motion](https://www.framer.com/motion/) (用于声明式动画和复杂交互)
*   **HTTP 客户端**: [reqwest](https://docs.rs/reqwest/latest/reqwest/) (Rust 端，用于 LLM API 调用)
*   **包管理器**: bun

### 2.2 项目初始化

基于模板的项目初始化流程（需排除模板自带的 `LICENSE`, `.gitignore`, `.git`）：

```bash
# 1. 克隆模板仓库到临时目录
git clone https://github.com/nomandhoni-cs/tauri-nextjs-shadcn-boilerplate temp_boilerplate

# 2. 将模板文件复制到项目目录
# 使用 --ignore-existing 参数，确保保留项目根目录下所有现有的文件和配置
# (包括 .figma, .idea, .trae, LICENSE, .gitignore, tray-icon.svg 等)
# 仅排除模板的 .git 目录，避免破坏当前仓库的版本控制
rsync -av --progress temp_boilerplate/ . --exclude .git --ignore-existing

# 3. 清理临时目录
rm -rf temp_boilerplate

# 4. 安装依赖
bun install

# 5. 更新项目元数据
# - package.json: name ("smartime"), version, description, author
# - src-tauri/tauri.conf.json: productName ("SmartIME"), identifier ("com.smartime.app"), version
# - src-tauri/Cargo.toml: name ("smartime"), version, authors

# 6. 安装额外 UI 依赖 (图标 & 动画)
bun add lucide-react framer-motion clsx tailwind-merge

# 7. 启动开发服务器
bun tauri dev
```

### 2.3 开发与调试流程

1.  **启动开发环境**:
    ```bash
    bun tauri dev
    ```
    该命令会同时启动 Next.js 前端热重载服务器 (localhost:3000) 和 Tauri 应用程序窗口。

2.  **前端调试**:
    *   **UI 检查**: 在 Tauri 窗口中右键点击 -> "Inspect Element" 打开 Web Inspector (Safari 风格)。
    *   **控制台**: 使用 `console.log` 输出日志，可在 Web Inspector 的 Console 面板查看。

3.  **后端调试 (Rust)**:
    *   **日志输出**: 使用 `println!` 或 `eprintln!` 宏打印日志，输出内容会直接显示在运行 `bun tauri dev` 的终端窗口中。
    *   **代码修改**: 修改 `src-tauri/src` 下的 Rust 代码后，Tauri 会自动重新编译并重启应用。

4.  **常见问题排查**:
    *   如果遇到 IPC 通信失败，请检查前端 `invoke` 的命令名称是否与后端 `#[tauri::command]` 宏定义的函数名完全一致。

## 3. 项目架构设计

### 3.1 目录结构

本项目遵循 `nomandhoni-cs/tauri-nextjs-shadcn-boilerplate` 的扁平化目录结构：

```
SmartIME/
├── app/                    # Next.js App Router 页面 (Frontend)
│   ├── globals.css         # 全局样式入口
│   ├── layout.tsx          # 根布局
│   └── page.tsx            # 主页
├── src-tauri/              # Rust 后端代码 (Tauri Core)
│   ├── src/
│   │   ├── main.rs         # 入口文件，注册命令和系统托盘
│   │   ├── command.rs      # 定义供前端调用的 Tauri Commands
│   │   ├── input_source.rs # macOS 输入法切换核心逻辑 (FFI)
│   │   ├── observer.rs     # 监听 NSWorkspace 活动应用变化
│   │   └── llm.rs          # (新增) LLM API 调用与配置管理
│   ├── tauri.conf.json     # Tauri 配置文件
│   └── Cargo.toml          # Rust 依赖管理
├── components/             # Shadcn UI 组件与业务组件
│   ├── ui/                 # 基础 UI 组件 (Button, Input, etc.)
│   └── ...                 # 业务组件 (Sidebar, Header, etc.)
├── lib/                    # 工具函数 (utils)
│   └── utils.ts            # 包含 cn() 等通用工具函数
├── hooks/                  # 自定义 React Hooks
├── public/                 # 静态资源 (Icons, Images)
├── styles/                 # 样式文件 (Tailwind CSS)
├── types/                  # TypeScript 类型定义
├── .github/                # GitHub 配置
│   └── workflows/          # CI/CD 工作流 (Build & Release)
├── components.json         # shadcn/ui 配置文件
├── next.config.mjs         # Next.js 配置文件
├── tailwind.config.ts      # Tailwind CSS 配置文件
├── package.json            # 项目依赖管理
└── tray-icon.svg           # macOS 菜单栏图标 (Template Icon)
```

### 3.2 核心模块依赖

1.  **Input Source Module (Rust)**:
    *   依赖 `core-foundation` 和 `core-graphics` crate。
    *   通过 `TISSelectInputSource` API 切换输入法。
    *   **新增**: 提供 `get_system_input_sources` 函数，调用 `TISCreateInputSourceList` 获取当前系统可用输入法列表。
2.  **App Observer (Rust)**:
    *   使用 `cocoa` 或 `objc` crate 监听 `NSWorkspaceDidActivateApplicationNotification`。
    *   当应用切换时，通过 `app_handle.emit` 向前端发送事件，或直接在后端查询配置并切换。
3.  **LLM Module (Rust)**:
    *   依赖 `reqwest` (feature: `json`, `rustls-tls`)。
    *   负责存储和读取 LLM 配置（API Key 建议使用 `tauri-plugin-store` 或加密存储）。
    *   负责向 OpenAI 兼容接口发送预测请求。
4.  **Config Manager (Hybrid)**:
    *   前端负责可视化配置（应用列表 -> 输入法映射）。
    *   配置存储在本地 JSON 文件中（使用 `tauri-plugin-store` 或 `fs` 模块）。

## 4. 系统详细设计 (System Design)

### 4.1 前后端通信 (Frontend-Backend Communication)

SmartIME 采用 Tauri 的 **IPC (Inter-Process Communication)** 机制。

#### 命令 (Commands) - 前端调用后端
| Command Name | 参数 (Payload) | 返回值 | 描述 |
| :--- | :--- | :--- | :--- |
| `save_llm_config` | `config: LLMConfig` | `Result<bool, String>` | 保存并校验 LLM 配置 (返回连通性结果) |
| `get_llm_config` | None | `LLMConfig` | 获取当前 LLM 配置 (API Key 需脱敏) |
| `get_system_input_sources` | None | `Vec<InputSource>` | 获取系统当前启用的输入法列表 |
| `scan_and_predict` | None | `Vec<AppRule>` | 执行扫描+AI预测流程 (需先配置 LLM) |
| `save_config` | `config: AppConfig` | `Result<(), String>` | 保存用户修改的规则 |
| `get_installed_apps` | None | `Vec<AppInfo>` | 扫描并返回系统应用列表 |
| `get_current_input_source` | None | `String` | 获取当前系统输入法 ID |
| `check_permissions` | None | `bool` | 检查是否有辅助功能权限 |
| `open_system_settings` | None | None | 打开 macOS 系统设置页 |

#### 事件 (Events) - 后端推送前端
| Event Name | Payload | 描述 |
| :--- | :--- | :--- |
| `app_focused` | `{ bundle_id: String, app_name: String }` | 当用户切换前台应用时触发，前端用于更新 UI 显示 |
| `input_switched` | `{ source_id: String }` | 当输入法发生变化时触发 |

### 4.2 数据流 (Data Flow)

1.  **初始化**: 
    *   前端检查 `get_llm_config` 是否为空。
    *   若空 -> 路由至 Onboarding 页。
    *   若非空 -> 路由至主页/后台模式。

2.  **AI 预测流**:
    *   前端调用 `scan_and_predict`。
    *   Rust 端获取应用列表 + 系统输入法列表 (`TISCreateInputSourceList`)。
    *   Rust 端读取 LLM 配置，构造 Prompt (包含系统输入法 ID 列表作为 Enum 约束)。
    *   Rust 端发送 HTTP 请求至 LLM Provider。
    *   解析 JSON 响应，生成 `AppRule` 列表返回前端。
    
3.  **应用监控循环 (Rust Side)**:
    *   `Observer Thread`: 使用 `NSWorkspace` 监听通知。
    *   **Trigger**: `NSWorkspaceDidActivateApplicationNotification`.
    *   **Action**: 
        1. 获取新 App 的 `bundle_id`。
        2. 读取内存中的 `Config HashMap`。
        3. 匹配规则 -> 调用 `TISSelectInputSource`。
        4. 发射 `app_focused` 事件给前端 (如果前端窗口打开)。

4.  **配置同步**:
    *   配置文件存储在 `$APP_DATA/config.json`。
    *   应用启动时，Rust 后端读取 JSON 加载到 `Mutex<Config>` 内存中，保证读取速度。
    *   前端修改配置 -> 调用 `save_config` -> Rust 更新内存 & 异步写入磁盘。

### 4.3 数据结构定义

#### LLMConfig
```typescript
interface LLMConfig {
  apiKey: string;
  model: string;
  baseUrl: string;
}
```

#### AppConfig (TypeScript Interface)
```typescript
interface AppConfig {
  version: number;
  globalSwitch: boolean; // 总开关
  defaultInput: "en" | "zh" | "keep"; // 默认策略
  rules: AppRule[];
}

interface AppRule {
  bundleId: string;   // e.g., "com.google.Chrome"
  appName: string;    // e.g., "Google Chrome"
  preferredInput: string; // 必须是系统存在的 InputSourceID
  isAiGenerated: boolean; // 标记是否为 AI 预测，允许用户覆盖
}
```

### 4.4 AI 预测模块 (逻辑设计)

*   **输入**: 
    *   应用列表 (`appName`, `appCategory`)
    *   **系统输入法列表** (e.g., `['com.apple.keylayout.ABC', 'com.apple.inputmethod.SCIM.ITABC']`)
*   **逻辑**:
    *   **纯 LLM 推断**: 系统不内置任何静态规则。完全依赖 Prompt 将应用特征（名称、类别）与输入法特性进行匹配。
*   **Prompt 策略**:
    *   指示 AI：“对于以下应用，请从给定的输入法 ID 列表中选择最合适的一个。如果是代码编辑器选英文 ID，如果是聊天软件选中文 ID。”
*   **输出**: 映射表 `Map<BundleID, InputSourceID>`。

### 4.5 目录与状态管理

*   **State Management**: 使用 `Zustand` 管理前端状态（当前应用列表、加载状态、搜索关键词）。
*   **Persistance**: 核心配置持久化由 Rust 后端负责，前端仅负责渲染和发送修改指令。

## 5. 构建与部署

### 5.1 环境要求
*   **OS**: macOS (仅支持 macOS，因为依赖特定的输入法 API)
*   **Runtime**: Bun v1.0+
*   **Rust**: 1.70+ (建议通过 `rustup` 安装)
*   **XCode Command Line Tools**: 必须安装以支持 macOS 系统库编译。

### 5.2 构建流程

1.  **生产环境构建**:
    ```bash
    bun tauri build
    ```
    该命令会先构建 Next.js 前端 (`next build` + `next export`)，然后编译 Rust 代码，最后打包成 `.dmg` 或 `.app` 文件。

3.  **产物位置**:
    生成的可执行文件位于 `src-tauri/target/release/bundle/macos/` 目录下。

### 5.3 发布与分发 (Homebrew Cask)

为了支持 `brew install --cask` 安装，需要维护一个 Homebrew Tap 仓库并定义 Cask。

1.  **创建 Tap 仓库**:
    在 GitHub 上创建一个名为 `homebrew-smartime` 或 `homebrew-tap` 的公共仓库。

2.  **定义 Cask (`Casks/smartime.rb`)**:
    在 Tap 仓库中创建 Ruby 脚本，指向 GitHub Release 的下载链接：

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

3.  **发布流程自动化**:
    配置 GitHub Actions 在发布新 Release 时自动更新 Tap 仓库中的版本号和 SHA256 校验和。
