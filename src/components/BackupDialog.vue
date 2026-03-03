<template>
  <el-dialog
    v-model="visible"
    title="备份管理"
    width="600px"
    destroy-on-close
  >
    <div v-loading="isBackingUp" element-loading-text="正在打包配置文件，请稍候..." class="backup-container">
      <div class="backup-actions">
        <el-button @click="handleOpenBackupDir">
          <el-icon><FolderOpened /></el-icon>
          查看目录
        </el-button>
        <el-button type="primary" @click="handleBackup" :loading="isBackingUp">
          <el-icon><DocumentCopy /></el-icon>
          {{ isBackingUp ? '正在备份...' : '立即备份' }}
        </el-button>
      </div>

      <el-divider />

      <div class="backup-list">
        <el-empty v-if="backups.length === 0" description="暂无备份" />
        
        <el-timeline v-else>
          <el-timeline-item
            v-for="backup in backups"
            :key="backup.id"
            :timestamp="formatDate(backup.created_at)"
            placement="top"
          >
            <el-card>
              <div class="backup-item">
                <div class="backup-info">
                  <el-icon class="backup-icon"><Document /></el-icon>
                  <div class="backup-details">
                    <span class="backup-name">{{ getBackupName(backup.backup_path) }}</span>
                    <span class="backup-size">{{ formatSize(backup.size_bytes) }}</span>
                  </div>
                </div>
                <div class="backup-actions-right">
                  <el-button
                    type="primary"
                    link
                    @click="handleRestore(backup)"
                    :loading="restoringId === backup.id"
                  >
                    恢复
                  </el-button>
                  <el-button
                    type="danger"
                    link
                    @click="handleDelete(backup)"
                  >
                    删除
                  </el-button>
                </div>
              </div>
            </el-card>
          </el-timeline-item>
        </el-timeline>
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { DocumentCopy, Document, FolderOpened } from '@element-plus/icons-vue';
import type { Profile, Backup } from '../types';
import { api } from '../api';
import { homeDir, join } from '@tauri-apps/api/path';

const props = defineProps<{
  modelValue: boolean;
  profile: Profile | null;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  success: [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const backups = ref<Backup[]>([]);
const isBackingUp = ref(false);
const restoringId = ref<string | null>(null);

const loadBackups = async () => {
  if (!props.profile) return;
  
  try {
    backups.value = await api.getBackups(props.profile.id);
  } catch (error) {
    console.error('Failed to load backups:', error);
  }
};

watch(
  () => props.modelValue,
  (show) => {
    if (show) {
      loadBackups();
    }
  }
);

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleString('zh-CN');
};

const formatSize = (bytes: number) => {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;
  
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  
  return `${size.toFixed(2)} ${units[unitIndex]}`;
};

const getBackupName = (path: string) => {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1];
};

const handleBackup = async () => {
  if (!props.profile) return;
  
  isBackingUp.value = true;
  
  try {
    // Use Downloads folder as default backup location
    const backupDir = await getDefaultBackupDir();
    const result = await api.backupProfile(props.profile.id, backupDir);
    
    if (result.success) {
      ElMessage.success('备份成功');
      await loadBackups();
      emit('success');
    } else {
      ElMessage.error(result.error || '备份失败');
    }
  } catch (error) {
    ElMessage.error('备份失败');
    console.error(error);
  } finally {
    isBackingUp.value = false;
  }
};

const getDefaultBackupDir = async (): Promise<string> => {
  const home = await homeDir();
  return await join(home, 'SpaceShift', 'Backups');
};

const handleOpenBackupDir = async () => {
  try {
    const backupDir = await getDefaultBackupDir();
    await api.openProfileDirectory(backupDir);
  } catch (error) {
    ElMessage.error('无法打开目录');
    console.error(error);
  }
};

const handleRestore = async (backup: Backup) => {
  if (!props.profile) return;
  
  try {
    await ElMessageBox.confirm(
      '恢复备份将覆盖当前配置的所有数据，是否继续？',
      '确认恢复',
      {
        confirmButtonText: '恢复',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );
    
    restoringId.value = backup.id;
    
    const result = await api.restoreProfile(props.profile.id, backup.backup_path);
    
    if (result.success) {
      ElMessage.success('恢复成功');
      emit('success');
    } else {
      ElMessage.error(result.error || '恢复失败');
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('恢复失败');
      console.error(error);
    }
  } finally {
    restoringId.value = null;
  }
};

const handleDelete = async (backup: Backup) => {
  try {
    await ElMessageBox.confirm(
      '确定要删除这个备份吗？此操作不可恢复。',
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );
    
    await api.deleteBackup(backup.id);
    ElMessage.success('备份已删除');
    await loadBackups();
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败');
      console.error(error);
    }
  }
};
</script>

<style scoped lang="scss">
:deep(.el-dialog__header) {
  background: linear-gradient(135deg, var(--secondary-500) 0%, var(--primary-500) 100%);
  padding: var(--space-5) var(--space-6);
  margin-right: 0;
  border-radius: var(--radius-2xl) var(--radius-2xl) 0 0;
}

:deep(.el-dialog__title) {
  color: white;
  font-weight: var(--font-semibold);
  font-size: var(--text-lg);
}

:deep(.el-dialog__headerbtn) {
  top: 50%;
  transform: translateY(-50%);

  .el-dialog__close {
    color: white;
    font-size: 20px;

    &:hover {
      color: rgba(255, 255, 255, 0.8);
    }
  }
}

:deep(.el-dialog__body) {
  padding: var(--space-6);
}

:deep(.el-dialog__footer) {
  padding: var(--space-4) var(--space-6);
  border-top: 1px solid var(--border-light);
}

.backup-actions {
  display: flex;
  justify-content: flex-end;
  margin-bottom: var(--space-5);

  .el-button {
    border-radius: var(--radius-lg);
    font-weight: var(--font-medium);
    padding: var(--space-3) var(--space-5);
  }
}

.backup-list {
  max-height: 400px;
  overflow-y: auto;

  &::-webkit-scrollbar {
    width: 6px;
  }

  &::-webkit-scrollbar-thumb {
    background: var(--border-medium);
    border-radius: var(--radius-full);
  }
}

.backup-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-4);
  border-radius: var(--radius-lg);
  transition: all var(--transition-fast);
  margin-bottom: var(--space-3);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);

  &:hover {
    background: var(--primary-50);
    border-color: var(--primary-200);
    transform: translateX(4px);
  }

  &:last-child {
    margin-bottom: 0;
  }
}

.backup-info {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.backup-icon {
  font-size: 28px;
  color: var(--primary-500);
  background: var(--primary-50);
  padding: var(--space-3);
  border-radius: var(--radius-lg);
}

.backup-details {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.backup-name {
  font-weight: var(--font-semibold);
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.backup-size {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.backup-actions-right {
  display: flex;
  gap: var(--space-2);

  .el-button {
    border-radius: var(--radius-md);
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;

  .el-button {
    padding: var(--space-3) var(--space-6);
    border-radius: var(--radius-lg);
    font-weight: var(--font-medium);
  }
}
</style>
