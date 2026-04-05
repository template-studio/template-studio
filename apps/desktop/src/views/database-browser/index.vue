<template>
  <div class="database-browser">
    <!-- 顶部工具栏 -->
    <div class="browser-toolbar">
      <div class="toolbar-left">
        <a-button type="text" @click="goBack" class="back-btn">
          <template #icon><ArrowLeftOutlined /></template>
        </a-button>
        <div class="toolbar-title">
          <DatabaseOutlined class="title-icon" />
          <span class="datasource-name">{{ datasource?.name || '数据库浏览器' }}</span>
          <a-tag v-if="datasource" :color="getTypeColor(datasource.type_)">{{ datasource.type_.toUpperCase() }}</a-tag>
        </div>
      </div>
      <div class="toolbar-right">
        <a-input-search
          v-model:value="treeSearch"
          placeholder="搜索表..."
          size="small"
          style="width: 200px"
          allow-clear
        />
        <a-button size="small" @click="refreshTree">
          <template #icon><ReloadOutlined /></template>
        </a-button>
      </div>
    </div>

    <div class="browser-body">
      <!-- 左侧树 -->
      <div class="browser-sidebar" :style="{ width: sidebarWidth + 'px' }">
        <div class="sidebar-content">
          <a-spin :spinning="treeLoading" size="small">
            <div v-if="filteredDatabases.length > 0" class="tree-list">
              <div
                v-for="db in filteredDatabases"
                :key="db.name"
                class="tree-node database-node"
                :class="{ 'is-expanded': expandedDbs.has(db.name), 'is-selected': selectedDb === db.name && !selectedTable }"
              >
                <div class="tree-node-header" @click="toggleDatabase(db.name)">
                  <span class="tree-arrow">
                    <RightOutlined v-if="!expandedDbs.has(db.name)" />
                    <DownOutlined v-else />
                  </span>
                  <DatabaseOutlined class="tree-icon db-icon" />
                  <span class="tree-label">{{ db.name }}</span>
                  <span class="tree-count" v-if="db.tables.length">{{ db.tables.length }}</span>
                </div>
                <div v-if="expandedDbs.has(db.name)" class="tree-children">
                  <div
                    v-for="table in filterTables(db.tables)"
                    :key="table"
                    class="tree-node table-node"
                    :class="{ 'is-selected': selectedDb === db.name && selectedTable === table }"
                    @click="selectTable(db.name, table)"
                  >
                    <span class="tree-indent"></span>
                    <TableOutlined class="tree-icon table-icon" />
                    <span class="tree-label">{{ table }}</span>
                  </div>
                  <div v-if="filterTables(db.tables).length === 0 && treeSearch" class="tree-empty">
                    无匹配表
                  </div>
                </div>
              </div>
            </div>
            <div v-else-if="!treeLoading" class="tree-empty">
              暂无数据库
            </div>
          </a-spin>
        </div>
        <!-- 拖拽调整手柄 -->
        <div class="resize-handle" @mousedown="startResize"></div>
      </div>

      <!-- 右侧内容区 -->
      <div class="browser-content">
        <template v-if="selectedTable">
          <!-- 表头信息 -->
          <div class="content-header">
            <div class="header-left">
              <h3 class="table-title">
                <TableOutlined />
                {{ selectedTable }}
              </h3>
              <span class="table-meta">
                <span v-if="tableData">{{ tableData.total }} 行</span>
                <span v-if="columns.length"> · {{ columns.length }} 列</span>
              </span>
            </div>
            <div class="header-right">
              <a-segmented v-model:value="viewMode" :options="viewOptions" size="small" />
            </div>
          </div>

          <!-- 数据视图 -->
          <div class="content-body" v-if="viewMode === 'data'">
            <a-spin :spinning="dataLoading">
              <div class="data-table-wrapper">
                <table class="data-table" v-if="tableData && tableData.columns.length > 0">
                  <thead>
                    <tr>
                      <th class="row-num-header">#</th>
                      <th v-for="col in tableData.columns" :key="col" class="data-header">
                        {{ col }}
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(row, idx) in tableData.rows" :key="idx">
                      <td class="row-num">{{ dataOffset + idx + 1 }}</td>
                      <td v-for="(val, colIdx) in row" :key="colIdx" class="data-cell">
                        <span v-if="val === null" class="null-value">NULL</span>
                        <span v-else>{{ val }}</span>
                      </td>
                    </tr>
                  </tbody>
                </table>
                <a-empty v-else-if="!dataLoading" description="表中暂无数据" />
              </div>
            </a-spin>
          </div>

          <!-- 列信息视图 -->
          <div class="content-body" v-else>
            <a-spin :spinning="columnsLoading">
              <div class="columns-table-wrapper">
                <table class="columns-table" v-if="columns.length > 0">
                  <thead>
                    <tr>
                      <th>列名</th>
                      <th>类型</th>
                      <th>可空</th>
                      <th>键</th>
                      <th>默认值</th>
                      <th>注释</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="col in columns" :key="col.name">
                      <td class="col-name">
                        <span class="col-key-icon" v-if="col.key === 'PRI'"><KeyOutlined /></span>
                        {{ col.name }}
                      </td>
                      <td><a-tag>{{ col.type }}</a-tag></td>
                      <td>{{ col.nullable ? 'YES' : 'NO' }}</td>
                      <td>
                        <a-tag v-if="col.key === 'PRI'" color="orange">PK</a-tag>
                        <span v-else>-</span>
                      </td>
                      <td class="col-default">{{ col.default || '-' }}</td>
                      <td class="col-comment">{{ col.comment || '-' }}</td>
                    </tr>
                  </tbody>
                </table>
                <a-empty v-else-if="!columnsLoading" description="暂无列信息" />
              </div>
            </a-spin>
          </div>
        </template>

        <!-- 未选择表时的空状态 -->
        <div v-else class="content-empty">
          <div class="empty-inner">
            <TableOutlined class="empty-icon" />
            <p class="empty-text">选择左侧的表以浏览数据</p>
            <p class="empty-hint">点击数据库展开表列表，然后选择要查看的表</p>
          </div>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { message } from 'ant-design-vue'
