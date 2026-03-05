use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("配置未找到")]
    ProfileNotFound,

    #[error("非法路径: {0}")]
    InvalidPath(String),

    #[error("网络请求失败: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Tauri 错误: {0}")]
    TauriError(#[from] tauri::Error),

    #[error("未知错误: {0}")]
    Other(String),
}

// 必须为 AppError 实现 Serialize，以便 Tauri 能够将其返回给前端
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type AppResult<T> = Result<T, AppError>;
