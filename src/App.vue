<template>
  <div class="app-container">
    <AppHeader @create="handleCreate" />

    <AppMain
      @create="handleCreate"
      @edit="handleEdit"
      @backup="handleBackup"
      @performance="handleShowPerformance"
    />

    <ProfileForm
      v-model="showForm"
      :profile="editingProfile"
      :existing-tags="profileStore.existingTags"
      @success="profileStore.loadProfiles"
    />

    <BackupDialog
      v-model="showBackupDialog"
      :profile="selectedProfile"
      @success="profileStore.loadProfiles"
    />

    <PerformanceDialog
      v-model="showPerformanceDialog"
      :profile="selectedProfile"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import AppHeader from './components/layout/AppHeader.vue';
import AppMain from './components/layout/AppMain.vue';
import ProfileForm from './components/ProfileForm.vue';
import BackupDialog from './components/BackupDialog.vue';
import PerformanceDialog from './components/PerformanceDialog.vue';
import type { Profile } from './types';
import { useProfileStore } from './store/profile';

const profileStore = useProfileStore();

const showForm = ref(false);
const showBackupDialog = ref(false);
const showPerformanceDialog = ref(false);
const editingProfile = ref<Profile | null>(null);
const selectedProfile = ref<Profile | null>(null);

const handleCreate = () => {
  editingProfile.value = null;
  showForm.value = true;
};

const handleEdit = (profile: Profile) => {
  editingProfile.value = profile;
  showForm.value = true;
};

const handleBackup = (profile: Profile) => {
  selectedProfile.value = profile;
  showBackupDialog.value = true;
};

const handleShowPerformance = (profile: Profile) => {
  selectedProfile.value = profile;
  showPerformanceDialog.value = true;
};

onMounted(async () => {
  await profileStore.loadProfiles();
  await profileStore.initStatusListener();
});
</script>

<style lang="scss">
.app-container {
  min-height: 100vh;
  background: var(--bg-secondary);
}
</style>
