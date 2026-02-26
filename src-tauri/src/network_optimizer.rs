/// Network optimization for Chrome profiles
pub struct NetworkOptimizer;

impl NetworkOptimizer {
    /// Get optimized Chrome launch arguments
    /// 只保留关键参数，避免影响运行性能
    pub fn get_optimized_args() -> Vec<String> {
        vec![
            // === 核心参数（保留）===
            "--disable-extensions".to_string(),
            "--disable-ipv6".to_string(),
            "--no-first-run".to_string(),
            
            // === 移除影响运行性能的参数 ===
            // "--dns-prefetch-disable" 已移除 - 导致每次访问新域名都要解析
            // "--disable-background-networking" 已移除 - 影响资源预加载
            
            // === 保留必要优化 ===
            "--disable-sync".to_string(),
            "--disable-default-apps".to_string(),
        ]
    }
}
