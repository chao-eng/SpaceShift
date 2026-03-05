import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { api } from '../api';
import type { Profile, ViewMode } from '../types';
import { ElMessage } from 'element-plus';
import { listen } from '@tauri-apps/api/event';

export const useProfileStore = defineStore('profile', () => {
    const profiles = ref<Profile[]>([]);
    const loading = ref(false);
    const searchQuery = ref('');
    const viewMode = ref<ViewMode>('grid');
    const launchingProfiles = ref<Set<string>>(new Set());

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

    const handleLaunch = async (profile: Profile) => {
        if (launchingProfiles.value.has(profile.id)) return;

        try {
            launchingProfiles.value.add(profile.id);
            const result = await api.launchChrome(profile.id);

            if (result.success) {
                ElMessage.success(`已启动: ${profile.name}`);
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

    const initStatusListener = async () => {
        await listen('browser-status-update', (event: any) => {
            const { id, is_running } = event.payload;
            const profile = profiles.value.find(p => p.id === id);
            if (profile) {
                profile.is_running = is_running;
            }
        });
    };

    return {
        profiles,
        loading,
        searchQuery,
        viewMode,
        launchingProfiles,
        filteredProfiles,
        existingTags,
        loadProfiles,
        handleLaunch,
        initStatusListener,
    };
});
