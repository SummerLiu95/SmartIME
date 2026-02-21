use std::fs;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager};

static PRIMARY_LISTENER: OnceLock<Mutex<Option<UnixListener>>> = OnceLock::new();

fn socket_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        return home
            .join("Library")
            .join("Caches")
            .join("com.smartime.app")
            .join("single-instance.sock");
    }

    std::env::temp_dir().join("smartime-single-instance.sock")
}

pub fn prepare_primary_instance() -> bool {
    let path = socket_path();

    // If another instance is alive, signal it to focus main window and exit this one.
    if let Ok(mut stream) = UnixStream::connect(&path) {
        let _ = stream.write_all(b"activate");
        return false;
    }

    if path.exists() {
        let _ = fs::remove_file(&path);
    }

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    match UnixListener::bind(&path) {
        Ok(listener) => {
            let _ = listener.set_nonblocking(false);
            let slot = PRIMARY_LISTENER.get_or_init(|| Mutex::new(None));
            if let Ok(mut guard) = slot.lock() {
                *guard = Some(listener);
            }
            true
        }
        Err(err) => {
            eprintln!("Failed to initialize single-instance socket: {}", err);
            // Fail open so app can still run even if socket init fails.
            true
        }
    }
}

pub fn start_activation_listener(app: AppHandle) {
    let listener = PRIMARY_LISTENER
        .get()
        .and_then(|slot| slot.lock().ok())
        .and_then(|mut guard| guard.take());

    if let Some(listener) = listener {
        std::thread::spawn(move || {
            for incoming in listener.incoming() {
                match incoming {
                    Ok(_) => {
                        focus_main_window(&app);
                    }
                    Err(err) => {
                        eprintln!("Single-instance listener error: {}", err);
                    }
                }
            }
        });
    }
}

pub fn focus_main_window(app: &AppHandle) {
    let scheduler = app.clone();
    let app_handle = app.clone();
    let _ = scheduler.run_on_main_thread(move || {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    });
}
