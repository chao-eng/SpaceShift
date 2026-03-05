<template>
  <el-dialog
    v-model="visible"
    title="性能分析报告"
    width="700px"
    destroy-on-close
    class="performance-dialog"
  >
    <div v-if="loading" class="loading-state">
      <el-skeleton :rows="5" animated />
    </div>

    <div v-else-if="logs.length === 0" class="empty-state">
      <el-empty description="暂无性能数据，请先启动浏览器" />
    </div>

    <div v-else class="performance-content">
      <!-- 概览卡片 -->
      <div class="metrics-overview">
        <div class="metric-card">
          <div class="metric-label">平均启动耗时</div>
          <div class="metric-value">{{ avgLaunchTime.toFixed(0) }}<span>ms</span></div>
        </div>
        <div class="metric-card">
          <div class="metric-label">进程创建耗时</div>
          <div class="metric-value">{{ avgSpawnTime.toFixed(0) }}<span>ms</span></div>
        </div>
        <div class="metric-card">
          <div class="metric-label">状态</div>
          <div class="metric-value status" :class="performanceStatus.class">
            {{ performanceStatus.text }}
          </div>
        </div>
      </div>

      <!-- 历史记录列表 -->
      <div class="history-section">
        <h4 class="section-title">最近 10 次启动分析</h4>
        <el-table :data="logs" stripe style="width: 100%" size="small">
          <el-table-column prop="created_at" label="时间" width="160">
            <template #default="scope">
              {{ formatDateTime(scope.row.created_at) }}
            </template>
          </el-table-column>
          <el-table-column label="总耗时">
            <template #default="scope">
              <el-tag :type="getDurationTag(scope.row.launch_duration_ms)">
                {{ scope.row.launch_duration_ms }} ms
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="spawn_duration_ms" label="进程创建" width="100">
            <template #default="scope">
              {{ scope.row.spawn_duration_ms }} ms
            </template>
          </el-table-column>
          <el-table-column label="网络/DOM (估算)">
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
        <h4 class="section-title">优化建议</h4>
        <el-alert
          v-if="avgLaunchTime > 3000"
          title="启动速度较慢"
          type="warning"
          description="平均启动耗时超过 3s。建议清理浏览器缓存或减少启动页依赖。"
          show-icon
          :closable="false"
        />
        <el-alert
          v-else
          title="系统响应良好"
          type="success"
          description="您的浏览器启动流程非常顺畅，目前的参数配置已接近最优。"
          show-icon
          :closable="false"
        />
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
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
  if (avg === 0) return { text: '未知', class: '' };
  if (avg < 2000) return { text: '极速', class: 'status-fast' };
  if (avg < 5000) return { text: '良好', class: 'status-good' };
  return { text: '缓慢', class: 'status-slow' };
});

const getDurationTag = (duration: number) => {
  if (duration < 2000) return 'success';
  if (duration < 5000) return 'warning';
  return 'danger';
};

const formatDateTime = (dateStr: string) => {
  const date = new Date(dateStr);
  return date.toLocaleString('zh-CN', {
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
