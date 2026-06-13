# SmartIME Rulebook

This file records mistakes that AI coding agents are likely to repeat in this project, plus the testing methods and hard-won lessons that prevent those mistakes from coming back.

Use this file as a prevention checklist. It is not the product requirements document, not the technical spec, and not the pre-development task plan.

## 1. What Belongs Here

Add content here when it helps a future AI agent avoid repeating a known failure.

Good entries include:

- AI mistake patterns observed in this project.
- The correct behavior or implementation habit that prevents the mistake.
- Test methods that caught or should catch the mistake.
- Release validation steps for macOS/Tauri behaviors.
- Bug-fix records where the root lesson should be remembered.

Do not use this file for:

- Product requirements. Put those in `docs/REQUIREMENTS.md`.
- Architecture contracts or module design. Put those in `docs/TECHNICAL_SPEC.md`.
- User-requested or Plan-mode task planning. Put that in `docs/TASKS.md`.
- Implementation details after a confirmed plan has been executed. Put those in `docs/exec-plan/`.
- Developer setup and commands. Put those in `README.md`.

## 2. AI Mistake Record Template

When a bug fix reveals a repeatable AI failure mode, add a short record using this template:

- `Mistake`: What the AI or implementation did wrong.
- `Trigger scenario`: When the mistake appears.
- `Why it is easy to miss`: The misleading assumption or local evidence that caused it.
- `Correct behavior`: What future work must do instead.
- `Test method`: The command, manual flow, bundled-app check, or regression case that verifies the fix.
- `Related files`: The main files or modules involved.

## 3. High-Risk AI Mistakes

### 3.1 Mixing Documentation Responsibilities

**Mistake**: Updating only code, or putting all notes into one convenient document.

**Correct behavior**:

- User-visible behavior belongs in `docs/REQUIREMENTS.md`.
- Architecture and runtime contracts belong in `docs/TECHNICAL_SPEC.md`.
- Lessons about repeated AI mistakes and testing methods belong in this file.
- `docs/TASKS.md` is updated only when the user explicitly asks AI to plan tasks, or when work is being planned in Plan mode.
- `docs/exec-plan/` records are created after one of those plans is confirmed for execution and the implementation has been completed.

**Test method**:

- Before handoff, check whether the changed behavior has an owner document update.
- Search for stale path references after moving or splitting docs.

### 3.2 Treating Dev Runtime as Release Evidence

**Mistake**: Assuming `bun tauri dev` proves macOS permission, login-item, Dock, tray, or bundle identity behavior.

**Why it is easy to miss**: The dev runtime is fast and can appear functionally correct, but macOS TCC and app lifecycle behavior are identity-sensitive.

**Correct behavior**:

- Use `bun tauri dev` for iteration only.
- Validate release-level OS integration with a bundled `.app` or `.dmg`.

**Test method**:

1.  Build the bundled app.
2.  Install or run the bundled app in the release-like location.
3.  Validate Accessibility registration, login-item behavior, Dock/tray activation, and app identity from the bundle.

### 3.3 Combining Permission Request and Permission Check

**Mistake**: Triggering native authorization, opening System Settings, and checking permission state from the same click path.

**Correct behavior**:

- Guide/request action may trigger the native authorization request only.
- Retry/check action must verify current permission state only.
- Opening System Settings must be an explicit manual fallback.

**Test method**:

1.  Reset permission state when needed:
    ```bash
    tccutil reset Accessibility com.smartime.app
    ```
2.  Launch the bundled app.
3.  Click the permission guide action and confirm it does not also perform check/navigation side effects.
4.  Click retry/check and confirm it does not trigger a new native prompt.

### 3.4 Trusting Cached Scan Results

**Mistake**: Treating app scan results or input-source options as append-only state.

**Why it is easy to miss**: The UI may look correct on the developer's machine while stale input methods or removed apps remain hidden in persisted config.

**Correct behavior**:

- On every onboarding scan and manual rescan, re-sync installed apps from `/Applications` and `~/Applications`.
- Re-sync enabled/selectable input sources from macOS APIs.
- Prune stale input-source IDs from rule options and persisted rules.
- Exclude helper or non-selectable input-source entries from dropdowns.

**Test method**:

- Compare rule dropdown options against currently enabled macOS input sources.
- Remove or disable an input method, rescan, and verify stale IDs disappear from options and persisted rules.
- Verify generated rules never invent input source IDs.

### 3.5 Allowing Panic Paths in Runtime Async Work

**Mistake**: Using `unwrap` or `expect` in scan/rescan/save/lifecycle async paths.

**Why it is easy to miss**: The happy path works locally, but error cases can crash the Tauri runtime worker.

**Correct behavior**:

- Return recoverable errors to the UI.
- Guard scan/rescan as a single in-flight task.
- Ensure rescan flags are reset even on early return.
- Persist only after validation and guarded merge.

