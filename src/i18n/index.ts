import { createI18n } from 'vue-i18n';
import zh from './locales/zh.json';
import en from './locales/en.json';

const messages = {
    zh,
    en,
};

const i18n = createI18n({
    legacy: false, // Use Composition API
    locale: 'zh', // Default language
    fallbackLocale: 'en',
    messages,
});

export default i18n;
