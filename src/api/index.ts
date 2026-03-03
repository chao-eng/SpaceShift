import { invoke } from '@tauri-apps/api/core';
import type { Profile, Backup, ChromeLaunchResult, BackupResult, RestoreResult } from '../types';

export const api = {
  // Profile operations
  getProfiles: (): Promise<Profile[]> => invoke('get_profiles'),

  getProfile: (id: string): Promise<Profile | null> => invoke('get_profile', { id }),

  createProfile: (name: string, chromePath?: string, iconBase64?: string, tags?: string): Promise<Profile> =>
    invoke('create_profile', { name, chromePath, iconBase64, tags }),

  updateProfile: (id: string, name?: string, chromePath?: string, iconBase64?: string, tags?: string): Promise<boolean> =>
    invoke('update_profile', { id, name, chromePath, iconBase64, tags }),

  deleteProfile: (id: string): Promise<boolean> => invoke('delete_profile', { id }),

  // Chrome operations
  launchChrome: (id: string, url?: string): Promise<ChromeLaunchResult> =>
    invoke('launch_chrome', { id, url }),

  // Backup operations
  backupProfile: (id: string, backupDir: string): Promise<BackupResult> =>
    invoke('backup_profile', { id, backupDir }),

  restoreProfile: (id: string, backupPath: string): Promise<RestoreResult> =>
    invoke('restore_profile', { id, backupPath }),

  getBackups: (id: string): Promise<Backup[]> => invoke('get_backups', { id }),

  deleteBackup: (id: string): Promise<boolean> => invoke('delete_backup', { id }),

  // Search operations
  searchProfiles: (query: string): Promise<Profile[]> => invoke('search_profiles', { query }),

  getProfilesByTag: (tag: string): Promise<Profile[]> => invoke('get_profiles_by_tag', { tag }),

  // Utility
  getProfileSize: (id: string): Promise<string> => invoke('get_profile_size', { id }),

  // Directory operations
  getAppDataDir: (): Promise<string> => invoke('get_app_data_dir'),

  openProfileDirectory: (profileDataDir: string): Promise<void> =>
    invoke('open_profile_directory', { profileDataDir }),
};