**Test method**:

- Trigger duplicate rescans quickly.
- Force or simulate scan/LLM/input-source errors.
- Confirm the app does not crash and the UI loading state eventually clears.

### 3.6 Losing Loading State Across Navigation

**Mistake**: Letting onboarding scan or manual rescan completion state depend on one page remaining mounted.

**Correct behavior**:

- Backend task lifecycle is authoritative for long-running rescan state.
- UI should poll or reload backend state after navigation.
- Loading state must clear after backend completion even if the user switched panels.

**Test method**:

- Start rescan from rules page.
- Navigate to another settings panel while rescan is running.
- Return to rules page and confirm loading state and saved rules reflect backend completion.

### 3.7 Coupling `autoStart` and `hideDockIcon`

**Mistake**: Treating login item behavior and Dock visibility as one setting, or closing the main window immediately when toggling hide-Dock mode.

**Correct behavior**:

- `autoStart` and `hideDockIcon` are independent settings.
- Toggling `hideDockIcon` must not close the visible settings window immediately.
- In hide-Dock mode, closing the window should keep the app alive in the menu bar.
- Dock/tray reactivation should restore the existing main window in the same process.

**Test method**:

- Toggle `hideDockIcon` while the settings window is open; verify the window stays usable.
- Close the window in hide-Dock mode; verify the app remains alive in the menu bar.
- Reactivate from Dock/tray/login item and verify no duplicate process or duplicate tray icon appears.

### 3.8 Letting App Identity Drift

**Mistake**: Updating only one metadata location during release or identity changes.

**Correct behavior**:

Keep identity aligned across:

- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- bundled `.app` metadata
- release artifact naming
- Homebrew cask metadata when applicable

**Test method**:

- Inspect bundled app metadata before release.
- Validate TCC permission entries are associated with the expected bundle identifier.
- Validate login-item and Dock/tray behavior with the packaged identity.

### 3.9 Listing Every System Bundle As A Rule Target

**Mistake**: Scanning system app directories and surfacing every discovered `.app` bundle in the Rules UI.

**Why it is easy to miss**: Directory traversal proves that the bundle exists, but many system bundles are internal agents, onboarding shells, or background utilities that are not meaningful input-method rule targets.

**Correct behavior**:

- Keep third-party and user-installed apps discoverable as usual.
- For system roots, expose only a curated set of common input-capable apps that users are likely to switch into and type in.
- Prefer localized display names for supported system apps so the Rules list stays recognizable.
- Include Cryptex-backed Safari locations when building the curated system-app set.

**Test method**:

1.  Build the bundled app.
2.  Open the Rules panel and run rescan.
3.  Confirm Safari, Notes, Reminders, TextEdit, Terminal, Mail, Messages, Calendar, and Finder can appear when present.
4.  Confirm internal bundles such as `SystemUIServer`, `Dock`, `ControlCenter`, and similar background components do not appear in the rules list.

### 3.10 Calling TIS Current-Input APIs Off The Main Thread

**Mistake**: Reading the current macOS input source from a background thread during observer work.

**Why it is easy to miss**: `TISSelectInputSource` was already scheduled onto the main thread, so it is easy to assume nearby `TISCopyCurrentKeyboardInputSource` calls are equally safe anywhere.

**Correct behavior**:

- Treat current-input-source reads as main-thread-only HIToolbox/TIS work.
- Reuse one main-thread scheduling helper for frontend commands and background-triggered observer flows.
- If a background worker needs current input-source state, marshal the work to the main thread and wait with a bounded timeout.

**Test method**:

1.  Build the bundled `.app`.
2.  Keep automatic switching on.
3.  Rapidly switch between managed apps that trigger automatic input-source changes.
4.  Confirm there is no `EXC_BREAKPOINT` crash involving `TISCopyCurrentKeyboardInputSource` or `dispatch_assert_queue`.

### 3.11 Blocking A Sync Command While It Schedules Main-Thread TIS Work

**Mistake**: Making a synchronous Tauri command enqueue TIS work with `run_on_main_thread`, then immediately blocking that same command while waiting for the result channel.

**Why it is easy to miss**: The helper looks correct because it keeps HIToolbox/TIS calls on the main thread, but frontend IPC commands can already be running on the event path that must process the queued main-thread task. The command can end up waiting for work that cannot run until the command returns.

**Correct behavior**:

- Frontend-facing commands that schedule main-thread TIS work must be `async`.
- Put the blocking channel wait inside `tauri::async_runtime::spawn_blocking` or an equivalent background wait boundary.
- Keep the actual TIS query/selection inside the `run_on_main_thread` closure.
- Preserve bounded timeouts and return recoverable `AppError::InputSource` errors instead of panicking.

