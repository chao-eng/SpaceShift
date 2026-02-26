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
            BrowserType::Chrome => vec![
                #[cfg(target_os = "macos")]
                "Google Chrome",
                #[cfg(target_os = "windows")]
                "chrome.exe",
                #[cfg(target_os = "linux")]
                "google-chrome",
                #[cfg(target_os = "linux")]
                "chrome",
            ],
            BrowserType::Firefox => vec![
                #[cfg(target_os = "macos")]
                "Firefox",
                #[cfg(target_os = "windows")]
                "firefox.exe",
                #[cfg(target_os = "linux")]
                "firefox",
            ],
            BrowserType::Edge => vec![
                #[cfg(target_os = "macos")]
                "Microsoft Edge",
                #[cfg(target_os = "windows")]
                "msedge.exe",
                #[cfg(target_os = "linux")]
                "microsoft-edge",
            ],
            BrowserType::Safari => vec![
                #[cfg(target_os = "macos")]
                "Safari",
            ],
            BrowserType::Opera => vec![
                #[cfg(target_os = "macos")]
                "Opera",
                #[cfg(target_os = "windows")]
                "opera.exe",
                #[cfg(target_os = "linux")]
                "opera",
            ],
            BrowserType::Brave => vec![
                #[cfg(target_os = "macos")]
                "Brave Browser",
                #[cfg(target_os = "windows")]
                "brave.exe",
                #[cfg(target_os = "linux")]
                "brave",
            ],
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
        #[cfg(target_os = "macos")]
        {
            self.detect_macos_processes()
        }
        
        #[cfg(target_os = "windows")]
        {
            self.detect_windows_processes()
        }
        
        #[cfg(target_os = "linux")]
        {
            self.detect_linux_processes()
        }
    }

    #[cfg(target_os = "macos")]
    fn detect_macos_processes(&self) -> Vec<BrowserProcess> {
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

    #[cfg(target_os = "windows")]
    fn detect_windows_processes(&self) -> Vec<BrowserProcess> {
        let mut processes = Vec::new();
        
        let output = Command::new("wmic")
            .args(&["process", "get", "ProcessId,Name,CommandLine", "/format:csv"])
            .output();
        
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 3 {
                    if let Ok(pid) = parts[parts.len()-1].trim().parse::<u32>() {
                        let name = parts[parts.len()-2].trim();
                        let command_line = if parts.len() > 3 {
                            parts[2..parts.len()-2].join(",")
                        } else {
                            String::new()
                        };
                        
                        let browser_type = self.identify_browser_type(name, &command_line);
                        if browser_type != BrowserType::Unknown {
                            let user_data_dir = self.extract_user_data_dir(&command_line);
                            
                            processes.push(BrowserProcess {
                                pid,
                                name: name.to_string(),
                                browser_type,
                                command: Some(command_line),
                                user_data_dir,
                            });
                        }
                    }
                }
            }
        }
        
        processes
    }

    #[cfg(target_os = "linux")]
    fn detect_linux_processes(&self) -> Vec<BrowserProcess> {
        let mut processes = Vec::new();
        
        let output = Command::new("ps")
            .args(&["-eo", "pid,comm,cmd"])
            .output();
        
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(pid) = parts[0].parse::<u32>() {
                        let comm = parts[1];
                        let cmd = if parts.len() > 2 {
                            parts[2..].join(" ")
                        } else {
                            String::new()
                        };
                        
                        let browser_type = self.identify_browser_type(comm, &cmd);
                        if browser_type != BrowserType::Unknown {
                            let user_data_dir = self.extract_user_data_dir(&cmd);
                            
                            processes.push(BrowserProcess {
                                pid,
                                name: comm.to_string(),
                                browser_type,
                                command: Some(cmd),
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
        // 支持 --user-data-dir=path 和 --user-data-dir="path" 格式
        if let Some(pos) = command_line.find("--user-data-dir=") {
            let start = pos + "--user-data-dir=".len();
            let rest = &command_line[start..];
            
            // 检查是否使用引号包围
            let (_end, path) = if rest.starts_with('"') {
                // 引号格式: --user-data-dir="path with spaces"
                let after_quote = &rest[1..];
                if let Some(quote_pos) = after_quote.find('"') {
                    (1 + quote_pos + 1, after_quote[..quote_pos].to_string())
                } else {
                    (rest.len(), rest[1..].to_string())
                }
            } else {
                // 无引号格式: --user-data-dir=path
                let end = rest.find(' ').unwrap_or(rest.len());
                (end, rest[..end].to_string())
            };
            
            // 展开用户主目录符号
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
        
        // 检查是否有多个同类型浏览器实例
        let unique_types: std::collections::HashSet<_> = processes
            .iter()
            .map(|p| p.browser_type)
            .collect();
        
        if unique_types.len() > 1 {
            warnings.push("检测到多个不同类型的浏览器进程".to_string());
        }
        
        // 检查是否有系统浏览器（如 Safari）
        if processes.iter().any(|p| p.browser_type == BrowserType::Safari) {
            warnings.push("警告：正在关闭系统浏览器 Safari，可能影响系统功能".to_string());
        }
        
        // 检查进程数量
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
        #[cfg(target_os = "macos")]
        {
            // 首先尝试优雅终止 (SIGTERM)
            let _term_result = Command::new("kill")
                .arg("-15")  // SIGTERM
                .arg(pid.to_string())
                .output();
            
            // 等待一小段时间让进程有机会优雅关闭
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            // 检查进程是否还在运行
            if self.is_process_running(pid) {
                // 强制终止 (SIGKILL)
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
        
        #[cfg(target_os = "windows")]
        {
            // 使用 /T 参数终止进程及其所有子进程
            match Command::new("taskkill")
                .args(&["/F", "/T", "/PID", &pid.to_string()])
                .output() 
            {
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    // 检查是否成功或进程已不存在
                    if output.status.success() || 
                       stderr.contains("已经") || 
                       stderr.contains("already") ||
                       !self.is_process_running(pid) {
                        Ok(())
                    } else {
                        Err(format!("Failed to kill process {}: {}", pid, stderr))
                    }
                }
                Err(e) => Err(format!("Failed to execute taskkill: {}", e)),
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            // 首先尝试优雅终止 (SIGTERM)
            let _ = Command::new("kill")
                .arg("-15")  // SIGTERM
                .arg(pid.to_string())
                .output();
            
            // 等待一小段时间
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            // 检查进程是否还在运行
            if self.is_process_running(pid) {
                // 强制终止 (SIGKILL)
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
    }
    
    /// 检查进程是否仍在运行
    fn is_process_running(&self, pid: u32) -> bool {
        #[cfg(target_os = "macos")]
        {
            match Command::new("ps").arg("-p").arg(pid.to_string()).output() {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            match Command::new("tasklist")
                .args(&["/FI", &format!("PID eq {}", pid), "/NH"])
                .output() 
            {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.contains(&pid.to_string())
                }
                Err(_) => false,
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            match Command::new("ps").arg("-p").arg(pid.to_string()).output() {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
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
            format!("关闭了 {} 个进程，{} 个进程关闭失败", killed_count, failed_pids.len())
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
            format!("关闭了 {} 个进程，{} 个进程关闭失败", killed_count, failed_pids.len())
        };
        
        KillBrowserResult {
            success,
            killed_count,
            failed_pids,
            message,
        }
    }

    pub fn kill_browser_by_profile(&self, user_data_dir: &str) -> KillBrowserResult {
        let processes = self.detect_browser_processes();
        
        // 标准化路径：移除末尾的斜杠，转换为小写进行比较
        let normalized_target = user_data_dir.trim_end_matches('/').trim_end_matches('\\').to_lowercase();
        
        let target_processes: Vec<_> = processes
            .into_iter()
            .filter(|p| {
                p.user_data_dir.as_ref()
                    .map(|dir| {
                        let normalized_dir = dir.trim_end_matches('/').trim_end_matches('\\').to_lowercase();
                        // 双向包含检查：目标路径包含在检测到的路径中，或反之
                        normalized_dir.contains(&normalized_target) || 
                        normalized_target.contains(&normalized_dir)
                    })
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
        
        // 按PID排序，先终止子进程（通常PID较大），再终止父进程
        let mut sorted_processes = target_processes.clone();
        sorted_processes.sort_by(|a, b| b.pid.cmp(&a.pid));
        
        let mut killed_count = 0;
        let mut failed_pids = Vec::new();
        let mut error_details = Vec::new();
        
        // 第一轮：尝试终止所有进程
        for process in &sorted_processes {
            match self.kill_browser_by_pid(process.pid) {
                Ok(_) => {
                    killed_count += 1;
                }
                Err(e) => {
                    failed_pids.push(process.pid);
                    error_details.push(format!("PID {} ({}): {}", process.pid, process.name, e));
                }
            }
        }
        
        // 等待一段时间，让进程有机会完全终止
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // 第二轮：重试失败的进程
        let mut still_failed = Vec::new();
        for pid in &failed_pids {
            if self.is_process_running(*pid) {
                match self.kill_browser_by_pid(*pid) {
                    Ok(_) => {
                        killed_count += 1;
                    }
                    Err(_) => {
                        still_failed.push(*pid);
                    }
                }
            } else {
                // 进程已经终止
                killed_count += 1;
            }
        }
        
        failed_pids = still_failed;
        
        let success = failed_pids.is_empty();
        let message = if success {
            format!("成功关闭 {} 个使用该配置文件的浏览器进程", killed_count)
        } else {
            format!(
                "关闭了 {} 个进程，{} 个进程关闭失败。失败的进程: {:?}", 
                killed_count, 
                failed_pids.len(),
                failed_pids
            )
        };
        
        KillBrowserResult {
            success,
            killed_count,
            failed_pids,
            message,
        }
    }
}
