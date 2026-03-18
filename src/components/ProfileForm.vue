<template>
  <el-dialog
    v-model="visible"
    :title="isEdit ? $t('profile.form.titleEdit') : $t('profile.form.titleCreate')"
    width="500px"
    destroy-on-close
  >
    <el-form
      ref="formRef"
      :model="form"
      :rules="rules"
      label-width="80px"
      label-position="top"
    >
      <el-form-item :label="$t('profile.form.name')" prop="name">
        <el-input
          v-model="form.name"
          :placeholder="$t('profile.form.namePlaceholder')"
          maxlength="50"
          show-word-limit
        />
      </el-form-item>

      <el-form-item :label="$t('profile.form.chromePath')" prop="chrome_path">
        <el-input
          v-model="form.chrome_path"
          :placeholder="$t('profile.form.chromePathPlaceholder')"
          clearable
        />
      </el-form-item>

      <el-form-item :label="$t('profile.form.homepage')" prop="homepage">
        <el-input
          v-model="form.homepage"
          :placeholder="$t('profile.form.homepagePlaceholder')"
          clearable
        >
          <template #prefix>
            <el-icon><Monitor /></el-icon>
          </template>
        </el-input>
      </el-form-item>

      <el-form-item :label="$t('profile.status.forwardPort')" prop="forward_port">
        <el-input-number
          v-model="form.forward_port"
          :placeholder="$t('profile.form.forwardPortPlaceholder')"
          :min="1"
          :max="65535"
        />
        <div class="form-tip">{{ $t('profile.form.forwardPortTip') }}</div>
      </el-form-item>

      <el-form-item :label="$t('profile.form.icon')">
        <div class="icon-selector">
          <div class="current-icon" @click="triggerFileInput">
            <img v-if="form.icon_base64" :src="form.icon_base64" alt="Icon" />
            <el-avatar v-else :size="80" :icon="UserFilled" />
            <div class="icon-overlay">
              <el-icon><Camera /></el-icon>
              <span>{{ $t('profile.actions.edit') }}</span>
            </div>
          </div>
          <input
            ref="fileInput"
            type="file"
            accept="image/*"
            style="display: none"
            @change="handleFileChange"
          />
        </div>
      </el-form-item>

      <el-form-item :label="$t('profile.form.tags')">
        <el-select
          v-model="tagList"
          multiple
          filterable
          allow-create
          default-first-option
          :placeholder="$t('profile.form.tagsPlaceholder')"
          style="width: 100%"
        >
          <el-option
            v-for="tag in existingTags"
            :key="tag"
            :label="tag"
            :value="tag"
          />
        </el-select>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="visible = false">{{ $t('common.cancel') }}</el-button>
      <el-button type="primary" @click="handleSubmit" :loading="isSubmitting">
        {{ isEdit ? $t('common.save') : $t('common.confirm') }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage } from 'element-plus';
import { UserFilled, Camera, Monitor } from '@element-plus/icons-vue';
import type { Profile } from '../types';
import { api } from '../api';

const props = defineProps<{
  modelValue: boolean;
  profile?: Profile | null;
  existingTags: string[];
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  success: [];
}>();

const { t } = useI18n();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const isEdit = computed(() => !!props.profile);

const formRef = ref();
const fileInput = ref<HTMLInputElement>();
const isSubmitting = ref(false);

const form = ref({
  name: '',
  chrome_path: '',
  homepage: '',
  icon_base64: '',
  tags: '',
  forward_port: undefined as number | undefined,
});

const tagList = ref<string[]>([]);

const rules = {
  name: [
    { required: true, message: t('profile.form.namePlaceholder'), trigger: 'blur' },
    { min: 1, max: 50, message: t('profile.form.namePlaceholder'), trigger: 'blur' },
  ],
  homepage: [
    { 
      pattern: /^(https?:\/\/)?([\da-z.-]+)\.([a-z.]{2,6})([\/\w .-]*)*\/?$/, 
      message: t('profile.form.homepagePlaceholder'), 
      trigger: 'blur' 
    }
  ]
};