import {
  ArrowLeftOutlined,
  DatabaseOutlined,
  TableOutlined,
  ReloadOutlined,
  RightOutlined,
  DownOutlined,
  KeyOutlined
} from '@ant-design/icons-vue'
import * as datasourcesApi from '@/api/datasources'
import { useLayoutStore } from '@/stores/layout'

const route = useRoute()
const router = useRouter()
const layoutStore = useLayoutStore()

// 数据源信息
const datasource = ref(null)

// 树状态
const treeLoading = ref(false)
const treeSearch = ref('')
const databases = ref([]) // [{ name, tables: [] }]
const expandedDbs = ref(new Set())
const selectedDb = ref('')
const selectedTable = ref('')

// 内容区状态
const viewMode = ref('data')
const viewOptions = [
  { label: '数据', value: 'data' },
  { label: '列信息', value: 'columns' }
]
const dataLoading = ref(false)
const columnsLoading = ref(false)
const tableData = ref(null) // { columns, rows, total }
const columns = ref([])

// 分页使用全局 store
const dataOffset = computed(() => {
  const { current, pageSize } = layoutStore.footerPagination
  return (current - 1) * pageSize
})

// 侧边栏宽度
const sidebarWidth = ref(240)
let resizing = false
let startX = 0
let startWidth = 0

// 筛选后的数据库列表
const filteredDatabases = computed(() => {
  if (!treeSearch.value) return databases.value
  const q = treeSearch.value.toLowerCase()
  return databases.value.filter(db => {
    if (db.name.toLowerCase().includes(q)) return true
    return db.tables.some(t => t.toLowerCase().includes(q))
  })
})

// 筛选表列表
const filterTables = (tables) => {
  if (!treeSearch.value) return tables
  const q = treeSearch.value.toLowerCase()
  return tables.filter(t => t.toLowerCase().includes(q))
}

// 获取类型颜色
const getTypeColor = (type) => {
  const colors = { mysql: 'blue', postgresql: 'cyan', sqlite: 'green' }
  return colors[type] || 'default'
}

