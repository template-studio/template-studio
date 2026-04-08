<template>
  <div class="config-section">
    <div class="section-header">
      <h3>软删除字段</h3>
      <a-switch v-model:checked="tableConfig.softDeleteEnabled" size="small" />
    </div>
    <a-divider />
    <template v-if="tableConfig.softDeleteEnabled">
      <div class="setting-row">
        <div class="setting-row-title">
          <div>启用软删除</div>
          <div class="setting-description">删除操作会更新此字段而非物理删除</div>
        </div>
      </div>
      <div class="setting-divider"></div>
      <div class="setting-row">
        <div class="setting-row-title">字段名</div>
        <a-input v-model:value="tableConfig.softDeleteField" style="width: 180px" />
      </div>
      <div class="setting-divider"></div>
      <div class="setting-row">
        <div class="setting-row-title">字段类型</div>
        <a-select v-model:value="tableConfig.softDeleteFieldType" style="width: 180px">
          <a-select-option value="TIMESTAMP">TIMESTAMP</a-select-option>
          <a-select-option value="DATETIME">DATETIME</a-select-option>
          <a-select-option value="BIGINT">BIGINT</a-select-option>
        </a-select>
      </div>
      <div class="setting-divider"></div>
      <div class="setting-row">
        <div class="setting-row-title">
          <div>允许空值</div>
          <div class="setting-description">NULL 表示未删除（建议勾选）</div>
        </div>
        <a-switch v-model:checked="tableConfig.softDeleteNullable" size="small" />
      </div>
      <div class="setting-divider"></div>
      <div class="setting-row">
        <div class="setting-row-title">字段注释</div>
        <a-input v-model:value="tableConfig.softDeleteComment" style="width: 180px" />
      </div>
    </template>
    <div v-else class="empty-tip">软删除已禁用，删除操作将物理删除数据</div>
  </div>
</template>

<script setup>
defineProps({
  tableConfig: {
    type: Object,
    required: true
  }
})
</script>

<style scoped>
.config-section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.section-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

.config-section :deep(.ant-divider) {
  margin: 12px 0;
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

.empty-tip {
  color: var(--color-text-secondary);
  font-size: 13px;
  padding: 12px 0;
}
</style>
