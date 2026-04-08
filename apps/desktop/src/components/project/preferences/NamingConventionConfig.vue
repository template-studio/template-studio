<template>
  <div class="setting-container">
    <div class="page-header">
      <h2>命名规范</h2>
      <p class="page-desc">配置字段命名的前缀和后缀规范，保持代码风格统一</p>
    </div>

    <div class="setting-group">
      <div class="setting-title">命名约定</div>
      <div class="setting-row">
        <div class="setting-row-title">
          <div>布尔字段前缀</div>
          <div class="setting-description">示例: is_active, is_deleted, is_verified</div>
        </div>
        <a-input v-model:value="tableConfig.booleanPrefix" style="width: 120px" />
      </div>
      <div class="setting-divider"></div>
      <div class="setting-row">
        <div class="setting-row-title">
          <div>时间字段后缀</div>
          <div class="setting-description">示例: created_at, updated_at, deleted_at</div>
        </div>
        <a-input v-model:value="tableConfig.datetimeSuffix" style="width: 120px" />
      </div>
    </div>

    <div class="action-bar">
      <a-button @click="emit('reset')">重置为默认</a-button>
      <a-button type="primary" @click="emit('save')" :loading="saving">
        <template #icon><SaveOutlined /></template>
        保存配置
      </a-button>
    </div>
  </div>
</template>

<script setup>
import { SaveOutlined } from '@ant-design/icons-vue'

defineProps({
  tableConfig: {
    type: Object,
    required: true
  },
  saving: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['reset', 'save'])
</script>

<style scoped>
.setting-container {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  padding-top: 15px;
}

.page-header {
  margin-bottom: 20px;
}

.page-header h2 {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
}

.page-desc {
  margin: 0;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.setting-group {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.setting-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 12px;
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
}

.setting-row-title {
  font-size: 13px;
  color: var(--color-text);
}

.setting-description {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 2px;
}

.setting-divider {
  height: 1px;
  background: var(--color-border);
  margin: 4px 0;
}

.action-bar {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 0;
  margin-top: 16px;
  border-top: 1px solid var(--color-border);
}
</style>
