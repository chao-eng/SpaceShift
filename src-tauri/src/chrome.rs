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

/// Store process info including the user data directory
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

    /// Check if Chrome is running for a specific profile by checking system processes
    /// Uses multiple detection methods for better reliability in sandboxed environments
    fn check_chrome_processes_by_profile(&self, user_data_dir: &PathBuf) -> bool {
        let dir_str = user_data_dir.to_string_lossy();
        let dir_str_ref: &str = &dir_str;
        
        // Normalize path for comparison (handle symlinks, relative paths, etc.)
        let normalized_dir = std::fs::canonicalize(user_data_dir).unwrap_or_else(|_| user_data_dir.clone());
        let normalized_dir_str = normalized_dir.to_string_lossy();

        #[cfg(target_os = "macos")]
        {
            // Method 1: Use pgrep with full command line
            if let Ok(output) = Command::new("/usr/bin/pgrep").args(&["-a", "-f", "Google Chrome"]).output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("--user-data-dir") {
                        // Extract the user-data-dir from the command line
                        if let Some(idx) = line.find("--user-data-dir=") {
                            let start = idx + "--user-data-dir=".len();
                            let end = line[start..].find(' ').map(|i| start + i).unwrap_or(line.len());
                            let process_dir = &line[start..end];
                            
                            // Compare normalized paths
                            if process_dir == dir_str_ref || process_dir == normalized_dir_str.as_ref() {
                                return true;
                            }
                            // Also check if one contains the other (for path variations)
                            if process_dir.contains(dir_str_ref) || dir_str_ref.contains(process_dir) {
                                return true;
                            }
                        }
                    }
                }
            }

            // Method 2: Use ps aux for broader search
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

            // Method 3: Use lsof to find open files in the profile directory
            if let Ok(output) = Command::new("/usr/sbin/lsof").args(&["+D", dir_str_ref]).output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("Google Chrome") || line.contains("Chrome") {
                        return true;
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Use wmic to query process command lines
            if let Ok(output) = Command::new("wmic")
                .args(&["process", "where", "name='chrome.exe'", "get", "ProcessId,CommandLine", "/format:csv"])
                .output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().skip(1) {
                    if line.contains("--user-data-dir") {
                        if let Some(idx) = line.find("--user-data-dir=") {
                            let start = idx + "--user-data-dir=".len();
                            let end = line[start..].find(',').map(|i| start + i).unwrap_or(line.len());
                            let process_dir = &line[start..end].trim_matches('"');
                            
                            if process_dir.eq_ignore_ascii_case(dir_str_ref) 
                                || process_dir.eq_ignore_ascii_case(&normalized_dir_str) {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Method 1: pgrep
            if let Ok(output) = Command::new("/usr/bin/pgrep").args(&["-a", "-f", "chrome"]).output() {
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
                        }
                    }
                }
            }

            // Method 2: ps aux
            if let Ok(output) = Command::new("/bin/ps").args(&["aux"]).output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if (line.contains("chrome") || line.contains("google-chrome"))
                        && line.contains("--user-data-dir") {
                        if let Some(idx) = line.find("--user-data-dir=") {
                            let start = idx + "--user-data-dir=".len();
                            let end = line[start..].find(' ').map(|i| start + i).unwrap_or(line.len());
                            let process_dir = &line[start..end];
                            
                            if process_dir == dir_str_ref || process_dir == normalized_dir_str.as_ref() {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    fn get_chrome_executable() -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            // Check common Chrome locations on macOS
            let user = std::env::var("USER").unwrap_or_default();
            let user_app_path = format!("/Users/{}/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", user);
            
            let paths = [
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                &user_app_path,
            ];
            
            for path in &paths {
                let pb = PathBuf::from(path);
                if pb.exists() {
                    return pb;
                }
            }
            
            // Default fallback
            PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
        }
        
        #[cfg(target_os = "windows")]
        {
            // Check common Chrome locations on Windows
            let paths = [
                "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
                "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
            ];
            
            for path in &paths {
                let pb = PathBuf::from(path);
                if pb.exists() {
                    return pb;
                }
            }
            
            PathBuf::from("C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe")
        }
        
        #[cfg(target_os = "linux")]
        {
            // Check common Chrome locations on Linux
            let paths = [
                "/usr/bin/google-chrome",
                "/usr/bin/google-chrome-stable",
                "/usr/bin/chromium",
                "/usr/bin/chromium-browser",
            ];
            
            for path in &paths {
                let pb = PathBuf::from(path);
                if pb.exists() {
                    return pb;
                }
            }
            
            PathBuf::from("/usr/bin/google-chrome")
        }
    }

    pub fn launch_chrome(&self, profile_id: &str, user_data_dir: &PathBuf, url: Option<&str>) -> ChromeLaunchResult {
        let chrome_path = Self::get_chrome_executable();
        
        if !chrome_path.exists() {
            return ChromeLaunchResult {
                success: false,
                pid: None,
                error: Some(format!("Chrome executable not found at: {:?}", chrome_path)),
            };
        }

        let mut cmd = Command::new(&chrome_path);
        
        // Inherit environment variables from parent process
        cmd.envs(std::env::vars());
        
        // Set working directory to user home
        if let Ok(home) = std::env::var("HOME") {
            cmd.current_dir(&home);
        }
        
        // IMPORTANT: Redirect stdout/stderr to prevent blocking
        // Chrome outputs a lot of logs which can block the pipe buffer
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
        
        cmd.arg(format!("--user-data-dir={}", user_data_dir.display()));
        
        // Add all optimized launch arguments
        let optimized_args = NetworkOptimizer::get_optimized_args();
        for arg in optimized_args {
            cmd.arg(arg);
        }
        
        
        if let Some(url) = url {
            cmd.arg(url);
        }

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
            // First check if the child process is still running
            match process_info.child.try_wait() {
                Ok(None) => {
                    // Process is still running, also verify by checking system processes
                    if self.check_chrome_processes_by_profile(&process_info.user_data_dir) {
                        return true;
                    }
                    // Child thinks it's running but no system process found
                    // This can happen if Chrome forked and parent exited
                    // Check using the provided user_data_dir as fallback
                    if let Some(dir) = user_data_dir {
                        if self.check_chrome_processes_by_profile(dir) {
                            return true;
                        }
                    }
                    false
                }
                Ok(Some(_)) => {
                    // Process has exited, but Chrome might have forked
                    // Check system processes before removing
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
            // No tracked process, but check system anyway
            self.check_chrome_processes_by_profile(dir)
        } else {
            false
        }
    }

    pub fn bring_to_front(&self, pid: u32) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            // Try using osascript to bring Chrome window to front
            let script = format!(
                r#"tell application "Google Chrome"
                    activate
                end tell"#
            );
            
            match Command::new("/usr/bin/osascript").arg("-e").arg(&script).output() {
                Ok(output) => {
                    if output.status.success() {
                        return Ok(());
                    }
                }
                Err(_) => {}
            }
            
            // Fallback: try using System Events
            let script = format!(
                r#"tell application "System Events"
                    set frontmost of every process whose unix id is {} to true
                end tell"#,
                pid
            );
            
            match Command::new("/usr/bin/osascript").arg("-e").arg(&script).output() {
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
        
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            
            let script = format!(
                r#"$hwnd = (Get-Process -Id {}).MainWindowHandle; 
                   [void][System.Reflection.Assembly]::LoadWithPartialName('Microsoft.VisualBasic');
                   [Microsoft.VisualBasic.Interaction]::AppActivate($hwnd)"#,
                pid
            );
            
            match Command::new("powershell")
                .arg("-Command")
                .arg(&script)
                .creation_flags(CREATE_NO_WINDOW)
                .output() 
            {
                Ok(output) => {
                    if output.status.success() {
                        Ok(())
                    } else {
                        Err(format!("PowerShell failed: {}", String::from_utf8_lossy(&output.stderr)))
                    }
                }
                Err(e) => Err(format!("Failed to execute PowerShell: {}", e)),
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            match Command::new("wmctrl")
                .args(&["-i", "-r", &format!("0x{:x}", pid), "-b", "add,above"])
                .output() 
            {
                Ok(output) => {
                    if output.status.success() {
                        Ok(())
                    } else {
                        Err(format!("wmctrl failed: {}", String::from_utf8_lossy(&output.stderr)))
                    }
                }
                Err(e) => Err(format!("Failed to execute wmctrl: {}", e)),
            }
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
