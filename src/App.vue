<template>
  <div class="app-container">
    <!-- Feishu-style Header -->
    <header class="app-header">
      <div class="header-left">
        <div class="brand">
          <div class="brand-icon">
            <el-icon><ChromeFilled /></el-icon>
          </div>
          <div class="brand-text">
            <h1 class="brand-title">SpaceShift</h1>
            <span class="brand-subtitle">Chrome 配置管理</span>
          </div>
        </div>
      </div>

      <div class="header-center">
        <el-input
          v-model="searchQuery"
          placeholder="搜索配置..."
          class="search-input"
          clearable
          @input="handleSearch"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
      </div>

      <div class="header-right">
        <el-radio-group v-model="viewMode" size="small" class="view-toggle">
          <el-radio-button label="grid">
            <el-icon><Grid /></el-icon>
          </el-radio-button>
          <el-radio-button label="list">
            <el-icon><List /></el-icon>
          </el-radio-button>
        </el-radio-group>

        <el-button type="primary" @click="handleCreate" class="create-btn">
          <el-icon><Plus /></el-icon>
          <span>新建配置</span>
        </el-button>
      </div>
    </header>

    <!-- Main Content -->
    <main class="app-main">
      <div v-if="loading" class="loading-container">
        <el-skeleton :rows="4" animated />
      </div>

      <el-empty
        v-else-if="filteredProfiles.length === 0"
        :description="searchQuery ? '未找到匹配的配置' : '暂无配置，点击右上角创建'"
        class="empty-state"
      >
        <el-button v-if="!searchQuery" type="primary" @click="handleCreate">
          创建第一个配置
        </el-button>
      </el-empty>

      <div
        v-else
        :class="['profiles-container', viewMode === 'grid' ? 'grid-view' : 'list-view']"
      >
        <ProfileCard
          v-for="(profile, index) in filteredProfiles"
          :key="profile.id"
          :profile="profile"
          :is-launching="launchingProfiles.has(profile.id)"
          :style="{ animationDelay: `${index * 50}ms` }"
          @click="handleCardClick"
          @launch="handleLaunch"
          @edit="handleEdit"
          @backup="handleBackup"
          @delete="handleDelete"
          @openDir="handleOpenDir"
        />
      </div>
    </main>

    <ProfileForm
      v-model="showForm"
      :profile="editingProfile"
      :existing-tags="existingTags"
      @success="loadProfiles"
    />

    <BackupDialog
      v-model="showBackupDialog"
      :profile="selectedProfile"
      @success="loadProfiles"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import {
  ChromeFilled,
  Search,
  Grid,
  List,
  Plus,
} from '@element-plus/icons-vue';
import ProfileCard from './components/ProfileCard.vue';
import ProfileForm from './components/ProfileForm.vue';
import BackupDialog from './components/BackupDialog.vue';
import type { Profile, ViewMode } from './types';
import { api } from './api';

const profiles = ref<Profile[]>([]);
const loading = ref(false);
const searchQuery = ref('');
const viewMode = ref<ViewMode>('grid');
const showForm = ref(false);
const showBackupDialog = ref(false);
const editingProfile = ref<Profile | null>(null);
const selectedProfile = ref<Profile | null>(null);

const launchingProfiles = ref<Set<string>>(new Set());

const handleOpenDir = async (profile: Profile) => {
  try {
    await api.openProfileDirectory(profile.data_dir_path);
  } catch (error) {
    ElMessage.error('无法打开目录');
    console.error(error);
  }
};

const filteredProfiles = computed(() => {
  if (!searchQuery.value) return profiles.value;

  const query = searchQuery.value.toLowerCase();
  return profiles.value.filter(
    p =>
      p.name.toLowerCase().includes(query) ||
      p.tags?.toLowerCase().includes(query)
  );
});

