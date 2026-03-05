use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

mod db;
mod chrome;
mod profile_manager;
mod network_optimizer;
mod performance_monitor;
mod browser_monitor;
mod error;

use db::{Database, Profile, Backup, PerformanceRecord};
use chrome::{ChromeManager, ChromeLaunchResult};
use profile_manager::{ProfileManager, BackupResult, RestoreResult};
use browser_monitor::BrowserMonitor;
use error::{AppError, AppResult};
use std::sync::Arc;

struct AppState {
    db: Arc<Mutex<Database>>,
    chrome_manager: ChromeManager,
}

#[tauri::command]
fn get_profiles(state: State<AppState>) -> AppResult<Vec<Profile>> {
    let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(db.get_all_profiles()?)
}

#[tauri::command]
fn get_profile(id: String, state: State<AppState>) -> AppResult<Option<Profile>> {
    let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(db.get_profile_by_id(&id)?)
}

#[tauri::command]
fn create_profile(
    name: String,
    chrome_path: Option<String>,
    homepage: Option<String>,
    icon_base64: Option<String>,
    tags: Option<String>,
    app_handle: AppHandle,
    state: State<AppState>,
) -> AppResult<Profile> {
    let app_data_dir = app_handle.path().app_data_dir()?;
    let profiles_dir = app_data_dir.join("profiles");
    
    std::fs::create_dir_all(&profiles_dir)?;
    
    let profile_dir = ProfileManager::create_profile_directory(&profiles_dir, &name)?;
    
    let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(db.create_profile(&name, &profile_dir.to_string_lossy(), chrome_path.as_deref(), homepage.as_deref(), icon_base64.as_deref(), tags.as_deref())?)
}

#[tauri::command]
fn update_profile(
    id: String,
    name: Option<String>,
    chrome_path: Option<String>,
    homepage: Option<String>,
    icon_base64: Option<String>,
    tags: Option<String>,
    state: State<AppState>,
) -> AppResult<bool> {
    let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(db.update_profile(&id, name.as_deref(), chrome_path.as_deref(), homepage.as_deref(), icon_base64.as_deref(), tags.as_deref())?)
}

#[tauri::command]
fn delete_profile(id: String, state: State<AppState>) -> AppResult<bool> {
    let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    
    if let Ok(Some(profile)) = db.get_profile_by_id(&id) {
        let profile_dir = PathBuf::from(&profile.data_dir_path);
        ProfileManager::delete_profile_directory(&profile_dir)?;
    }
    
    Ok(db.delete_profile(&id)?)
}

#[tauri::command]
fn launch_chrome(
    id: String,
    url: Option<String>,
    state: State<AppState>,
) -> AppResult<ChromeLaunchResult> {
    let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;

    let profile = db.get_profile_by_id(&id)?
        .ok_or(AppError::ProfileNotFound)?;

    let profile_dir = PathBuf::from(&profile.data_dir_path);
    
    // Use the provided URL if available, otherwise use the profile's homepage
    let launch_url = url.as_deref().or(profile.homepage.as_deref());

    // Find a free port for CDP
    let debug_port = std::net::TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|l| l.local_addr().ok())
        .map(|a| a.port());

    let start_inst = std::time::Instant::now();
    let result = state.chrome_manager.launch_chrome(
        &id, 
        &profile_dir, 
        profile.chrome_path.as_deref(),
        launch_url,
        debug_port
    );

    if result.success {
        db.update_profile_status(&id, true, result.pid.map(|p| p as i32))?;
        
        // Start async monitoring task using debug_port
        if let Some(port) = debug_port {
            let db_clone = state.db.clone();
            let profile_id = id.clone();
            let spawn_ms = result.spawn_duration_ms;

            tauri::async_runtime::spawn(async move {
                println!("[Lib] Monitoring performance for port: {}", port);
                let client = reqwest::Client::new();
                let url = format!("http://127.0.0.1:{}/json/version", port);
                
                let mut ready_ms = None;
                // Poll for up to 10 seconds
                for _ in 0..20 {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    if let Ok(resp) = client.get(&url).send().await {
                        if resp.status().is_success() {
                            ready_ms = Some(start_inst.elapsed().as_millis() as u64);
                            break;
                        }
                    }
                }

                if let Some(total_ms) = ready_ms {
                    let record = PerformanceRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        profile_id: profile_id.clone(),
                        launch_duration_ms: total_ms,
                        spawn_duration_ms: spawn_ms,
                        dns_duration_ms: Some(total_ms / 10), // Mocked sub-metrics for now
                        tcp_duration_ms: Some(total_ms / 5),
                        dom_ready_ms: Some(total_ms),
                        page_load_ms: Some(total_ms + 200),
                        created_at: chrono::Utc::now().to_rfc3339(),
                    };

                    let db_lock = db_clone.lock().unwrap();
                    let _ = db_lock.save_performance_record(&record);
                    println!("[Lib] Performance record saved for {}: {}ms", profile_id, total_ms);
                }
            });
        }
    }

    Ok(result)
}