// 返回数据源页面
const goBack = () => {
  router.push('/datasource')
}

// 加载数据源信息和数据库列表
const loadDatasource = async () => {
  const id = Number(route.params.id)
  if (!id) return

  try {
    // 获取数据源列表，找到匹配的
    const all = await datasourcesApi.getAllDatasources()
    datasource.value = all.find(d => d.id === id)
    if (!datasource.value) {
      message.error('数据源不存在')
      router.push('/datasource')
      return
    }
    await loadDatabases()
  } catch (error) {
    message.error('加载数据源失败: ' + error)
  }
}

// 更新数据库概览 footer
const updateDbOverview = () => {
  const dbCount = databases.value.length
  const tableCount = databases.value.reduce((sum, db) => sum + db.tables.length, 0)
  layoutStore.showFooterOverview([
    { label: '数据源', value: datasource.value?.name || '-' },
    { label: '类型', value: datasource.value?.type_?.toUpperCase() || '-' },
    { label: '数据库', value: dbCount },
    { label: '已加载表', value: tableCount }
  ])
}

// 加载数据库列表
const loadDatabases = async () => {
  if (!datasource.value) return
  treeLoading.value = true
  databases.value = []

  try {
    const ds = datasource.value
    if (ds.type_ === 'sqlite') {
      // SQLite 直接获取表列表
      const tables = await datasourcesApi.getDatabaseTables(ds)
      databases.value = [{ name: ds.sqlite_file || 'main', tables }]
      expandedDbs.value = new Set([databases.value[0].name])
      selectedDb.value = databases.value[0].name
    } else {
      // MySQL/PostgreSQL 获取数据库列表
      const dbNames = await datasourcesApi.getDatabaseTables({ ...ds, database: '' })
      databases.value = dbNames.map(name => ({ name, tables: [] }))

      // 如果数据源已指定数据库，自动展开
      if (ds.database) {
        expandedDbs.value = new Set([ds.database])
        selectedDb.value = ds.database
        await loadTables(ds.database)
      }
    }
    // 显示数据库概览 footer
    updateDbOverview()
  } catch (error) {
    message.error('加载数据库列表失败: ' + error)
  } finally {
    treeLoading.value = false
  }
}

// 展开/折叠数据库
const toggleDatabase = async (dbName) => {
  if (expandedDbs.value.has(dbName)) {
    expandedDbs.value.delete(dbName)
    expandedDbs.value = new Set(expandedDbs.value)
    return
  }

  expandedDbs.value.add(dbName)
  expandedDbs.value = new Set(expandedDbs.value)
  selectedDb.value = dbName
  selectedTable.value = ''
  tableData.value = null
  columns.value = []
  // 恢复概览 footer
  updateDbOverview()

  // 如果还没有加载表，加载
  const db = databases.value.find(d => d.name === dbName)
  if (db && db.tables.length === 0) {
    await loadTables(dbName)
  }
}

// 加载指定数据库的表
const loadTables = async (dbName) => {
  try {
    const ds = datasource.value
    const tables = await datasourcesApi.getDatabaseTables({ ...ds, database: dbName })
    const db = databases.value.find(d => d.name === dbName)
    if (db) {
      db.tables = tables
    }
    // 更新概览（未选中表时）
    if (!selectedTable.value) {
      updateDbOverview()
    }
  } catch (error) {
    message.error('加载表列表失败: ' + error)
  }
}

// 选择表
const selectTable = async (dbName, tableName) => {
  selectedDb.value = dbName
  selectedTable.value = tableName
  // 先显示 footer（数据加载完成前就展示分页区域）
  layoutStore.showFooterPagination(0, 1, 100, ['50', '100', '200', '500'])
  await Promise.all([loadColumns(), loadData()])
}

// 加载列信息
const loadColumns = async () => {
  if (!datasource.value || !selectedTable.value) return
  columnsLoading.value = true
  try {
    const ds = { ...datasource.value, database: selectedDb.value }
    columns.value = await datasourcesApi.getTableColumns(ds, selectedTable.value)
    // 如果当前是列信息视图，更新概览 footer
    if (viewMode.value === 'columns') {
      updateFooterOverview()
    }
  } catch (error) {
    message.error('获取列信息失败: ' + error)
  } finally {
    columnsLoading.value = false
  }
}

