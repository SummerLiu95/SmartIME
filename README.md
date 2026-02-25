# SmartIME

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
- Apple Silicon + Intel (universal target)

## Installation

### Option 1: DMG (recommended for end users)

1. Download the latest `.dmg` from Releases.
2. Drag `SmartIME.app` into `/Applications`.
3. Launch `SmartIME.app` from Applications.

### Option 2: Homebrew

```bash
brew install --cask smartime
```

## First-Run Setup

1. Launch SmartIME.
2. Complete Accessibility permission authorization in onboarding.
3. Configure LLM settings (`API Key`, `Model`, `Base URL`) and run connection test.
4. Start first scan to generate initial rules.

## Development Setup

### Prerequisites

- Bun
- Rust toolchain (`rustup`)
- Xcode Command Line Tools

### Install dependencies

```bash
bun install
```

### Run in development mode

```bash
bun tauri dev
```

## Build

```bash
bun tauri build --target universal-apple-darwin
```

DMG output is generated at:

`src-tauri/target/universal-apple-darwin/release/bundle/dmg/SmartIME_<version>_universal.dmg`

## Release Automation (Git Tag)

1. Update and commit version metadata in:
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`
2. Create and push a tag in format `v<version>` (for example `v0.1.0`).
3. GitHub Actions workflow `Release DMG` is triggered automatically and will:
   - run `bun tauri build --target universal-apple-darwin`
   - upload `SmartIME_<version>_universal.dmg` and checksum file to GitHub Release.

## Disclaimer

- This project requires users to provide their own LLM API credentials (`LLM_API_KEY`), and any credential misuse, leakage, billing loss, account suspension, or related damages are the user's own responsibility.
- SmartIME is a personal-interest project provided "as is", without warranty of any kind, and the maintainer is not liable for any direct or indirect loss caused by installing, running, or relying on this software.
