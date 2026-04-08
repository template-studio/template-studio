<template>
  <div class="config-section">
    <div class="section-header">
      <h3>审计字段</h3>
      <a-switch v-model:checked="tableConfig.auditEnabled" size="small" />
    </div>
    <a-divider />
    <template v-if="tableConfig.auditEnabled">
      <a-table
        :columns="auditColumns"
        :data-source="auditData"
        :pagination="false"
        size="small"
        bordered
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'enabled'">
            <a-switch v-model:checked="record.enabled" size="small" />
          </template>
          <template v-else-if="column.key === 'fieldName'">
            <a-input v-model:value="record.fieldName" size="small" />
          </template>
          <template v-else-if="column.key === 'fieldType'">
            <a-select v-model:value="record.fieldType" size="small" style="width: 100%">
              <a-select-option v-for="t in getTypeOptions(record.key)" :key="t" :value="t">{{ t }}</a-select-option>
            </a-select>
          </template>
          <template v-else-if="column.key === 'comment'">
            <a-input v-model:value="record.comment" size="small" />
          </template>
        </template>
      </a-table>
    </template>
    <div v-else class="empty-tip">审计字段已禁用，新建表时不会自动添加创建时间、更新时间等字段</div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  tableConfig: {
    type: Object,
    required: true
  }
})

const auditColumns = [
  { title: '启用', key: 'enabled', width: 60, align: 'center' },
  { title: '字段', dataIndex: 'label', width: 100 },
  { title: '字段名', key: 'fieldName', width: 120 },
  { title: '类型', key: 'fieldType', width: 120 },
  { title: '注释', key: 'comment' }
]

const auditData = computed(() => [
  { key: 'createdAt', label: '创建时间', get enabled() { return props.tableConfig.auditFields.createdAt.enabled }, set enabled(v) { props.tableConfig.auditFields.createdAt.enabled = v }, get fieldName() { return props.tableConfig.auditFields.createdAt.fieldName }, set fieldName(v) { props.tableConfig.auditFields.createdAt.fieldName = v }, get fieldType() { return props.tableConfig.auditFields.createdAt.fieldType }, set fieldType(v) { props.tableConfig.auditFields.createdAt.fieldType = v }, get comment() { return props.tableConfig.auditFields.createdAt.comment }, set comment(v) { props.tableConfig.auditFields.createdAt.comment = v } },
  { key: 'updatedAt', label: '更新时间', get enabled() { return props.tableConfig.auditFields.updatedAt.enabled }, set enabled(v) { props.tableConfig.auditFields.updatedAt.enabled = v }, get fieldName() { return props.tableConfig.auditFields.updatedAt.fieldName }, set fieldName(v) { props.tableConfig.auditFields.updatedAt.fieldName = v }, get fieldType() { return props.tableConfig.auditFields.updatedAt.fieldType }, set fieldType(v) { props.tableConfig.auditFields.updatedAt.fieldType = v }, get comment() { return props.tableConfig.auditFields.updatedAt.comment }, set comment(v) { props.tableConfig.auditFields.updatedAt.comment = v } },
  { key: 'createdBy', label: '创建人', get enabled() { return props.tableConfig.auditFields.createdBy.enabled }, set enabled(v) { props.tableConfig.auditFields.createdBy.enabled = v }, get fieldName() { return props.tableConfig.auditFields.createdBy.fieldName }, set fieldName(v) { props.tableConfig.auditFields.createdBy.fieldName = v }, get fieldType() { return props.tableConfig.auditFields.createdBy.fieldType }, set fieldType(v) { props.tableConfig.auditFields.createdBy.fieldType = v }, get comment() { return props.tableConfig.auditFields.createdBy.comment }, set comment(v) { props.tableConfig.auditFields.createdBy.comment = v } },
  { key: 'updatedBy', label: '更新人', get enabled() { return props.tableConfig.auditFields.updatedBy.enabled }, set enabled(v) { props.tableConfig.auditFields.updatedBy.enabled = v }, get fieldName() { return props.tableConfig.auditFields.updatedBy.fieldName }, set fieldName(v) { props.tableConfig.auditFields.updatedBy.fieldName = v }, get fieldType() { return props.tableConfig.auditFields.updatedBy.fieldType }, set fieldType(v) { props.tableConfig.auditFields.updatedBy.fieldType = v }, get comment() { return props.tableConfig.auditFields.updatedBy.comment }, set comment(v) { props.tableConfig.auditFields.updatedBy.comment = v } }
])

const getTypeOptions = (key) => {
  if (key === 'createdAt' || key === 'updatedAt') {
    return ['TIMESTAMP', 'DATETIME']
  }
  return ['BIGINT', 'INT', 'VARCHAR(64)']
}
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

.empty-tip {
  color: var(--color-text-secondary);
  font-size: 13px;
  padding: 12px 0;
}

:deep(.ant-table) {
  font-size: 13px;
}

:deep(.ant-table-thead > tr > th) {
  background: var(--color-bg-secondary);
  font-weight: 500;
}

:deep(.ant-table-tbody > tr > td) {
  padding: 8px 12px;
}
</style>
