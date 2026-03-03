#![allow(dead_code)]

use std::time::Instant;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Performance metrics for Chrome launch and page load
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub launch_start_time: u64,
    pub launch_end_time: u64,
    pub total_launch_duration_ms: u64,
    pub dns_lookup_time_ms: Option<u64>,
    pub tcp_connect_time_ms: Option<u64>,
    pub ssl_handshake_time_ms: Option<u64>,
    pub first_byte_time_ms: Option<u64>,
    pub dom_content_loaded_ms: Option<u64>,
    pub page_load_complete_ms: Option<u64>,
    pub chrome_process_spawn_ms: u64,
    pub chrome_window_ready_ms: Option<u64>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            launch_start_time: 0,
            launch_end_time: 0,
            total_launch_duration_ms: 0,
            dns_lookup_time_ms: None,
            tcp_connect_time_ms: None,
            ssl_handshake_time_ms: None,
            first_byte_time_ms: None,
            dom_content_loaded_ms: None,
            page_load_complete_ms: None,
            chrome_process_spawn_ms: 0,
            chrome_window_ready_ms: None,
        }
    }
}

/// Performance monitor for tracking Chrome launch and page load performance
pub struct PerformanceMonitor {
    start_time: Option<Instant>,
    checkpoints: HashMap<String, Instant>,
    metrics: PerformanceMetrics,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: None,
            checkpoints: HashMap::new(),
            metrics: PerformanceMetrics::new(),
        }
    }

    /// Start performance monitoring
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.metrics.launch_start_time = Self::current_timestamp_ms();
    }

    /// Record a checkpoint
    pub fn checkpoint(&mut self, name: &str) {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            self.checkpoints.insert(name.to_string(), Instant::now());
            
            match name {
                "chrome_spawned" => {
                    self.metrics.chrome_process_spawn_ms = elapsed.as_millis() as u64;
                }
                "window_ready" => {
                    self.metrics.chrome_window_ready_ms = Some(elapsed.as_millis() as u64);
                }
                "dns_complete" => {
                    self.metrics.dns_lookup_time_ms = Some(elapsed.as_millis() as u64);
                }
                "tcp_connected" => {
                    self.metrics.tcp_connect_time_ms = Some(elapsed.as_millis() as u64);
                }
                "ssl_complete" => {
                    self.metrics.ssl_handshake_time_ms = Some(elapsed.as_millis() as u64);
                }
                "first_byte" => {
                    self.metrics.first_byte_time_ms = Some(elapsed.as_millis() as u64);
                }
                "dom_ready" => {
                    self.metrics.dom_content_loaded_ms = Some(elapsed.as_millis() as u64);
                }
                "page_complete" => {
                    self.metrics.page_load_complete_ms = Some(elapsed.as_millis() as u64);
                }
                _ => {}
            }
        }
    }

    /// End performance monitoring
    pub fn end(&mut self) {
        if let Some(start) = self.start_time {
            let duration = start.elapsed();
            self.metrics.total_launch_duration_ms = duration.as_millis() as u64;
            self.metrics.launch_end_time = Self::current_timestamp_ms();
        }
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::from("=== Chrome Performance Report ===\n\n");
        
        report.push_str(&format!("Total Launch Duration: {} ms\n", self.metrics.total_launch_duration_ms));
        report.push_str(&format!("Chrome Process Spawn: {} ms\n", self.metrics.chrome_process_spawn_ms));
        
        if let Some(window_ready) = self.metrics.chrome_window_ready_ms {
            report.push_str(&format!("Window Ready: {} ms\n", window_ready));
        }
        
        if let Some(dns) = self.metrics.dns_lookup_time_ms {
            report.push_str(&format!("DNS Lookup: {} ms\n", dns));
        }
        
        if let Some(tcp) = self.metrics.tcp_connect_time_ms {
            report.push_str(&format!("TCP Connect: {} ms\n", tcp));
        }
        
        if let Some(ssl) = self.metrics.ssl_handshake_time_ms {
            report.push_str(&format!("SSL Handshake: {} ms\n", ssl));
        }
        
        if let Some(first_byte) = self.metrics.first_byte_time_ms {
            report.push_str(&format!("First Byte: {} ms\n", first_byte));
        }
        
        if let Some(dom) = self.metrics.dom_content_loaded_ms {
            report.push_str(&format!("DOM Content Loaded: {} ms\n", dom));
        }
        
        if let Some(page) = self.metrics.page_load_complete_ms {
            report.push_str(&format!("Page Load Complete: {} ms\n", page));
        }
        
        // Performance analysis
        report.push_str("\n=== Performance Analysis ===\n");
        
        if self.metrics.total_launch_duration_ms > 5000 {
            report.push_str("⚠️  WARNING: Launch time exceeds 5 seconds\n");
        }
        
        if let Some(dns) = self.metrics.dns_lookup_time_ms {
            if dns > 1000 {
                report.push_str("⚠️  WARNING: DNS lookup is slow (>1s)\n");
                report.push_str("   Suggestion: Check DNS configuration or use faster DNS servers\n");
            }
        }
        
        if let Some(tcp) = self.metrics.tcp_connect_time_ms {
            if let Some(dns) = self.metrics.dns_lookup_time_ms {
                let connect_time = tcp - dns;
                if connect_time > 500 {
                    report.push_str("⚠️  WARNING: TCP connection is slow (>500ms)\n");
                    report.push_str("   Suggestion: Check network latency or firewall settings\n");
                }
            }
        }
        
        report
    }

    fn current_timestamp_ms() -> u64 {
        use std::time::SystemTime;
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// Performance optimization recommendations
pub struct PerformanceOptimizer;

impl PerformanceOptimizer {
    /// Analyze system for performance issues
    pub fn analyze_system() -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Check available memory
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("vm_stat").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Parse memory info
                if stdout.contains("page size of") {
                    // Memory check passed
                } else {
                    recommendations.push("Unable to determine memory status".to_string());
                }
            }
        }
        
        // Check DNS configuration
        recommendations.push("Use fast DNS servers: 223.5.5.5 (Aliyun) or 119.29.29.29 (Tencent)".to_string());
        
        // Chrome optimization
        recommendations.push("Ensure Chrome is up to date".to_string());
        recommendations.push("Disable unnecessary Chrome extensions".to_string());
        recommendations.push("Clear Chrome cache periodically".to_string());
        
        recommendations
    }
    
    /// Get optimal Chrome flags based on system analysis
    pub fn get_optimal_flags() -> Vec<String> {
        #[allow(unused_mut)]
        let mut flags = vec![
            "--disable-extensions".to_string(),
            "--disable-ipv6".to_string(),
            "--dns-prefetch-disable".to_string(),
            "--no-first-run".to_string(),
            "--disable-background-networking".to_string(),
            "--disable-sync".to_string(),
            "--disable-default-apps".to_string(),
        ];
        
        // Add memory optimization for low-memory systems
        #[cfg(target_os = "macos")]
        {
            flags.push("--memory-model=low".to_string());
            flags.push("--max_old_space_size=512".to_string());
        }
        
        flags
    }
}
