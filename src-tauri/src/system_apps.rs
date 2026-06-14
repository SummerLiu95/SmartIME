#![allow(deprecated)] // Keep using the existing cocoa/objc bridge until the project migrates to objc2.

use crate::error::Result;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSAutoreleasePool, NSString};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[cfg(target_os = "macos")]
use std::ffi::CStr;
use std::fs;
#[cfg(target_os = "macos")]
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

const INPUT_CAPABLE_SYSTEM_BUNDLE_IDS: &[&str] = &[
    "com.apple.Safari",
    "com.apple.Notes",
    "com.apple.reminders",
    "com.apple.TextEdit",
    "com.apple.mail",
    "com.apple.MobileSMS",
    "com.apple.iCal",
    "com.apple.AddressBook",
    "com.apple.Maps",
    "com.apple.freeform",
    "com.apple.Photos",
    "com.apple.Stickies",
    "com.apple.shortcuts",
    "com.apple.Dictionary",
    "com.apple.Passwords",
    "com.apple.AppStore",
    "com.apple.Terminal",
    "com.apple.finder",
    "com.apple.Preview",
    "com.apple.iBooksX",
    "com.apple.journal",
    "com.apple.iWork.Pages",
    "com.apple.iWork.Numbers",
    "com.apple.iWork.Keynote",
];

const ZH_HANS_SYSTEM_APP_NAMES: &[(&str, &str)] = &[
    ("com.apple.Safari", "Safari 浏览器"),
    ("com.apple.Notes", "备忘录"),
    ("com.apple.reminders", "提醒事项"),
    ("com.apple.TextEdit", "文本编辑"),
    ("com.apple.mail", "邮件"),
    ("com.apple.MobileSMS", "信息"),
    ("com.apple.iCal", "日历"),
    ("com.apple.AddressBook", "通讯录"),
    ("com.apple.Maps", "地图"),
    ("com.apple.freeform", "无边记"),
    ("com.apple.Photos", "照片"),
    ("com.apple.Stickies", "便笺"),
    ("com.apple.shortcuts", "快捷指令"),
    ("com.apple.Dictionary", "词典"),
    ("com.apple.Passwords", "密码"),
    ("com.apple.AppStore", "App Store"),
    ("com.apple.Terminal", "终端"),
    ("com.apple.finder", "访达"),
    ("com.apple.Preview", "预览"),
    ("com.apple.iBooksX", "图书"),
    ("com.apple.journal", "日记"),
    ("com.apple.iWork.Pages", "Pages 文稿"),
    ("com.apple.iWork.Numbers", "Numbers 表格"),
    ("com.apple.iWork.Keynote", "Keynote 讲演"),
];

const LOCALIZED_INFO_PLIST_DIRS: &[&str] = &[
    "zh-Hans.lproj",
    "zh_CN.lproj",
    "zh-Hans-CN.lproj",
    "zh.lproj",
    "zh-Hant.lproj",
    "zh_TW.lproj",
    "Base.lproj",
    "en.lproj",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemApp {
    pub name: String,
    pub bundle_id: String,
    pub path: PathBuf,
}

#[derive(Deserialize)]
struct AppPlist {
    #[serde(rename = "CFBundleDisplayName")]
    display_name: Option<String>,
    #[serde(rename = "CFBundleName")]
    bundle_name: Option<String>,
    #[serde(rename = "CFBundleIdentifier")]
    bundle_identifier: Option<String>,
}

#[derive(Deserialize)]
struct LocalizedAppPlist {
    #[serde(rename = "CFBundleDisplayName")]
    display_name: Option<String>,
    #[serde(rename = "CFBundleName")]
    bundle_name: Option<String>,
}

#[derive(Clone, Copy)]
enum ScanRootKind {
    User,
    System,
}

struct ScanRoot {
    path: PathBuf,
    kind: ScanRootKind,
}

fn is_app_bundle(entry: &DirEntry) -> bool {
    entry.file_type().is_dir() && entry.path().extension().map_or(false, |ext| ext == "app")
}

fn parse_app_plist(app_path: &Path) -> Option<SystemApp> {
    let plist_path = app_path.join("Contents/Info.plist");
    if !plist_path.exists() {
        return None;
    }

    let app_plist: AppPlist = plist::from_file(&plist_path).ok()?;

    let bundle_id = app_plist.bundle_identifier?.trim().to_string();
    // Ignore empty bundle IDs
    if bundle_id.is_empty() {
        return None;
    }

    let system_name = system_display_name(app_path);
    let localized_name = localized_app_name(app_path, &bundle_id);
    let name = preferred_app_name(app_path, system_name, localized_name)
        .or_else(|| clean_app_name(app_plist.display_name))
        .or_else(|| clean_app_name(app_plist.bundle_name))
        .or_else(|| path_stem_app_name(app_path))?;

    Some(SystemApp {
        name,
        bundle_id,
        path: app_path.to_path_buf(),
    })
}

#[cfg(target_os = "macos")]
fn system_display_name(app_path: &Path) -> Option<String> {
    let path = app_path.to_str()?;

    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let result = system_display_name_with_pool(path);
        let _: () = msg_send![pool, drain];
        result
    }
}

