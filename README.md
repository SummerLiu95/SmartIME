# SmartIME

<p align="center">
  <img width="2414" height="2824" src="https://github.com/user-attachments/assets/ed7158dd-b13a-4121-8622-d0cb646dd40e" alt="SmartIME Screenshot"/>
</p>

SmartIME is a macOS desktop app (Tauri v2 + Next.js) that automatically switches input methods based on the active application.

It is designed for users who frequently move between coding tools, browsers, and chat apps and want input method switching to happen automatically in the background.

## Key Capabilities

- Monitors foreground app changes on macOS and applies per-app input method rules.
- Uses LLM-based initial prediction to bootstrap rules on first setup.
- Allows manual rule override in the rules list panel.
- Supports rescanning installed apps and refreshing AI-generated rules.
- Supports menu bar background mode and login-at-startup behavior.

## Platform Support

- macOS 12+ (Monterey and above)
- Apple Silicon (M1/M2/M3+)

## Installation

### Option 1: DMG (recommended for end users)

1. Download the latest `.dmg` from Releases.
2. Drag `SmartIME.app` into `/Applications`.
3. Launch `SmartIME.app` from Applications.

### Option 2: Homebrew

```bash
brew tap SummerLiu95/smartime
brew install --cask smartime
```

## First-Run Setup

SmartIME is currently a technical exploration project and does not ship with Apple Developer ID signing or Apple notarization. After installing from Homebrew or downloading the DMG, macOS Gatekeeper may report that `SmartIME.app` is damaged and cannot be opened.

If this happens, remove the quarantine attribute manually before the first launch:

```bash
xattr -dr com.apple.quarantine /Applications/SmartIME.app
open /Applications/SmartIME.app
```

Then continue setup:

1. Complete Accessibility permission authorization in onboarding.
2. Configure LLM settings (`API Key`, `Model`, `Base URL`) and run connection test.
3. Start first scan to generate initial rules.

## Development Setup

### Prerequisites

- Bun
- Rust toolchain (`rustup`)
- Xcode Command Line Tools

### Install dependencies

```bash
bun install
```

### Configure LLM credentials

```bash
cp .env.llm.example .env.llm
```

Then edit `.env.llm` with your local credentials:

- `LLM_API_KEY`: LLM service API key
- `LLM_MODEL`: Model name, such as `gpt-4o-mini`
- `LLM_BASE_URL`: API base URL, defaulting to OpenAI-compatible endpoints

### Run in development mode

```bash
bun tauri dev
```

Equivalent package-script form:

```bash
bun run tauri dev
```

This starts both the Next.js frontend and the Tauri desktop shell. For frontend-only work, use:

```bash
bun run dev
```

## Build

```bash
bun tauri build
```

For the frontend static build only:

```bash
bun run build
```

## Lint

```bash
bun run lint
```

## Debugging

- Frontend debugging: right-click the Tauri window and choose "Inspect Element" to open the Web Inspector.
- Backend debugging: use `println!` or `eprintln!` in Rust; output appears in the terminal running `bun tauri dev`.
- IPC debugging: command names in frontend `invoke` calls must match backend `#[tauri::command]` names exactly.
- Permission testing: validate macOS Accessibility behavior with a bundled `.app` before release decisions.

## Release Automation (Git Tag)

1. Update and commit version metadata in:
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`
2. Create and push a tag in format `v<version>` (for example `v0.1.0`).
3. GitHub Actions workflow `Release DMG` is triggered automatically and will:
   - run `bun tauri build`
   - upload `SmartIME_<version>_aarch64.dmg` and checksum file to GitHub Release.


## Milestone

Future development plans include but are not limited to:

- [x] **System Apps Support**: Extend support to macOS system applications (e.g., Safari).
- [ ] **Universal Build Support**: Support Intel chips (x86_64) to provide universal binary packaging.
- [ ] ~~**Focus Indicator**: Display a visual indicator of the current input method when the input cursor is focused.~~
- [ ] **Website-Based Switching**: Enable automatic input method switching in browsers based on the specific website being visited.
- [ ] **Built-in Local Model**: Integrate a built-in tiny LLM or lightweight classification model to remove dependency on third-party API keys.
- [ ] **UI Enhancement**: Improve the user interface for better usability and aesthetics.
- [ ] **i18n**: Add support for multiple languages.

> **Note**: These features are part of the roadmap but are not guaranteed to be implemented.

## Disclaimer

- This project requires users to provide their own LLM API credentials (`LLM_API_KEY`), and any credential misuse, leakage, billing loss, account suspension, or related damages are the user's own responsibility.
- SmartIME is a personal-interest project provided "as is", without warranty of any kind, and the maintainer is not liable for any direct or indirect loss caused by installing, running, or relying on this software.
