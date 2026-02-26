use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupResult {
    pub success: bool,
    pub backup_path: Option<String>,
    pub size_bytes: i64,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreResult {
    pub success: bool,
    pub error: Option<String>,
}

pub struct ProfileManager;

impl ProfileManager {
    pub fn create_profile_directory(base_dir: &PathBuf, _profile_name: &str) -> io::Result<PathBuf> {
        let uuid = uuid::Uuid::new_v4().to_string().replace('-', "");
        let profile_dir = base_dir.join(&uuid);
        fs::create_dir_all(&profile_dir)?;
        
        Ok(profile_dir)
    }

    pub fn delete_profile_directory(dir_path: &PathBuf) -> io::Result<()> {
        if dir_path.exists() {
            fs::remove_dir_all(dir_path)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn rename_profile_directory(old_path: &PathBuf, new_name: &str) -> io::Result<PathBuf> {
        let parent = old_path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Invalid path")
        })?;
        
        let sanitized_name = new_name
            .replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-', "_")
            .replace(' ', "_");
        
        let new_path = parent.join(&sanitized_name);
        
        if old_path.exists() {
            fs::rename(old_path, &new_path)?;
        }
        
        Ok(new_path)
    }

    pub fn backup_profile(profile_dir: &PathBuf, backup_dir: &PathBuf, profile_name: &str) -> BackupResult {
        if !profile_dir.exists() {
            return BackupResult {
                success: false,
                backup_path: None,
                size_bytes: 0,
                error: Some(format!("Profile directory does not exist: {:?}", profile_dir)),
            };
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("{}_{}.zip", profile_name.replace(' ', "_"), timestamp);
        let backup_path = backup_dir.join(&backup_filename);

        if let Some(parent) = backup_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return BackupResult {
                    success: false,
                    backup_path: None,
                    size_bytes: 0,
                    error: Some(format!("Failed to create backup directory: {}", e)),
                };
            }
        }

        let file = match fs::File::create(&backup_path) {
            Ok(f) => f,
            Err(e) => {
                return BackupResult {
                    success: false,
                    backup_path: None,
                    size_bytes: 0,
                    error: Some(format!("Failed to create backup file: {}", e)),
                };
            }
        };

        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        let mut total_size: i64 = 0;

        for entry in WalkDir::new(profile_dir) {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    let name = path.strip_prefix(profile_dir).unwrap_or(path);
                    let name_str = name.to_string_lossy();

                    if path.is_file() {
                        match fs::read(path) {
                            Ok(contents) => {
                                total_size += contents.len() as i64;
                                if let Err(e) = zip.start_file(name_str.as_ref(), options) {
                                    return BackupResult {
                                        success: false,
                                        backup_path: None,
                                        size_bytes: 0,
                                        error: Some(format!("Failed to add file to zip: {}", e)),
                                    };
                                }
                                if let Err(e) = zip.write_all(&contents) {
                                    return BackupResult {
                                        success: false,
                                        backup_path: None,
                                        size_bytes: 0,
                                        error: Some(format!("Failed to write file contents: {}", e)),
                                    };
                                }
                            }
                            Err(e) => {
                                return BackupResult {
                                    success: false,
                                    backup_path: None,
                                    size_bytes: 0,
                                    error: Some(format!("Failed to read file: {}", e)),
                                };
                            }
                        }
                    } else if path.is_dir() && !name_str.is_empty() {
                        let dir_name = format!("{}/", name_str);
                        if let Err(e) = zip.add_directory(&dir_name, options) {
                            return BackupResult {
                                success: false,
                                backup_path: None,
                                size_bytes: 0,
                                error: Some(format!("Failed to add directory to zip: {}", e)),
                            };
                        }
                    }
                }
                Err(e) => {
                    return BackupResult {
                        success: false,
                        backup_path: None,
                        size_bytes: 0,
                        error: Some(format!("Failed to walk directory: {}", e)),
                    };
                }
            }
        }

        match zip.finish() {
            Ok(_) => BackupResult {
                success: true,
                backup_path: Some(backup_path.to_string_lossy().to_string()),
                size_bytes: total_size,
                error: None,
            },
            Err(e) => BackupResult {
                success: false,
                backup_path: None,
                size_bytes: 0,
                error: Some(format!("Failed to finalize zip: {}", e)),
            },
        }
    }

    pub fn restore_profile(backup_path: &PathBuf, target_dir: &PathBuf) -> RestoreResult {
        if !backup_path.exists() {
            return RestoreResult {
                success: false,
                error: Some(format!("Backup file does not exist: {:?}", backup_path)),
            };
        }

        let file = match fs::File::open(backup_path) {
            Ok(f) => f,
            Err(e) => {
                return RestoreResult {
                    success: false,
                    error: Some(format!("Failed to open backup file: {}", e)),
                };
            }
        };

        let mut archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(e) => {
                return RestoreResult {
                    success: false,
                    error: Some(format!("Failed to read zip archive: {}", e)),
                };
            }
        };

        if let Err(e) = fs::create_dir_all(target_dir) {
            return RestoreResult {
                success: false,
                error: Some(format!("Failed to create target directory: {}", e)),
            };
        }

        for i in 0..archive.len() {
            let mut file = match archive.by_index(i) {
                Ok(f) => f,
                Err(e) => {
                    return RestoreResult {
                        success: false,
                        error: Some(format!("Failed to read file from archive: {}", e)),
                    };
                }
            };

            let outpath = target_dir.join(file.name());

            if file.name().ends_with('/') {
                if let Err(e) = fs::create_dir_all(&outpath) {
                    return RestoreResult {
                        success: false,
                        error: Some(format!("Failed to create directory: {}", e)),
                    };
                }
            } else {
                if let Some(parent) = outpath.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        return RestoreResult {
                            success: false,
                            error: Some(format!("Failed to create parent directory: {}", e)),
                        };
                    }
                }

                let mut outfile = match fs::File::create(&outpath) {
                    Ok(f) => f,
                    Err(e) => {
                        return RestoreResult {
                            success: false,
                            error: Some(format!("Failed to create output file: {}", e)),
                        };
                    }
                };

                if let Err(e) = io::copy(&mut file, &mut outfile) {
                    return RestoreResult {
                        success: false,
                        error: Some(format!("Failed to write file: {}", e)),
                    };
                }
            }
        }

        RestoreResult {
            success: true,
            error: None,
        }
    }

    pub fn get_directory_size(dir: &PathBuf) -> io::Result<u64> {
        let mut total_size: u64 = 0;

        for entry in WalkDir::new(dir) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                }
            }
        }

        Ok(total_size)
    }

    pub fn format_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }
}
