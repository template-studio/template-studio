<template>
  <div>
    <a-alert v-if="!remoteExists" message="远程表不存在" :description="`数据库中未找到表 \`${currentTable?.name}\`，可能已被删除`" type="warning" show-icon style="margin-bottom: 16px">
      <template #action>
        <a-button size="small" type="primary" danger @click="emit('delete-local', currentTable)">删除本地表</a-button>
      </template>
    </a-alert>

    <template v-if="diffResult.total > 0">
      <div class="diff-summary">
        <a-tag v-if="diffResult.added.length > 0" color="success">+{{ diffResult.added.length }} 新增</a-tag>
        <a-tag v-if="diffResult.removed.length > 0" color="error">-{{ diffResult.removed.length }} 删除</a-tag>
        <a-tag v-if="diffResult.modified.length > 0" color="warning">~{{ diffResult.modified.length }} 修改</a-tag>
        <a-tag v-if="diffResult.unchanged.length > 0">{{ diffResult.unchanged.length }} 未变</a-tag>
        <a-tag v-if="!diffResult.hasChanges" color="success">一致</a-tag>
      </div>

      <a-table :columns="diffColumns" :data-source="tableData" :row-key="record => record.name" :pagination="false" size="small" :row-class-name="rowClassName">
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'localType'">
            <span v-if="record.local" :class="{ 'text-deleted': record.status === 'removed' }">{{ formatType(record.local) }}</span>
            <span v-else class="text-muted">-</span>
          </template>
          <template v-else-if="column.key === 'remoteType'">
            <span v-if="record.remote" :class="{ 'text-added': record.status === 'added' }">{{ formatType(record.remote) }}</span>
            <span v-else class="text-muted">-</span>
          </template>
          <template v-else-if="column.key === 'status'">
            <a-tag v-if="record.status === 'added'" color="success">新增</a-tag>
            <a-tag v-else-if="record.status === 'removed'" color="error">删除</a-tag>
            <a-tag v-else-if="record.status === 'modified'" color="warning">修改</a-tag>
            <a-tag v-else>未变</a-tag>
          </template>
          <template v-else-if="column.key === 'detail'">
            <span v-if="record.status === 'modified'" class="diff-detail">{{ record.detail }}</span>
            <span v-else class="text-muted">-</span>
          </template>
        </template>
      </a-table>
    </template>
  </div>
</template>

<script setup>
defineProps({
  currentTable: { type: Object, default: null },
  remoteExists: { type: Boolean, default: true },
  diffResult: { type: Object, default: () => ({ added: [], removed: [], modified: [], unchanged: [], total: 0, hasChanges: false }) },
  tableData: { type: Array, default: () => [] }
})

const emit = defineEmits(['delete-local'])

const diffColumns = [
  { title: '列名', dataIndex: 'name', key: 'name', width: 100, ellipsis: true },
  { title: '本地类型', key: 'localType', width: 110 },
  { title: '远程类型', key: 'remoteType', width: 110 },
  { title: '状态', key: 'status', width: 60, align: 'center' },
  { title: '差异', key: 'detail', ellipsis: true }
]

const rowClassName = (record) => {
  if (record.status === 'added') return 'row-added'
  if (record.status === 'removed') return 'row-removed'
  if (record.status === 'modified') return 'row-modified'
  return ''
}

const formatType = (col) => {
  let type = col.data_type.toUpperCase()
  if (col.length) type += `(${col.length})`
  return type
}
</script>

<style scoped>
.diff-summary {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
.diff-detail {
  color: var(--color-text-secondary);
  font-size: 12px;
}
.text-muted { color: var(--color-text-muted); }
.text-deleted { text-decoration: line-through; color: var(--color-error); }
.text-added { color: var(--color-success); font-weight: 500; }
:deep(.ant-table-wrapper) { overflow-x: hidden; }
:deep(.row-added td) { background: rgba(82, 196, 26, 0.06) !important; }
:deep(.row-removed td) { background: rgba(255, 77, 79, 0.06) !important; }
:deep(.row-modified td) { background: rgba(250, 173, 20, 0.06) !important; }
</style>
