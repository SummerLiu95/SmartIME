use crate::config::GeneralSettings;
use crate::error::{AppError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::image::Image;
use tauri::{AppHandle, Manager};

pub const TRAY_ICON_ID: &str = "smartime-status";
const TRAY_ICON_BYTES: &[u8] = include_bytes!("../icons/tray/32x32.png");

pub fn apply_general_settings(app: &AppHandle, settings: &GeneralSettings) -> Result<()> {
    apply_dock_visibility(app, settings.hide_dock_icon)?;
    sync_tray_icon_visibility(app, settings.hide_dock_icon)?;
    apply_auto_start(app, settings.auto_start)?;
    Ok(())
}

pub fn apply_general_settings_delta(
    app: &AppHandle,
    previous: &GeneralSettings,
    next: &GeneralSettings,
) -> Result<()> {
    if previous.hide_dock_icon != next.hide_dock_icon {
        apply_dock_visibility(app, next.hide_dock_icon)?;
        sync_tray_icon_visibility(app, next.hide_dock_icon)?;
    }

    if previous.auto_start != next.auto_start {
        apply_auto_start(app, next.auto_start)?;
    }

    Ok(())
}

fn apply_dock_visibility(app: &AppHandle, hide_dock_icon: bool) -> Result<()> {
    // Keep the dock visible while the settings window is currently open,
    // and only hide the dock after users close the window into tray mode.
    let should_show_dock = if hide_dock_icon {
        app.get_webview_window("main")
            .and_then(|window| window.is_visible().ok())
            .unwrap_or(false)
    } else {
        true
    };

    app.set_dock_visibility(should_show_dock)
        .map_err(|e| AppError::Config(format!("Failed to update dock visibility: {}", e)))
}

fn sync_tray_icon_visibility(app: &AppHandle, visible: bool) -> Result<()> {
    #[cfg(desktop)]
    {
        use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

        if app.tray_by_id(TRAY_ICON_ID).is_none() && visible {
            let icon = Image::from_bytes(TRAY_ICON_BYTES)
                .map_err(|e| AppError::Config(format!("Failed to load tray icon bytes: {}", e)))?;
            TrayIconBuilder::with_id(TRAY_ICON_ID)
                .icon(icon)
                .tooltip("SmartIME")
                .icon_as_template(true)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)
                .map_err(|e| AppError::Config(format!("Failed to create tray icon: {}", e)))?;
        }

        if let Some(tray) = app.tray_by_id(TRAY_ICON_ID) {
            tray.set_visible(visible).map_err(|e| {
                AppError::Config(format!("Failed to update tray visibility: {}", e))
            })?;
        }

        Ok(())
    }

    #[cfg(not(desktop))]
    {
        let _ = app;
        Ok(())
    }
}

fn apply_auto_start(app: &AppHandle, auto_start: bool) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let label = app.config().identifier.clone();
        let plist_path = launch_agent_path(&label)?;

        if auto_start {
            let exec_path = std::env::current_exe().map_err(|e| {
                AppError::Config(format!("Failed to resolve executable path: {}", e))
            })?;
            let plist_content = build_launch_agent_plist(&label, &exec_path);
            if let Some(parent) = plist_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    AppError::Config(format!("Failed to create LaunchAgents dir: {}", e))
                })?;
            }
            fs::write(&plist_path, plist_content).map_err(|e| {
                AppError::Config(format!("Failed to write LaunchAgent plist: {}", e))
            })?;
        } else {
            if plist_path.exists() {
                // Best-effort unload to avoid lingering managed state.
                let _ = disable_launch_agent(&plist_path);
                let _ = fs::remove_file(&plist_path);
            }
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = app; // suppress unused warnings on non-macOS
        let _ = auto_start;
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn launch_agent_path(label: &str) -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Config("Failed to resolve home directory".to_string()))?;
    Ok(home
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{}.plist", label)))
}

#[cfg(target_os = "macos")]
fn build_launch_agent_plist(label: &str, exec_path: &Path) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exec}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
</dict>
</plist>
"#,
        label = label,
        exec = exec_path.display()
    )
}

#[cfg(target_os = "macos")]
fn disable_launch_agent(plist_path: &Path) -> Result<()> {
    let uid = current_uid()?;
    let target = format!("gui/{}", uid);

    if run_launchctl(["bootout", &target, plist_path.to_string_lossy().as_ref()])? {
        return Ok(());
    }

    if run_launchctl(["unload", "-w", plist_path.to_string_lossy().as_ref()])? {
        return Ok(());
    }

    Err(AppError::Config(
        "Failed to disable auto-start via launchctl".to_string(),
    ))
}

#[cfg(target_os = "macos")]
fn run_launchctl<I, S>(args: I) -> Result<bool>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let output = Command::new("launchctl")
        .args(args)
        .output()
        .map_err(|e| AppError::Config(format!("launchctl failed to start: {}", e)))?;

    if output.status.success() {
        return Ok(true);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        eprintln!("launchctl error: {}", stderr.trim());
    }
    Ok(false)
}

#[cfg(target_os = "macos")]
fn current_uid() -> Result<String> {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .map_err(|e| AppError::Config(format!("Failed to run id -u: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::Config(
            "Failed to resolve UID via id -u".to_string(),
        ));
    }

    let uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if uid.is_empty() {
        return Err(AppError::Config("UID is empty".to_string()));
    }
    Ok(uid)
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use super::*;

    #[test]
    fn test_build_launch_agent_plist() {
        let label = "com.smartime.app";
        let exec_path = Path::new("/Applications/SmartIME.app/Contents/MacOS/SmartIME");
        let plist = build_launch_agent_plist(label, exec_path);
        assert!(!plist.contains("\\\""));
        assert!(plist.contains("<key>Label</key>"));
        assert!(plist.contains(label));
        assert!(plist.contains(exec_path.to_str().unwrap()));
        assert!(plist.contains("<key>RunAtLoad</key>"));
        assert!(plist.contains("<true/>"));
    }
}