#[cfg(target_os = "macos")]
unsafe fn system_display_name_with_pool(path: &str) -> Option<String> {
    let path = NSString::alloc(nil).init_str(path);
    let _: id = msg_send![path, autorelease];
    let file_manager: id = msg_send![class!(NSFileManager), defaultManager];
    if file_manager == nil {
        return None;
    }

    let display_name: id = msg_send![file_manager, displayNameAtPath: path];
    ns_string_to_string(display_name).and_then(|name| clean_app_name(Some(name)))
}

#[cfg(target_os = "macos")]
unsafe fn ns_string_to_string(value: id) -> Option<String> {
    if value == nil {
        return None;
    }

    let utf8: *const c_char = msg_send![value, UTF8String];
    if utf8.is_null() {
        return None;
    }

    CStr::from_ptr(utf8).to_str().ok().map(str::to_string)
}

#[cfg(not(target_os = "macos"))]
fn system_display_name(_app_path: &Path) -> Option<String> {
    None
}

fn localized_app_name(app_path: &Path, bundle_id: &str) -> Option<String> {
    localized_name_from_info_plist_strings(app_path)
        .or_else(|| zh_hans_system_app_name(bundle_id).map(str::to_string))
}

fn preferred_app_name(
    app_path: &Path,
    system_name: Option<String>,
    localized_name: Option<String>,
) -> Option<String> {
    match (system_name, localized_name) {
        (Some(system_name), Some(localized_name)) if is_plain_path_stem(app_path, &system_name) => {
            Some(localized_name)
        }
        (Some(system_name), _) => Some(system_name),
        (None, localized_name) => localized_name,
    }
}

fn localized_name_from_info_plist_strings(app_path: &Path) -> Option<String> {
    for dir in LOCALIZED_INFO_PLIST_DIRS {
        let path = app_path
            .join("Contents")
            .join("Resources")
            .join(dir)
            .join("InfoPlist.strings");
        if !path.exists() {
            continue;
        }

        if let Some(name) = localized_name_from_info_plist_strings_file(&path) {
            return Some(name);
        }
    }

    None
}

fn localized_name_from_info_plist_strings_file(path: &Path) -> Option<String> {
    if let Ok(localized) = plist::from_file::<_, LocalizedAppPlist>(path) {
        if let Some(name) = clean_app_name(localized.display_name.or(localized.bundle_name)) {
            return Some(name);
        }
    }

    let content = read_strings_file(path)?;
    parse_strings_value(&content, "CFBundleDisplayName")
        .or_else(|| parse_strings_value(&content, "CFBundleName"))
        .and_then(|name| clean_app_name(Some(name)))
}

fn read_strings_file(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    if bytes.starts_with(&[0xff, 0xfe]) {
        return decode_utf16_bytes(&bytes[2..], Utf16Endian::Little);
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        return decode_utf16_bytes(&bytes[2..], Utf16Endian::Big);
    }
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return String::from_utf8(bytes[3..].to_vec()).ok();
    }

    String::from_utf8(bytes).ok()
}

enum Utf16Endian {
    Little,
    Big,
}

fn decode_utf16_bytes(bytes: &[u8], endian: Utf16Endian) -> Option<String> {
    let mut chunks = bytes.chunks_exact(2);
    let units = chunks
        .by_ref()
        .map(|chunk| match endian {
            Utf16Endian::Little => u16::from_le_bytes([chunk[0], chunk[1]]),
            Utf16Endian::Big => u16::from_be_bytes([chunk[0], chunk[1]]),
        })
        .collect::<Vec<_>>();
    if !chunks.remainder().is_empty() {
        return None;
    }

    String::from_utf16(&units).ok()
}

