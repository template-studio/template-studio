<template>
  <a-drawer :open="open" :title="view === 'overview' ? '表结构同步' : `列对比 - ${currentTable?.name}`" :width="780" placement="right"
    @update:open="$emit('update:open', $event)" :body-style="{ position: 'relative' }">
    <div class="drawer-resize-handle" @mousedown="startResize"></div>
    <template #title>
      <div style="display: flex; align-items: center; gap: 8px">
        <DiffOutlined />
        <template v-if="view === 'overview'">
          <span style="font-weight: 600">表结构同步</span>
          <a-tag>{{ project?.name }}</a-tag>
        </template>
        <template v-else>
          <a-button type="link" size="small" @click="fetchOverview" style="padding: 0"><ArrowLeftOutlined /> 返回</a-button>
          <span style="font-weight: 600">{{ currentTable?.name }}</span>
          <a-tag>本地 vs 远程</a-tag>
        </template>
      </div>
    </template>

    <a-spin :spinning="loading" tip="正在检测表结构差异...">
      <a-alert v-if="error" :message="error" type="error" show-icon style="margin-bottom: 16px" />

      <!-- 总览视图 -->
      <template v-if="view === 'overview' && !loading && !error">
        <div class="overview-summary">
          <a-tag v-if="newTables.length > 0" color="success">远程新增 {{ newTables.length }}</a-tag>
          <a-tag v-if="removedTables.length > 0" color="warning">本地新增 {{ removedTables.length }}</a-tag>
          <a-tag v-if="syncedTables.length > 0" color="blue">已同步 {{ syncedTables.length }}</a-tag>
        </div>

        <!-- 远程新增表 -->
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
                <a-button type="link" size="small" @click="handleImportSingle(record)" :loading="importingName === record.name">导入</a-button>
              </template>
            </template>
          </a-table>
        </template>

        <!-- 本地多余表 -->
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
                  <a-button type="link" size="small" @click="handlePushToRemote(record)" :loading="pushingName === record.name">同步到远程</a-button>
                  <a-popconfirm title="确定删除本地表？" ok-text="确定" cancel-text="取消" @confirm="handleDeleteLocal(record)">
                    <a-button type="link" size="small" danger>删除</a-button>
                  </a-popconfirm>
                </a-space>
              </template>
            </template>
          </a-table>
        </template>

        <!-- 已同步表 -->
        <template v-if="syncedTables.length > 0">
          <h4 class="section-title"><SyncOutlined style="color: var(--color-primary)" /> 已同步（可对比列）</h4>
          <a-table :columns="syncedColumns" :data-source="syncedTables" :row-key="r => r.name" :pagination="false" size="small">
            <template #bodyCell="{ column, record }">
              <template v-if="column.key === 'name'">
                <span style="font-weight: 500">{{ record.name }}</span>
              </template>
              <template v-else-if="column.key === 'action'">
                <a-button type="link" size="small" @click="openDetail(record)">对比列</a-button>
              </template>
            </template>
          </a-table>
        </template>
      </template>

      <!-- 列对比视图 -->
      <template v-if="view === 'detail' && !loading && !error">
        <a-alert v-if="!remoteExists" message="远程表不存在" :description="`数据库中未找到表 \`${currentTable?.name}\`，可能已被删除`" type="warning" show-icon style="margin-bottom: 16px">
          <template #action>
            <a-button size="small" type="primary" danger @click="handleDeleteLocal(currentTable)">删除本地表</a-button>
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
      </template>
    </a-spin>

    <template #footer>
      <div style="display: flex; justify-content: space-between; align-items: center">
        <span v-if="syncing" style="color: var(--color-text-secondary)">同步中...</span>
        <span v-else-if="view === 'detail' && !remoteExists" style="color: var(--color-warning)">远程表不存在，可删除本地表</span>
        <span v-else-if="view === 'detail' && diffResult.hasChanges" style="color: var(--color-text-secondary)">将同步 {{ diffResult.added.length + diffResult.modified.length + diffResult.removed.length }} 项变更</span>
        <span v-else-if="view === 'detail' && diffResult.total > 0" style="color: var(--color-success)">表结构一致</span>
        <span v-else></span>
        <a-space>
          <a-button @click="$emit('update:open', false)">关闭</a-button>
          <a-button v-if="view === 'detail' && remoteExists" type="primary" :disabled="!diffResult.hasChanges || syncing" :loading="syncing" @click="handleSyncColumns">同步到本地</a-button>
        </a-space>
      </div>
    </template>
  </a-drawer>