**Test method**:

1.  Run `cd src-tauri && cargo test`.
2.  From the frontend, trigger input-source list loading and manual input-source selection.
3.  Run onboarding scan and manual rescan, which both depend on system input-source retrieval.
4.  Confirm these flows do not hit the 500ms or 5s main-thread timeout errors.

### 3.12 Assuming TIS Localized Names Match System Settings

**Mistake**: Using only `kTISPropertyLocalizedName` as the input method label and assuming it matches the label macOS shows in System Settings or the input menu.

**Why it is easy to miss**: The property name says "localized", but some built-in input methods can still return English fallback labels such as `Pinyin - Simplified` on a Chinese system.

**Correct behavior**:

- Resolve display names with AppKit `NSTextInputContext.localizedNameForInputSource:` first.
- If AppKit still returns a known English fallback for a built-in Apple input method, apply a small locale-gated built-in localization fallback.
- Fall back to `kTISPropertyLocalizedName` when neither AppKit nor the built-in fallback provides a better label.
- Keep rule persistence based on stable input source IDs, not display names.

**Test method**:

1.  Enable Simplified Chinese Pinyin in macOS input sources.
2.  Open SmartIME Rules and inspect the input method dropdown.
3.  Confirm the label follows the system-localized name, for example `简体拼音`, while rule values still store the original input source ID.

### 3.13 Treating App Icons As Persisted Rule Data Or Ignoring Cocoa Ownership

**Mistake**: Rendering placeholder initials for managed apps, storing large app icon payloads directly in persisted rule config, or forgetting Cocoa retain/release ownership when converting native icons to frontend-safe image data.

**Why it is easy to miss**: Rule rows already have enough identity to function (`bundle_id` and `app_name`), so a placeholder can survive for a long time. The tempting quick fix is to attach icon data to `AppRule`, but that makes `config.json` larger and stale when apps move or update their icons. On the native side, Rust ownership does not automatically manage Objective-C objects created with `alloc/init`; an autorelease pool only drains objects that were actually autoreleased.

**Correct behavior**:

- Resolve app icons from the currently installed app bundle path at runtime.
- Keep app icon data as a frontend display cache only.
- Do not persist PNG/base64 icon data in `config.json`.
- When using Cocoa APIs from Rust, balance `alloc/init`, `copy`, `new`, or `mutableCopy` ownership with explicit `release` or `autorelease` after copied Rust-owned bytes are produced.
- Wrap repeated native rendering work in an `NSAutoreleasePool`, but do not treat the pool as a substitute for marking owned objects as autoreleased.
- If icon lookup fails, keep a stable fallback avatar and preserve rule editing behavior.
- Validate with a bundled app because Finder/System app icon resolution is a macOS integration behavior.

**Test method**:

1.  Build and launch the bundled `.app`.
2.  Open SmartIME Rules and confirm common third-party apps and supported system apps show their real macOS icons.
3.  Confirm a missing or unresolved app icon falls back to the initial-letter avatar without blocking input-method selection, deletion, or rescan.
4.  Inspect `config.json` and confirm rules still contain only stable rule fields, not icon data URLs.
5.  Repeatedly revisit Rules or trigger rescans and confirm native memory does not grow monotonically from leaked retained Cocoa objects.

## 4. Incident Catalog

