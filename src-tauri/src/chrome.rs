use std::process::{Command, Child};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

use crate::network_optimizer::NetworkOptimizer;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChromeLaunchResult {
    pub success: bool,
    pub pid: Option<u32>,
    pub error: Option<String>,
}

struct ProcessInfo {
    child: Child,
    user_data_dir: PathBuf,
}

impl ProcessInfo {
    fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }
}

pub struct ChromeManager {
    running_processes: Mutex<HashMap<String, ProcessInfo>>,
}

impl ChromeManager {
    pub fn new() -> Self {
        ChromeManager {
            running_processes: Mutex::new(HashMap::new()),
        }
    }

    fn check_chrome_processes_by_profile(&self, user_data_dir: &PathBuf) -> bool {
        let dir_str = user_data_dir.to_string_lossy();
        let dir_str_ref: &str = &dir_str;
        
        let normalized_dir = std::fs::canonicalize(user_data_dir).unwrap_or_else(|_| user_data_dir.clone());
        let normalized_dir_str = normalized_dir.to_string_lossy();

        if let Ok(output) = Command::new("/usr/bin/pgrep").args(&["-a", "-f", "Google Chrome"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("--user-data-dir") {
                    if let Some(idx) = line.find("--user-data-dir=") {
                        let start = idx + "--user-data-dir=".len();
                        let end = line[start..].find(' ').map(|i| start + i).unwrap_or(line.len());
                        let process_dir = &line[start..end];
                        
                        if process_dir == dir_str_ref || process_dir == normalized_dir_str.as_ref() {
                            return true;
                        }
                        if process_dir.contains(dir_str_ref) || dir_str_ref.contains(process_dir) {
                            return true;
                        }
                    }
                }
            }
        }

        if let Ok(output) = Command::new("/bin/ps").args(&["aux"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if (line.contains("Google Chrome") || line.contains("Chrome"))
                    && line.contains("--user-data-dir") {
                    if let Some(idx) = line.find("--user-data-dir=") {
                        let start = idx + "--user-data-dir=".len();
                        let end = line[start..].find(' ').map(|i| start + i).unwrap_or(line.len());
                        let process_dir = &line[start..end];
                        
                        if process_dir == dir_str_ref || process_dir == normalized_dir_str.as_ref() {
                            return true;
                        }
                        if process_dir.contains(dir_str_ref) || dir_str_ref.contains(process_dir) {
                            return true;
                        }
                    }
                }
            }
        }

        if let Ok(output) = Command::new("/usr/sbin/lsof").args(&["+D", dir_str_ref]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("Google Chrome") || line.contains("Chrome") {
                    return true;
                }
            }
        }

        false
    }

    pub fn launch_chrome(&self, profile_id: &str, user_data_dir: &PathBuf, url: Option<&str>) -> ChromeLaunchResult {
        let mut cmd = Command::new("open");
        cmd.arg("-n");
        cmd.arg("-a");
        cmd.arg("Google Chrome");
        cmd.arg("--args");
        cmd.arg(format!("--user-data-dir={}", user_data_dir.display()));
        
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

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                let mut processes = self.running_processes.lock().unwrap();
                processes.insert(profile_id.to_string(), ProcessInfo {
                    child,
                    user_data_dir: user_data_dir.clone(),
                });

                ChromeLaunchResult {
                    success: true,
                    pid: Some(pid),
                    error: None,
                }
            }
            Err(e) => ChromeLaunchResult {
                success: false,
                pid: None,
                error: Some(format!("Failed to launch Chrome: {}", e)),
            },
        }
    }

    pub fn is_chrome_running(&self, profile_id: &str, user_data_dir: Option<&PathBuf>) -> bool {
        let mut processes = self.running_processes.lock().unwrap();

        if let Some(process_info) = processes.get_mut(profile_id) {
            match process_info.child.try_wait() {
                Ok(None) => {
                    if self.check_chrome_processes_by_profile(&process_info.user_data_dir) {
                        return true;
                    }
                    if let Some(dir) = user_data_dir {
                        if self.check_chrome_processes_by_profile(dir) {
                            return true;
                        }
                    }
                    false
                }
                Ok(Some(_)) => {
                    if self.check_chrome_processes_by_profile(&process_info.user_data_dir) {
                        return true;
                    }
                    processes.remove(profile_id);
                    false
                }
                Err(_) => {
                    processes.remove(profile_id);
                    false
                }
            }
        } else if let Some(dir) = user_data_dir {
            self.check_chrome_processes_by_profile(dir)
        } else {
            false
        }
    }

    pub fn bring_to_front(&self, _pid: u32) -> Result<(), String> {
        let script = r#"tell application "Google Chrome" to activate"#;
        
        match Command::new("/usr/bin/osascript").arg("-e").arg(script).output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(format!("osascript failed: {}", String::from_utf8_lossy(&output.stderr)))
                }
            }
            Err(e) => Err(format!("Failed to execute osascript: {}", e)),
        }
    }

    #[allow(dead_code)]
    pub fn kill_chrome(&self, profile_id: &str) -> Result<(), String> {
        let mut processes = self.running_processes.lock().unwrap();

        if let Some(mut process_info) = processes.remove(profile_id) {
            match process_info.kill() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to kill Chrome process: {}", e)),
            }
        } else {
            Err("No running Chrome process found for this profile".to_string())
        }
    }

    #[allow(dead_code)]
    pub fn get_all_running_pids(&self) -> Vec<(String, u32)> {
        let mut processes = self.running_processes.lock().unwrap();
        let mut result = Vec::new();
        let mut to_remove = Vec::new();

        for (profile_id, process_info) in processes.iter_mut() {
            match process_info.child.try_wait() {
                Ok(None) => {
                    result.push((profile_id.clone(), process_info.child.id()));
                }
                _ => {
                    to_remove.push(profile_id.clone());
                }
            }
        }

        for id in to_remove {
            processes.remove(&id);
        }

        result
    }
}
