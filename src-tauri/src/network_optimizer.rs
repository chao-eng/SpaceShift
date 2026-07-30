/// Network optimization for Chrome profiles
pub struct NetworkOptimizer;

impl NetworkOptimizer {
    /// Get optimized Chrome launch arguments
    /// 只保留关键参数，避免影响运行性能
    /// 注意：--disable-extensions 由调用方根据配置决定是否添加
    pub fn get_optimized_args() -> Vec<String> {
        vec![
            "--disable-ipv6".to_string(),
            "--dns-prefetch-disable".to_string(),
            "--no-first-run".to_string(),
        ]
    }
}