</template>

<script setup>
import { ref, watch } from 'vue'
import { DiffOutlined, ArrowLeftOutlined, PlusCircleOutlined, MinusCircleOutlined, SyncOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { notify } from '@/utils/notify'
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
const view = ref('overview')
const importingName = ref(null)
const pushingName = ref(null)

// 总览数据
const newTables = ref([])
const removedTables = ref([])
const syncedTables = ref([])

// 列对比数据
const currentTable = ref(null)
const remoteExists = ref(true)
const localColumns = ref([])
const remoteColumns = ref([])
const diffResult = ref({ added: [], removed: [], modified: [], unchanged: [], total: 0, hasChanges: false })
const tableData = ref([])

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
  return {
    name: col.name,
    data_type: dataType,
    length,
    is_nullable: col.nullable === true || col.nullable === 'YES',
    is_primary_key: col.key === 'PRI',
    is_unique: col.key === 'PRI' || col.key === 'UNI',
    default_value: col.default || null,
    comment: col.comment || null
  }
}

const normalizeLocalColumn = (col) => {
  const { dataType, length } = parseColumnType(col.data_type)
  return {
    ...col,
    name: stripBackticks(col.name),
    data_type: dataType,
    length: length != null ? length : (typeof col.length === 'number' ? col.length : null)
  }
}

const getConnParams = () => {
  const ds = props.project.datasource
  const dbName = props.project.database_name
  return {
    type: ds.type_,
    host: ds.host || null,
    port: ds.port || null,
    username: ds.username || null,
    password: ds.password || null,
    database: dbName || ds.database || null,
    sqlite_file: ds.sqlite_file || null
  }
}

// ===== 总览逻辑 =====

const fetchOverview = async () => {
  if (!props.project) return
  loading.value = true
  error.value = ''
  view.value = 'overview'
  newTables.value = []
  removedTables.value = []
  syncedTables.value = []

  try {
    const [remote, local] = await Promise.all([fetchRemoteTables(), fetchLocalTables()])
    const remoteMap = new Map(remote.map(t => [t.name, t]))
    const localMap = new Map(local.map(t => [stripBackticks(t.name), t]))

    for (const [name, rTable] of remoteMap) {
      if (!localMap.has(name)) newTables.value.push(rTable)
      else syncedTables.value.push(localMap.get(name))
    }
    for (const [name, lTable] of localMap) {
      if (!remoteMap.has(name)) removedTables.value.push(lTable)
    }
  } catch (e) {
    error.value = '检测失败: ' + e
  } finally {
    loading.value = false
  }
}

const fetchRemoteTables = async () => {
  const ds = props.project.datasource
  const dbName = props.project.database_name
  if (ds.type_ === 'mysql') {
    return JSON.parse(await invoke('cmd_fetch_mysql_tables', { datasourceId: ds.id, databaseName: dbName }))
  } else if (ds.type_ === 'postgresql') {
    return JSON.parse(await invoke('cmd_fetch_postgresql_tables', { datasourceId: ds.id, databaseName: dbName }))
  } else if (ds.type_ === 'sqlite') {
    return JSON.parse(await invoke('cmd_fetch_sqlite_tables', { datasourceId: ds.id }))
  }
  throw new Error(`不支持的数据源类型: ${ds.type_}`)
}

const fetchLocalTables = async () => {
  return await projectsApi.getProjectTables(props.project.id)
}

