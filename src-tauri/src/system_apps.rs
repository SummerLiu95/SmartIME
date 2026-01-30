use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};
use serde::Deserialize;
use crate::error::Result;

#[derive(Debug, Clone)]
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

fn is_app_bundle(entry: &DirEntry) -> bool {
    entry.file_type().is_dir() && entry.path().extension().map_or(false, |ext| ext == "app")
}

fn parse_app_plist(app_path: &Path) -> Option<SystemApp> {
    let plist_path = app_path.join("Contents/Info.plist");
    if !plist_path.exists() {
        return None;
    }

    let app_plist: AppPlist = plist::from_file(&plist_path).ok()?;
    
    let bundle_id = app_plist.bundle_identifier?;
    // Ignore empty bundle IDs
    if bundle_id.trim().is_empty() {
        return None;
    }

    let name = app_plist.display_name
        .or(app_plist.bundle_name)
        .unwrap_or_else(|| app_path.file_stem().unwrap().to_string_lossy().to_string());

    Some(SystemApp {
        name,
        bundle_id,
        path: app_path.to_path_buf(),
    })
}

pub fn get_installed_apps() -> Result<Vec<SystemApp>> {
    let mut apps = Vec::new();
    let mut seen_bundle_ids = HashSet::new();
    
    // Scan paths
    let mut scan_paths = vec![
        PathBuf::from("/Applications"),
    ];
    
    if let Some(home) = dirs::home_dir() {
        scan_paths.push(home.join("Applications"));
    }

    for root_path in scan_paths {
        if !root_path.exists() {
            continue;
        }

        // Use WalkDir to traverse
        // max_depth 3 is usually enough (e.g. /Applications/Utilities/Terminal.app)
        let mut it = WalkDir::new(&root_path)
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
                     if !seen_bundle_ids.contains(&app_info.bundle_id) {
                        seen_bundle_ids.insert(app_info.bundle_id.clone());
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

    #[test]
    fn test_get_installed_apps() {
        let apps = get_installed_apps().expect("Failed to get installed apps");
        assert!(!apps.is_empty(), "Should find at least one app");
        
        // Check for common apps (optional, but good for verification)
        let has_finder = apps.iter().any(|app| app.bundle_id == "com.apple.finder");
        let has_safari = apps.iter().any(|app| app.bundle_id == "com.apple.Safari");
        
        // Finder might be in /System/Library/CoreServices, not scanned by default in /Applications
        // But Safari should be in /Applications
        if !has_safari {
             println!("Warning: Safari not found in /Applications. This might be normal on some systems or if Safari is moved.");
        }
        
        for app in apps.iter().take(5) {
            println!("Found app: {} ({}) at {:?}", app.name, app.bundle_id, app.path);
        }
    }
}
