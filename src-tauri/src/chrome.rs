use std::process::Command;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::network_optimizer::NetworkOptimizer;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChromeLaunchResult {
    pub success: bool,
    pub pid: Option<u32>,
    pub error: Option<String>,
    pub spawn_duration_ms: u64,
}

pub struct ChromeManager;

impl ChromeManager {
    pub fn new() -> Self {
        ChromeManager
    }

    pub fn launch_chrome(&self, _profile_id: &str, user_data_dir: &PathBuf, chrome_path: Option<&str>, url: Option<&str>, debug_port: Option<u16>) -> ChromeLaunchResult {
        let mut cmd = if let Some(path) = chrome_path {
            Command::new(path)
        } else {
            let found_path = self.find_chrome_executable();
            
            #[cfg(target_os = "macos")]
            {
                let mut c = Command::new("open");
                c.arg("-n");
                c.arg("-a");
                c.arg(found_path.unwrap_or_else(|| "Google Chrome".to_string()));
                c.arg("--args");
                c
            }
            
            #[cfg(not(target_os = "macos"))]
            {
                Command::new(found_path.unwrap_or_else(|| {
                    #[cfg(target_os = "windows")] { "chrome.exe".to_string() }
                    #[cfg(target_os = "linux")] { "google-chrome".to_string() }
                    #[cfg(not(any(target_os = "windows", target_os = "linux")))] { "chrome".to_string() }
                }))
            }
        };

        cmd.arg(format!("--user-data-dir={}", user_data_dir.display()));
        
        if let Some(port) = debug_port {
            cmd.arg(format!("--remote-debugging-port={}", port));
        }

        let optimized_args = NetworkOptimizer::get_optimized_args();
        for arg in optimized_args {
            cmd.arg(arg);
        }
        
        if let Some(url) = url {
            cmd.arg(url);
        }
        
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());

        let start = std::time::Instant::now();
        match cmd.spawn() {
            Ok(child) => {
                let duration = start.elapsed().as_millis() as u64;
                let pid = child.id();
                ChromeLaunchResult {
                    success: true,
                    pid: Some(pid),
                    error: None,
                    spawn_duration_ms: duration,
                }
            }
            Err(e) => {
                let duration = start.elapsed().as_millis() as u64;
                ChromeLaunchResult {
                    success: false,
                    pid: None,
                    error: Some(format!("Failed to launch Chrome: {}", e)),
                    spawn_duration_ms: duration,
                }
            }
        }
    }

    pub fn find_chrome_executable(&self) -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            use winreg::enums::*;
            use winreg::RegKey;

            // 1. Try Registry (HKEY_LOCAL_MACHINE)
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            if let Ok(key) = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths\\chrome.exe") {
                if let Ok(path) = key.get_value::<String, _>("") {
                    if std::path::Path::new(&path).exists() {
                        return Some(path);
                    }
                }
            }

            // 2. Try Registry (HKEY_CURRENT_USER)
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(key) = hkcu.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths\\chrome.exe") {
                if let Ok(path) = key.get_value::<String, _>("") {
                    if std::path::Path::new(&path).exists() {
                        return Some(path);
                    }
                }
            }

            // 3. Fallback to common paths
            let common_paths = vec![
                "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_string(),
                "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe".to_string(),
                format!("{}\\Google\\Chrome\\Application\\chrome.exe", std::env::var("LOCALAPPDATA").unwrap_or_default()),
            ];

            for path in common_paths {
                if !path.is_empty() && std::path::Path::new(&path).exists() {
                    return Some(path);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let apps = [
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
                "/Applications/Chromium.app/Contents/MacOS/Chromium",
            ];
            for path in apps {
                if std::path::Path::new(path).exists() {
                    return Some(path.to_string());
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let binaries = ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser"];
            for bin in binaries {
                if std::process::Command::new("which").arg(bin).output().map(|o| o.status.success()).unwrap_or(false) {
                    return Some(bin.to_string());
                }
            }
        }

        None
    }

    pub fn unlock_profile(&self, user_data_dir: &PathBuf) -> Result<(), String> {
        let lock_files = ["SingletonLock", "Parent.lock", "lockfile"];
        for file in lock_files {
            let mut path = user_data_dir.clone();
            path.push(file);
            if path.exists() {
                if let Err(e) = std::fs::remove_file(&path) {
                    return Err(format!("Failed to remove lock file {}: {}", file, e));
                }
            }
        }
        Ok(())
    }
}
