<template>
  <div>
    <div class="overview-summary">
      <a-tag v-if="newTables.length > 0" color="success">远程新增 {{ newTables.length }}</a-tag>
      <a-tag v-if="removedTables.length > 0" color="warning">本地新增 {{ removedTables.length }}</a-tag>
      <a-tag v-if="syncedTables.length > 0" color="blue">已同步 {{ syncedTables.length }}</a-tag>
    </div>

    <template v-if="newTables.length > 0">
      <h4 class="section-title"><PlusCircleOutlined style="color: var(--color-success)" /> 远程新增（可导入）</h4>
      <a-table :columns="overviewColumns" :data-source="newTables" :row-key="r => r.name" :pagination="false" size="small">
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'name'">
            <span style="font-weight: 500">{{ record.name }}</span>
          </template>
          <template v-else-if="column.key === 'type'">
            <a-tag :color="record.table_type === 'view' ? 'purple' : 'blue'">{{ record.table_type === 'view' ? '视图' : '表' }}</a-tag>
          </template>
          <template v-else-if="column.key === 'action'">
            <a-button type="link" size="small" @click="emit('import-single', record)" :loading="importingName === record.name">导入</a-button>
          </template>
        </template>
      </a-table>
    </template>

    <template v-if="removedTables.length > 0">
      <h4 class="section-title"><MinusCircleOutlined style="color: var(--color-warning)" /> 本地新增（未同步到远程）</h4>
      <a-table :columns="overviewColumns" :data-source="removedTables" :row-key="r => r.name" :pagination="false" size="small">
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'name'">
            <span style="font-weight: 500">{{ record.name }}</span>
          </template>
          <template v-else-if="column.key === 'type'">
            <a-tag :color="record.table_type === 'view' ? 'purple' : 'blue'">{{ record.table_type === 'view' ? '视图' : '表' }}</a-tag>
          </template>
          <template v-else-if="column.key === 'action'">
            <a-space>
              <a-button type="link" size="small" @click="emit('push-to-remote', record)" :loading="pushingName === record.name">同步到远程</a-button>
              <a-popconfirm title="确定删除本地表？" ok-text="确定" cancel-text="取消" @confirm="emit('delete-local', record)">
                <a-button type="link" size="small" danger>删除</a-button>
              </a-popconfirm>
            </a-space>
          </template>
        </template>
      </a-table>
    </template>

    <template v-if="syncedTables.length > 0">
      <h4 class="section-title"><SyncOutlined style="color: var(--color-primary)" /> 已同步（可对比列）</h4>
      <a-table :columns="syncedColumns" :data-source="syncedTables" :row-key="r => r.name" :pagination="false" size="small">
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'name'">
            <span style="font-weight: 500">{{ record.name }}</span>
          </template>
          <template v-else-if="column.key === 'action'">
            <a-button type="link" size="small" @click="emit('compare-table', record)">对比列</a-button>
          </template>
        </template>
      </a-table>
    </template>
  </div>
</template>

<script setup>
import { PlusCircleOutlined, MinusCircleOutlined, SyncOutlined } from '@ant-design/icons-vue'

defineProps({
  newTables: { type: Array, default: () => [] },
  removedTables: { type: Array, default: () => [] },
  syncedTables: { type: Array, default: () => [] },
  importingName: { type: String, default: null },
  pushingName: { type: String, default: null }
})

const emit = defineEmits(['import-single', 'push-to-remote', 'compare-table', 'delete-local'])

const overviewColumns = [
  { title: '表名', dataIndex: 'name', key: 'name', ellipsis: true },
  { title: '类型', dataIndex: 'table_type', key: 'type', width: 60, align: 'center' },
  { title: '说明', dataIndex: 'comment', key: 'comment', ellipsis: true },
  { title: '操作', key: 'action', width: 150, align: 'center' }
]

const syncedColumns = [
  { title: '表名', dataIndex: 'name', key: 'name', ellipsis: true },
  { title: '说明', dataIndex: 'comment', key: 'comment', ellipsis: true },
  { title: '操作', key: 'action', width: 80, align: 'center' }
]
</script>

<style scoped>
.overview-summary {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}
.section-title {
  font-size: 14px;
  font-weight: 600;
  margin: 16px 0 8px;
  display: flex;
  align-items: center;
  gap: 6px;
}
</style>
