<template>
  <div class="setting-container">
    <div class="page-header">
      <h2>存储配置</h2>
      <p class="page-desc">配置数据库存储引擎、字符集等，新建表时自动应用</p>
    </div>

    <div class="setting-group">
      <div class="setting-title">数据库配置</div>
      <div class="setting-row">
        <div class="setting-row-title">存储引擎</div>
        <a-select v-model:value="tableConfig.engineType" style="width: 200px">
          <a-select-option value="InnoDB">InnoDB（推荐，支持事务）</a-select-option>
          <a-select-option value="MyISAM">MyISAM（不支持事务）</a-select-option>
        </a-select>
      </div>
      <div class="setting-divider"></div>
      <div class="setting-row">
        <div class="setting-row-title">字符集</div>
        <a-select v-model:value="tableConfig.charset" style="width: 200px">
          <a-select-option value="utf8mb4">utf8mb4（推荐，支持 emoji）</a-select-option>
          <a-select-option value="utf8">utf8</a-select-option>
        </a-select>
      </div>
      <div class="setting-divider"></div>
      <div class="setting-row">
        <div class="setting-row-title">排序规则</div>
        <a-select v-model:value="tableConfig.collation" style="width: 200px">
          <a-select-option value="utf8mb4_unicode_ci">utf8mb4_unicode_ci</a-select-option>
          <a-select-option value="utf8mb4_general_ci">utf8mb4_general_ci</a-select-option>
        </a-select>
      </div>
      <div class="setting-divider"></div>
      <div class="setting-row">
        <div class="setting-row-title">行格式</div>
        <a-select v-model:value="tableConfig.rowFormat" style="width: 200px">
          <a-select-option value="DYNAMIC">DYNAMIC</a-select-option>
          <a-select-option value="COMPRESSED">COMPRESSED</a-select-option>
        </a-select>
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
