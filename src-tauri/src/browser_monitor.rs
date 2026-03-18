use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use crate::db::Database;
use std::fs;

pub struct BrowserMonitor {
    db: Arc<Mutex<Database>>,
    app_handle: AppHandle,
}

impl BrowserMonitor {
    pub fn new(db: Arc<Mutex<Database>>, app_handle: AppHandle) -> Self {
        Self { db, app_handle }
    }

    pub fn start_monitoring(self) {
        let db = self.db.clone();
        let app_handle = self.app_handle.clone();

        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                
                let profiles = {
                    let db_lock = db.lock().unwrap();
                    match db_lock.get_all_profiles() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to get profiles for monitoring: {}", e);
                            continue;
                        }
                    }
                };

                for profile in profiles {
                    let mut is_running = Self::check_if_profile_running(&profile.data_dir_path, &profile.name);
                    
                    // DEBOUNCE: If it seems stopped, don't trust it.
                    // This is usually due to OS file system lag or Chrome's internal lock cycling.
                    if !is_running && profile.is_running {
                        println!("[Monitor] '{}' appears stopped, entering 4.5s triple-check period...", profile.name);
                        for i in 1..=2 {
                            tokio::time::sleep(Duration::from_millis(1500)).await;
                            is_running = Self::check_if_profile_running(&profile.data_dir_path, &profile.name);
                            if is_running { 
                                println!("[Monitor] '{}' recovered via retry #{}", profile.name, i);
                                break; 
                            }
                        }
                    }
                    
                    if is_running != profile.is_running {
                        println!("[Monitor] '{}' status confirm change: {} -> {}", profile.name, profile.is_running, is_running);
                        let db_lock = db.lock().unwrap();
                        let _ = db_lock.update_profile_status(&profile.id, is_running, None, None);
                        
                        let _ = app_handle.emit("browser-status-update", serde_json::json!({
                            "id": profile.id,
                            "is_running": is_running
                        }));
                    }
                }
            }
        });
    }

    fn check_if_profile_running(data_dir: &str, _name: &str) -> bool {
        let path = PathBuf::from(data_dir);
        let lock_file = path.join("SingletonLock");
        
        // 1. File & PID check (Reliable & Fast)
        if let Ok(_) = fs::symlink_metadata(&lock_file) {
            #[cfg(any(target_os = "macos", target_os = "linux"))]
            {
                if let Ok(target) = fs::read_link(&lock_file) {
                    if let Some(pid) = target.to_string_lossy().split('-').last().and_then(|p| p.parse::<i32>().ok()) {
                        // Directly ask kernel if PID is alive
                        if unsafe { libc::kill(pid, 0) } == 0 {
                            return true;
                        }
                    }
                }
            }
            // If we have metadata but PID check failed, still consider running 
            // because the lock file is physically there.
            return true;
        }

        // 2. Heavy Fallback: Process scan (Slow but extremely hard to trick)
        let folder_id = path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();

        if !folder_id.is_empty() {
            #[cfg(target_os = "macos")]
            {
                // -ww: huge output width, -ax: all processes.
                let output = std::process::Command::new("ps")
                    .args(["-axww", "-o", "command"])
                    .output();
                
                if let Ok(out) = output {
                    let cmd_list = String::from_utf8_lossy(&out.stdout);
                    if cmd_list.contains(&folder_id) && 
                       (cmd_list.contains("Google Chrome") || cmd_list.contains("chrome")) {
                        return true;
                    }
                }
            }

            #[cfg(target_os = "windows")]
            {
                // Windows fallback to lockfile existence
                if path.join("lockfile").exists() { return true; }
            }
        }

        false
    }
}
