<template>
  <el-dialog
    v-model="visible"
    title="⚠️ 强制关闭浏览器"
    width="500px"
    :close-on-click-modal="false"
    class="kill-browser-dialog"
  >
    <div class="warning-content">
      <el-alert
        :title="warningTitle"
        type="warning"
        :description="warningDescription"
        show-icon
        :closable="false"
        class="warning-alert"
      />
      
      <div v-if="safetyCheck" class="safety-info">
        <h4>检测到的浏览器进程：</h4>
        <el-descriptions :column="1" border size="small">
          <el-descriptions-item label="浏览器类型">
            {{ getBrowserTypeDisplay(safetyCheck.browser_type) }}
          </el-descriptions-item>
          <el-descriptions-item label="进程数量">
            {{ safetyCheck.process_count }} 个
          </el-descriptions-item>
          <el-descriptions-item label="安全状态">
            <el-tag :type="safetyCheck.is_safe_to_kill ? 'success' : 'danger'">
              {{ safetyCheck.is_safe_to_kill ? '安全' : '存在风险' }}
            </el-tag>
          </el-descriptions-item>
        </el-descriptions>
        
        <div v-if="safetyCheck.warnings.length > 0" class="warnings-list">
          <h4>⚠️ 警告信息：</h4>
          <ul>
            <li v-for="(warning, index) in safetyCheck.warnings" :key="index" class="warning-item">
              {{ warning }}
            </li>
          </ul>
        </div>
      </div>
      
      <div v-if="browserProcesses.length > 0" class="process-list">
        <h4>进程详情：</h4>
        <el-table :data="browserProcesses" size="small" max-height="200">
          <el-table-column prop="pid" label="PID" width="80" />
          <el-table-column prop="name" label="进程名" />
          <el-table-column prop="browser_type" label="类型" width="100">
            <template #default="{ row }">
              <el-tag size="small">{{ getBrowserTypeDisplay(row.browser_type) }}</el-tag>
            </template>
          </el-table-column>
        </el-table>
      </div>
      
      <div class="risk-notice">
        <h4>📋 风险提示：</h4>
        <ul>
          <li>强制关闭浏览器可能导致未保存的数据丢失</li>
          <li>正在进行的下载任务将被中断</li>
          <li>某些网站的登录状态可能需要重新验证</li>
          <li>建议先尝试正常关闭浏览器，强制关闭作为最后手段</li>
        </ul>
      </div>
      
      <el-checkbox v-model="confirmed" class="confirm-checkbox">
        我已了解风险，确认强制关闭浏览器
      </el-checkbox>
    </div>
    
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleCancel">取消</el-button>
        <el-button 
          type="danger" 
          :disabled="!confirmed || killing"
          :loading="killing"
          @click="handleConfirm"
        >
          {{ killing ? '关闭中...' : '强制关闭' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { ElMessage } from 'element-plus';
import type { BrowserProcess, BrowserSafetyCheck, BrowserType, KillBrowserResult } from '../types';
import { api } from '../api';

interface Props {
  modelValue: boolean;
  profileDataDir?: string;
  browserType?: BrowserType;
  mode?: 'profile' | 'type' | 'all';
}

const props = withDefaults(defineProps<Props>(), {
  mode: 'profile',
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'success', result: KillBrowserResult): void;
  (e: 'error', error: string): void;
}>();

const visible = ref(false);
const confirmed = ref(false);
const killing = ref(false);
const browserProcesses = ref<BrowserProcess[]>([]);
const safetyCheck = ref<BrowserSafetyCheck | null>(null);

const warningTitle = '您即将强制关闭浏览器进程';
const warningDescription = '此操作将立即终止浏览器进程，可能导致数据丢失。请确保已保存所有重要数据。';

watch(() => props.modelValue, (val) => {
  if (visible.value !== val) {
    visible.value = val;
  }
  if (val) {
    loadBrowserInfo();
  } else {
    resetState();
  }
});

watch(visible, (val) => {
  if (props.modelValue !== val) {
    emit('update:modelValue', val);
  }
});

const resetState = () => {
  confirmed.value = false;
  killing.value = false;
  browserProcesses.value = [];
  safetyCheck.value = null;
};

const loadBrowserInfo = async () => {
  try {
    const [processes, safety] = await Promise.all([
      api.detectBrowserProcesses(),
      api.safetyCheckBrowserKill(),
    ]);
    browserProcesses.value = processes;
    safetyCheck.value = safety;
  } catch (error) {
    console.error('Failed to load browser info:', error);
    ElMessage.warning('无法获取浏览器进程信息，请谨慎操作');
  }
};

const getBrowserTypeDisplay = (type: BrowserType): string => {
  const displayMap: Record<BrowserType, string> = {
    chrome: 'Google Chrome',
    firefox: 'Mozilla Firefox',
    edge: 'Microsoft Edge',
    safari: 'Apple Safari',
    opera: 'Opera',
    brave: 'Brave Browser',
    unknown: '未知浏览器',
  };
  return displayMap[type] || '未知浏览器';
};

const handleCancel = () => {
  visible.value = false;
};

const handleConfirm = async () => {
  if (!confirmed.value) return;
  
  killing.value = true;
  
  try {
    let result: KillBrowserResult;
    
    switch (props.mode) {
      case 'profile':
        if (props.profileDataDir) {
          result = await api.killBrowserByProfile(props.profileDataDir);
        } else {
          throw new Error('未指定配置文件目录');
        }
        break;
      case 'type':
        if (props.browserType) {
          result = await api.killBrowserByType(props.browserType);
        } else {
          throw new Error('未指定浏览器类型');
        }
        break;
      case 'all':
        result = await api.killAllBrowsers();
        break;
      default:
        throw new Error('未知的关闭模式');
    }
    
    if (result.success) {
      // 成功消息由父组件统一显示，避免重复
      emit('success', result);
      visible.value = false;
    } else {
      // 失败消息在对话框显示，不传递给父组件避免重复
      ElMessage.error(result.message);
      emit('error', result.message);
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : '强制关闭失败';
    ElMessage.error(errorMsg);
    emit('error', errorMsg);
  } finally {
    killing.value = false;
  }
};
</script>

<style scoped lang="scss">
.kill-browser-dialog {
  :deep(.el-dialog__header) {
    background: linear-gradient(135deg, var(--danger-color) 0%, #f97316 100%);
    color: white;
    padding: var(--space-5) var(--space-6);
    margin-right: 0;
    border-radius: var(--radius-2xl) var(--radius-2xl) 0 0;
  }
  
  :deep(.el-dialog__title) {
    color: white;
    font-weight: var(--font-semibold);
    font-size: var(--text-lg);
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  
  :deep(.el-dialog__headerbtn) {
    top: 50%;
    transform: translateY(-50%);
    
    .el-dialog__close {
      color: white;
      font-size: 20px;
      
      &:hover {
        color: rgba(255, 255, 255, 0.8);
      }
    }
  }
  
  :deep(.el-dialog__body) {
    padding: var(--space-6);
  }
  
  :deep(.el-dialog__footer) {
    padding: var(--space-4) var(--space-6);
    border-top: 1px solid var(--border-light);
  }
}

.warning-content {
  max-height: 60vh;
  overflow-y: auto;
  
  &::-webkit-scrollbar {
    width: 6px;
  }
  
  &::-webkit-scrollbar-thumb {
    background: var(--border-medium);
    border-radius: var(--radius-full);
  }
}

.warning-alert {
  margin-bottom: var(--space-5);
  border-radius: var(--radius-lg);
  
  :deep(.el-alert__title) {
    font-weight: var(--font-semibold);
    font-size: var(--text-base);
  }
  
  :deep(.el-alert__description) {
    font-size: var(--text-sm);
    line-height: var(--leading-relaxed);
  }
}

.safety-info {
  margin-bottom: var(--space-5);
  
  h4 {
    margin: 0 0 var(--space-3) 0;
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  :deep(.el-descriptions) {
    border-radius: var(--radius-lg);
    overflow: hidden;
    
    .el-descriptions__cell {
      padding: var(--space-3) var(--space-4);
    }
    
    .el-descriptions__label {
      color: var(--text-secondary);
      font-weight: var(--font-medium);
    }
    
    .el-descriptions__content {
      color: var(--text-primary);
      font-weight: var(--font-semibold);
    }
  }
}

.warnings-list {
  margin-top: var(--space-4);
  padding: var(--space-4);
  background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
  border-radius: var(--radius-lg);
  border: 1px solid #fcd34d;
  
  h4 {
    margin: 0 0 var(--space-3) 0;
    color: #d97706;
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  
  ul {
    margin: 0;
    padding-left: var(--space-5);
  }
}

.warning-item {
  color: #92400e;
  font-size: var(--text-sm);
  margin-bottom: var(--space-2);
  line-height: var(--leading-relaxed);
  
  &:last-child {
    margin-bottom: 0;
  }
}

.process-list {
  margin-bottom: var(--space-5);
  
  h4 {
    margin: 0 0 var(--space-3) 0;
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  :deep(.el-table) {
    border-radius: var(--radius-lg);
    overflow: hidden;
    border: 1px solid var(--border-light);
    
    .el-table__header {
      th {
        background: var(--bg-tertiary);
        color: var(--text-secondary);
        font-weight: var(--font-semibold);
        font-size: var(--text-xs);
        text-transform: uppercase;
        letter-spacing: 0.5px;
      }
    }
    
    .el-table__row {
      transition: background var(--transition-fast);
      
      &:hover {
        background: var(--primary-50);
      }
    }
  }
}

.risk-notice {
  margin-bottom: var(--space-5);
  padding: var(--space-4);
  background: linear-gradient(135deg, var(--bg-tertiary) 0%, var(--bg-secondary) 100%);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-light);
  
  h4 {
    margin: 0 0 var(--space-3) 0;
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  
  ul {
    margin: 0;
    padding-left: var(--space-5);
  }
  
  li {
    color: var(--text-secondary);
    font-size: var(--text-sm);
    margin-bottom: var(--space-2);
    line-height: var(--leading-relaxed);
    
    &:last-child {
      margin-bottom: 0;
    }
  }
}

.confirm-checkbox {
  margin-top: var(--space-5);
  padding: var(--space-4);
  background: var(--danger-light);
  border-radius: var(--radius-lg);
  border: 1px solid var(--danger-color);
  
  :deep(.el-checkbox__input) {
    .el-checkbox__inner {
      border-color: var(--danger-color);
      
      &:hover {
        border-color: var(--danger-color);
      }
    }
    
    &.is-checked .el-checkbox__inner {
      background-color: var(--danger-color);
      border-color: var(--danger-color);
    }
  }
  
  :deep(.el-checkbox__label) {
    color: var(--danger-color);
    font-weight: var(--font-semibold);
    font-size: var(--text-sm);
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
  
  .el-button {
    padding: var(--space-3) var(--space-6);
    border-radius: var(--radius-lg);
    font-weight: var(--font-medium);
    
    &--danger {
      background: linear-gradient(135deg, var(--danger-color), #dc2626);
      border: none;
      box-shadow: var(--shadow-md);
      
      &:hover:not(:disabled) {
        background: linear-gradient(135deg, #dc2626, #b91c1c);
        transform: translateY(-1px);
        box-shadow: var(--shadow-lg);
      }
      
      &:disabled {
        opacity: 0.6;
        cursor: not-allowed;
      }
    }
  }
}
</style>
