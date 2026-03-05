<template>
  <div
    class="profile-card"
    :class="{ 'is-launching': props.isLaunching, 'is-running': profile.is_running }"
    @click="$emit('click', profile)"
  >
    <!-- 状态指示条 -->
    <div class="status-bar"></div>
    
    <div class="profile-card-content">
      <!-- 头像区域 -->
      <div class="profile-avatar-wrapper">
        <div class="profile-avatar">
          <img v-if="profile.icon_base64" :src="profile.icon_base64" :alt="profile.name" />
          <el-icon v-else class="avatar-icon"><UserFilled /></el-icon>
        </div>
      </div>
      
      <!-- 信息区域 -->
      <div class="profile-info">
        <h3 class="profile-name" :title="profile.name">{{ profile.name }}</h3>
        <div class="profile-tags" v-if="tagList.length > 0">
          <span
            v-for="tag in tagList.slice(0, 3)"
            :key="tag"
            class="profile-tag"
          >
            {{ tag }}
          </span>
        </div>
        <div class="profile-meta">
          <span v-if="profile.is_running" class="status-tag running">
            <span class="status-dot"></span>
            {{ $t('profile.status.running') }}
          </span>
          <span v-else-if="profile.last_opened_at" class="last-opened">
            <el-icon><Timer /></el-icon>
            <span>{{ formatDate(profile.last_opened_at) }}</span>
          </span>
          <span v-else class="last-opened never">
            <el-icon><Timer /></el-icon>
            <span>{{ $t('profile.status.unused') }}</span>
          </span>
          <span v-if="profile.homepage" class="homepage-indicator" :title="profile.homepage">
            <el-icon><Link /></el-icon>
          </span>
        </div>
      </div>
      
      <!-- 操作区域 -->
      <div class="profile-actions">
        <el-tooltip
          :content="$t('profile.actions.launch')"
          placement="top"
          :show-after="300"
        >
          <button
            :class="['action-btn', 'launch-btn', { 'is-loading': props.isLaunching }]"
            @click.stop="handleLaunch"
            :disabled="props.isLaunching"
          >
            <el-icon v-if="props.isLaunching" class="loading-icon"><Loading /></el-icon>
            <el-icon v-else><VideoPlay /></el-icon>
          </button>
        </el-tooltip>
        
        <el-dropdown trigger="click" @command="handleCommand" @click.stop>
          <button class="action-btn more-btn" @click.stop>
            <el-icon><MoreFilled /></el-icon>
          </button>
          <template #dropdown>
            <el-dropdown-menu class="profile-dropdown-menu">
              <el-dropdown-item command="edit">
                <el-icon><Edit /></el-icon>
                <span>{{ $t('profile.actions.edit') }}</span>
              </el-dropdown-item>
              <el-dropdown-item command="performance">
                <el-icon><DataLine /></el-icon>
                <span>{{ $t('profile.actions.performance') }}</span>
              </el-dropdown-item>
              <el-dropdown-item command="backup">
                <el-icon><DocumentCopy /></el-icon>
                <span>{{ $t('profile.actions.backup') }}</span>
              </el-dropdown-item>
              <el-dropdown-item command="openDir">
                <el-icon><FolderOpened /></el-icon>
                <span>{{ $t('profile.actions.openDir') }}</span>
              </el-dropdown-item>
              <el-dropdown-item divided command="delete" class="danger-item">
                <el-icon><Delete /></el-icon>
                <span>{{ $t('profile.actions.delete') }}</span>
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { 
  UserFilled, 
  Timer, 
  Edit, 
  Delete, 
  DocumentCopy, 
  FolderOpened,
  VideoPlay,
  MoreFilled,
  Loading,
  Link,
  DataLine
} from '@element-plus/icons-vue';
import type { Profile } from '../types';

const props = defineProps<{
  profile: Profile;
  isLaunching?: boolean;
}>();

const emit = defineEmits<{
  click: [profile: Profile];
  launch: [profile: Profile];
  edit: [profile: Profile];
  backup: [profile: Profile];
  delete: [profile: Profile];
  openDir: [profile: Profile];
  performance: [profile: Profile];
}>();

const { t, locale } = useI18n();

const tagList = computed(() => {
  return props.profile.tags?.split(',').map(t => t.trim()).filter(Boolean) || [];
});

const formatDate = (dateStr: string) => {
  const date = new Date(dateStr);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);
  
  if (minutes < 1) return t('profile.status.justNow');
  if (minutes < 60) return t('profile.status.minutesAgo', { n: minutes });
  if (hours < 24) return t('profile.status.hoursAgo', { n: hours });
  if (days < 30) return t('profile.status.daysAgo', { n: days });
  
  return date.toLocaleDateString(locale.value === 'zh' ? 'zh-CN' : 'en-US', { month: 'short', day: 'numeric' });
};

const handleLaunch = async () => {
  if (props.isLaunching) {
    return;
  }
  emit('launch', props.profile);
};

const handleCommand = (command: string) => {
  switch (command) {
    case 'edit':
      emit('edit', props.profile);
      break;
    case 'backup':
      emit('backup', props.profile);
      break;
    case 'openDir':
      emit('openDir', props.profile);
      break;
    case 'delete':
      emit('delete', props.profile);
      break;
    case 'performance':
      emit('performance', props.profile);
      break;
  }
};
</script>

