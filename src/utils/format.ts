/**
 * Format bytes into human readable string
 */
export const formatSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
        size /= 1024;
        unitIndex++;
    }

    return `${size.toFixed(2)} ${units[unitIndex]}`;
};

/**
 * Format ISO date string into locale date string
 */
export const formatDate = (dateStr: string, locale: string = 'zh'): string => {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    return date.toLocaleDateString(locale === 'zh' ? 'zh-CN' : 'en-US');
};

/**
 * Format ISO date string into locale date time string
 */
export const formatDateTime = (dateStr: string, locale: string = 'zh'): string => {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    return date.toLocaleString(locale === 'zh' ? 'zh-CN' : 'en-US', {
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
    });
};
