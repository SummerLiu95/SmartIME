#![allow(deprecated)] // Suppress warnings for deprecated cocoa APIs

use crate::config::AppState;
use crate::input_source::select_input_source;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use once_cell::sync::OnceCell;
use std::ffi::CStr;
use std::sync::mpsc::Sender;
use std::sync::Once;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

// 定义事件数据结构
#[derive(Debug, Clone, serde::Serialize)]
pub struct AppFocusedEvent {
    pub bundle_id: String,
    pub app_name: String,
}

// 全局 Channel Sender，用于从 FFI 回调向主线程发送消息
static APP_EVENT_TX: OnceCell<Sender<AppFocusedEvent>> = OnceCell::new();
static REGISTER_OBSERVER_CLASS: Once = Once::new();

/// 初始化监听器
///
/// * `app_handle`: Tauri App Handle，用于发送事件到前端
pub fn setup_observer(app_handle: AppHandle) {
    // 创建一个 Channel
    let (tx, rx) = std::sync::mpsc::channel::<AppFocusedEvent>();

    // 保存 Sender 到全局变量
    if APP_EVENT_TX.set(tx).is_err() {
        eprintln!("Failed to set global sender for app observer");
        return;
    }

    // 启动一个线程来处理事件并发送给前端
    std::thread::spawn(move || {
        let mut last_bundle_id = String::new();
        let mut last_selected_input = String::new();

        while let Ok(event) = rx.recv() {
            if event.bundle_id == last_bundle_id {
                continue;
            }
            last_bundle_id = event.bundle_id.clone();

            // println!("App focused: {:?}", event);
            // 发送事件到前端
            if let Err(e) = app_handle.emit("app_focused", &event) {
                eprintln!("Failed to emit app_focused event: {}", e);
            }

            match apply_input_source_for_bundle_on_main_thread(
                &app_handle,
                event.bundle_id.clone(),
                if last_selected_input.is_empty() {
                    None
                } else {
                    Some(last_selected_input.clone())
                },
            ) {
                Ok(Some(selected_input)) => {
                    last_selected_input = selected_input;
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!(
                        "Failed to switch input source for {} ({}): {}",
                        event.app_name, event.bundle_id, e
                    );
                }
            }
        }
    });

    unsafe {
        // 注册 Objective-C 类
        REGISTER_OBSERVER_CLASS.call_once(|| {
            let superclass = class!(NSObject);
            let mut decl =
                ClassDecl::new("RustAppObserver", superclass).expect("Failed to declare class");

            decl.add_method(
                sel!(appActivated:),
                app_activated_impl as extern "C" fn(&Object, Sel, id),
            );

            decl.register();
        });

        let pool = NSAutoreleasePool::new(nil);

        // 实例化 Observer
        let observer_class = class!(RustAppObserver);
        let observer: id = msg_send![observer_class, new];

        // 故意泄漏 observer 引用，防止被回收 (它是单例)
        let _ = Box::leak(Box::new(observer));

        // 获取 NotificationCenter
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let notification_center: id = msg_send![workspace, notificationCenter];

        // 添加观察者
        // 使用 NSString 创建通知名称，避免链接错误
        let notification_name =
            NSString::alloc(nil).init_str("NSWorkspaceDidActivateApplicationNotification");

        let _: () = msg_send![notification_center,
            addObserver: observer
            selector: sel!(appActivated:)
            name: notification_name
            object: nil
        ];

        pool.drain();
    }
}

fn resolve_target_input_source(app_handle: &AppHandle, bundle_id: &str) -> Option<String> {
    let state = app_handle.state::<AppState>();
    let manager = state.config.lock().ok()?;
    if !manager.get_config().global_switch {
        return None;
    }

    manager.get_rule(bundle_id)
}

fn apply_input_source_for_bundle_on_main_thread(
    app_handle: &AppHandle,
    bundle_id: String,
    previous_selected_input: Option<String>,
) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel::<Result<Option<String>, String>>();
    let schedule_handle = app_handle.clone();
    let resolve_handle = app_handle.clone();

    schedule_handle
        .run_on_main_thread(move || {
            let result = (|| -> Result<Option<String>, String> {
                let Some(target_input) = resolve_target_input_source(&resolve_handle, &bundle_id)
                else {
                    return Ok(None);
                };

                if previous_selected_input
                    .as_ref()
                    .is_some_and(|current| current == &target_input)
                {
                    return Ok(Some(target_input));
                }

                select_input_source(&target_input).map_err(|e| e.to_string())?;
                Ok(Some(target_input))
            })();
            let _ = tx.send(result);
        })
        .map_err(|e| format!("Failed to schedule main-thread input resolution: {}", e))?;

    rx.recv_timeout(Duration::from_millis(500))
        .map_err(|e| format!("Timed out waiting main-thread input resolution: {}", e))?
}

// Objective-C 回调函数
extern "C" fn app_activated_impl(_this: &Object, _cmd: Sel, notification: id) {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let user_info: id = msg_send![notification, userInfo];
        let key = NSString::alloc(nil).init_str("NSWorkspaceApplicationKey");
        let app: id = msg_send![user_info, objectForKey: key];

        if app != nil {
            let bundle_id: id = msg_send![app, bundleIdentifier];
            let app_name: id = msg_send![app, localizedName];

            let b_id = nsstring_to_owned(bundle_id).unwrap_or_else(|| "unknown".to_string());
            let a_name = nsstring_to_owned(app_name).unwrap_or_else(|| "unknown".to_string());

            // 发送到 Channel
            if let Some(tx) = APP_EVENT_TX.get() {
                let _ = tx.send(AppFocusedEvent {
                    bundle_id: b_id,
                    app_name: a_name,
                });
            }
        }
        pool.drain();
    }
}

unsafe fn nsstring_to_owned(value: id) -> Option<String> {
    if value == nil {
        return None;
    }
    let ptr = cocoa::foundation::NSString::UTF8String(value);
    if ptr.is_null() {
        return None;
    }
    Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
}
