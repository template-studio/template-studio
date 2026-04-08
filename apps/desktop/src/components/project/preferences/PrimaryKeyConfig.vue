<template>
  <div class="config-section">
    <div class="section-header">
      <h3>主键规范</h3>
      <a-switch v-model:checked="tableConfig.pkEnabled" size="small" />
    </div>
    <a-divider />
    <template v-if="tableConfig.pkEnabled">
      <div class="setting-row">
        <div class="setting-row-title">字段名</div>
        <a-input v-model:value="tableConfig.pkFieldName" style="width: 180px" />
      </div>
      <div class="setting-divider"></div>
      <div class="setting-row">
        <div class="setting-row-title">字段类型</div>
        <a-select v-model:value="tableConfig.pkFieldType" style="width: 180px">
          <a-select-option value="INT">INT</a-select-option>
          <a-select-option value="BIGINT">BIGINT（推荐）</a-select-option>
          <a-select-option value="CHAR(36)">CHAR(36) - UUID</a-select-option>
          <a-select-option value="VARCHAR(32)">VARCHAR(32) - 雪花ID</a-select-option>
        </a-select>
      </div>
      <div class="setting-divider"></div>
      <div class="setting-row">
        <div class="setting-row-title">
          <div>自增 (AUTO_INCREMENT)</div>
          <div class="setting-description">主键自动递增</div>
        </div>
        <a-switch v-model:checked="tableConfig.pkAutoIncrement" size="small" />
      </div>
      <div class="setting-divider"></div>
      <div class="setting-row">
        <div class="setting-row-title">字段注释</div>
        <a-input v-model:value="tableConfig.pkComment" style="width: 180px" />
      </div>
    </template>
    <div v-else class="empty-tip">主键规范已禁用，新建表时不会自动添加主键字段</div>
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
