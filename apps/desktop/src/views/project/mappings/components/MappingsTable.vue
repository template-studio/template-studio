<template>
  <a-table
    :columns="columns"
    :data-source="mappings"
    :pagination="false"
    :scroll="{ x: 600 }"
    row-key="id"
    class="mapping-table"
    :loading="loading"
  >
    <template #bodyCell="{ column, record }">
      <template v-if="column.key === 'sourceType'">
        <code class="type-code">{{ record.source_type }}</code>
      </template>
      <template v-else-if="column.key === 'targetType'">
        <div v-if="editingKey === record.id" class="editing-cell">
          <a-input
            v-model:value="record.target_type"
            @blur="emit('save-edit', record)"
            @keyup.enter="emit('save-edit', record)"
            @keyup.esc="emit('cancel-edit', record)"
            size="small"
            autofocus
          />
        </div>
        <div v-else class="target-type-cell" @dblclick="emit('start-edit', record)">
          <code class="type-code target">{{ record.target_type }}</code>
          <EditOutlined class="edit-hint" />
        </div>
      </template>
      <template v-else-if="column.key === 'action'">
        <a-space>
          <a-button type="link" size="small" @click="emit('edit', record)">
            <template #icon><EditOutlined /></template>
            编辑
          </a-button>
          <a-popconfirm
            title="确定要删除这个映射吗？"
            ok-text="确定"
            cancel-text="取消"
            @confirm="emit('delete', record)"
          >
            <a-button type="link" size="small" danger>
              <template #icon><DeleteOutlined /></template>
              删除
            </a-button>
          </a-popconfirm>
        </a-space>
      </template>
    </template>
  </a-table>
</template>

<script setup>
import { EditOutlined, DeleteOutlined } from '@ant-design/icons-vue'

defineProps({
  mappings: { type: Array, default: () => [] },
  loading: { type: Boolean, default: false },
  editingKey: { type: [Number, String, null], default: null },
  editingValue: { type: String, default: '' }
})

const emit = defineEmits(['edit', 'save-edit', 'cancel-edit', 'delete', 'start-edit'])

const columns = [
  { title: '数据库字段类型', dataIndex: 'source_type', key: 'sourceType', width: 200 },
  { title: '语言字段类型', dataIndex: 'target_type', key: 'targetType', width: 200 },
  { title: '操作', key: 'action', width: 120, fixed: 'right' }
]
</script>

<style scoped>
.mapping-table {
  background: transparent;
}
.mapping-table :deep(.ant-table-thead > tr > th) {
  background: var(--color-background);
  border-bottom: 2px solid var(--color-border);
  font-weight: 600;
  color: var(--color-text);
}
.mapping-table :deep(.ant-table-tbody > tr:hover > td) {
  background: var(--color-hover);
}
.mapping-table :deep(.ant-table-tbody > tr > td) {
  border-bottom: 1px solid var(--color-border-light);
}
.type-code {
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  padding: 2px 8px;
  background: var(--color-primary-bg);
  color: var(--color-primary);
  border-radius: 4px;
}
.type-code.target {
  background: var(--color-success-bg, #f6ffed);
  color: var(--color-success);
}
.editing-cell {
  display: flex;
  align-items: center;
}
.target-type-cell {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  cursor: pointer;
  padding: 4px 0;
}
.target-type-cell:hover .edit-hint {
  opacity: 1;
}
.edit-hint {
  font-size: 12px;
  color: var(--color-text-muted);
  opacity: 0;
  transition: opacity 0.2s;
}
</style>
