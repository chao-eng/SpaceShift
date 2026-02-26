export interface Profile {
  id: string;
  name: string;
  data_dir_path: string;
  icon_path?: string;
  icon_base64?: string;
  tags?: string;
  created_at: string;
  updated_at: string;
  last_opened_at?: string;
  is_running: boolean;
  pid?: number;
}

export interface Backup {
  id: string;
  profile_id: string;
  backup_path: string;
  created_at: string;
  size_bytes: number;
}

export interface ChromeLaunchResult {
  success: boolean;
  pid?: number;
  error?: string;
}

export interface BackupResult {
  success: boolean;
  backup_path?: string;
  size_bytes: number;
  error?: string;
}

export interface RestoreResult {
  success: boolean;
  error?: string;
}

export type ViewMode = 'grid' | 'list';

export type BrowserType = 'chrome' | 'firefox' | 'edge' | 'safari' | 'opera' | 'brave' | 'unknown';

export interface BrowserProcess {
  pid: number;
  name: string;
  browser_type: BrowserType;
  command?: string;
  user_data_dir?: string;
}

export interface KillBrowserResult {
  success: boolean;
  killed_count: number;
  failed_pids: number[];
  message: string;
}

export interface BrowserSafetyCheck {
  is_safe_to_kill: boolean;
  warnings: string[];
  browser_type: BrowserType;
  process_count: number;
}
