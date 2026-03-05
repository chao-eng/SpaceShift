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
            #[cfg(target_os = "macos")]
            {
                let mut c = Command::new("open");
                c.arg("-n");
                c.arg("-a");
                c.arg("Google Chrome");
                c.arg("--args");
                c
            }
            #[cfg(target_os = "windows")]
            {
                // Typical Windows paths for Chrome
                let paths = vec![
                    "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
                    "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
                ];
                let mut found_path = "chrome.exe".to_string(); // Fallback to PATH
                for p in paths {
                    if std::path::Path::new(p).exists() {
                        found_path = p.to_string();
                        break;
                    }
                }
                Command::new(found_path)
            }
            #[cfg(target_os = "linux")]
            {
                Command::new("google-chrome")
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
}
