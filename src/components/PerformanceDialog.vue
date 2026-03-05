<template>
  <el-dialog
    v-model="visible"
    :title="$t('performance.title')"
    width="700px"
    destroy-on-close
    class="performance-dialog"
  >
    <div v-if="loading" class="loading-state">
      <el-skeleton :rows="5" animated />
    </div>

    <div v-else-if="logs.length === 0" class="empty-state">
      <el-empty :description="$t('performance.noData')" />
    </div>

    <div v-else class="performance-content">
      <!-- 概览卡片 -->
      <div class="metrics-overview">
        <div class="metric-card">
          <div class="metric-label">{{ $t('performance.metrics.avgLaunch') }}</div>
          <div class="metric-value">{{ avgLaunchTime.toFixed(0) }}<span>ms</span></div>
        </div>
        <div class="metric-card">
          <div class="metric-label">{{ $t('performance.metrics.avgSpawn') }}</div>
          <div class="metric-value">{{ avgSpawnTime.toFixed(0) }}<span>ms</span></div>
        </div>
        <div class="metric-card">
          <div class="metric-label">{{ $t('performance.metrics.status') }}</div>
          <div class="metric-value status" :class="performanceStatus.class">
            {{ performanceStatus.text }}
          </div>
        </div>
      </div>

      <!-- 历史记录列表 -->
      <div class="history-section">
        <h4 class="section-title">{{ $t('performance.historyTitle') }}</h4>
        <el-table :data="logs" stripe style="width: 100%" size="small">
          <el-table-column prop="created_at" :label="$t('performance.table.time')" width="160">
            <template #default="scope">
              {{ formatDateTime(scope.row.created_at) }}
            </template>
          </el-table-column>
          <el-table-column :label="$t('performance.table.total')">
            <template #default="scope">
              <el-tag :type="getDurationTag(scope.row.launch_duration_ms)">
                {{ scope.row.launch_duration_ms }} ms
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="spawn_duration_ms" :label="$t('performance.table.spawn')" width="100">
            <template #default="scope">
              {{ scope.row.spawn_duration_ms }} ms
            </template>
          </el-table-column>
          <el-table-column :label="$t('performance.table.network')">
            <template #default="scope">
              <span class="sub-metrics">
                DNS: {{ scope.row.dns_duration_ms }}ms | DOM: {{ scope.row.dom_ready_ms }}ms
              </span>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <!-- 优化建议 -->
      <div class="suggestions-section">
        <h4 class="section-title">{{ $t('performance.suggestions.title') }}</h4>
        <el-alert
          v-if="avgLaunchTime > 3000"
          :title="$t('performance.suggestions.title')"
          type="warning"
          :description="$t('performance.suggestions.slow')"
          show-icon
          :closable="false"
        />
        <el-alert
          v-else
          :title="$t('common.success')"
          type="success"
          :description="$t('performance.suggestions.fast')"
          show-icon
          :closable="false"
        />
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '../api';
import type { Profile, PerformanceRecord } from '../types';

const props = defineProps<{
  modelValue: boolean;
  profile: Profile | null;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const logs = ref<PerformanceRecord[]>([]);
const loading = ref(false);
const { t, locale } = useI18n();

const loadLogs = async () => {
  if (!props.profile) return;
  loading.value = true;
  try {
    logs.value = await api.getPerformanceLogs(props.profile.id, 10);
  } catch (error) {
    console.error('Failed to load performance logs:', error);
  } finally {
    loading.value = false;
  }
};

watch(
  () => props.modelValue,
  (val) => {
    if (val && props.profile) {
      loadLogs();
    }
  }
);

const avgLaunchTime = computed(() => {
  if (logs.value.length === 0) return 0;
  const sum = logs.value.reduce((acc, log) => acc + log.launch_duration_ms, 0);
  return sum / logs.value.length;
});

const avgSpawnTime = computed(() => {
  if (logs.value.length === 0) return 0;
  const sum = logs.value.reduce((acc, log) => acc + log.spawn_duration_ms, 0);
  return sum / logs.value.length;
});

const performanceStatus = computed(() => {
  const avg = avgLaunchTime.value;
  if (avg === 0) return { text: t('performance.status.unknown'), class: '' };
  if (avg < 2000) return { text: t('performance.status.fast'), class: 'status-fast' };
  if (avg < 5000) return { text: t('performance.status.good'), class: 'status-good' };
  return { text: t('performance.status.slow'), class: 'status-slow' };
});

const getDurationTag = (duration: number) => {
  if (duration < 2000) return 'success';
  if (duration < 5000) return 'warning';
  return 'danger';
};

const formatDateTime = (dateStr: string) => {
  const date = new Date(dateStr);
  return date.toLocaleString(locale.value === 'zh' ? 'zh-CN' : 'en-US', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
};
</script>

<style scoped lang="scss">
.performance-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
}

.metrics-overview {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-4);
}

.metric-card {
  background: var(--bg-secondary);
  padding: var(--space-4);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-1);
  border: 1px solid var(--border-light);

  .metric-label {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .metric-value {
    font-size: 24px;
    font-weight: var(--font-bold);
    color: var(--text-primary);

    span {
      font-size: 14px;
      font-weight: var(--font-normal);
      margin-left: 2px;
      color: var(--text-tertiary);
    }

    &.status {
      font-size: 20px;
    }
  }
}

.status-fast { color: var(--success-color, #67C23A); }
.status-good { color: var(--primary-color, #409EFF); }
.status-slow { color: var(--warning-color, #E6A23C); }

.section-title {
  margin: 0 0 var(--space-3) 0;
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: var(--font-semibold);
}

.sub-metrics {
  font-size: 11px;
  color: var(--text-tertiary);
}

.suggestions-section {
  .el-alert {
    border-radius: var(--radius-md);
  }
}

.loading-state, .empty-state {
  padding: var(--space-10) 0;
}
</style>
