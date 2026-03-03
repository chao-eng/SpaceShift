use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub data_dir_path: String,
    pub chrome_path: Option<String>,
    pub icon_path: Option<String>,
    pub icon_base64: Option<String>,
    pub tags: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_opened_at: Option<String>,
    pub is_running: bool,
    pub pid: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Backup {
    pub id: String,
    pub profile_id: String,
    pub backup_path: String,
    pub created_at: String,
    pub size_bytes: i64,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(app_data_dir: &PathBuf) -> SqliteResult<Self> {
        let db_path = app_data_dir.join("profiles.db");
        let conn = Connection::open(&db_path)?;
        
        let db = Database { conn };
        db.init_tables()?;
        
        Ok(db)
    }

    fn init_tables(&self) -> SqliteResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                data_dir_path TEXT NOT NULL UNIQUE,
                chrome_path TEXT,
                icon_path TEXT,
                icon_base64 TEXT,
                tags TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_opened_at TEXT,
                is_running INTEGER DEFAULT 0,
                pid INTEGER
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS backups (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                backup_path TEXT NOT NULL,
                created_at TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_profiles_name ON profiles(name)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_profiles_tags ON profiles(tags)",
            [],
        )?;

        // Migrations
        let _ = self.conn.execute("ALTER TABLE profiles ADD COLUMN chrome_path TEXT", []);

        Ok(())
    }

    pub fn create_profile(&self, name: &str, data_dir_path: &str, chrome_path: Option<&str>, icon_base64: Option<&str>, tags: Option<&str>) -> SqliteResult<Profile> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO profiles (id, name, data_dir_path, chrome_path, icon_base64, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (&id, name, data_dir_path, chrome_path, icon_base64, tags, &now, &now),
        )?;

        Ok(Profile {
            id,
            name: name.to_string(),
            data_dir_path: data_dir_path.to_string(),
            chrome_path: chrome_path.map(|s| s.to_string()),
            icon_path: None,
            icon_base64: icon_base64.map(|s| s.to_string()),
            tags: tags.map(|s| s.to_string()),
            created_at: now.clone(),
            updated_at: now,
            last_opened_at: None,
            is_running: false,
            pid: None,
        })
    }

    pub fn get_all_profiles(&self) -> SqliteResult<Vec<Profile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, data_dir_path, chrome_path, icon_path, icon_base64, tags, 
                    created_at, updated_at, last_opened_at, is_running, pid 
             FROM profiles ORDER BY updated_at DESC"
        )?;

        let profiles = stmt.query_map([], |row| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                data_dir_path: row.get(2)?,
                chrome_path: row.get(3)?,
                icon_path: row.get(4)?,
                icon_base64: row.get(5)?,
                tags: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                last_opened_at: row.get(9)?,
                is_running: row.get::<_, i32>(10)? != 0,
                pid: row.get(11)?,
            })
        })?;

        profiles.collect()
    }

    pub fn get_profile_by_id(&self, id: &str) -> SqliteResult<Option<Profile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, data_dir_path, chrome_path, icon_path, icon_base64, tags, 
                    created_at, updated_at, last_opened_at, is_running, pid 
             FROM profiles WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map([id], |row| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                data_dir_path: row.get(2)?,
                chrome_path: row.get(3)?,
                icon_path: row.get(4)?,
                icon_base64: row.get(5)?,
                tags: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                last_opened_at: row.get(9)?,
                is_running: row.get::<_, i32>(10)? != 0,
                pid: row.get(11)?,
            })
        })?;

        rows.next().transpose()
    }

    pub fn update_profile(&self, id: &str, name: Option<&str>, chrome_path: Option<&str>, icon_base64: Option<&str>, tags: Option<&str>) -> SqliteResult<bool> {
        let now = Utc::now().to_rfc3339();
        
        if let Some(name) = name {
            self.conn.execute(
                "UPDATE profiles SET name = ?1, updated_at = ?2 WHERE id = ?3",
                (name, &now, id),
            )?;
        }

        if let Some(path) = chrome_path {
            self.conn.execute(
                "UPDATE profiles SET chrome_path = ?1, updated_at = ?2 WHERE id = ?3",
                (path, &now, id),
            )?;
        }

        if let Some(icon) = icon_base64 {
            self.conn.execute(
                "UPDATE profiles SET icon_base64 = ?1, updated_at = ?2 WHERE id = ?3",
                (icon, &now, id),
            )?;
        }

        if let Some(tags) = tags {
            self.conn.execute(
                "UPDATE profiles SET tags = ?1, updated_at = ?2 WHERE id = ?3",
                (tags, &now, id),
            )?;
        }

        Ok(self.conn.changes() > 0)
    }

    pub fn delete_profile(&self, id: &str) -> SqliteResult<bool> {
        self.conn.execute("DELETE FROM profiles WHERE id = ?1", [id])?;
        Ok(self.conn.changes() > 0)
    }

    pub fn update_profile_status(&self, id: &str, is_running: bool, pid: Option<i32>) -> SqliteResult<bool> {
        let now = Utc::now().to_rfc3339();
        let running = if is_running { 1 } else { 0 };
        
        self.conn.execute(
            "UPDATE profiles SET is_running = ?1, pid = ?2, last_opened_at = ?3, updated_at = ?4 WHERE id = ?5",
            (running, pid, &now, &now, id),
        )?;

        Ok(self.conn.changes() > 0)
    }

    pub fn create_backup(&self, profile_id: &str, backup_path: &str, size_bytes: i64) -> SqliteResult<Backup> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO backups (id, profile_id, backup_path, created_at, size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (&id, profile_id, backup_path, &now, size_bytes),
        )?;

        Ok(Backup {
            id,
            profile_id: profile_id.to_string(),
            backup_path: backup_path.to_string(),
            created_at: now,
            size_bytes,
        })
    }

    pub fn get_backups_by_profile(&self, profile_id: &str) -> SqliteResult<Vec<Backup>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, profile_id, backup_path, created_at, size_bytes 
             FROM backups WHERE profile_id = ?1 ORDER BY created_at DESC"
        )?;

        let backups = stmt.query_map([profile_id], |row| {
            Ok(Backup {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                backup_path: row.get(2)?,
                created_at: row.get(3)?,
                size_bytes: row.get(4)?,
            })
        })?;

        backups.collect()
    }

    pub fn delete_backup(&self, id: &str) -> SqliteResult<bool> {
        self.conn.execute("DELETE FROM backups WHERE id = ?1", [id])?;
        Ok(self.conn.changes() > 0)
    }

    pub fn search_profiles(&self, query: &str) -> SqliteResult<Vec<Profile>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, name, data_dir_path, chrome_path, icon_path, icon_base64, tags, 
                    created_at, updated_at, last_opened_at, is_running, pid 
             FROM profiles 
             WHERE name LIKE ?1 OR tags LIKE ?1
             ORDER BY updated_at DESC"
        )?;

        let profiles = stmt.query_map([&search_pattern], |row| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                data_dir_path: row.get(2)?,
                chrome_path: row.get(3)?,
                icon_path: row.get(4)?,
                icon_base64: row.get(5)?,
                tags: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                last_opened_at: row.get(9)?,
                is_running: row.get::<_, i32>(10)? != 0,
                pid: row.get(11)?,
            })
        })?;

        profiles.collect()
    }

    pub fn get_profiles_by_tag(&self, tag: &str) -> SqliteResult<Vec<Profile>> {
        let search_pattern = format!("%{}%", tag);
        let mut stmt = self.conn.prepare(
            "SELECT id, name, data_dir_path, chrome_path, icon_path, icon_base64, tags, 
                    created_at, updated_at, last_opened_at, is_running, pid 
             FROM profiles 
             WHERE tags LIKE ?1
             ORDER BY updated_at DESC"
        )?;

        let profiles = stmt.query_map([&search_pattern], |row| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                data_dir_path: row.get(2)?,
                chrome_path: row.get(3)?,
                icon_path: row.get(4)?,
                icon_base64: row.get(5)?,
                tags: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                last_opened_at: row.get(9)?,
                is_running: row.get::<_, i32>(10)? != 0,
                pid: row.get(11)?,
            })
        })?;

        profiles.collect()
    }
}
