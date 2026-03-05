import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

export const useThemeStore = defineStore('theme', () => {
    const isDark = ref(localStorage.getItem('theme') === 'dark' ||
        (!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches));

    const toggleTheme = () => {
        isDark.value = !isDark.value;
    };

    watch(isDark, (val) => {
        const root = document.documentElement;
        const appWindow = getCurrentWindow();

        if (val) {
            root.classList.add('dark');
            localStorage.setItem('theme', 'dark');
            appWindow.setTheme('dark').catch(console.error);
        } else {
            root.classList.remove('dark');
            localStorage.setItem('theme', 'light');
            appWindow.setTheme('light').catch(console.error);
        }
    }, { immediate: true });

    return {
        isDark,
        toggleTheme
    };
});
