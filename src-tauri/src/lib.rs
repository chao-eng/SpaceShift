use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

mod db;
mod chrome;
mod profile_manager;
mod network_optimizer;
mod performance_monitor;

use db::{Database, Profile, Backup};
use chrome::{ChromeManager, ChromeLaunchResult};
use profile_manager::{ProfileManager, BackupResult, RestoreResult};

struct AppState {
    db: Mutex<Database>,
    chrome_manager: ChromeManager,
}

#[tauri::command]
fn get_profiles(state: State<AppState>) -> Result<Vec<Profile>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_profiles().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_profile(id: String, state: State<AppState>) -> Result<Option<Profile>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_profile_by_id(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_profile(
    name: String,
    icon_base64: Option<String>,
    tags: Option<String>,
    app_handle: AppHandle,
    state: State<AppState>,
) -> Result<Profile, String> {
    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let profiles_dir = app_data_dir.join("profiles");
    
    std::fs::create_dir_all(&profiles_dir).map_err(|e| e.to_string())?;
    
    let profile_dir = ProfileManager::create_profile_directory(&profiles_dir, &name)
        .map_err(|e| e.to_string())?;
    
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_profile(&name, &profile_dir.to_string_lossy(), icon_base64.as_deref(), tags.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_profile(
    id: String,
    name: Option<String>,
    icon_base64: Option<String>,
    tags: Option<String>,
    state: State<AppState>,
) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_profile(&id, name.as_deref(), icon_base64.as_deref(), tags.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_profile(id: String, state: State<AppState>) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    if let Ok(Some(profile)) = db.get_profile_by_id(&id) {
        let profile_dir = PathBuf::from(&profile.data_dir_path);
        ProfileManager::delete_profile_directory(&profile_dir).map_err(|e| e.to_string())?;
    }
    
    db.delete_profile(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn launch_chrome(
    id: String,
    url: Option<String>,
    state: State<AppState>,
) -> Result<ChromeLaunchResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let profile = db.get_profile_by_id(&id)
        .map_err(|e| e.to_string())?
        .ok_or("Profile not found")?;

    let profile_dir = PathBuf::from(&profile.data_dir_path);

    let result = state.chrome_manager.launch_chrome(&id, &profile_dir, url.as_deref());

    if result.success {
        db.update_profile_status(&id, true, result.pid.map(|p| p as i32))
            .map_err(|e| e.to_string())?;
    }

    Ok(result)
}

#[tauri::command]
fn backup_profile(
    id: String,
    backup_dir: String,
    state: State<AppState>,
) -> Result<BackupResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let profile = db.get_profile_by_id(&id)
        .map_err(|e| e.to_string())?;
    
    if let Some(profile) = profile {
        let profile_dir = PathBuf::from(&profile.data_dir_path);
        let backup_path = PathBuf::from(&backup_dir);
        
        let result = ProfileManager::backup_profile(&profile_dir, &backup_path, &profile.name);
        
        if result.success {
            if let Some(ref backup_file_path) = result.backup_path {
                let _ = db.create_backup(&id, backup_file_path, result.size_bytes);
            }
        }
        
        Ok(result)
    } else {
        Ok(BackupResult {
            success: false,
            backup_path: None,
            size_bytes: 0,
            error: Some("Profile not found".to_string()),
        })
    }
}

#[tauri::command]
fn restore_profile(
    id: String,
    backup_path: String,
    state: State<AppState>,
) -> Result<RestoreResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let profile = db.get_profile_by_id(&id)
        .map_err(|e| e.to_string())?;
    
    if let Some(profile) = profile {
        let backup_file = PathBuf::from(&backup_path);
        let target_dir = PathBuf::from(&profile.data_dir_path);
        
        let result = ProfileManager::restore_profile(&backup_file, &target_dir);
        Ok(result)
    } else {
        Ok(RestoreResult {
            success: false,
            error: Some("Profile not found".to_string()),
        })
    }
}

#[tauri::command]
fn get_backups(id: String, state: State<AppState>) -> Result<Vec<Backup>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_backups_by_profile(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_backup(id: String, state: State<AppState>) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_backup(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn search_profiles(query: String, state: State<AppState>) -> Result<Vec<Profile>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.search_profiles(&query).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_profiles_by_tag(tag: String, state: State<AppState>) -> Result<Vec<Profile>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_profiles_by_tag(&tag).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_profile_size(id: String, state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    if let Ok(Some(profile)) = db.get_profile_by_id(&id) {
        let profile_dir = PathBuf::from(&profile.data_dir_path);
        if let Ok(size) = ProfileManager::get_directory_size(&profile_dir) {
            return Ok(ProfileManager::format_size(size));
        }
    }
    
    Ok("0 B".to_string())
}

#[tauri::command]
fn get_app_data_dir(app_handle: AppHandle) -> Result<String, String> {
    app_handle.path()
        .app_data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_profile_directory(profile_data_dir: String) -> Result<(), String> {
    let path = PathBuf::from(&profile_data_dir);
    if !path.exists() {
        return Err("目录不存在".to_string());
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开目录: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开目录: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开目录: {}", e))?;
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            
            let db = Database::new(&app_data_dir)?;
            let chrome_manager = ChromeManager::new();
            
            app.manage(AppState {
                db: Mutex::new(db),
                chrome_manager,
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_profiles,
            get_profile,
            create_profile,
            update_profile,
            delete_profile,
            launch_chrome,
            backup_profile,
            restore_profile,
            get_backups,
            delete_backup,
            search_profiles,
            get_profiles_by_tag,
            get_profile_size,
            get_app_data_dir,
            open_profile_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
