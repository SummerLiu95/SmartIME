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

## 4. Incident Catalog

| Incident ID | AI-prone mistake | What Happened | Corrective Lesson | Regression Test |
| :--- | :--- | :--- | :--- | :--- |
| INC-001 | Trusting dev identity for Accessibility behavior | In dev/debug scenarios, users could not reliably add the app from Accessibility settings or could not locate the expected app identity. | macOS TCC behavior is identity-sensitive; validate permission behavior with bundled metadata, not only dev runtime. | Build bundled app, reset Accessibility permission, rerun permission onboarding, verify expected bundle identity. |
| INC-002 | Combining permission request/check/navigation | Permission guide action triggered both native prompt and system settings navigation, while retry/check also triggered prompt. | Request, check, and settings navigation must be separate user actions. | Verify guide action is request-only and retry/check is check-only after TCC reset. |
| INC-003 | Leaving panic-prone async paths | Clicking rescan could crash app with `EXC_BREAKPOINT` / `SIGTRAP` on a tokio worker. | Scan/rescan paths must be panic-free, single in-flight, and recoverable on error. | Trigger duplicate rescans and forced error paths; verify no crash and loading state clears. |
| INC-004 | Tying async completion to page lifecycle | After onboarding scan success and redirect, rules panel could remain in a perpetual loading state. | Cross-page async completion must have one authoritative backend state transition. | Complete onboarding scan, redirect to rules, verify rules load and loading state clears. |
| INC-005 | Treating system state as append-only | Input method options showed stale or helper entries that did not match currently enabled input methods. | Apps and input sources must be re-synced from system truth on every scan/rescan. | Change enabled input methods, rescan, verify dropdown and persisted rules are pruned. |
| INC-006 | Coupling Dock/tray/autostart lifecycle | Hide Dock mode, login item flow, and relaunch/reactivation had duplicate icon or wrong reopen behavior. | Lifecycle settings must remain independent and all entry points must restore one existing window/process. | Validate hide-Dock close, tray reopen, Dock reopen, and login item startup on bundled app. |

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
3.  Rules rescan: no crash, duplicate triggers blocked, loading lifecycle correct across panel switches.
4.  Dock/tray behavior: hide/show Dock transitions and window reactivation behavior are stable.
5.  Login item behavior: startup works without duplicate process/icon side effects.
6.  Identity and distribution: metadata aligns across Rust, Tauri, bundled app, release artifact, and cask surfaces.