const existingTags = computed(() => {
  const tags = new Set<string>();
  profiles.value.forEach(p => {
    p.tags?.split(',').forEach(t => {
      const tag = t.trim();
      if (tag) tags.add(tag);
    });
  });
  return Array.from(tags);
});

const loadProfiles = async () => {
  loading.value = true;
  try {
    profiles.value = await api.getProfiles();
  } catch (error) {
    ElMessage.error('加载配置失败');
    console.error(error);
  } finally {
    loading.value = false;
  }
};

const handleSearch = () => {
  // Debounce could be added here
};

const handleCreate = () => {
  editingProfile.value = null;
  showForm.value = true;
};

const handleCardClick = (_profile: Profile) => {
  // Optional: Show profile details
  console.log('Card clicked:', _profile.name);
};

const handleLaunch = async (profile: Profile) => {
  if (launchingProfiles.value.has(profile.id)) {
    return;
  }

  try {
    launchingProfiles.value.add(profile.id);
    const result = await api.launchChrome(profile.id);

    if (result.success) {
      ElMessage.success(`已启动: ${profile.name}`);
      const profileIndex = profiles.value.findIndex(p => p.id === profile.id);
      if (profileIndex !== -1) {
        profiles.value[profileIndex].is_running = true;
        profiles.value[profileIndex].pid = result.pid ? Number(result.pid) : undefined;
        profiles.value[profileIndex].last_opened_at = new Date().toISOString();
      }
      await loadProfiles();
    } else {
      ElMessage.error(result.error || '启动失败');
    }
  } catch (error) {
    ElMessage.error('启动失败');
    console.error(error);
  } finally {
    launchingProfiles.value.delete(profile.id);
  }
};

const handleEdit = (profile: Profile) => {
  editingProfile.value = profile;
  showForm.value = true;
};

const handleBackup = (profile: Profile) => {
  selectedProfile.value = profile;
  showBackupDialog.value = true;
};

const handleDelete = async (profile: Profile) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除配置 "${profile.name}" 吗？此操作将删除所有相关数据，不可恢复。`,
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    await api.deleteProfile(profile.id);
    ElMessage.success('配置已删除');
    await loadProfiles();
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败');
      console.error(error);
    }
  }
};

onMounted(() => {
  loadProfiles();
});
</script>

<style lang="scss">
// Feishu-style App Container
.app-container {
  min-height: 100vh;
  background: var(--bg-secondary);
}

// Feishu-style Header
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
    width: 36px;
    height: 36px;
    border-radius: var(--radius-lg);
    background: linear-gradient(135deg, var(--primary-500), var(--primary-600));
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-size: 20px;
    box-shadow: var(--shadow-sm);
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
  gap: var(--space-3);
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

// Main Content Area
.app-main {
  padding: var(--space-5);
  max-width: 1400px;
  margin: 0 auto;
  min-height: calc(100vh - 60px);
}

.loading-container {
  padding: var(--space-10);
}

.empty-state {
  padding: var(--space-16) 0;

  :deep(.el-empty__image) {
    width: 100px;
    height: 100px;
  }

  :deep(.el-empty__description) {
    margin-top: var(--space-4);
    margin-bottom: var(--space-4);
  }
}

// Profile Grid Layout
.profiles-container {
  display: grid;
  gap: var(--space-4);

  &.grid-view {
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  }

  &.list-view {
    grid-template-columns: 1fr;
    max-width: 800px;
    margin: 0 auto;
  }

  > * {
    animation: fadeInUp 0.4s ease-out backwards;
  }
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

// Responsive Design
@media (max-width: 1024px) {
  .app-main {
    padding: var(--space-4);
  }

  .profiles-container.grid-view {
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: var(--space-3);
  }

  .header-center {
    max-width: 300px;
    margin: 0 var(--space-4);
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

  .app-main {
    padding: var(--space-3);
  }

  .profiles-container.grid-view {
    grid-template-columns: 1fr;
    gap: var(--space-3);
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
