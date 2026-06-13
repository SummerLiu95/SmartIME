#![allow(deprecated)] // Keep using the existing cocoa/objc bridge until the project migrates to objc2.

use crate::error::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use cocoa::appkit::{NSCompositingOperation, NSImage};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use objc::{class, msg_send, sel, sel_impl};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const ICON_SIZE: f64 = 64.0;
const NSPNG_FILE_TYPE: usize = 4;

pub fn app_icon_data_urls(apps: &[(String, PathBuf)]) -> Result<HashMap<String, String>> {
    let mut icons = HashMap::with_capacity(apps.len());

    for (bundle_id, path) in apps {
        if let Some(data_url) = app_icon_data_url(path)? {
            icons.insert(bundle_id.clone(), data_url);
        }
    }

    Ok(icons)
}

#[cfg(target_os = "macos")]
fn app_icon_data_url(app_path: &Path) -> Result<Option<String>> {
    if !app_path.exists() {
        return Ok(None);
    }

    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let result = render_app_icon_png(app_path).map(|maybe_bytes| {
            maybe_bytes.map(|bytes| {
                let encoded = STANDARD.encode(bytes);
                format!("data:image/png;base64,{encoded}")
            })
        });
        let _: () = msg_send![pool, drain];
        result
    }
}

#[cfg(not(target_os = "macos"))]
fn app_icon_data_url(_app_path: &Path) -> Result<Option<String>> {
    Ok(None)
}

#[cfg(target_os = "macos")]
unsafe fn render_app_icon_png(app_path: &Path) -> Result<Option<Vec<u8>>> {
    let Some(path) = app_path.to_str() else {
        return Ok(None);
    };

    let path = autorelease_owned(NSString::alloc(nil).init_str(path));
    let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
    let icon: id = msg_send![workspace, iconForFile: path];
    if icon == nil {
        return Ok(None);
    }

    let size = NSSize::new(ICON_SIZE, ICON_SIZE);
    let rect = NSRect::new(NSPoint::new(0.0, 0.0), size);
    let source_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0));
    let target: id = autorelease_owned(NSImage::alloc(nil).initWithSize_(size));
    if target == nil {
        return Ok(None);
    }

    target.lockFocus();
    icon.drawInRect_fromRect_operation_fraction_(
        rect,
        source_rect,
        NSCompositingOperation::NSCompositeSourceOver,
        1.0,
    );
    let bitmap: id = msg_send![class!(NSBitmapImageRep), alloc];
    let bitmap: id = autorelease_owned(msg_send![bitmap, initWithFocusedViewRect: rect]);
    target.unlockFocus();

    if bitmap == nil {
        return Ok(None);
    }

    let png_data: id = msg_send![bitmap, representationUsingType: NSPNG_FILE_TYPE properties: nil];
    if png_data == nil {
        return Ok(None);
    }

    let bytes: *const u8 = msg_send![png_data, bytes];
    let len: usize = msg_send![png_data, length];
    if bytes.is_null() || len == 0 {
        return Ok(None);
    }

    let png = std::slice::from_raw_parts(bytes, len).to_vec();
    Ok(Some(png))
}

#[cfg(target_os = "macos")]
unsafe fn autorelease_owned(object: id) -> id {
    if object != nil {
        let _: id = msg_send![object, autorelease];
    }
    object
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn missing_app_path_returns_no_icon() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let app_path = std::env::temp_dir().join(format!("smartime-missing-icon-{now}.app"));

        let icons = app_icon_data_urls(&[("com.example.missing".to_string(), app_path)])
            .expect("icon lookup should not fail for missing app");

        assert!(icons.is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn safari_icon_can_be_encoded_when_installed() {
        let safari_path = PathBuf::from("/System/Cryptexes/App/System/Applications/Safari.app");
        if !safari_path.exists() {
            return;
        }

        let icons = app_icon_data_urls(&[("com.apple.Safari".to_string(), safari_path)])
            .expect("encode Safari icon");

        let icon = icons
            .get("com.apple.Safari")
            .expect("Safari icon should resolve");
        assert!(icon.starts_with("data:image/png;base64,"));
    }
}
