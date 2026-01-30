#![allow(deprecated)] // Suppress warnings for deprecated cocoa APIs

use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use once_cell::sync::OnceCell;
use std::ffi::CStr;
use std::sync::mpsc::Sender;
use std::sync::Once;
use tauri::{AppHandle, Emitter};

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
        while let Ok(event) = rx.recv() {
            // println!("App focused: {:?}", event);
            // 发送事件到前端
            if let Err(e) = app_handle.emit("app_focused", &event) {
                eprintln!("Failed to emit app_focused event: {}", e);
            }
        }
    });

    unsafe {
        // 注册 Objective-C 类
        REGISTER_OBSERVER_CLASS.call_once(|| {
            let superclass = class!(NSObject);
            let mut decl = ClassDecl::new("RustAppObserver", superclass).expect("Failed to declare class");

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
        let notification_name = NSString::alloc(nil).init_str("NSWorkspaceDidActivateApplicationNotification");

        let _: () = msg_send![notification_center, 
            addObserver: observer
            selector: sel!(appActivated:)
            name: notification_name
            object: nil
        ];

        pool.drain();
    }
}

// Objective-C 回调函数
extern "C" fn app_activated_impl(_this: &Object, _cmd: Sel, notification: id) {
    unsafe {
        let user_info: id = msg_send![notification, userInfo];
        let key = NSString::alloc(nil).init_str("NSWorkspaceApplicationKey");
        let app: id = msg_send![user_info, objectForKey: key];
        
        if app != nil {
            let bundle_id: id = msg_send![app, bundleIdentifier];
            let app_name: id = msg_send![app, localizedName];

            let b_id = if bundle_id != nil {
                let bytes = CStr::from_ptr(cocoa::foundation::NSString::UTF8String(bundle_id));
                bytes.to_string_lossy().into_owned()
            } else {
                "unknown".to_string()
            };
            
            let a_name = if app_name != nil {
                let bytes = CStr::from_ptr(cocoa::foundation::NSString::UTF8String(app_name));
                bytes.to_string_lossy().into_owned()
            } else {
                "unknown".to_string()
            };

            // 发送到 Channel
            if let Some(tx) = APP_EVENT_TX.get() {
                let _ = tx.send(AppFocusedEvent {
                    bundle_id: b_id,
                    app_name: a_name,
                });
            }
        }
    }
}