// ===== 列对比逻辑 =====

const openDetail = async (table) => {
  currentTable.value = table
  view.value = 'detail'
  await fetchAndDiff()
}

const fetchAndDiff = async () => {
  if (!currentTable.value || !props.project) return
  loading.value = true
  error.value = ''
  remoteExists.value = true
  diffResult.value = { added: [], removed: [], modified: [], unchanged: [], total: 0, hasChanges: false }
  tableData.value = []

  try {
    const cleanName = stripBackticks(currentTable.value.name)
    const remoteTableNames = (await fetchRemoteTables()).map(t => t.name)
    if (!remoteTableNames.includes(cleanName)) {
      // 远程表不存在，切回总览视图（展示远程新增表等完整信息）
      loading.value = false
      await fetchOverview()
      return
    }

    const [local, remote] = await Promise.all([
      projectsApi.getTableColumns(currentTable.value.id),
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
  const params = getConnParams()
  const result = await invoke('cmd_get_table_columns', { params, tableName: stripBackticks(currentTable.value.name) })
  return JSON.parse(result)
}

const computeDiff = () => {
  const local = localColumns.value.map(normalizeLocalColumn)
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
      // MySQL 整数类型 CHARACTER_MAXIMUM_LENGTH 为 NULL，但 COLUMN_TYPE 含显示宽度如 bigint(20)，忽略单侧为空的长度差异
      if (lCol.length != null && rCol.length != null && lCol.length !== rCol.length) details.push(`长度: ${lCol.length} → ${rCol.length}`)
      if (lCol.is_nullable !== rCol.is_nullable) details.push(`可空: ${lCol.is_nullable ? '是' : '否'} → ${rCol.is_nullable ? '是' : '否'}`)
      if (lCol.is_primary_key !== rCol.is_primary_key) details.push(`主键: ${lCol.is_primary_key ? '是' : '否'} → ${rCol.is_primary_key ? '是' : '否'}`)
      const normalizeDefault = (v) => { const s = (v ?? '').trim().toUpperCase(); return (s === '' || s === 'NULL' || s === '-') ? '' : v }
      const lDef = normalizeDefault(lCol.default_value)
      const rDef = normalizeDefault(rCol.default_value)
      if (lDef !== rDef) details.push(`默认值: ${lCol.default_value || '-'} → ${rCol.default_value || '-'}`)

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

// ===== 操作 =====

const handleImportSingle = async (table) => {
  importingName.value = table.name
  try {
    const ds = props.project.datasource
    await invoke('cmd_import_single_table', {
      projectId: props.project.id,
      datasourceId: ds.id,
      databaseName: props.project.database_name,
      tableName: table.name,
      tableComment: table.comment || null,
      tableType: table.table_type,
      engine: table.engine || null,
      rowCount: table.row_count || 0
    })
    notify({ type: 'success', title: '表导入成功', content: `表 "${table.name}" 已导入到本地` })
    newTables.value = newTables.value.filter(t => t.name !== table.name)
    syncedTables.value.push(table)
    emit('synced')
  } catch (e) {
    notify({ type: 'error', title: '导入失败', content: String(e) })
  } finally {
    importingName.value = null
  }
}

const stripBackticks = (s) => s ? s.replace(/^`|`$/g, '') : s

const handlePushToRemote = async (table) => {
  pushingName.value = table.name
  try {
    const tableName = stripBackticks(table.name)
    if (!tableName) throw new Error('表名为空')
    const cols = await projectsApi.getTableColumns(table.id)
    if (!cols.length) throw new Error(`表 "${tableName}" 没有字段定义`)
    const params = getConnParams()
    const columns = cols.map(c => {
      let dataType = c.data_type || ''
      let length = typeof c.length === 'number' ? c.length : null
      // data_type 可能包含长度信息，如 "tinyint(none)" 或 "varchar(50)" 或 "tinyint(some(1))"
      const parenMatch = dataType.match(/^(.+?)\((.+)\)$/)
      if (parenMatch) {
        dataType = parenMatch[1]
        if (length == null) {
          const inner = parenMatch[2].trim()
          if (inner !== 'none' && inner !== 'None') {
            const someMatch = inner.match(/^some\((\d+)\)$/)
            if (someMatch) {
              length = parseInt(someMatch[1], 10)
            } else {
              const num = parseInt(inner, 10)
              if (!isNaN(num)) length = num
            }
          }
        }
      }
      return {
        name: stripBackticks(c.name),
        data_type: dataType,
        length,
        is_nullable: !!c.is_nullable,
        is_primary_key: !!c.is_primary_key,
        default_value: c.default_value || null,
        comment: c.comment || null
      }
    })
    await invoke('cmd_push_table_to_remote', {
      params,
      tableName,
      tableEngine: table.engine || null,
      tableComment: table.comment || null,
      columns
    })
    notify({ type: 'success', title: '同步到远程成功', content: `表 "${tableName}" 已同步到远程数据库` })
    removedTables.value = removedTables.value.filter(t => t.name !== table.name)
    syncedTables.value.push(table)
  } catch (e) {
    notify({ type: 'error', title: '同步到远程失败', content: String(e) })
  } finally {
    pushingName.value = null
  }
}

const handleDeleteLocal = async (table) => {
  try {
    await projectsApi.deleteTable(table.id)
    notify({ type: 'success', title: '本地表已删除', content: `本地表 "${table.name}" 已删除` })
    removedTables.value = removedTables.value.filter(t => t.name !== table.name)
    emit('synced')
  } catch (e) {
    notify({ type: 'error', title: '删除失败', content: String(e) })
  }
}

const handleSyncColumns = async () => {
  syncing.value = true
  try {
    const diff = diffResult.value
    const maxPos = localColumns.value.reduce((max, c) => Math.max(max, c.ordinal_position || 0), 0)
    let pos = maxPos

    for (const item of diff.added) {
      pos++
      await projectsApi.createColumn({
        tableId: currentTable.value.id,
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
    notify({ type: 'success', title: '列同步完成', content: parts.join(', ') })

    view.value = 'overview'
    await fetchOverview()
    emit('synced')
  } catch (e) {
    notify({ type: 'error', title: '列同步失败', content: String(e) })
  } finally {
    syncing.value = false
  }
}

// ===== 抽屉宽度拖拽 =====
const drawerWidth = ref(780)
let resizing = false
let resizeStartX = 0
let resizeStartWidth = 0

const startResize = (e) => {
  resizing = true
  resizeStartX = e.clientX
  resizeStartWidth = drawerWidth.value
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}
const onResize = (e) => {
  if (!resizing) return
  const diff = resizeStartX - e.clientX
  drawerWidth.value = Math.min(Math.max(resizeStartWidth + diff, 600), 1400)
}
const stopResize = () => {
  resizing = false
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

watch(() => props.open, (val) => {
  if (val) {
    if (props.table) {
      currentTable.value = props.table
      view.value = 'detail'
      fetchAndDiff()
    } else {
      fetchOverview()
    }
  }
})
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
:deep(.ant-table-wrapper) { overflow-x: hidden; }
.text-deleted { text-decoration: line-through; color: var(--color-error); }
.text-added { color: var(--color-success); font-weight: 500; }
:deep(.row-added td) { background: rgba(82, 196, 26, 0.06) !important; }
:deep(.row-removed td) { background: rgba(255, 77, 79, 0.06) !important; }
:deep(.row-modified td) { background: rgba(250, 173, 20, 0.06) !important; }
.drawer-resize-handle {
  position: absolute;
  top: 0;
  left: 0;
  width: 4px;
  height: 100%;
  cursor: col-resize;
  z-index: 10;
}
.drawer-resize-handle:hover,
.drawer-resize-handle:active {
  background: rgba(0, 0, 0, 0.15);
}
</style>
