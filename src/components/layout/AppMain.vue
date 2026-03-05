<template>
  <main class="app-main">
    <div v-if="profileStore.loading" class="loading-container">
      <el-skeleton :rows="4" animated />
    </div>

    <el-empty
      v-else-if="profileStore.filteredProfiles.length === 0"
      :description="profileStore.searchQuery ? $t('common.error') : $t('main.noProfiles')"
      class="empty-state"
    >
      <el-button v-if="!profileStore.searchQuery" type="primary" @click="$emit('create')">
        {{ $t('header.newProfile') }}
      </el-button>
    </el-empty>

    <div
      v-else
      :class="['profiles-container', profileStore.viewMode === 'grid' ? 'grid-view' : 'list-view']"
    >
      <ProfileCard
        v-for="(profile, index) in profileStore.filteredProfiles"
        :key="profile.id"
        :profile="profile"
        :is-launching="profileStore.launchingProfiles.has(profile.id)"
        :is-selected="profileStore.selectedIds.has(profile.id)"
        :style="{ animationDelay: `${index * 50}ms` }"
        @launch="profileStore.handleLaunch"
        @toggleSelection="profileStore.toggleSelection"
        @edit="$emit('edit', profile)"
        @backup="$emit('backup', profile)"
        @performance="$emit('performance', profile)"
        @delete="handleDelete"
        @openDir="handleOpenDir"
        @repair="handleRepair"
      />
    </div>

    <!-- 批量操作栏 -->
    <transition name="slide-up">
      <div v-if="profileStore.selectedIds.size > 0" class="batch-bar">
        <div class="batch-info">
          <span class="batch-count">{{ $t('main.batchSelected', { n: profileStore.selectedIds.size }) }}</span>
        </div>
        <div class="batch-actions">
          <el-button type="primary" plain @click="profileStore.handleBatchLaunch">
            <el-icon><VideoPlay /></el-icon>
            {{ $t('profile.actions.launch') }}
          </el-button>
          <el-button type="info" plain @click="profileStore.clearSelection">
            {{ $t('common.cancel') }}
          </el-button>
        </div>
      </div>
    </transition>
  </main>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useProfileStore } from '../../store/profile';
import ProfileCard from '../ProfileCard.vue';
import { api } from '../../api';
import { ElMessage, ElMessageBox } from 'element-plus';
import { VideoPlay } from '@element-plus/icons-vue';
import type { Profile } from '../../types';

defineEmits(['create', 'edit', 'backup', 'performance']);
const profileStore = useProfileStore();
const { t } = useI18n();

const handleOpenDir = async (profile: Profile) => {
  try {
    await api.openProfileDirectory(profile.data_dir_path);
  } catch (error) {
    ElMessage.error(t('common.error'));
    console.error(error);
  }
};

const handleRepair = async (profile: Profile) => {
  try {
    await api.unlockProfile(profile.id);
    ElMessage.success(t('common.success'));
  } catch (error) {
    ElMessage.error(t('common.error'));
    console.error(error);
  }
};

const handleDelete = async (profile: Profile) => {
  try {
    await ElMessageBox.confirm(
      t('main.deleteConfirm.content'),
      t('main.deleteConfirm.title'),
      {
        confirmButtonText: t('common.delete'),
        cancelButtonText: t('common.cancel'),
        type: 'warning',
      }
    );

    await api.deleteProfile(profile.id);
    ElMessage.success(t('common.success'));
    await profileStore.loadProfiles();
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(t('common.error'));
      console.error(error);
    }
  }
};
</script>

<style lang="scss" scoped>
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

// 批量操作栏样式
.batch-bar {
  position: fixed;
  bottom: var(--space-8);
  left: 50%;
  transform: translateX(-50%);
  background: var(--bg-primary);
  border: 1px solid var(--primary-200);
  border-radius: var(--radius-2xl);
  padding: var(--space-3) var(--space-6);
  display: flex;
  align-items: center;
  gap: var(--space-10);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  z-index: var(--z-fixed);
  backdrop-filter: blur(12px);
}

.batch-info {
  .batch-count {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--primary-600);
  }
}

.batch-actions {
  display: flex;
  gap: var(--space-3);
  
  .el-button {
    height: 36px;
    border-radius: var(--radius-lg);
  }
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-up-enter-from,
.slide-up-leave-to {
  transform: translate(-50%, 100%);
  opacity: 0;
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

@media (max-width: 1024px) {
  .app-main {
    padding: var(--space-4);
  }

  .profiles-container.grid-view {
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: var(--space-3);
  }
}

@media (max-width: 768px) {
  .app-main {
    padding: var(--space-3);
  }

  .profiles-container.grid-view {
    grid-template-columns: 1fr;
    gap: var(--space-3);
  }

  .batch-bar {
    width: 90%;
    bottom: var(--space-4);
    gap: var(--space-4);
    padding: var(--space-3) var(--space-4);
  }
}
</style>
