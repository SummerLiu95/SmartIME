# AGENTS.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

SmartIME is a macOS desktop application built with Tauri v2 + Next.js that automatically switches input methods based on the active application window. The app uses AI/LLM to predict appropriate input methods for different applications and provides a clean UI for configuration management.

## Development Commands

### Setup & Installation
```bash
# Install frontend dependencies
bun install

# Install Rust dependencies (if needed)
cargo install --path src-tauri
```

### Development
```bash
# Start development server (starts both Next.js frontend and Tauri app)
bun run tauri dev

# Alternative development command
bun tauri dev
```

### Build & Test
```bash
# Build for production
bun run tauri build

# Build Next.js frontend only
bun run build

# Start Next.js in production mode
bun run start

# Lint frontend code
bun run lint
```

### LLM Configuration
```bash
# Copy LLM configuration template
cp .env.llm.example .env.llm
```
Then edit `.env.llm` with your actual API credentials:
- `LLM_API_KEY`: Your LLM service API key
- `LLM_MODEL`: Model name (e.g., gpt-4o-mini)
- `LLM_BASE_URL`: API base URL (defaults to OpenAI)

## Architecture Overview

### Frontend (Next.js)
- **App Router**: Uses Next.js app directory structure
- **UI Components**: Shadcn/ui components with Tailwind CSS and Radix UI
- **Animation**: Framer Motion for smooth transitions
- **Icons**: Lucide React icon library
- **State**: React hooks and Zustand for state management

### Backend (Tauri/Rust)
- **Core Modules**: 
  - `input_source.rs`: macOS input method switching via CoreFoundation APIs
  - `observer.rs`: NSWorkspace application focus monitoring
  - `llm.rs`: HTTP client for LLM API communication
  - `system_apps.rs`: System application scanning and parsing
  - `config.rs`: Configuration persistence management
- **IPC Commands**: Registered in `command.rs` for frontend-backend communication
- **State Management**: Global app state via `AppState`

### Key Dependencies
- **Rust**: `reqwest`, `cocoa`, `objc`, `core-foundation`, `tauri-plugin-store`
- **Frontend**: `framer-motion`, `lucide-react`, `@radix-ui/*`, `tailwindcss`

## File Organization

### Configuration Files
- `tauri.conf.json`: Tauri application configuration
- `components.json`: Shadcn/ui component configuration
- `next.config.ts`: Next.js configuration with static export for Tauri
- `.env.llm.example`: LLM configuration template (actual config in `.env.llm`)

### Core Directories
- `app/`: Next.js pages (onboarding, main app)
- `components/`: UI components (shadcn/ui + custom components)
- `src-tauri/src/`: Rust backend modules
- `lib/`: Shared utilities and API helpers

## Development Guidelines

### Frontend Development
- Use shadcn/ui components consistently
- Follow the established animation patterns with Framer Motion
- Maintain responsive design principles even for fixed desktop windows
- Use the established color scheme that adapts to macOS Dark/Light mode

### Backend Development
- All system interactions should be macOS-specific and use appropriate APIs
- LLM configuration must be handled securely (never commit API keys)
- Input method switching should complete within 100ms for good UX
- Use proper error handling with the established error types

### Configuration Management
- App rules and LLM config are stored locally using `tauri-plugin-store`
- All user preferences should persist across app restarts
- API keys should be masked in UI and stored securely

### Testing Approach
- Frontend debugging: Use Web Inspector in Tauri window
- Backend debugging: Use `println!` macros visible in terminal during `bun tauri dev`
- For LLM testing: Use mock configurations via environment variables
- Permission testing: Verify macOS accessibility permissions are properly checked

## Platform Requirements

- **Target OS**: macOS 12.0+ (Monterey and above)
- **Architecture**: Universal Binary (Apple Silicon + Intel)
- **Permissions**: Accessibility permissions for input method switching and application monitoring
- **Package Manager**: Bun for frontend, Cargo for Rust dependencies

## Deployment Notes

- The app builds as a native macOS .app bundle
- Supports distribution via Homebrew Cask
- Uses static export configuration for Next.js integration with Tauri
- GitHub Actions configured for automated builds and releases