watch(
  () => props.profile,
  (profile) => {
    if (profile) {
      form.value = {
        name: profile.name,
        chrome_path: profile.chrome_path || '',
        homepage: profile.homepage || '',
        icon_base64: profile.icon_base64 || '',
        tags: profile.tags || '',
        forward_port: profile.forward_port,
      };
      tagList.value = profile.tags?.split(',').map(t => t.trim()).filter(Boolean) || [];
    } else {
      form.value = {
        name: '',
        chrome_path: '',
        homepage: '',
        icon_base64: '',
        tags: '',
        forward_port: undefined,
      };
      tagList.value = [];
    }
  },
  { immediate: true }
);

const triggerFileInput = () => {
  fileInput.value?.click();
};

const handleFileChange = (event: Event) => {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  
  if (file) {
    if (file.size > 2 * 1024 * 1024) {
      ElMessage.error(t('common.error'));
      return;
    }
    
    const reader = new FileReader();
    reader.onload = (e) => {
      form.value.icon_base64 = e.target?.result as string;
    };
    reader.readAsDataURL(file);
  }
  
  // Reset input
  target.value = '';
};

const handleSubmit = async () => {
  const valid = await formRef.value?.validate().catch(() => false);
  if (!valid) return;

  isSubmitting.value = true;
  
  try {
    const tags = tagList.value.join(',');
    
    if (isEdit.value && props.profile) {
      await api.updateProfile(
        props.profile.id,
        form.value.name,
        form.value.chrome_path || undefined,
        form.value.homepage || undefined,
        form.value.icon_base64 || undefined,
        tags || undefined,
        form.value.forward_port || undefined
      );
      ElMessage.success(t('common.success'));
    } else {
      await api.createProfile(
        form.value.name,
        form.value.chrome_path || undefined,
        form.value.homepage || undefined,
        form.value.icon_base64 || undefined,
        tags || undefined,
        form.value.forward_port || undefined
      );
      ElMessage.success(t('common.success'));
    }
    
    emit('success');
    visible.value = false;
  } catch (error) {
    ElMessage.error(t('common.error'));
    console.error(error);
  } finally {
    isSubmitting.value = false;
  }
};
</script>

<style scoped lang="scss">
:deep(.el-dialog__header) {
  background: linear-gradient(135deg, var(--primary-500) 0%, var(--secondary-500) 100%);
  padding: var(--space-5) var(--space-6);
  margin-right: 0;
  border-radius: var(--radius-2xl) var(--radius-2xl) 0 0;
}

:deep(.el-dialog__title) {
  color: white;
  font-weight: var(--font-semibold);
  font-size: var(--text-lg);
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

:deep(.el-form-item__label) {
  font-weight: var(--font-medium);
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

:deep(.el-input__wrapper) {
  border-radius: var(--radius-lg);
  padding: var(--space-2) var(--space-4);
  
  &.is-focus {
    box-shadow: 0 0 0 3px rgba(33, 150, 243, 0.1);
  }
}

:deep(.el-textarea__inner) {
  border-radius: var(--radius-lg);
  padding: var(--space-3) var(--space-4);
  
  &:focus {
    box-shadow: 0 0 0 3px rgba(33, 150, 243, 0.1);
  }
}

.icon-selector {
  display: flex;
  justify-content: center;
}

.current-icon {
  position: relative;
  width: 96px;
  height: 96px;
  border-radius: var(--radius-full);
  cursor: pointer;
  overflow: hidden;
  border: 3px solid var(--border-light);
  transition: all var(--transition-fast);
  box-shadow: var(--shadow-md);
  
  &:hover {
    border-color: var(--primary-400);
    transform: scale(1.05);
    box-shadow: var(--shadow-lg);
  }
  
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: var(--radius-full);
  }
  
  :deep(.el-avatar) {
    width: 100%;
    height: 100%;
    font-size: 36px;
    background: linear-gradient(135deg, var(--primary-100), var(--secondary-100));
    color: var(--primary-600);
  }
}

.icon-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(0, 0, 0, 0.6), rgba(0, 0, 0, 0.4));
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: white;
  opacity: 0;
  transition: all var(--transition-fast);
  border-radius: var(--radius-full);
  
  .el-icon {
    font-size: 24px;
    margin-bottom: var(--space-1);
  }
  
  span {
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
  }
}

.current-icon:hover .icon-overlay {
  opacity: 1;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
  
  .el-button {
    padding: var(--space-3) var(--space-6);
    border-radius: var(--radius-lg);
    font-weight: var(--font-medium);
  }
}

.form-tip {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin-top: var(--space-1);
}
</style>
