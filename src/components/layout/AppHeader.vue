<template>
  <header class="app-header">
    <div class="header-left">
      <div class="brand">
        <div class="brand-icon">
          <img src="../../assets/logo.svg" alt="SpaceShift logo" class="logo-image" />
        </div>
        <div class="brand-text">
          <h1 class="brand-title">SpaceShift</h1>
          <span class="brand-subtitle">{{ $t('profile.status.running') }}</span>
        </div>
      </div>
    </div>

    <div class="header-center">
      <el-input
        v-model="profileStore.searchQuery"
        :placeholder="$t('header.search')"
        class="search-input"
        clearable
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
    </div>

    <div class="header-right">
      <!-- 语言切换 -->
      <el-dropdown trigger="click" @command="handleLanguageChange">
        <button class="icon-btn">
          <el-icon><i class="el-icon-language"></i><span class="lang-text">{{ currentLocale === 'zh' ? '中' : 'EN' }}</span></el-icon>
        </button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="zh" :disabled="currentLocale === 'zh'">简体中文</el-dropdown-item>
            <el-dropdown-item command="en" :disabled="currentLocale === 'en'">English</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

      <el-tooltip :content="$t('header.settings')" placement="bottom" :show-after="300">
        <button class="icon-btn" @click="handleOpenGithub">
          <svg height="20" width="20" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z"></path>
          </svg>
        </button>
      </el-tooltip>

      <el-radio-group v-model="profileStore.viewMode" size="small" class="view-toggle">
        <el-tooltip :content="$t('main.gridView')" placement="bottom">
          <el-radio-button label="grid">
            <el-icon><Grid /></el-icon>
          </el-radio-button>
        </el-tooltip>
        <el-tooltip :content="$t('main.listView')" placement="bottom">
          <el-radio-button label="list">
            <el-icon><List /></el-icon>
          </el-radio-button>
        </el-tooltip>
      </el-radio-group>

      <el-button type="primary" @click="$emit('create')" class="create-btn">
        <el-icon><Plus /></el-icon>
        <span>{{ $t('header.newProfile') }}</span>
      </el-button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Search, Grid, List, Plus } from '@element-plus/icons-vue';
import { openUrl } from '@tauri-apps/plugin-opener';
import { ElMessage } from 'element-plus';
import { useProfileStore } from '../../store/profile';

defineEmits(['create']);
const profileStore = useProfileStore();
const { locale } = useI18n();

const currentLocale = computed(() => locale.value);

const handleLanguageChange = (lang: string) => {
  locale.value = lang;
};

const handleOpenGithub = async () => {
  try {
    await openUrl('https://github.com/chao-eng/SpaceShift');
  } catch (error) {
    ElMessage.error('无法打开浏览器');
    console.error(error);
  }
};
</script>

<style lang="scss" scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--space-5);
  height: 60px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border-light);
  position: sticky;
  top: 0;
  z-index: var(--z-sticky);
  box-shadow: var(--shadow-xs);
}

.header-left {
  display: flex;
  align-items: center;
}

.brand {
  display: flex;
  align-items: center;
  gap: var(--space-3);

  &-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.25);
    transition: all 0.3s ease;

    &:hover {
      transform: scale(1.05);
      box-shadow: 0 6px 16px rgba(59, 130, 246, 0.35);
    }

    .logo-image {
      width: 100%;
      height: 100%;
      object-fit: contain;
    }
  }

  &-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  &-title {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    line-height: 1.2;
    margin: 0;
  }

  &-subtitle {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    line-height: 1.2;
  }
}

.header-center {
  flex: 1;
  max-width: 400px;
  margin: 0 var(--space-6);
}

.search-input {
  width: 100%;

  :deep(.el-input__wrapper) {
    border-radius: var(--radius-md);
    background: var(--bg-secondary);
    border: 1px solid transparent;
    box-shadow: none;
    transition: all var(--transition-fast);

    &:hover {
      background: var(--bg-tertiary);
    }

    &.is-focus {
      background: var(--bg-primary);
      border-color: var(--primary-400);
      box-shadow: 0 0 0 2px var(--primary-100);
    }
  }

  :deep(.el-input__inner) {
    font-size: var(--text-sm);
    color: var(--text-primary);

    &::placeholder {
      color: var(--text-tertiary);
    }
  }

  :deep(.el-input__prefix) {
    color: var(--text-tertiary);
    margin-right: var(--space-2);
  }
}

.header-right {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 50%;
  transition: all 0.2s ease;

  .lang-text {
    font-size: 11px;
    font-weight: var(--font-bold);
    font-family: var(--font-mono, monospace);
  }

  &:hover {
    background: var(--gray-100);
    color: var(--text-primary);
  }
  
  &:active {
    transform: scale(0.95);
  }
}

.view-toggle {
  :deep(.el-radio-button__inner) {
    padding: 6px 12px;
    font-size: var(--text-sm);

    .el-icon {
      font-size: 16px;
    }
  }
}

.create-btn {
  height: 34px;
  padding: 0 16px;
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  display: flex;
  align-items: center;
  gap: var(--space-1);

  .el-icon {
    font-size: 16px;
  }
}

@media (max-width: 768px) {
  .app-header {
    flex-wrap: wrap;
    height: auto;
    padding: var(--space-3);
    gap: var(--space-3);
  }

  .brand-subtitle {
    display: none;
  }

  .header-center {
    order: 3;
    width: 100%;
    max-width: none;
    margin: 0;
  }

  .header-right {
    margin-left: auto;
  }
}

@media (max-width: 480px) {
  .brand-icon {
    width: 32px;
    height: 32px;
    font-size: 18px;
  }

  .brand-title {
    font-size: var(--text-sm);
  }

  .create-btn span {
    display: none;
  }

  .create-btn {
    padding: 0 10px;
  }
}
</style>