<style scoped lang="scss">
// 飞书风格 ProfileCard
.profile-card {
  position: relative;
  background: var(--bg-primary);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-light);
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;

  &:hover {
    border-color: var(--primary-300);
    box-shadow: var(--shadow-md);
    transform: translateY(-1px);

    .status-bar {
      opacity: 1;
    }
  }

  &:active {
    transform: translateY(0);
    box-shadow: var(--shadow-sm);
  }

  // 启动中状态
  &.is-launching {
    pointer-events: none;
    opacity: 0.8;

    .launch-btn {
      background: var(--primary-50);
      color: var(--primary-500);
    }
  }

  // 运行中状态
  &.is-running {
    border-color: var(--primary-400);
    
    .status-bar {
      opacity: 1;
    }
  }
}

// 顶部状态条
.status-bar {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--primary-500);
  opacity: 0;
  transition: opacity 0.2s ease;
}

// 状态标签
.status-tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: var(--font-medium);
  
  &.running {
    color: var(--primary-600);
    
    .status-dot {
      width: 6px;
      height: 6px;
      border-radius: 50%;
      background: var(--primary-500);
      box-shadow: 0 0 4px var(--primary-300);
      animation: pulse 2s infinite;
    }
  }
}

@keyframes pulse {
  0% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(var(--primary-500-rgb, 0, 100, 250), 0.4); }
  70% { transform: scale(1); box-shadow: 0 0 0 6px rgba(var(--primary-500-rgb, 0, 100, 250), 0); }
  100% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(var(--primary-500-rgb, 0, 100, 250), 0); }
}

// 卡片内容
.profile-card-content {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4);
}

// 头像包装器
.profile-avatar-wrapper {
  position: relative;
  flex-shrink: 0;
}

// 头像
.profile-avatar {
  width: 52px;
  height: 52px;
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: var(--gray-100);
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-light);
  transition: all 0.2s ease;

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-icon {
    font-size: 24px;
    color: var(--gray-400);
  }
}

// 信息区域
.profile-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

// 配置名称
.profile-name {
  margin: 0;
  font-size: var(--text-base);
  font-weight: var(--font-semibold);
  color: var(--text-primary);
  line-height: 1.4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

// 标签区域
.profile-tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
}

.profile-tag {
  font-size: 12px;
  font-weight: var(--font-medium);
  color: var(--primary-600);
  background: var(--primary-50);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--primary-100);
  line-height: 1.4;
}

// 元信息
.profile-meta {
  font-size: 12px;
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--space-1);
}

.last-opened {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);

  .el-icon {
    font-size: 13px;
  }

  &.never {
    color: var(--text-quaternary);
  }
}

.homepage-indicator {
  display: flex;
  align-items: center;
  margin-left: var(--space-1);
  color: var(--primary-400);
  
  .el-icon {
    font-size: 14px;
    cursor: help;
  }
}

// 操作区域
.profile-actions {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  align-items: center;
}

// 操作按钮基础样式
.action-btn {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border-medium);
  background: var(--bg-primary);
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  outline: none;

  .el-icon {
    font-size: 16px;
  }

  &:hover {
    border-color: var(--primary-300);
    color: var(--primary-600);
    background: var(--primary-50);
  }

  &:active {
    transform: scale(0.95);
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
}

// 启动按钮
.launch-btn {
  background: var(--primary-500);
  color: white;
  border-color: var(--primary-500);

  &:hover:not(:disabled) {
    background: var(--primary-600);
    border-color: var(--primary-600);
    color: white;
  }

  &:active:not(:disabled) {
    background: var(--primary-700);
  }

  &.is-loading {
    background: var(--primary-50);
    color: var(--primary-500);
    border-color: var(--primary-200);
  }

  .loading-icon {
    animation: spin 1s linear infinite;
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

// 更多按钮
.more-btn {
  &:hover {
    transform: rotate(90deg);
  }
}

// 下拉菜单样式
:deep(.profile-dropdown-menu) {
  padding: var(--space-2);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-lg);
  min-width: 140px;

  .el-dropdown-menu__item {
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: var(--space-2);
    transition: all 0.15s ease;

    .el-icon {
      font-size: 14px;
      color: var(--text-tertiary);
    }

    &:hover {
      background: var(--gray-100);
      color: var(--text-primary);

      .el-icon {
        color: var(--text-secondary);
      }
    }

    &.danger-item {
      color: var(--danger-color);

      .el-icon {
        color: var(--danger-color);
      }

      &:hover {
        background: var(--danger-light);
      }
    }
  }

  .el-dropdown-menu__item--divided {
    margin: var(--space-2) 0;
    border-top: 1px solid var(--border-light);

    &::before {
      display: none;
    }
  }
}

// 列表视图适配
:deep(.list-view) {
  .profile-card-content {
    padding: var(--space-3) var(--space-4);
  }

  .profile-avatar {
    width: 48px;
    height: 48px;
  }
}

// 响应式适配
@media (max-width: 640px) {
  .profile-card-content {
    padding: var(--space-3);
    gap: var(--space-2);
  }

  .profile-avatar {
    width: 44px;
    height: 44px;

    .avatar-icon {
      font-size: 20px;
    }
  }

  .profile-name {
    font-size: var(--text-sm);
  }

  .profile-tag {
    font-size: 11px;
    padding: 1px 6px;
  }

  .profile-meta {
    font-size: 11px;
  }

  .last-opened .el-icon {
    font-size: 12px;
  }

  .action-btn {
    width: 32px;
    height: 32px;

    .el-icon {
      font-size: 14px;
    }
  }
}
</style>
