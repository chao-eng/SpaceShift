use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Edge,
    Safari,
    Opera,
    Brave,
    Unknown,
}

impl BrowserType {
    #[allow(dead_code)]
    pub fn process_names(&self) -> Vec<&'static str> {
        match self {
            BrowserType::Chrome => vec!["Google Chrome"],
            BrowserType::Firefox => vec!["Firefox"],
            BrowserType::Edge => vec!["Microsoft Edge"],
            BrowserType::Safari => vec!["Safari"],
            BrowserType::Opera => vec!["Opera"],
            BrowserType::Brave => vec!["Brave Browser"],
            BrowserType::Unknown => vec![],
        }
    }

    #[allow(dead_code)]
    pub fn from_process_name(name: &str) -> Self {
        let name_lower = name.to_lowercase();
        if name_lower.contains("chrome") {
            BrowserType::Chrome
        } else if name_lower.contains("firefox") {
            BrowserType::Firefox
        } else if name_lower.contains("edge") || name_lower.contains("msedge") {
            BrowserType::Edge
        } else if name_lower.contains("safari") {
            BrowserType::Safari
        } else if name_lower.contains("opera") {
            BrowserType::Opera
        } else if name_lower.contains("brave") {
            BrowserType::Brave
        } else {
            BrowserType::Unknown
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            BrowserType::Chrome => "Google Chrome",
            BrowserType::Firefox => "Mozilla Firefox",
            BrowserType::Edge => "Microsoft Edge",
            BrowserType::Safari => "Apple Safari",
            BrowserType::Opera => "Opera",
            BrowserType::Brave => "Brave Browser",
            BrowserType::Unknown => "Unknown Browser",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserProcess {
    pub pid: u32,
    pub name: String,
    pub browser_type: BrowserType,
    pub command: Option<String>,
    pub user_data_dir: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KillBrowserResult {
    pub success: bool,
    pub killed_count: usize,
    pub failed_pids: Vec<u32>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowserSafetyCheck {
    pub is_safe_to_kill: bool,
    pub warnings: Vec<String>,
    pub browser_type: BrowserType,
    pub process_count: usize,
}

pub struct BrowserKiller;

impl BrowserKiller {
    pub fn new() -> Self {
        BrowserKiller
    }

    pub fn detect_browser_processes(&self) -> Vec<BrowserProcess> {
        let mut processes = Vec::new();
        
        let output = Command::new("ps")
            .args(&["-eo", "pid,comm,args"])
            .output();
        
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(pid) = parts[0].parse::<u32>() {
                        let comm = parts[1];
                        let args = if parts.len() > 2 {
                            parts[2..].join(" ")
                        } else {
                            String::new()
                        };
                        
                        let browser_type = self.identify_browser_type(comm, &args);
                        if browser_type != BrowserType::Unknown {
                            let user_data_dir = self.extract_user_data_dir(&args);
                            
                            processes.push(BrowserProcess {
                                pid,
                                name: comm.to_string(),
                                browser_type,
                                command: Some(args),
                                user_data_dir,
                            });
                        }
                    }
                }
            }
        }
        
        processes
    }

    fn identify_browser_type(&self, process_name: &str, command_line: &str) -> BrowserType {
        let name_lower = process_name.to_lowercase();
        let cmd_lower = command_line.to_lowercase();
        
        if name_lower.contains("chrome") || cmd_lower.contains("google chrome") {
            BrowserType::Chrome
        } else if name_lower.contains("firefox") {
            BrowserType::Firefox
        } else if name_lower.contains("edge") || name_lower.contains("msedge") {
            BrowserType::Edge
        } else if name_lower.contains("safari") {
            BrowserType::Safari
        } else if name_lower.contains("opera") {
            BrowserType::Opera
        } else if name_lower.contains("brave") {
            BrowserType::Brave
        } else {
            BrowserType::Unknown
        }
    }

    fn extract_user_data_dir(&self, command_line: &str) -> Option<String> {
        if let Some(pos) = command_line.find("--user-data-dir=") {
            let start = pos + "--user-data-dir=".len();
            let rest = &command_line[start..];
            
            let (_end, path) = if rest.starts_with('"') {
                let after_quote = &rest[1..];
                if let Some(quote_pos) = after_quote.find('"') {
                    (1 + quote_pos + 1, after_quote[..quote_pos].to_string())
                } else {
                    (rest.len(), rest[1..].to_string())
                }
            } else {
                let end = rest.find(' ').unwrap_or(rest.len());
                (end, rest[..end].to_string())
            };
            
            let expanded_path = if path.starts_with("~/") {
                dirs::home_dir()
                    .map(|home| home.join(&path[2..]).to_string_lossy().to_string())
                    .unwrap_or(path)
            } else {
                path
            };
            
            Some(expanded_path)
        } else {
            None
        }
    }

    pub fn safety_check(&self, processes: &[BrowserProcess]) -> BrowserSafetyCheck {
        let mut warnings = Vec::new();
        let browser_type = if !processes.is_empty() {
            processes[0].browser_type
        } else {
            BrowserType::Unknown
        };
        
        let unique_types: std::collections::HashSet<_> = processes
            .iter()
            .map(|p| p.browser_type)
            .collect();
        
        if unique_types.len() > 1 {
            warnings.push("检测到多个不同类型的浏览器进程".to_string());
        }
        
        if processes.iter().any(|p| p.browser_type == BrowserType::Safari) {
            warnings.push("警告：正在关闭系统浏览器 Safari，可能影响系统功能".to_string());
        }
        
        if processes.len() > 5 {
            warnings.push(format!("检测到 {} 个浏览器进程，数量异常", processes.len()));
        }
        
        BrowserSafetyCheck {
            is_safe_to_kill: warnings.len() < 3,
            warnings,
            browser_type,
            process_count: processes.len(),
        }
    }

    pub fn kill_browser_by_pid(&self, pid: u32) -> Result<(), String> {
        let _term_result = Command::new("kill")
            .arg("-15")
            .arg(pid.to_string())
            .output();
        
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        if self.is_process_running(pid) {
            match Command::new("kill").arg("-9").arg(pid.to_string()).output() {
                Ok(output) => {
                    if output.status.success() || !self.is_process_running(pid) {
                        Ok(())
                    } else {
                        Err(format!("Failed to kill process {}: {}", 
                            pid, String::from_utf8_lossy(&output.stderr)))
                    }
                }
                Err(e) => Err(format!("Failed to execute kill command: {}", e)),
            }
        } else {
            Ok(())
        }
    }
    
    fn is_process_running(&self, pid: u32) -> bool {
        match Command::new("ps").arg("-p").arg(pid.to_string()).output() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    pub fn kill_browsers_by_type(&self, browser_type: BrowserType) -> KillBrowserResult {
        let processes = self.detect_browser_processes();
        let target_processes: Vec<_> = processes
            .into_iter()
            .filter(|p| p.browser_type == browser_type)
            .collect();
        
        if target_processes.is_empty() {
            return KillBrowserResult {
                success: true,
                killed_count: 0,
                failed_pids: Vec::new(),
                message: format!("未找到运行的 {}", browser_type.display_name()),
            };
        }
        
        let _safety_check = self.safety_check(&target_processes);
        let mut killed_count = 0;
        let mut failed_pids = Vec::new();
        
        for process in &target_processes {
            match self.kill_browser_by_pid(process.pid) {
                Ok(_) => killed_count += 1,
                Err(_) => failed_pids.push(process.pid),
            }
        }
        
        let success = failed_pids.is_empty();
        let message = if success {
            format!("成功关闭 {} 个 {} 进程", killed_count, browser_type.display_name())
        } else {
            format!("关闭了 {} 个进程，{} 个失败", killed_count, failed_pids.len())
        };
        
        KillBrowserResult {
            success,
            killed_count,
            failed_pids,
            message,
        }
    }

    pub fn kill_browser_by_profile(&self, profile_data_dir: &str) -> KillBrowserResult {
        let processes = self.detect_browser_processes();
        let target_processes: Vec<_> = processes
            .into_iter()
            .filter(|p| {
                p.user_data_dir.as_ref()
                    .map(|dir| dir == profile_data_dir || dir.contains(profile_data_dir))
                    .unwrap_or(false)
            })
            .collect();
        
        if target_processes.is_empty() {
            return KillBrowserResult {
                success: true,
                killed_count: 0,
                failed_pids: Vec::new(),
                message: "未找到使用该配置文件的浏览器进程".to_string(),
            };
        }
        
        let mut killed_count = 0;
        let mut failed_pids = Vec::new();
        
        for process in &target_processes {
            match self.kill_browser_by_pid(process.pid) {
                Ok(_) => killed_count += 1,
                Err(_) => failed_pids.push(process.pid),
            }
        }
        
        let success = failed_pids.is_empty();
        let message = if success {
            format!("成功关闭 {} 个浏览器进程", killed_count)
        } else {
            format!("关闭了 {} 个进程，{} 个失败", killed_count, failed_pids.len())
        };
        
        KillBrowserResult {
            success,
            killed_count,
            failed_pids,
            message,
        }
    }

    pub fn kill_all_browsers(&self) -> KillBrowserResult {
        let processes = self.detect_browser_processes();
        
        if processes.is_empty() {
            return KillBrowserResult {
                success: true,
                killed_count: 0,
                failed_pids: Vec::new(),
                message: "未找到运行的浏览器进程".to_string(),
            };
        }
        
        let _safety_check = self.safety_check(&processes);
        let mut killed_count = 0;
        let mut failed_pids = Vec::new();
        
        for process in &processes {
            match self.kill_browser_by_pid(process.pid) {
                Ok(_) => killed_count += 1,
                Err(_) => failed_pids.push(process.pid),
            }
        }
        
        let success = failed_pids.is_empty();
        let message = if success {
            format!("成功关闭 {} 个浏览器进程", killed_count)
        } else {
            format!("关闭了 {} 个进程，{} 个失败", killed_count, failed_pids.len())
        };
        
        KillBrowserResult {
            success,
            killed_count,
            failed_pids,
            message,
        }
    }
}
