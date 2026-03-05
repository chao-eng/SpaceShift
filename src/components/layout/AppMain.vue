<template>
  <main class="app-main">
    <div v-if="profileStore.loading" class="loading-container">
      <el-skeleton :rows="4" animated />
    </div>

    <el-empty
      v-else-if="profileStore.filteredProfiles.length === 0"
      :description="profileStore.searchQuery ? '未找到匹配的配置' : '暂无配置，点击右上角创建'"
      class="empty-state"
    >
      <el-button v-if="!profileStore.searchQuery" type="primary" @click="$emit('create')">
        创建第一个配置
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
        :style="{ animationDelay: `${index * 50}ms` }"
        @launch="profileStore.handleLaunch"
        @edit="$emit('edit', profile)"
        @backup="$emit('backup', profile)"
        @performance="$emit('performance', profile)"
        @delete="handleDelete"
        @openDir="handleOpenDir"
      />
    </div>
  </main>
</template>

<script setup lang="ts">
import { useProfileStore } from '../../store/profile';
import ProfileCard from '../ProfileCard.vue';
import { api } from '../../api';
import { ElMessage, ElMessageBox } from 'element-plus';
import type { Profile } from '../../types';

defineEmits(['create', 'edit', 'backup', 'performance']);
const profileStore = useProfileStore();

const handleOpenDir = async (profile: Profile) => {
  try {
    await api.openProfileDirectory(profile.data_dir_path);
  } catch (error) {
    ElMessage.error('无法打开目录');
    console.error(error);
  }
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
    await profileStore.loadProfiles();
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败');
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
}
</style>