fn parse_strings_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with("/*") {
            continue;
        }

        let Some((raw_key, raw_value)) = line.split_once('=') else {
            continue;
        };
        if parse_strings_token(raw_key.trim()).as_deref() != Some(key) {
            continue;
        }

        let raw_value = raw_value.trim().trim_end_matches(';').trim();
        if let Some(value) = parse_strings_token(raw_value) {
            return Some(value);
        }
    }

    None
}

fn parse_strings_token(token: &str) -> Option<String> {
    let token = token.trim();
    let token = token
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(token);
    let value = token
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
        .replace("\\n", "\n");

    (!value.trim().is_empty()).then_some(value)
}

fn zh_hans_system_app_name(bundle_id: &str) -> Option<&'static str> {
    ZH_HANS_SYSTEM_APP_NAMES
        .iter()
        .find_map(|(id, name)| (*id == bundle_id).then_some(*name))
}

fn clean_app_name(name: Option<String>) -> Option<String> {
    let mut name = name?.trim().to_string();
    if name.is_empty() {
        return None;
    }

    if let Some(stripped) = name.strip_suffix(".app") {
        name = stripped.trim().to_string();
    }

    (!name.is_empty()).then_some(name)
}

fn path_stem_app_name(app_path: &Path) -> Option<String> {
    clean_app_name(
        app_path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string()),
    )
}

fn is_plain_path_stem(app_path: &Path, name: &str) -> bool {
    path_stem_app_name(app_path).is_some_and(|stem| stem.eq_ignore_ascii_case(name))
}

fn is_input_capable_system_app(bundle_id: &str) -> bool {
    INPUT_CAPABLE_SYSTEM_BUNDLE_IDS.contains(&bundle_id)
}

fn should_include_app(app: &SystemApp, root_kind: ScanRootKind) -> bool {
    if matches!(root_kind, ScanRootKind::System) {
        return is_input_capable_system_app(&app.bundle_id);
    }

    true
}

pub fn get_installed_apps() -> Result<Vec<SystemApp>> {
    scan_apps_in_roots(&default_scan_roots())
}

fn default_scan_roots() -> Vec<ScanRoot> {
    let mut scan_roots = Vec::with_capacity(5);
    scan_roots.push(ScanRoot {
        path: PathBuf::from("/Applications"),
        kind: ScanRootKind::User,
    });

    if let Some(home) = dirs::home_dir() {
        scan_roots.push(ScanRoot {
            path: home.join("Applications"),
            kind: ScanRootKind::User,
        });
    }

    scan_roots.push(ScanRoot {
        path: PathBuf::from("/System/Applications"),
        kind: ScanRootKind::System,
    });
    scan_roots.push(ScanRoot {
        path: PathBuf::from("/System/Cryptexes/App/System/Applications"),
        kind: ScanRootKind::System,
    });
    scan_roots.push(ScanRoot {
        path: PathBuf::from("/System/Library/CoreServices"),
        kind: ScanRootKind::System,
    });

    scan_roots
}

