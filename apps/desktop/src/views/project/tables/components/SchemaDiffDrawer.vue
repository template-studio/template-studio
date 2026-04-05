<template>
  <a-drawer :open="open" title="表结构对比" :width="720" placement="right" @update:open="$emit('update:open', $event)">
    <template #title>
      <div style="display: flex; align-items: center; gap: 8px">
        <DiffOutlined />
        <span style="font-weight: 600">{{ table?.name }}</span>
        <a-tag>本地 vs 远程</a-tag>
      </div>
    </template>

    <a-spin :spinning="loading" tip="正在对比表结构...">
      <a-empty v-if="!loading && !error && diffResult.total === 0 && !diffResult.hasChanges" description="暂无数据" />

      <a-alert v-if="error" :message="error" type="error" show-icon style="margin-bottom: 16px" />

      <template v-if="!loading && !error && diffResult.total > 0">
        <!-- 差异统计 -->
        <div class="diff-summary">
          <a-tag v-if="diffResult.added.length > 0" color="success">+{{ diffResult.added.length }} 新增</a-tag>
          <a-tag v-if="diffResult.removed.length > 0" color="error">-{{ diffResult.removed.length }} 删除</a-tag>
          <a-tag v-if="diffResult.modified.length > 0" color="warning">~{{ diffResult.modified.length }} 修改</a-tag>
          <a-tag v-if="diffResult.unchanged.length > 0">{{ diffResult.unchanged.length }} 未变</a-tag>
          <a-tag v-if="!diffResult.hasChanges" color="success">一致</a-tag>
        </div>

        <!-- 差异表格 -->
        <a-table :columns="columns" :data-source="tableData" :row-key="record => record.name" :pagination="false" size="small" :row-class-name="rowClassName">
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
    </a-spin>

    <template #footer>
      <div style="display: flex; justify-content: space-between; align-items: center">
        <span v-if="syncing" style="color: var(--color-text-secondary)">同步中...</span>
        <span v-else-if="diffResult.hasChanges" style="color: var(--color-text-secondary)">将同步 {{ diffResult.added.length + diffResult.modified.length + diffResult.removed.length }} 项变更</span>
        <span v-else-if="!loading && diffResult.total > 0" style="color: var(--color-success)">表结构一致，无需同步</span>
        <span v-else></span>
        <a-space>
          <a-button @click="$emit('update:open', false)">关闭</a-button>
          <a-button type="primary" :disabled="!diffResult.hasChanges || syncing" :loading="syncing" @click="handleSync">同步到本地</a-button>
        </a-space>
      </div>
    </template>
  </a-drawer>
</template>

