<template>
  <div class="database-browser">
    <DatabaseToolbar
      :datasource="datasource"
      v-model:tree-search="treeSearch"
      @go-back="goBack"
      @refresh="refreshTree"
    />

    <div class="browser-body">
      <DatabaseTreePanel
        :databases="databases"
        :filtered-databases="filteredDatabases"
        :expanded-dbs="expandedDbs"
        :selected-db="selectedDb"
        :selected-table="selectedTable"
        :tree-search="treeSearch"
        :tree-loading="treeLoading"
        :sidebar-width="sidebarWidth"
        @toggle-database="toggleDatabase"
        @select-table="selectTable"
        @start-resize="startResize"
      />

      <TableContentView
        :selected-table="selectedTable"
        :table-data="tableData"
        :columns="columns"
        :view-mode="viewMode"
        :view-options="viewOptions"
        :data-loading="dataLoading"
        :columns-loading="columnsLoading"
        :data-offset="dataOffset"
        @update:view-mode="viewMode = $event"
      />
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { message } from 'ant-design-vue'
import * as datasourcesApi from '@/api/datasources'
import { useLayoutStore } from '@/stores/layout'
import DatabaseToolbar from './components/DatabaseToolbar.vue'
import DatabaseTreePanel from './components/DatabaseTreePanel.vue'
import TableContentView from './components/TableContentView.vue'

const route = useRoute()
const router = useRouter()
const layoutStore = useLayoutStore()

const datasource = ref(null)

const treeLoading = ref(false)
const treeSearch = ref('')
const databases = ref([])
const expandedDbs = ref(new Set())
const selectedDb = ref('')
const selectedTable = ref('')

const viewMode = ref('data')
const viewOptions = [
  { label: '数据', value: 'data' },
  { label: '列信息', value: 'columns' }
]
const dataLoading = ref(false)
const columnsLoading = ref(false)
const tableData = ref(null)
const columns = ref([])

const dataOffset = computed(() => {
  const { current, pageSize } = layoutStore.footerPagination
  return (current - 1) * pageSize
})

const sidebarWidth = ref(240)
let resizing = false
let startX = 0
let startWidth = 0

const filteredDatabases = computed(() => {
  if (!treeSearch.value) return databases.value
  const q = treeSearch.value.toLowerCase()
  return databases.value.filter(db => {
    if (db.name.toLowerCase().includes(q)) return true
    return db.tables.some(t => t.toLowerCase().includes(q))
  })
})

const goBack = () => {
  router.push('/datasource')
}

const loadDatasource = async () => {
  const id = Number(route.params.id)
  if (!id) return

  try {
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

const loadDatabases = async () => {
  if (!datasource.value) return
  treeLoading.value = true
  databases.value = []

  try {
    const ds = datasource.value
    if (ds.type_ === 'sqlite') {
      const tables = await datasourcesApi.getDatabaseTables(ds)
      databases.value = [{ name: ds.sqlite_file || 'main', tables }]
      expandedDbs.value = new Set([databases.value[0].name])
      selectedDb.value = databases.value[0].name
    } else {
      const dbNames = await datasourcesApi.getDatabaseTables({ ...ds, database: '' })
      databases.value = dbNames.map(name => ({ name, tables: [] }))

      if (ds.database) {
        expandedDbs.value = new Set([ds.database])
        selectedDb.value = ds.database
        await loadTables(ds.database)
      }
    }
    updateDbOverview()
  } catch (error) {
    message.error('加载数据库列表失败: ' + error)
  } finally {
    treeLoading.value = false
  }
}

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
  updateDbOverview()

  const db = databases.value.find(d => d.name === dbName)
  if (db && db.tables.length === 0) {
    await loadTables(dbName)
  }
}

const loadTables = async (dbName) => {
  try {
    const ds = datasource.value
    const tables = await datasourcesApi.getDatabaseTables({ ...ds, database: dbName })
    const db = databases.value.find(d => d.name === dbName)
    if (db) {
      db.tables = tables
    }
    if (!selectedTable.value) {
      updateDbOverview()
    }
  } catch (error) {
    message.error('加载表列表失败: ' + error)
  }
}

const selectTable = async (dbName, tableName) => {
  selectedDb.value = dbName
  selectedTable.value = tableName
  layoutStore.showFooterPagination(0, 1, 100, ['50', '100', '200', '500'])
  await Promise.all([loadColumns(), loadData()])
}

const loadColumns = async () => {
  if (!datasource.value || !selectedTable.value) return
  columnsLoading.value = true
  try {
    const ds = { ...datasource.value, database: selectedDb.value }
    columns.value = await datasourcesApi.getTableColumns(ds, selectedTable.value)
    if (viewMode.value === 'columns') {
      updateFooterOverview()
    }
  } catch (error) {
    message.error('获取列信息失败: ' + error)
  } finally {
    columnsLoading.value = false
  }
}

const loadData = async () => {
  if (!datasource.value || !selectedTable.value) return
  dataLoading.value = true
  try {
    const ds = { ...datasource.value, database: selectedDb.value }
    const { pageSize } = layoutStore.footerPagination
    tableData.value = await datasourcesApi.queryTableData(
      ds, selectedTable.value, pageSize, dataOffset.value
    )
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

const refreshTree = async () => {
  selectedTable.value = ''
  tableData.value = null
  columns.value = []
  await loadDatabases()
}

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

watch(viewMode, (mode) => {
  if (!selectedTable.value) return
  if (mode === 'data') {
    if (!tableData.value) {
      loadData()
    } else {
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
.browser-body {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
}
</style>
