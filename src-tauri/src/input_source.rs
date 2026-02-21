use crate::error::{AppError, Result};
use core_foundation::array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation::base::{CFRelease, CFTypeRef, TCFType};
use core_foundation::boolean::{CFBoolean, CFBooleanRef};
use core_foundation::dictionary::CFDictionaryRef;
use core_foundation::number::{CFNumber, CFNumberRef};
use core_foundation::string::{CFString, CFStringRef};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ffi::c_void;
use std::io::Cursor;
use std::process::Command;
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
    pub static kTISPropertyInputSourceType: CFStringRef;
    pub static kTISPropertyBundleID: CFStringRef;
    pub static kTISPropertyInputModeID: CFStringRef;
    pub static kTISPropertyInputSourceIsEnabled: CFStringRef;
    pub static kTISPropertyInputSourceIsSelectCapable: CFStringRef;
    pub static kTISCategoryKeyboardInputSource: CFStringRef;
    pub static kTISTypeKeyboardInputMode: CFStringRef;
    pub static kTISTypeKeyboardLayout: CFStringRef;
    pub static kTISTypeKeyboardInputMethodWithoutModes: CFStringRef;

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
    let mut seen_ids = HashSet::new();
    let menu_enabled = load_menu_enabled_sources();

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
            if !is_selectable_keyboard_source(source) {
                continue;
            }

            if let Some(parsed) = parse_input_source(source) {
                if let Some(filter) = &menu_enabled {
                    if !filter.matches(
                        &parsed.source,
                        &parsed.source_type,
                        parsed.keyboard_layout_id,
                        parsed.bundle_id.as_deref(),
                        parsed.input_mode_id.as_deref(),
                    ) {
                        continue;
                    }
                }

                if seen_ids.insert(parsed.source.id.clone()) {
                    sources.push(parsed.source);
                }
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

#[derive(Debug)]
struct ParsedInputSource {
    source: InputSource,
    source_type: String,
    keyboard_layout_id: Option<i64>,
    bundle_id: Option<String>,
    input_mode_id: Option<String>,
}

#[derive(Debug, Default)]
struct MenuEnabledSources {
    input_mode_ids: HashSet<String>,
    input_mode_parent_bundles: std::collections::HashMap<String, String>,
    keyboard_layout_ids: HashSet<i64>,
    keyboard_layout_names: HashSet<String>,
    keyboard_input_method_bundle_ids: HashSet<String>,
}

impl MenuEnabledSources {
    fn is_empty(&self) -> bool {
        self.input_mode_ids.is_empty()
            && self.input_mode_parent_bundles.is_empty()
            && self.keyboard_layout_ids.is_empty()
            && self.keyboard_layout_names.is_empty()
            && self.keyboard_input_method_bundle_ids.is_empty()
    }

    fn matches(
        &self,
        source: &InputSource,
        source_type: &str,
        keyboard_layout_id: Option<i64>,
        bundle_id: Option<&str>,
        input_mode_id: Option<&str>,
    ) -> bool {
        if source_type == "TISTypeKeyboardInputMode" {
            let mode_key = if self.input_mode_ids.contains(&source.id) {
                Some(source.id.as_str())
            } else if let Some(mode_id) = input_mode_id {
                if self.input_mode_ids.contains(mode_id) {
                    Some(mode_id)
                } else {
                    None
                }
            } else {
                None
            };

            let Some(mode_key) = mode_key else {
                return false;
            };

            if let Some(parent_bundle) = self.input_mode_parent_bundles.get(mode_key) {
                // Keep only sources that belong to the same input-method bundle
                // selected in HIToolbox for this mode.
                let Some(source_bundle) = bundle_id else {
                    return false;
                };
                if source_bundle != parent_bundle {
                    return false;
                }

                if !self.keyboard_input_method_bundle_ids.is_empty()
                    && !self
                        .keyboard_input_method_bundle_ids
                        .contains(parent_bundle)
                {
                    return false;
                }
            }

            return true;
        }

        if source_type == "TISTypeKeyboardInputMethodWithoutModes" {
            if self.keyboard_input_method_bundle_ids.is_empty() {
                return true;
            }

            if self.keyboard_input_method_bundle_ids.contains(&source.id) {
                return true;
            }

            if let Some(bundle) = bundle_id {
                return self.keyboard_input_method_bundle_ids.contains(bundle);
            }

            return false;
        }

        if source_type != "TISTypeKeyboardLayout" {
            return false;
        }

        let has_layout_filters =
            !self.keyboard_layout_ids.is_empty() || !self.keyboard_layout_names.is_empty();
        if !has_layout_filters {
            // Fail-open for layouts only if we could not parse layout entries
            // from HIToolbox preferences on this machine.
            return true;
        }

        if let Some(layout_id) = keyboard_layout_id {
            if self.keyboard_layout_ids.contains(&layout_id) {
                return true;
            }
        }

        self.keyboard_layout_names.contains(&source.name)
    }
}

fn load_menu_enabled_sources() -> Option<MenuEnabledSources> {
    let output = Command::new("defaults")
        .args(["export", "com.apple.HIToolbox", "-"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let plist = plist::Value::from_reader(Cursor::new(output.stdout)).ok()?;
    let root = plist.as_dictionary()?;
    let enabled = root.get("AppleEnabledInputSources")?.as_array()?;

    let mut sources = MenuEnabledSources::default();

    for item in enabled {
        let Some(dict) = item.as_dictionary() else {
            continue;
        };

        let Some(kind) = dict
            .get("InputSourceKind")
            .and_then(|value| value.as_string())
        else {
            continue;
        };

        match kind {
            "Input Mode" => {
                if let Some(id) = dict.get("Input Mode").and_then(|value| value.as_string()) {
                    sources.input_mode_ids.insert(id.to_string());
                    if let Some(bundle_id) =
                        dict.get("Bundle ID").and_then(|value| value.as_string())
                    {
                        sources
                            .input_mode_parent_bundles
                            .insert(id.to_string(), bundle_id.to_string());
                    }
                }
            }
            "Keyboard Layout" => {
                if let Some(id) = dict
                    .get("KeyboardLayout ID")
                    .and_then(|value| value.as_signed_integer())
                {
                    sources.keyboard_layout_ids.insert(id);
                }
                if let Some(name) = dict
                    .get("KeyboardLayout Name")
                    .and_then(|value| value.as_string())
                {
                    sources.keyboard_layout_names.insert(name.to_string());
                }
            }
            "Keyboard Input Method" => {
                if let Some(bundle_id) = dict.get("Bundle ID").and_then(|value| value.as_string()) {
                    sources
                        .keyboard_input_method_bundle_ids
                        .insert(bundle_id.to_string());
                }
            }
            _ => {}
        }
    }

    if sources.is_empty() {
        None
    } else {
        Some(sources)
    }
}

unsafe fn parse_input_source(source: TISInputSourceRef) -> Option<ParsedInputSource> {
    let id_ptr = TISGetInputSourceProperty(source, kTISPropertyInputSourceID);
    let name_ptr = TISGetInputSourceProperty(source, kTISPropertyLocalizedName);
    let cat_ptr = TISGetInputSourceProperty(source, kTISPropertyInputSourceCategory);
    let source_type_ptr = TISGetInputSourceProperty(source, kTISPropertyInputSourceType);
    let bundle_id_ptr = TISGetInputSourceProperty(source, kTISPropertyBundleID);
    let input_mode_id_ptr = TISGetInputSourceProperty(source, kTISPropertyInputModeID);

    if id_ptr.is_null() || name_ptr.is_null() || cat_ptr.is_null() || source_type_ptr.is_null() {
        return None;
    }

    let id = CFString::wrap_under_get_rule(id_ptr as CFStringRef).to_string();
    let name = CFString::wrap_under_get_rule(name_ptr as CFStringRef).to_string();
    let category = CFString::wrap_under_get_rule(cat_ptr as CFStringRef).to_string();
    let source_type = CFString::wrap_under_get_rule(source_type_ptr as CFStringRef).to_string();
    let keyboard_layout_key = CFString::new("KeyboardLayout ID");
    let keyboard_layout_ptr =
        TISGetInputSourceProperty(source, keyboard_layout_key.as_concrete_TypeRef());
    let keyboard_layout_id = if keyboard_layout_ptr.is_null() {
        None
    } else {
        CFNumber::wrap_under_get_rule(keyboard_layout_ptr as CFNumberRef).to_i64()
    };
    let bundle_id = if bundle_id_ptr.is_null() {
        None
    } else {
        Some(CFString::wrap_under_get_rule(bundle_id_ptr as CFStringRef).to_string())
    };
    let input_mode_id = if input_mode_id_ptr.is_null() {
        None
    } else {
        Some(CFString::wrap_under_get_rule(input_mode_id_ptr as CFStringRef).to_string())
    };

    Some(ParsedInputSource {
        source: InputSource { id, name, category },
        source_type,
        keyboard_layout_id,
        bundle_id,
        input_mode_id,
    })
}

unsafe fn is_selectable_keyboard_source(source: TISInputSourceRef) -> bool {
    let category_ptr = TISGetInputSourceProperty(source, kTISPropertyInputSourceCategory);
    let source_type_ptr = TISGetInputSourceProperty(source, kTISPropertyInputSourceType);
    let is_enabled_ptr = TISGetInputSourceProperty(source, kTISPropertyInputSourceIsEnabled);
    let is_select_capable_ptr =
        TISGetInputSourceProperty(source, kTISPropertyInputSourceIsSelectCapable);

    if category_ptr.is_null()
        || source_type_ptr.is_null()
        || is_enabled_ptr.is_null()
        || is_select_capable_ptr.is_null()
    {
        return false;
    }

    let category = CFString::wrap_under_get_rule(category_ptr as CFStringRef).to_string();
    let source_type = CFString::wrap_under_get_rule(source_type_ptr as CFStringRef).to_string();
    let keyboard_category =
        CFString::wrap_under_get_rule(kTISCategoryKeyboardInputSource).to_string();
    let keyboard_layout_type = CFString::wrap_under_get_rule(kTISTypeKeyboardLayout).to_string();
    let keyboard_input_mode_type =
        CFString::wrap_under_get_rule(kTISTypeKeyboardInputMode).to_string();
    let keyboard_input_method_without_modes =
        CFString::wrap_under_get_rule(kTISTypeKeyboardInputMethodWithoutModes).to_string();

    let is_enabled = CFBoolean::wrap_under_get_rule(is_enabled_ptr as CFBooleanRef).into();
    let is_select_capable =
        CFBoolean::wrap_under_get_rule(is_select_capable_ptr as CFBooleanRef).into();

    category == keyboard_category
        && (source_type == keyboard_layout_type
            || source_type == keyboard_input_mode_type
            || source_type == keyboard_input_method_without_modes)
        && is_enabled
        && is_select_capable
}

#[cfg(test)]
fn should_include_source(
    category: &str,
    source_type: &str,
    keyboard_category: &str,
    keyboard_layout_type: &str,
    keyboard_input_mode_type: &str,
    is_enabled: bool,
    is_select_capable: bool,
) -> bool {
    category == keyboard_category
        && (source_type == keyboard_layout_type || source_type == keyboard_input_mode_type)
        && is_enabled
        && is_select_capable
}

#[cfg(test)]
fn should_include_menu_enabled_source(
    source_id: &str,
    source_name: &str,
    source_type: &str,
    source_layout_id: Option<i64>,
    source_bundle_id: Option<&str>,
    source_input_mode_id: Option<&str>,
    input_mode_ids: &[&str],
    input_mode_parent_pairs: &[(&str, &str)],
    keyboard_layout_ids: &[i64],
    keyboard_layout_names: &[&str],
    keyboard_input_method_bundle_ids: &[&str],
) -> bool {
    let filter = MenuEnabledSources {
        input_mode_ids: input_mode_ids.iter().map(|id| id.to_string()).collect(),
        input_mode_parent_bundles: input_mode_parent_pairs
            .iter()
            .map(|(mode_id, bundle_id)| (mode_id.to_string(), bundle_id.to_string()))
            .collect(),
        keyboard_layout_ids: keyboard_layout_ids.iter().copied().collect(),
        keyboard_layout_names: keyboard_layout_names
            .iter()
            .map(|name| name.to_string())
            .collect(),
        keyboard_input_method_bundle_ids: keyboard_input_method_bundle_ids
            .iter()
            .map(|bundle_id| bundle_id.to_string())
            .collect(),
    };

    let source = InputSource {
        id: source_id.to_string(),
        name: source_name.to_string(),
        category: "TISCategoryKeyboardInputSource".to_string(),
    };

    filter.matches(
        &source,
        source_type,
        source_layout_id,
        source_bundle_id,
        source_input_mode_id,
    )
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

    #[test]
    fn test_should_include_source_filters_non_selectable_sources() {
        let keyboard_category = "TISCategoryKeyboardInputSource";
        let keyboard_layout = "TISTypeKeyboardLayout";
        let keyboard_input_mode = "TISTypeKeyboardInputMode";

        assert!(should_include_source(
            keyboard_category,
            keyboard_layout,
            keyboard_category,
            keyboard_layout,
            keyboard_input_mode,
            true,
            true,
        ));

        assert!(should_include_source(
            keyboard_category,
            keyboard_input_mode,
            keyboard_category,
            keyboard_layout,
            keyboard_input_mode,
            true,
            true,
        ));

        assert!(!should_include_source(
            "TISCategoryPaletteInputSource",
            keyboard_input_mode,
            keyboard_category,
            keyboard_layout,
            keyboard_input_mode,
            true,
            true,
        ));

        assert!(!should_include_source(
            keyboard_category,
            "TISTypeInk",
            keyboard_category,
            keyboard_layout,
            keyboard_input_mode,
            true,
            true,
        ));

        assert!(!should_include_source(
            keyboard_category,
            keyboard_layout,
            keyboard_category,
            keyboard_layout,
            keyboard_input_mode,
            false,
            true,
        ));

        assert!(!should_include_source(
            keyboard_category,
            keyboard_layout,
            keyboard_category,
            keyboard_layout,
            keyboard_input_mode,
            true,
            false,
        ));
    }

    #[test]
    fn test_should_include_menu_enabled_source_filters_to_menu_items() {
        assert!(should_include_menu_enabled_source(
            "com.apple.inputmethod.SCIM.ITABC",
            "Pinyin - Simplified",
            "TISTypeKeyboardInputMode",
            None,
            Some("com.apple.inputmethod.SCIM"),
            Some("com.apple.inputmethod.SCIM.ITABC"),
            &["com.apple.inputmethod.SCIM.ITABC"],
            &[(
                "com.apple.inputmethod.SCIM.ITABC",
                "com.apple.inputmethod.SCIM",
            )],
            &[],
            &[],
            &["com.apple.inputmethod.SCIM"],
        ));

        assert!(should_include_menu_enabled_source(
            "com.apple.keylayout.ABC",
            "ABC",
            "TISTypeKeyboardLayout",
            Some(252),
            None,
            None,
            &["com.apple.inputmethod.SCIM.ITABC"],
            &[(
                "com.apple.inputmethod.SCIM.ITABC",
                "com.apple.inputmethod.SCIM",
            )],
            &[252],
            &[],
            &["com.apple.inputmethod.SCIM"],
        ));

        assert!(!should_include_menu_enabled_source(
            "com.apple.inputmethod.Kotoeri.Japanese",
            "Hiragana",
            "TISTypeKeyboardInputMode",
            None,
            Some("com.apple.inputmethod.Kotoeri"),
            Some("com.apple.inputmethod.Japanese"),
            &["com.apple.inputmethod.SCIM.ITABC"],
            &[(
                "com.apple.inputmethod.SCIM.ITABC",
                "com.apple.inputmethod.SCIM",
            )],
            &[],
            &[],
            &["com.apple.inputmethod.SCIM"],
        ));

        assert!(should_include_menu_enabled_source(
            "com.apple.inputmethod.Kotoeri.KanaTyping.Hiragana",
            "Hiragana",
            "TISTypeKeyboardInputMode",
            None,
            Some("com.apple.inputmethod.Kotoeri.KanaTyping"),
            Some("com.apple.inputmethod.Japanese"),
            &["com.apple.inputmethod.Japanese"],
            &[(
                "com.apple.inputmethod.Japanese",
                "com.apple.inputmethod.Kotoeri.KanaTyping",
            )],
            &[],
            &[],
            &["com.apple.inputmethod.Kotoeri.KanaTyping"],
        ));

        assert!(!should_include_menu_enabled_source(
            "com.apple.inputmethod.Kotoeri.Japanese",
            "Hiragana",
            "TISTypeKeyboardInputMode",
            None,
            Some("com.apple.inputmethod.Kotoeri"),
            Some("com.apple.inputmethod.Japanese"),
            &["com.apple.inputmethod.Japanese"],
            &[(
                "com.apple.inputmethod.Japanese",
                "com.apple.inputmethod.Kotoeri.KanaTyping",
            )],
            &[],
            &[],
            &["com.apple.inputmethod.Kotoeri.KanaTyping"],
        ));

        assert!(should_include_menu_enabled_source(
            "com.thirdparty.ime",
            "Third-Party IME",
            "TISTypeKeyboardInputMethodWithoutModes",
            None,
            Some("com.thirdparty.ime"),
            None,
            &[],
            &[],
            &[],
            &[],
            &["com.thirdparty.ime"],
        ));

        assert!(!should_include_menu_enabled_source(
            "com.thirdparty.ime.pinyin",
            "Third-Party IME Pinyin",
            "TISTypeKeyboardInputMode",
            None,
            Some("com.thirdparty.ime"),
            Some("com.thirdparty.ime.pinyin"),
            &[],
            &[],
            &[],
            &[],
            &["com.thirdparty.ime"],
        ));

        assert!(!should_include_menu_enabled_source(
            "com.apple.inputmethod.Korean.2SetKorean",
            "2-Set Korean",
            "TISTypeKeyboardInputMode",
            None,
            Some("com.apple.inputmethod.Korean"),
            Some("com.apple.inputmethod.Korean.2SetKorean"),
            &[
                "com.apple.inputmethod.SCIM.ITABC",
                "com.apple.inputmethod.Korean.2SetKorean",
            ],
            &[
                (
                    "com.apple.inputmethod.SCIM.ITABC",
                    "com.apple.inputmethod.SCIM",
                ),
                (
                    "com.apple.inputmethod.Korean.2SetKorean",
                    "com.apple.inputmethod.Korean",
                ),
            ],
            &[252],
            &["ABC"],
            &["com.apple.inputmethod.SCIM"],
        ));
    }
}