fn scan_apps_in_roots(scan_roots: &[ScanRoot]) -> Result<Vec<SystemApp>> {
    let mut apps = Vec::new();
    let mut seen_bundle_ids = HashSet::new();

    for root in scan_roots {
        let root_path = &root.path;
        if !root_path.exists() {
            continue;
        }

        // Use WalkDir to traverse
        // max_depth 3 is usually enough (e.g. /Applications/Utilities/Terminal.app)
        let mut it = WalkDir::new(root_path)
            .min_depth(1)
            .max_depth(3)
            .into_iter();

        loop {
            let entry = match it.next() {
                None => break,
                Some(Err(_)) => continue,
                Some(Ok(entry)) => entry,
            };

            if is_app_bundle(&entry) {
                // Try to parse Info.plist
                if let Some(app_info) = parse_app_plist(entry.path()) {
                    if should_include_app(&app_info, root.kind)
                        && seen_bundle_ids.insert(app_info.bundle_id.clone())
                    {
                        apps.push(app_info);
                    }
                }

                // Don't look inside the .app
                it.skip_current_dir();
                continue;
            }
        }
    }

    // Sort by name for better UX
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(apps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_app(root: &Path, app_name: &str, bundle_id: &str) -> PathBuf {
        let app_path = root.join(format!("{app_name}.app"));
        let contents_path = app_path.join("Contents");
        fs::create_dir_all(&contents_path).expect("create test app contents");
        fs::write(
            contents_path.join("Info.plist"),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>{app_name}</string>
  <key>CFBundleIdentifier</key>
  <string>{bundle_id}</string>
</dict>
</plist>"#
            ),
        )
        .expect("write test Info.plist");
        app_path
    }

    fn create_localized_info_plist(app_path: &Path, display_name: &str) {
        let localized_path = app_path
            .join("Contents")
            .join("Resources")
            .join("zh_CN.lproj");
        fs::create_dir_all(&localized_path).expect("create localized resources");
        fs::write(
            localized_path.join("InfoPlist.strings"),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDisplayName</key>
  <string>{display_name}</string>
</dict>
</plist>"#
            ),
        )
        .expect("write localized InfoPlist.strings");
    }

    fn unique_temp_dir() -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("smartime-system-app-test-{now}"));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn test_get_installed_apps() {
        let apps = get_installed_apps().expect("Failed to get installed apps");
        assert!(!apps.is_empty(), "Should find at least one app");

        // Check for common apps (optional, but good for verification)
        let _has_finder = apps.iter().any(|app| app.bundle_id == "com.apple.finder");
        let has_safari = apps.iter().any(|app| app.bundle_id == "com.apple.Safari");

        if !has_safari {
            println!("Warning: Safari not found. This might be normal on some systems or if Safari is moved.");
        }

        for app in apps.iter().take(5) {
            println!(
                "Found app: {} ({}) at {:?}",
                app.name, app.bundle_id, app.path
            );
        }
        for bundle_id in [
            "com.bot.pc.doubao",
            "com.netease.163music",
            "com.tencent.xinWeChat",
        ] {
            if let Some(app) = apps.iter().find(|app| app.bundle_id == bundle_id) {
                println!(
                    "Localized app check: {} ({}) at {:?}",
                    app.name, app.bundle_id, app.path
                );
            }
        }
    }

    #[test]
    fn scan_apps_in_paths_deduplicates_bundle_ids_across_roots() {
        let temp_root = unique_temp_dir();
        let apps_root = temp_root.join("Applications");
        let system_root = temp_root.join("SystemApplications");
        fs::create_dir_all(&apps_root).expect("create apps root");
        fs::create_dir_all(&system_root).expect("create system root");

        create_test_app(&apps_root, "Safari", "com.apple.Safari");
        create_test_app(&system_root, "Safari Copy", "com.apple.Safari");
        create_test_app(&system_root, "Terminal", "com.apple.Terminal");

        let apps = scan_apps_in_roots(&[
            ScanRoot {
                path: apps_root,
                kind: ScanRootKind::User,
            },
            ScanRoot {
                path: system_root,
                kind: ScanRootKind::System,
            },
        ])
        .expect("scan apps");

        assert_eq!(apps.len(), 2);
        assert!(apps.iter().any(|app| app.bundle_id == "com.apple.Safari"));
        assert!(apps.iter().any(|app| app.bundle_id == "com.apple.Terminal"));

        fs::remove_dir_all(temp_root).expect("remove temp dir");
    }

    #[test]
    fn scan_apps_in_paths_limits_apple_apps_to_input_capable_allowlist() {
        let temp_root = unique_temp_dir();
        let apps_root = temp_root.join("Applications");
        let system_root = temp_root.join("SystemApplications");
        fs::create_dir_all(&apps_root).expect("create apps root");
        fs::create_dir_all(&system_root).expect("create system root");

        let safari = create_test_app(&system_root, "Safari", "com.apple.Safari");
        create_localized_info_plist(&safari, "Safari 浏览器");
        create_test_app(&system_root, "Siri", "com.apple.Siri");
        create_test_app(&system_root, "Notes", "com.apple.Notes");
        create_test_app(&apps_root, "Chrome", "com.google.Chrome");

        let apps = scan_apps_in_roots(&[
            ScanRoot {
                path: apps_root,
                kind: ScanRootKind::User,
            },
            ScanRoot {
                path: system_root,
                kind: ScanRootKind::System,
            },
        ])
        .expect("scan apps");

        assert_eq!(apps.len(), 3);
        assert!(apps
            .iter()
            .any(|app| { app.bundle_id == "com.apple.Safari" && app.name == "Safari 浏览器" }));
        assert!(apps
            .iter()
            .any(|app| app.bundle_id == "com.apple.Notes" && app.name == "备忘录"));
        assert!(apps.iter().any(|app| app.bundle_id == "com.google.Chrome"));
        assert!(!apps.iter().any(|app| app.bundle_id == "com.apple.Siri"));

        fs::remove_dir_all(temp_root).expect("remove temp dir");
    }

    #[test]
    fn preferred_app_name_uses_localized_name_when_system_name_is_only_path_stem() {
        let app_path = PathBuf::from("/Applications/WeChat.app");
        let name = preferred_app_name(
            &app_path,
            Some("WeChat".to_string()),
            Some("微信".to_string()),
        );

        assert_eq!(name, Some("微信".to_string()));
    }

    #[test]
    fn preferred_app_name_treats_path_stem_case_changes_as_unlocalized() {
        let app_path = PathBuf::from("/Applications/doubao.app");
        let name = preferred_app_name(
            &app_path,
            Some("Doubao".to_string()),
            Some("豆包".to_string()),
        );

        assert_eq!(name, Some("豆包".to_string()));

        let app_path = PathBuf::from("/Applications/NeteaseMusic.app");
        let name = preferred_app_name(
            &app_path,
            Some("NetEaseMusic".to_string()),
            Some("网易云音乐".to_string()),
        );

        assert_eq!(name, Some("网易云音乐".to_string()));
    }

    #[test]
    fn preferred_app_name_uses_system_name_when_it_is_localized() {
        let app_path = PathBuf::from("/Applications/WeChat.app");
        let name = preferred_app_name(
            &app_path,
            Some("微信".to_string()),
            Some("WeChat".to_string()),
        );

        assert_eq!(name, Some("微信".to_string()));
    }

    #[test]
    fn clean_app_name_strips_bundle_extension_from_display_names() {
        assert_eq!(
            clean_app_name(Some("aDrive.app".to_string())),
            Some("aDrive".to_string())
        );
        assert_eq!(clean_app_name(Some("  ".to_string())), None);
    }

    #[test]
    fn parse_strings_value_prefers_display_name() {
        let content = r#"
            "CFBundleName" = "aDrive";
            "CFBundleDisplayName" = "阿里云盘";
        "#;

        assert_eq!(
            parse_strings_value(content, "CFBundleDisplayName"),
            Some("阿里云盘".to_string())
        );
    }

    #[test]
    fn localized_name_from_text_info_plist_strings_falls_back_to_bundle_name() {
        let temp_root = unique_temp_dir();
        let strings_path = temp_root.join("InfoPlist.strings");
        fs::write(
            &strings_path,
            r#"
            "CFBundleName" = "微信.app";
            "#,
        )
        .expect("write strings file");

        let name = localized_name_from_info_plist_strings_file(&strings_path);

        assert_eq!(name, Some("微信".to_string()));
        fs::remove_dir_all(temp_root).expect("remove temp dir");
    }

    #[test]
    fn localized_name_from_utf16_info_plist_strings_is_supported() {
        let temp_root = unique_temp_dir();
        let strings_path = temp_root.join("InfoPlist.strings");
        let content = r#""CFBundleDisplayName" = "网易云音乐";"#;
        let mut bytes = vec![0xff, 0xfe];
        for unit in content.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        fs::write(&strings_path, bytes).expect("write utf16 strings file");

        let name = localized_name_from_info_plist_strings_file(&strings_path);

        assert_eq!(name, Some("网易云音乐".to_string()));
        fs::remove_dir_all(temp_root).expect("remove temp dir");
    }

    #[test]
    fn installed_chinese_app_names_are_localized_when_resources_exist() {
        let cases = [
            (PathBuf::from("/Applications/doubao.app"), "豆包"),
            (
                PathBuf::from("/Applications/NeteaseMusic.app"),
                "网易云音乐",
            ),
            (PathBuf::from("/Applications/WeChat.app"), "微信"),
        ];

        for (path, expected_name) in cases {
            if !path.exists() {
                continue;
            }

            let app = parse_app_plist(&path).expect("parse installed app plist");
            assert_eq!(app.name, expected_name, "unexpected name for {path:?}");
        }
    }
}
