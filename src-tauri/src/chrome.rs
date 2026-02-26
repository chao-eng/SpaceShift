use std::process::Command;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::network_optimizer::NetworkOptimizer;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChromeLaunchResult {
    pub success: bool,
    pub pid: Option<u32>,
    pub error: Option<String>,
}

pub struct ChromeManager;

impl ChromeManager {
    pub fn new() -> Self {
        ChromeManager
    }

    pub fn launch_chrome(&self, _profile_id: &str, user_data_dir: &PathBuf, url: Option<&str>) -> ChromeLaunchResult {
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
}