#[tauri::command]
async fn backup_profile(
    id: String,
    backup_dir: String,
    state: State<'_, AppState>,
) -> AppResult<BackupResult> {
    let (profile_dir, profile_name) = {
        let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
        let profile = db.get_profile_by_id(&id)?
            .ok_or(AppError::ProfileNotFound)?;
        (PathBuf::from(&profile.data_dir_path), profile.name.clone())
    };
    
    let backup_path = PathBuf::from(&backup_dir);
    
    // Perform heavy backup operation in a blocking thread to keep the async executor free
    let result = tauri::async_runtime::spawn_blocking(move || {
        ProfileManager::backup_profile(&profile_dir, &backup_path, &profile_name)
    }).await.map_err(|e| AppError::Other(e.to_string()))?;
    
    if result.success {
        if let Some(backup_file_path) = &result.backup_path {
            let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
            let _ = db.create_backup(&id, backup_file_path, result.size_bytes);
        }
    }
    
    Ok(result)
}

#[tauri::command]
async fn restore_profile(
    id: String,
    backup_path: String,
    state: State<'_, AppState>,
) -> AppResult<RestoreResult> {
    let target_dir = {
        let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
        let profile = db.get_profile_by_id(&id)?
            .ok_or(AppError::ProfileNotFound)?;
        PathBuf::from(&profile.data_dir_path)
    };
    
    let backup_file = PathBuf::from(&backup_path);
    
    // Perform heavy restore operation in a blocking thread
    let result = tauri::async_runtime::spawn_blocking(move || {
        ProfileManager::restore_profile(&backup_file, &target_dir)
    }).await.map_err(|e| AppError::Other(e.to_string()))?;
    
    Ok(result)
}

#[tauri::command]
fn get_backups(id: String, state: State<AppState>) -> AppResult<Vec<Backup>> {
    let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(db.get_backups_by_profile(&id)?)
}

#[tauri::command]
fn delete_backup(id: String, state: State<AppState>) -> AppResult<bool> {
    let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    
    // Fetch backup info to get the file path first
    if let Ok(Some(backup)) = db.get_backup_by_id(&id) {
        let path = PathBuf::from(&backup.backup_path);
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
    }
    
    Ok(db.delete_backup(&id)?)
}

#[tauri::command]
fn search_profiles(query: String, state: State<AppState>) -> AppResult<Vec<Profile>> {
    let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(db.search_profiles(&query)?)
}

#[tauri::command]
fn get_profiles_by_tag(tag: String, state: State<AppState>) -> AppResult<Vec<Profile>> {
    let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(db.get_profiles_by_tag(&tag)?)
}

#[tauri::command]
fn get_performance_logs(id: String, limit: Option<i32>, state: State<AppState>) -> AppResult<Vec<PerformanceRecord>> {
    let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(db.get_performance_logs(&id, limit.unwrap_or(10))?)
}

#[tauri::command]
fn get_profile_size(id: String, state: State<AppState>) -> AppResult<String> {
    let db = state.db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    
    if let Ok(Some(profile)) = db.get_profile_by_id(&id) {
        let profile_dir = PathBuf::from(&profile.data_dir_path);
        if let Ok(size) = ProfileManager::get_directory_size(&profile_dir) {
            return Ok(ProfileManager::format_size(size));
        }
    }
    
    Ok("0 B".to_string())
}

#[tauri::command]
fn get_app_data_dir(app_handle: AppHandle) -> AppResult<String> {
    Ok(app_handle.path()
        .app_data_dir()?
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
fn open_profile_directory(profile_data_dir: String) -> AppResult<()> {
    let path = PathBuf::from(&profile_data_dir);
    if !path.exists() {
        return Err(AppError::InvalidPath("目录不存在".to_string()));
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()?;
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
            
            let db = Arc::new(Mutex::new(Database::new(&app_data_dir)?));
            let chrome_manager = ChromeManager::new();
            
            let monitor_db = db.clone();
            let app_handle = app.handle().clone();
            let monitor = BrowserMonitor::new(monitor_db, app_handle);
            monitor.start_monitoring();
            
            app.manage(AppState {
                db,
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
            get_performance_logs,
            get_profile_size,
            get_app_data_dir,
            open_profile_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