| Incident ID | AI-prone mistake | What Happened | Corrective Lesson | Regression Test |
| :--- | :--- | :--- | :--- | :--- |
| INC-001 | Trusting dev identity for Accessibility behavior | In dev/debug scenarios, users could not reliably add the app from Accessibility settings or could not locate the expected app identity. | macOS TCC behavior is identity-sensitive; validate permission behavior with bundled metadata, not only dev runtime. | Build bundled app, reset Accessibility permission, rerun permission onboarding, verify expected bundle identity. |
| INC-002 | Combining permission request/check/navigation | Permission guide action triggered both native prompt and system settings navigation, while retry/check also triggered prompt. | Request, check, and settings navigation must be separate user actions. | Verify guide action is request-only and retry/check is check-only after TCC reset. |
| INC-003 | Leaving panic-prone async paths | Clicking rescan could crash app with `EXC_BREAKPOINT` / `SIGTRAP` on a tokio worker. | Scan/rescan paths must be panic-free, single in-flight, and recoverable on error. | Trigger duplicate rescans and forced error paths; verify no crash and loading state clears. |
| INC-004 | Tying async completion to page lifecycle | After onboarding scan success and redirect, rules panel could remain in a perpetual loading state. | Cross-page async completion must have one authoritative backend state transition. | Complete onboarding scan, redirect to rules, verify rules load and loading state clears. |
| INC-005 | Treating system state as append-only | Input method options showed stale or helper entries that did not match currently enabled input methods. | Apps and input sources must be re-synced from system truth on every scan/rescan. | Change enabled input methods, rescan, verify dropdown and persisted rules are pruned. |
| INC-006 | Coupling Dock/tray/autostart lifecycle | Hide Dock mode, login item flow, and relaunch/reactivation had duplicate icon or wrong reopen behavior. | Lifecycle settings must remain independent and all entry points must restore one existing window/process. | Validate hide-Dock close, tray reopen, Dock reopen, and login item startup on bundled app. |
| INC-007 | Treating all system bundles as user-facing app targets | System scan surfaced internal/background Apple bundles while still risking missing Safari on Cryptex-backed systems. | System roots must be filtered to a curated input-capable allowlist, with Safari Cryptex locations and localized display names handled deliberately. | Rescan bundled app and confirm common typing apps appear while background system bundles stay hidden. |
| INC-008 | Calling current-input-source APIs off the main thread | Current-input-source reads in app-switch handling crashed bundled app with `EXC_BREAKPOINT` / `dispatch_assert_queue` inside `TISCopyCurrentKeyboardInputSource`. | HIToolbox current-input-source reads must be marshaled to the main thread just like input-source selection. | Rapid app switching on bundled app should not crash while automatic switching continues to work. |
| INC-009 | Blocking a sync command after scheduling main-thread TIS work | Frontend-facing input-source commands could enqueue TIS work back to the main thread and then wait synchronously, risking a timeout because the queued task could not run until the command returned. | Make frontend TIS commands async and move the channel wait into `tauri::async_runtime::spawn_blocking`, while keeping the actual TIS call in `run_on_main_thread`. | Load input sources, manually select an input source, and run onboarding/manual rescans without 500ms or 5s main-thread timeout errors. |
| INC-010 | Trusting TIS localized names as final UI labels | Built-in input methods could show English fallback labels such as `Pinyin - Simplified` instead of the system-localized label users see in macOS. | Prefer AppKit input-source localized names, use a locale-gated built-in Apple fallback for known English labels, and fall back to TIS only when needed. | On a Chinese macOS system, Rules dropdown should show `简体拼音` or the current system-localized equivalent for Simplified Pinyin. |
| INC-011 | Treating app icons as persisted rule data or ignoring Cocoa ownership | Rule rows showed initial-letter placeholders instead of the same app icons users see in macOS, and review found the native rendering path initially leaked retained Objective-C objects. | Resolve real app icons from installed bundle paths at runtime, keep icon payloads out of persisted rules, fall back visually when lookup fails, and explicitly release/autorelease Cocoa objects created with ownership transfer. | In the bundled app, Rules rows should show real icons, `config.json` remains free of icon data, and repeated Rules visits/rescans should not leak native image memory. |

## 5. Testing Methods AI Should Prefer

### 5.1 Fast Iteration Checks

Use these while coding:

- `bun run lint` for frontend lint checks.
- Targeted Rust tests or `cargo test` when touching Rust logic.
- Manual Tauri dev checks for quick UI/IPC feedback.

These checks are useful but not enough for release-level macOS behavior.

### 5.2 Bundled-App Checks

Use bundled-app checks for:

- Accessibility permission flow.
- TCC identity behavior.
- Login item behavior.
- Dock/tray lifecycle.
- Single-instance reactivation.
- Release artifact naming and metadata.

### 5.3 Crash Investigation Baseline

When investigating a crash, capture:

- exception type/code
- crashing thread name
- process identifier
- bundle identifier
- task lifecycle at the moment of crash: scan, merge, persist, permission flow, UI state propagation, or lifecycle transition

Do not fix only the visible symptom. Record the AI-prone mistake in this file if it is likely to recur.

## 6. Release Regression Matrix

Run this matrix on a bundled app before release:

1.  Permission onboarding: request-only and check-only actions are independent.
2.  First scan output: app list and input method options match current system state.
3.  Input method labels: options use system-localized display names while persisted values remain stable source IDs.
4.  Rule app icons: installed app rows show real macOS icons, unresolved icons fall back cleanly, `config.json` does not persist icon payloads, and repeated icon loads do not leak retained Cocoa objects.
5.  Rules rescan: no crash, duplicate triggers blocked, loading lifecycle correct across panel switches.
6.  System app scope: curated input-capable Apple apps appear with recognizable names; internal/system utility bundles stay hidden.
7.  Input-source stability: repeated automatic input-source switches do not crash the app, current-input-source reads do not leave the main thread, and frontend input-source commands do not time out while waiting for main-thread TIS work.
8.  Dock/tray behavior: hide/show Dock transitions and window reactivation behavior are stable.
9.  Login item behavior: startup works without duplicate process/icon side effects.
10.  Identity and distribution: metadata aligns across Rust, Tauri, bundled app, release artifact, and cask surfaces.
