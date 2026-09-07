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

### 3.13 Treating App Names And Icons As Raw Bundle Metadata

**Mistake**: Rendering placeholder initials for managed apps, showing raw bundle fallback names such as `WeChat` or `aDrive` when macOS has localized names, storing large app icon payloads directly in persisted rule config, or forgetting Cocoa retain/release ownership when converting native icons to frontend-safe image data.

**Why it is easy to miss**: Rule rows already have enough identity to function (`bundle_id` and `app_name`), so raw `CFBundleName` values and placeholder icons can survive for a long time. Third-party apps may keep English `CFBundleName` values while shipping localized `InfoPlist.strings`, and system APIs can sometimes return only the plain `.app` filename. The tempting quick icon fix is to attach icon data to `AppRule`, but that makes `config.json` larger and stale when apps move or update their icons. On the native side, Rust ownership does not automatically manage Objective-C objects created with `alloc/init`; an autorelease pool only drains objects that were actually autoreleased.

**Correct behavior**:

- Resolve app display names from the installed app bundle at scan/rescan time.
- Prefer the macOS localized display name when available, but if it is only the plain `.app` filename, continue checking localized `InfoPlist.strings` before falling back to raw plist names.
- Treat plain `.app` filename comparisons as case-insensitive, because apps can return `Doubao` for `doubao.app` or `NetEaseMusic` for `NeteaseMusic.app`.
- Parse localized `InfoPlist.strings` as both UTF-8 and UTF-16 with BOM; Chinese third-party apps commonly ship UTF-16 strings files.
- Keep `AppRule.app_name` as a persisted display-name snapshot and `bundle_id` as the stable matching key; do not add a separate display-name field without a schema reason.
- Resolve app icons from the currently installed app bundle path at runtime.
- Keep app icon data as a frontend display cache only.
- Do not persist PNG/base64 icon data in `config.json`.
- When using Cocoa APIs from Rust, balance `alloc/init`, `copy`, `new`, or `mutableCopy` ownership with explicit `release` or `autorelease` after copied Rust-owned bytes are produced.
- Wrap repeated native rendering work in an `NSAutoreleasePool`, but do not treat the pool as a substitute for marking owned objects as autoreleased.
- If icon lookup fails, keep a stable fallback avatar and preserve rule editing behavior.
- Validate with a bundled app because Finder/System app icon resolution is a macOS integration behavior.

**Test method**:

1.  Build and launch the bundled `.app`.
2.  Open SmartIME Rules and confirm common third-party apps and supported system apps show localized names and real macOS icons.
3.  On a Chinese macOS system, confirm examples such as Doubao, NetEaseMusic, WeChat, and aDrive show as `豆包`, `网易云音乐`, `微信`, and `阿里云盘` when those localized resources exist.
4.  Confirm a missing or unresolved app icon falls back to the initial-letter avatar without blocking input-method selection, deletion, or rescan.
5.  Inspect `config.json` and confirm rules still contain only stable rule fields, not icon data URLs.
6.  Repeatedly revisit Rules or trigger rescans and confirm native memory does not grow monotonically from leaked retained Cocoa objects.

### 3.14 Sending Batchable Data To External Services One Item At A Time

**Mistake**: Sending each app to the LLM one by one, sending the complete installed-app set as one timeout-prone request, converting failed predictions into valid-looking AI fallback rules, reusing rescan gap detection in first onboarding, or parsing structured batch objects as generic string maps.

**Why it is easy to miss**: Per-item code is straightforward to write and easy to test with a small local sample, but it creates terrible user experience when the real data set is dozens of apps and each item triggers network latency, provider queueing, or rate limits. First scan and manual rescan also both produce `AppRule` lists, but only rescan has existing rules that can be reused. Batch LLM responses can arrive as either a direct `{ bundle_id: input_source_id }` map or structured objects like `{ bundle_id, preferred_input }`; checking the generic map shape first can silently drop valid structured entries.

**Correct behavior**:

