use crate::error::{AppError, Result};
use core_foundation::array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation::base::{CFRelease, CFTypeRef, TCFType};
use core_foundation::dictionary::CFDictionaryRef;
use core_foundation::string::{CFString, CFStringRef};
use serde::{Deserialize, Serialize};
use std::ffi::c_void;
use std::ptr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSource {
    pub id: String,
    pub name: String,
    pub category: String,
}

#[repr(C)]
pub struct __TISInputSource(c_void);
pub type TISInputSourceRef = *const __TISInputSource;
pub type OSStatus = i32;

#[link(name = "Carbon", kind = "framework")]
extern "C" {
    pub static kTISPropertyInputSourceID: CFStringRef;
    pub static kTISPropertyLocalizedName: CFStringRef;
    pub static kTISPropertyInputSourceCategory: CFStringRef;
    pub static kTISCategoryKeyboardInputSource: CFStringRef;

    pub fn TISCreateInputSourceList(
        properties: CFDictionaryRef,
        includeAllInstalled: bool,
    ) -> CFArrayRef;

    pub fn TISSelectInputSource(inputSource: TISInputSourceRef) -> OSStatus;
    pub fn TISGetInputSourceProperty(
        inputSource: TISInputSourceRef,
        propertyKey: CFStringRef,
    ) -> CFTypeRef;
}

/// 获取当前系统所有已启用的键盘输入法
pub fn get_system_input_sources() -> Result<Vec<InputSource>> {
    let mut sources = Vec::new();

    unsafe {
        let source_list = TISCreateInputSourceList(ptr::null(), false);
        if source_list.is_null() {
            return Err(AppError::InputSource(
                "Failed to create input source list".to_string(),
            ));
        }

        let count = CFArrayGetCount(source_list);
        for i in 0..count {
            let source = CFArrayGetValueAtIndex(source_list, i) as TISInputSourceRef;
            if let Some(s) = parse_input_source(source) {
                // 仅保留键盘输入法，过滤掉其他类型的 Input Source
                // if s.category == "TISCategoryKeyboardInputSource" {
                sources.push(s);
                // }
            }
        }

        CFRelease(source_list as CFTypeRef);
    }

    Ok(sources)
}

/// 切换到指定的输入法 ID
pub fn select_input_source(source_id: &str) -> Result<()> {
    unsafe {
        let source_list = TISCreateInputSourceList(ptr::null(), true);
        if source_list.is_null() {
            return Err(AppError::InputSource(
                "Failed to create input source list".to_string(),
            ));
        }

        let count = CFArrayGetCount(source_list);
        let target_id = CFString::new(source_id);
        let mut found_source: TISInputSourceRef = ptr::null();

        for i in 0..count {
            let source = CFArrayGetValueAtIndex(source_list, i) as TISInputSourceRef;
            let id_ptr = TISGetInputSourceProperty(source, kTISPropertyInputSourceID);
            if id_ptr.is_null() {
                continue;
            }

            let id_cfstr = CFString::wrap_under_get_rule(id_ptr as CFStringRef);
            if id_cfstr == target_id {
                found_source = source;
                break;
            }
        }

        let result = if !found_source.is_null() {
            let status = TISSelectInputSource(found_source);
            if status == 0 {
                Ok(())
            } else {
                Err(AppError::InputSource(format!(
                    "TISSelectInputSource failed with status: {}",
                    status
                )))
            }
        } else {
            Err(AppError::InputSource(format!(
                "Input source not found: {}",
                source_id
            )))
        };

        CFRelease(source_list as CFTypeRef);
        result
    }
}

unsafe fn parse_input_source(source: TISInputSourceRef) -> Option<InputSource> {
    let id_ptr = TISGetInputSourceProperty(source, kTISPropertyInputSourceID);
    let name_ptr = TISGetInputSourceProperty(source, kTISPropertyLocalizedName);
    let cat_ptr = TISGetInputSourceProperty(source, kTISPropertyInputSourceCategory);

    if id_ptr.is_null() || name_ptr.is_null() || cat_ptr.is_null() {
        return None;
    }

    let id = CFString::wrap_under_get_rule(id_ptr as CFStringRef).to_string();
    let name = CFString::wrap_under_get_rule(name_ptr as CFStringRef).to_string();
    let category = CFString::wrap_under_get_rule(cat_ptr as CFStringRef).to_string();

    Some(InputSource { id, name, category })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_input_sources() {
        // 仅在 macOS 下运行此测试
        if cfg!(target_os = "macos") {
            let sources = get_system_input_sources().unwrap();
            assert!(!sources.is_empty());
            for source in sources {
                println!("Found source: {:?}", source);
            }
        }
    }
}