// 加载表数据
const loadData = async () => {
  if (!datasource.value || !selectedTable.value) return
  dataLoading.value = true
  try {
    const ds = { ...datasource.value, database: selectedDb.value }
    const { pageSize } = layoutStore.footerPagination
    tableData.value = await datasourcesApi.queryTableData(
      ds, selectedTable.value, pageSize, dataOffset.value
    )
    // 更新全局 footer 分页
    layoutStore.showFooterPagination(
      tableData.value.total,
      layoutStore.footerPagination.current,
      pageSize,
      ['50', '100', '200', '500']
    )
  } catch (error) {
    message.error('查询表数据失败: ' + error)
  } finally {
    dataLoading.value = false
  }
}

// 更新列信息概览 footer
const updateFooterOverview = () => {
  if (!columns.value.length) return
  const pkCols = columns.value.filter(c => c.key === 'PRI').map(c => c.name).join(', ') || '无'
  const nullableCount = columns.value.filter(c => c.nullable).length
  layoutStore.showFooterOverview([
    { label: '数据库', value: selectedDb.value },
    { label: '表', value: selectedTable.value },
    { label: '列数', value: columns.value.length },
    { label: '主键', value: pkCols },
    { label: '可空列', value: `${nullableCount} / ${columns.value.length}` }
  ])
}

// 刷新树
const refreshTree = async () => {
  selectedTable.value = ''
  tableData.value = null
  columns.value = []
  await loadDatabases() // loadDatabases 末尾会调用 updateDbOverview
}

// 拖拽调整侧边栏宽度
const startResize = (e) => {
  resizing = true
  startX = e.clientX
  startWidth = sidebarWidth.value
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

const onResize = (e) => {
  if (!resizing) return
  const diff = e.clientX - startX
  const newWidth = Math.min(Math.max(startWidth + diff, 180), 500)
  sidebarWidth.value = newWidth
}

const stopResize = () => {
  resizing = false
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

// 监听 viewMode 切换
watch(viewMode, (mode) => {
  if (!selectedTable.value) return
  if (mode === 'data') {
    if (!tableData.value) {
      loadData()
    } else {
      // 恢复分页 footer
      layoutStore.showFooterPagination(
        tableData.value.total,
        layoutStore.footerPagination.current,
        layoutStore.footerPagination.pageSize,
        ['50', '100', '200', '500']
      )
    }
  } else if (mode === 'columns') {
    if (columns.value.length === 0) {
      loadColumns()
    } else {
      updateFooterOverview()
    }
  }
})

// 监听全局分页变化，重新加载数据
watch(
  () => [layoutStore.footerPagination.current, layoutStore.footerPagination.pageSize],
  ([newPage, newPageSize], [oldPage, oldPageSize]) => {
    if (!selectedTable.value || viewMode.value !== 'data') return
    if (newPage !== oldPage || newPageSize !== oldPageSize) {
      loadData()
    }
  }
)

onMounted(() => {
  loadDatasource()
})

onUnmounted(() => {
  layoutStore.hideFooter()
})
</script>

<style scoped>
.database-browser {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--color-background);
}

/* 顶部工具栏 */
.browser-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
  flex-shrink: 0;
  height: 44px;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.back-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
}

.toolbar-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-icon {
  font-size: 16px;
  color: var(--color-primary);
}

.datasource-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 主体区域 */
.browser-body {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
}

/* 左侧树 */
.browser-sidebar {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--color-border);
  position: relative;
  flex-shrink: 0;
  overflow: hidden;
}

.sidebar-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.tree-list {
  display: flex;
  flex-direction: column;
}

/* 树节点 */
.tree-node-header {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  cursor: pointer;
  transition: background 0.15s;
  user-select: none;
}

.tree-node-header:hover {
  background: var(--color-hover);
}

.database-node.is-selected > .tree-node-header {
  background: var(--color-primary-bg);
}