- When a feature processes many similar records, first evaluate batching, caching, deduplication, and incremental gap processing before writing a per-item external request loop.
- Treat LLM/API calls as expensive UX boundaries; avoid N serial network calls when one batch call or a small number of bounded batches can produce the same result.
- Bound both batch size and concurrency. One batch timeout must not discard successful results from unrelated batches.
- First onboarding scan has no rule cache: batch-predict the full target app set, then align with an empty existing-rule list.
- Manual rescan must read existing persisted rules first, preserve manual rules, reuse valid existing AI rules, predict only missing/new/invalid AI gaps, then align and persist.
- Batch response parsing must validate both target bundle IDs and currently available input source IDs.
- Parse structured rule items before generic string maps, so `{ bundle_id, preferred_input }` entries are not misclassified.
- Provider errors, malformed JSON, or partial responses should not panic or fall back to serial N-request prediction. Keep valid partial results, leave failed apps without a rule, do not retry failed batches during the same scan, and request those gaps only on a later user-triggered rescan.
- DeepSeek rule classification must explicitly disable thinking, request JSON output, and bound output tokens. Its V4 models otherwise default to high-effort thinking, which can consume the full client timeout even for simple classification.
- Never label a deterministic fallback as `is_ai_generated: true`; never label it manual either, because both values would suppress correct retry behavior.
- Emit real settled-app progress for long-running batches and record failures through the application logger rather than terminal-only output.

**Test method**:

1.  Run `cd src-tauri && cargo test`.
2.  Confirm parser tests cover direct maps, structured arrays, malformed JSON, unknown bundle IDs, and invalid input source IDs.
3.  Confirm batching tests cover the 20-app limit, missing-prediction omission, manual preservation, valid AI-rule reuse, invalid AI-rule gaps, new app gaps, stale app pruning, and legacy all-fallback recovery.
4.  Run onboarding scan and manual rescan on the bundled app; unchanged rescans should avoid LLM calls for already valid rules.
5.  During review, search for loops that call LLM/API/network functions per record and require a clear reason if they are intentionally serial.

### 3.15 Inferring The LLM Provider From URLs Or Model Names

**Mistake**: Exposing a free-form Base URL in settings while the backend speaks only one wire protocol, adding a provider selector only to the frontend form, or inferring the provider/protocol from the model name string.

**Why it is easy to miss**: DeepSeek and many gateways are OpenAI-compatible, so a single-protocol client appears to work during development. Model-name inference also looks correct for common names such as `deepseek-*` or `claude-*`, but silently misroutes arbitrary valid model names, and a form-only provider selector still sends OpenAI-shaped payloads to Anthropic or Gemini native endpoints.

**Correct behavior**:

- Treat the provider as part of the persisted config model, the Keychain credential binding, and the IPC contract — never as a frontend-only concern.
- Resolve the explicit provider to a native protocol adapter (`genai::adapter::AdapterKind`) and a provider-managed endpoint; do not expose custom service addresses in the UI.
- Bind saved-key reuse to the unchanged provider: a blank API key reuses the stored key only for the same provider, and switching providers requires a new key.
- Migrate legacy `base_url` configs and Base-URL-bound credentials through one-time inference (`LLMProvider::infer_legacy`), then drop the legacy field on the next successful save. Never keep inference on the hot request path.
- Apply the shared request policy on every provider: HTTPS-only, no redirects, 60-second timeout, bounded output tokens, JSON mode for batch prediction, and disabled reasoning for DeepSeek classification.

**Test method**:

1.  Run `cd src-tauri && cargo test`.
2.  Confirm `explicit_provider_selects_native_genai_adapter` maps each provider to its native adapter without model-string inference.
3.  Confirm `legacy_provider_is_inferred_from_model_or_service_address` covers legacy migration inference only.
4.  Confirm `cannot_reuse_key_for_different_provider_and_delete_survives_reload` proves provider-bound key reuse.
5.  Confirm `batch_options_are_bounded_and_request_json_for_every_provider` proves the shared bounded/JSON policy and DeepSeek-only reasoning disablement.
6.  On the bundled app, verify onboarding shows the provider selector with no Base URL field, and a legacy Base-URL config migrates on first credential access.