<script setup>
import { ref, watch } from 'vue'
import { DiffOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import * as projectsApi from '@/api/projects'

const props = defineProps({
  open: { type: Boolean, default: false },
  table: { type: Object, default: null },
  project: { type: Object, default: null }
})
const emit = defineEmits(['update:open', 'synced'])

const loading = ref(false)
const syncing = ref(false)
const error = ref('')
const localColumns = ref([])
const remoteColumns = ref([])
const diffResult = ref({ added: [], removed: [], modified: [], unchanged: [], total: 0, hasChanges: false })
const tableData = ref([])

const columns = [
  { title: '列名', dataIndex: 'name', key: 'name', width: 120, ellipsis: true },
  { title: '本地类型', key: 'localType', width: 130 },
  { title: '远程类型', key: 'remoteType', width: 130 },
  { title: '状态', key: 'status', width: 70, align: 'center' },
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

const parseColumnType = (typeStr) => {
  if (!typeStr) return { dataType: typeStr, length: null }
  const match = typeStr.match(/^(\w+)(?:\(([^)]+)\))?/)
  if (match) {
    const lengthVal = match[2] ? parseInt(match[2], 10) : null
    return { dataType: match[1].toLowerCase(), length: isNaN(lengthVal) ? null : lengthVal }
  }
  return { dataType: typeStr.toLowerCase(), length: null }
}

const normalizeRemoteColumn = (col) => {
  const { dataType, length } = parseColumnType(col.type)
  const isPrimaryKey = col.key === 'PRI'
  const isUnique = col.key === 'PRI' || col.key === 'UNI'
  return {
    name: col.name,
    data_type: dataType,
    length,
    is_nullable: col.nullable === true || col.nullable === 'YES',
    is_primary_key: isPrimaryKey,
    is_unique: isUnique,
    default_value: col.default || null,
    comment: col.comment || null
  }
}

const computeDiff = () => {
  const local = localColumns.value
  const remote = remoteColumns.value.map(normalizeRemoteColumn)

  const localMap = new Map(local.map(c => [c.name, c]))
  const remoteMap = new Map(remote.map(c => [c.name, c]))

  const added = []
  const removed = []
  const modified = []
  const unchanged = []

  for (const [name, rCol] of remoteMap) {
    const lCol = localMap.get(name)
    if (!lCol) {
      added.push({ name, remote: rCol, local: null, status: 'added', detail: '' })
    } else {
      const details = []
      if (lCol.data_type !== rCol.data_type) details.push(`类型: ${lCol.data_type} → ${rCol.data_type}`)
      if (lCol.length !== rCol.length && !(lCol.length == null && rCol.length == null)) details.push(`长度: ${lCol.length ?? '-'} → ${rCol.length ?? '-'}`)
      if (lCol.is_nullable !== rCol.is_nullable) details.push(`可空: ${lCol.is_nullable ? '是' : '否'} → ${rCol.is_nullable ? '是' : '否'}`)
      if (lCol.is_primary_key !== rCol.is_primary_key) details.push(`主键: ${lCol.is_primary_key ? '是' : '否'} → ${rCol.is_primary_key ? '是' : '否'}`)
      const lDef = lCol.default_value || ''
      const rDef = rCol.default_value || ''
      if (lDef !== rDef) details.push(`默认值: ${lDef || '-'} → ${rDef || '-'}`)

      if (details.length > 0) {
        modified.push({ name, remote: rCol, local: lCol, status: 'modified', detail: details.join('; ') })
      } else {
        unchanged.push({ name, remote: rCol, local: lCol, status: 'unchanged', detail: '' })
      }
    }
  }

  for (const [name, lCol] of localMap) {
    if (!remoteMap.has(name)) {
      removed.push({ name, remote: null, local: lCol, status: 'removed', detail: '' })
    }
  }

  const result = { added, removed, modified, unchanged, total: added.length + removed.length + modified.length + unchanged.length, hasChanges: added.length + removed.length + modified.length > 0 }
  diffResult.value = result
  tableData.value = [...added, ...modified, ...removed, ...unchanged]
}

const fetchAndDiff = async () => {
  if (!props.table || !props.project) return
  loading.value = true
  error.value = ''
  try {
    const [local, remote] = await Promise.all([
      projectsApi.getTableColumns(props.table.id),
      fetchRemoteColumns()
    ])
    localColumns.value = local
    remoteColumns.value = remote
    computeDiff()
  } catch (e) {
    error.value = '对比失败: ' + e
  } finally {
    loading.value = false
  }
}

const fetchRemoteColumns = async () => {
  const ds = props.project.datasource
  const dbName = props.project.database_name
  const params = {
    type_: ds.type_,
    host: ds.host || null,
    port: ds.port || null,
    username: ds.username || null,
    password: ds.password || null,
    database: dbName || ds.database || null,
    sqlite_file: ds.sqlite_file || null
  }
  const result = await invoke('cmd_get_table_columns', { params, tableName: props.table.name })
  return JSON.parse(result)
}

const handleSync = async () => {
  syncing.value = true
  try {
    const diff = diffResult.value
    const maxPos = localColumns.value.reduce((max, c) => Math.max(max, c.ordinal_position || 0), 0)
    let pos = maxPos

    for (const item of diff.added) {
      pos++
      await projectsApi.createColumn({
        tableId: props.table.id,
        name: item.remote.name,
        dataType: item.remote.data_type,
        length: item.remote.length,
        isNullable: item.remote.is_nullable,
        isPrimaryKey: item.remote.is_primary_key,
        isUnique: item.remote.is_unique,
        defaultValue: item.remote.default_value || '',
        comment: item.remote.comment || '',
        ordinalPosition: pos
      })
    }

    for (const item of diff.modified) {
      await projectsApi.updateColumn(item.local.id, {
        name: item.remote.name,
        dataType: item.remote.data_type,
        length: item.remote.length,
        isNullable: item.remote.is_nullable,
        isPrimaryKey: item.remote.is_primary_key,
        isUnique: item.remote.is_unique,
        defaultValue: item.remote.default_value || '',
        comment: item.remote.comment || item.local.comment || ''
      })
    }

    for (const item of diff.removed) {
      await projectsApi.deleteColumn(item.local.id)
    }

    const parts = []
    if (diff.added.length > 0) parts.push(`${diff.added.length} 新增`)
    if (diff.modified.length > 0) parts.push(`${diff.modified.length} 修改`)
    if (diff.removed.length > 0) parts.push(`${diff.removed.length} 删除`)
    message.success('同步完成: ' + parts.join(', '))

    emit('synced')
    emit('update:open', false)
  } catch (e) {
    message.error('同步失败: ' + e)
  } finally {
    syncing.value = false
  }
}

watch(() => props.open, (val) => {
  if (val) fetchAndDiff()
})
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
:deep(.row-added td) { background: rgba(82, 196, 26, 0.06) !important; }
:deep(.row-removed td) { background: rgba(255, 77, 79, 0.06) !important; }
:deep(.row-modified td) { background: rgba(250, 173, 20, 0.06) !important; }
</style>