.tree-arrow {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.tree-icon {
  font-size: 14px;
  flex-shrink: 0;
}

.db-icon {
  color: var(--color-primary);
}

.table-icon {
  color: var(--color-success);
}

.tree-label {
  font-size: 13px;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.tree-count {
  font-size: 11px;
  color: var(--color-text-muted);
  background: var(--color-surface);
  padding: 0 6px;
  border-radius: 10px;
  flex-shrink: 0;
}

.tree-children {
  display: flex;
  flex-direction: column;
}

.table-node {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 12px 5px 28px;
  cursor: pointer;
  transition: background 0.15s;
  user-select: none;
}

.table-node:hover {
  background: var(--color-hover);
}

.table-node.is-selected {
  background: var(--color-primary-bg);
}

.tree-indent {
  width: 16px;
  flex-shrink: 0;
}

.tree-empty {
  padding: 12px 16px;
  color: var(--color-text-muted);
  font-size: 13px;
  text-align: center;
}

/* 拖拽调整手柄 */
.resize-handle {
  position: absolute;
  top: 0;
  right: -2px;
  width: 4px;
  height: 100%;
  cursor: col-resize;
  z-index: 10;
}

.resize-handle:hover,
.resize-handle:active {
  background: var(--color-primary);
}

/* 右侧内容区 */
.browser-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

/* 内容头 */
.content-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.table-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  display: flex;
  align-items: center;
  gap: 6px;
}

.table-meta {
  font-size: 13px;
  color: var(--color-text-secondary);
}

/* 内容主体 */
.content-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.content-body :deep(.ant-spin-nested-loading),
.content-body :deep(.ant-spin-container) {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* 数据表格 */
.data-table-wrapper {
  flex: 1;
  overflow: auto;
  min-height: 0;
  height: 100%;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
  table-layout: auto;
  color: var(--color-text);
}

.data-table th,
.data-table td {
  padding: 6px 12px;
  border-bottom: 1px solid var(--color-border);
  border-right: 1px solid var(--color-border-light);
  text-align: left;
  white-space: nowrap;
  color: var(--color-text);
}

.data-table thead {
  position: sticky;
  top: 0;
  z-index: 1;
}

.data-header {
  background: var(--color-surface);
  font-weight: 600;
  color: var(--color-text);
  border-right: 1px solid var(--color-border);
}

.row-num-header {
  background: var(--color-surface);
  width: 50px;
  text-align: center;
  color: var(--color-text-muted);
  border-right: 1px solid var(--color-border);
}

.row-num {
  width: 50px;
  text-align: center;
  color: var(--color-text-muted);
  font-size: 11px;
  background: var(--color-surface);
  border-right: 1px solid var(--color-border);
}

.data-cell {
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-text);
}

.null-value {
  color: var(--color-text-muted);
  font-style: italic;
  font-size: 12px;
}


/* 列信息表格 */
.columns-table-wrapper {
  flex: 1;
  overflow: auto;
  padding: 0;
  height: 100%;
}

.columns-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
  color: var(--color-text);
}

.columns-table th {
  padding: 8px 16px;
  background: var(--color-surface);
  font-weight: 600;
  color: var(--color-text);
  border-bottom: 1px solid var(--color-border);
  text-align: left;
  position: sticky;
  top: 0;
  z-index: 1;
}

.columns-table td {
  padding: 8px 16px;
  border-bottom: 1px solid var(--color-border);
  color: var(--color-text);
}

.col-name {
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 4px;
}

.col-key-icon {
  color: var(--color-warning);
  font-size: 12px;
}

.col-default {
  color: var(--color-text-secondary);
  font-family: 'Courier New', monospace;
  font-size: 12px;
}

.col-comment {
  color: var(--color-text-secondary);
}

/* 空状态 */
.content-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-inner {
  text-align: center;
}

.empty-icon {
  font-size: 48px;
  color: var(--color-text-muted);
  margin-bottom: 16px;
}

.empty-text {
  font-size: 16px;
  color: var(--color-text);
  margin: 0 0 8px;
}

.empty-hint {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin: 0;
}
</style>