**Related files**: `src-tauri/src/llm.rs`, `src-tauri/src/credentials.rs`, `lib/api.ts`, `app/onboarding/llm/page.tsx`

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
| INC-011 | Treating app names and icons as raw bundle metadata | Rule rows showed initial-letter placeholders and raw English bundle fallback names instead of the localized names and icons users see in macOS; review also found the native icon rendering path initially leaked retained Objective-C objects. | Resolve localized app names and real app icons from installed bundle paths at scan/rescan time, keep icon payloads out of persisted rules, fall back visually when lookup fails, and explicitly release/autorelease Cocoa objects created with ownership transfer. | In the bundled app, Rules rows should show localized names and real icons, `config.json` remains free of icon data, and repeated Rules visits/rescans should not leak native image memory. |
| INC-012 | Sending batchable data to external services one item at a time | Initial rule generation sent each app to the LLM separately, making first scan and rescan painfully slow; the later optimization also briefly applied rescan-only gap detection to the first scan path, and batch parsing initially misclassified structured rule objects as a generic string map. | For similar records, consider batching/caching/incremental gaps before writing per-item API loops. Keep first scan as full batch prediction with empty existing rules; keep rescan as incremental gap prediction; parse structured batch entries before generic maps. | `cargo test` must cover first/rescan helper behavior and batch parser response shapes, and review should flag serial per-record LLM/API loops unless there is a clear product reason. |
| INC-013 | Persisting a failed whole-set prediction as AI output | A 79-app request hit the 60-second provider timeout; the error became an empty result, alignment filled every app with the first input source (`ABC`), and rescan reused those fake AI rules forever. After batching, DeepSeek's default high-effort thinking still caused two 20-app batches to hit the same timeout, leaving 40 legitimate gaps. | Use bounded 20-app batches with concurrency 2, retain independent successes, omit failed predictions, and request missing rules only on a later rescan. For DeepSeek classification, disable thinking, require JSON output, cap output tokens, and preserve the timeout stage in safe logs. | Tests must prove batch sizing, omission instead of AI fallback, progress payloads, manual-safe recovery of the legacy full-fallback signature, and DeepSeek-specific request serialization without leaking those fields into generic providers. |

## 5. Testing Methods AI Should Prefer

### Credential Storage Regression

- Password fields and masked IPC responses do not protect plaintext files. Keep the disk schema separate from secret-bearing request types.
- Run `cargo test --offline --locked --manifest-path src-tauri/Cargo.toml` for migration/readback failure, atomic-write failure, replacement/deletion, provider binding, legacy Base URL inference, and corrupt-file behavior. These use fake credentials and a fake Keychain.
- Run `bun test lib/api.test.mjs` to verify browser preview clears old storage and retains no submitted secret.
- Native integration: `cargo test --offline --locked --manifest-path src-tauri/Cargo.toml --bin smartime credentials::tests::native_keychain_roundtrip -- --ignored --exact` creates and deletes only a random disposable entry. Run outside a sandbox that blocks Keychain; never use a user's real key in test output.
- Before release, verify the signed/bundled app's Keychain authorization, denied-access recovery, legacy migration, replacement/deletion across restart, and CSP-protected onboarding. Unit tests or an unsigned test binary do not establish bundled-app access behavior.
- A successful migration cannot erase plaintext copies in existing backups or development `.env.llm`; do not create new plaintext backups or claim those old copies were securely erased.

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
4.  Rule app names and icons: installed app rows show localized macOS app names and real icons, unresolved icons fall back cleanly, `config.json` does not persist icon payloads, and repeated icon loads do not leak retained Cocoa objects.
5.  Rules rescan: no crash, duplicate triggers blocked, loading lifecycle correct across panel switches.
6.  System app scope: curated input-capable Apple apps appear with recognizable names; internal/system utility bundles stay hidden.
7.  Scan performance: first scan avoids serial per-app LLM calls; unchanged manual rescan reuses valid existing rules and predicts only gaps.
8.  Input-source stability: repeated automatic input-source switches do not crash the app, current-input-source reads do not leave the main thread, and frontend input-source commands do not time out while waiting for main-thread TIS work.
9.  Dock/tray behavior: hide/show Dock transitions and window reactivation behavior are stable.
10.  Login item behavior: startup works without duplicate process/icon side effects.
11.  Identity and distribution: metadata aligns across Rust, Tauri, bundled app, release artifact, and cask surfaces.
12.  LLM provider setup: onboarding offers DeepSeek, OpenAI, Anthropic, and Gemini with provider-managed endpoints and no Base URL field; a blank API key reuses the saved key only for the unchanged provider; a legacy Base-URL-based `llm_config.json` migrates to a provider-bound Keychain credential on first access; connection test and first scan succeed with a real key on the configured provider.
