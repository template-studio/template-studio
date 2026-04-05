<template>
  <div class="tables-view">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <h2 class="page-title">{{ project?.name }}</h2>
        <a-tag v-if="project?.datasource" :color="getDatabaseColor(project.datasource.type_)">{{ getDatabaseLabel(project.datasource.type_) }}</a-tag>
        <span class="database-name">{{ project?.database_name }}</span>
      </div>
      <div class="header-right">
        <a-button type="primary" @click="aiCreateTableVisible = true">
          <template #icon><RobotOutlined /></template>AI 建表
        </a-button>
        <a-button type="primary" @click="showAddTableDialog">
          <template #icon><PlusOutlined /></template>新增表
        </a-button>
        <a-dropdown>
          <a-button><template #icon><MoreOutlined /></template></a-button>
          <template #overlay>
            <a-menu @click="handleHeaderMenu">
              <a-menu-item key="sql-import"><FileTextOutlined /> 从SQL导入</a-menu-item>
              <a-menu-item key="import-structure"><ImportOutlined /> 导入表结构</a-menu-item>
              <a-menu-item key="export"><ExportOutlined /> 导出SQL</a-menu-item>
              <a-menu-divider />
              <a-menu-item key="refresh"><ReloadOutlined /> 刷新</a-menu-item>
            </a-menu>
          </template>
        </a-dropdown>
        <a-button v-if="selectedRowKeys.length > 0" danger @click="batchDeleteTables">批量删除 ({{ selectedRowKeys.length }})</a-button>
      </div>
    </div>

    <!-- 搜索筛选排序工具栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <a-input v-model:value="searchQuery" placeholder="搜索表名或说明..." allow-clear style="width: 240px" @change="handleSearch">
          <template #prefix><SearchOutlined /></template>
        </a-input>
        <a-select v-model:value="filterValue" placeholder="筛选引擎" allow-clear style="width: 140px" @change="handleFilter">
          <a-select-option v-for="f in engineFilters" :key="f.value" :value="f.value">{{ f.label }}</a-select-option>
        </a-select>
        <a-select v-model:value="sortValue" style="width: 160px" @change="handleSort">
          <a-select-option v-for="s in sortOptions" :key="s.value" :value="s.value">{{ s.label }}</a-select-option>
        </a-select>
      </div>
      <div class="toolbar-right"><span class="result-count">共 {{ filteredTables.length }} 张表</span></div>
    </div>

    <!-- 表列表 -->
    <a-card :bordered="false" class="table-card">
      <a-table :columns="columns" :data-source="paginatedTables" :row-key="record => record.id" :row-selection="rowSelection" :pagination="false" :loading="loading" size="small">
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'name'">
            <a @click="viewColumns(record)" style="font-weight: 500"><TableOutlined style="margin-right: 4px" />{{ record.name }}</a>
            <a-tag v-if="record.table_type === 'view'" color="purple" style="margin-left: 8px">视图</a-tag>
          </template>
          <template v-else-if="column.key === 'comment'">
            <a-tooltip v-if="record.comment" :title="record.comment"><span class="comment-text">{{ record.comment }}</span></a-tooltip>
            <span v-else class="comment-empty">-</span>
          </template>
          <template v-else-if="column.key === 'engine'">
            <a-tag v-if="record.engine" color="cyan">{{ record.engine }}</a-tag>
            <span v-else style="color: var(--color-text-secondary)">-</span>
          </template>
          <template v-else-if="column.key === 'column_count'"><a-tag color="blue">{{ record.column_count }}</a-tag></template>
          <template v-else-if="column.key === 'updated_at'"><span style="color: var(--color-text-secondary)">{{ formatDate(record.updated_at) }}</span></template>
          <template v-else-if="column.key === 'action'">
            <a-space>
              <a-button type="link" size="small" @click="openTableConfig(record)"><SettingOutlined /> 配置</a-button>
              <a-button type="link" size="small" @click="viewColumns(record)">查看字段</a-button>
              <a-button type="link" size="small" @click="editTable(record)">编辑</a-button>
              <a-popconfirm title="确定要删除这张表吗？" ok-text="确定" cancel-text="取消" @confirm="deleteTable(record)">
                <a-button type="link" size="small" danger>删除</a-button>
              </a-popconfirm>
            </a-space>
          </template>
        </template>
      </a-table>
    </a-card>

    <!-- 子组件 -->
    <ImportProgressModal ref="importProgressRef" v-model:open="importDialogVisible" :project="project" :existing-table-names="existingTableNames" @imported="loadTables" />
    <SqlImportModal v-model:open="sqlImportDialogVisible" :project="project" @imported="loadTables" />
    <TableDialog v-model:open="addTableDialogVisible" mode="add" :project="project" @saved="loadTables" />
    <TableDialog v-model:open="editTableDialogVisible" mode="edit" :table="currentEditTable" @saved="loadTables" />
    <ColumnsDrawer v-model:open="columnsDrawerVisible" :table="currentTable" @columns-updated="onColumnsUpdated" />
    <AiCreateTableDrawer v-model:open="aiCreateTableVisible" :project="project" @tables-created="loadTables" />
    <TableConfigDrawer v-model:open="tableConfigVisible" :table="currentConfigTable" @saved="onTableConfigSaved" />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useLayoutStore } from '@/stores/layout'
import {
  ImportOutlined, MoreOutlined, ReloadOutlined, TableOutlined,
  SearchOutlined, PlusOutlined, FileTextOutlined, RobotOutlined,
  SettingOutlined, ExportOutlined
} from '@ant-design/icons-vue'
import { TableConfigDrawer } from '@/components/tableConfig'
import { message, Modal } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import * as projectsApi from '@/api/projects'
import ImportProgressModal from './components/ImportProgressModal.vue'
import SqlImportModal from './components/SqlImportModal.vue'
import TableDialog from './components/TableDialog.vue'
import ColumnsDrawer from './components/ColumnsDrawer.vue'
import AiCreateTableDrawer from './components/AiCreateTableDrawer.vue'

const route = useRoute()
const layoutStore = useLayoutStore()

// 核心状态
const loading = ref(false)
const project = ref(null)
const tables = ref([])

// 搜索、筛选、排序
const searchQuery = ref('')
const filterValue = ref(undefined)
const sortValue = ref('name:asc')
const engineFilters = [
  { label: 'InnoDB', value: 'InnoDB' },
  { label: 'MyISAM', value: 'MyISAM' },
  { label: 'Memory', value: 'Memory' }
]
const sortOptions = [
  { label: '名称 A-Z', value: 'name:asc' },
  { label: '名称 Z-A', value: 'name:desc' },
  { label: '列数最多', value: 'column_count:desc' },
  { label: '列数最少', value: 'column_count:asc' },
  { label: '最新更新', value: 'updated_at:desc' },
  { label: '最早更新', value: 'updated_at:asc' }
]

const filteredTables = computed(() => {
  let result = [...tables.value]
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(t => t.name.toLowerCase().includes(query) || (t.comment && t.comment.toLowerCase().includes(query)))
  }
  if (filterValue.value) result = result.filter(t => t.engine === filterValue.value)
  if (sortValue.value) {
    const [field, order] = sortValue.value.split(':')
    result.sort((a, b) => {
      let valueA, valueB
      switch (field) {
        case 'name': valueA = a.name.toLowerCase(); valueB = b.name.toLowerCase(); break
        case 'column_count': valueA = a.column_count || 0; valueB = b.column_count || 0; break
        case 'updated_at': valueA = new Date(a.updated_at).getTime(); valueB = new Date(b.updated_at).getTime(); break
        default: return 0
      }
      return order === 'asc' ? (valueA > valueB ? 1 : -1) : (valueA < valueB ? 1 : -1)
    })
  }
  return result
})

const paginatedTables = computed(() => {
  const { current, pageSize } = layoutStore.footerPagination
  const start = (current - 1) * pageSize
  return filteredTables.value.slice(start, start + pageSize)
})

const existingTableNames = computed(() => tables.value.map(t => t.name))

// 行选择
const selectedRowKeys = ref([])
const rowSelection = {
  selectedRowKeys,
  onChange: (selectedKeys) => { selectedRowKeys.value = selectedKeys },
  columnWidth: 30, columnTitle: ' '
}

// 表格列定义
const columns = [
  { title: '表名', dataIndex: 'name', key: 'name', width: 140, ellipsis: true },
  { title: '说明', dataIndex: 'comment', key: 'comment', width: 100, ellipsis: true },
  { title: '引擎', dataIndex: 'engine', key: 'engine', width: 80 },
  { title: '列数', dataIndex: 'column_count', key: 'column_count', width: 60, align: 'center' },
  { title: '更新时间', dataIndex: 'updated_at', key: 'updated_at', width: 120 },
  { title: '操作', key: 'action', width: 160, fixed: 'right', align: 'center' }
]

// 子组件对话框状态
const importDialogVisible = ref(false)
const sqlImportDialogVisible = ref(false)
const addTableDialogVisible = ref(false)
const editTableDialogVisible = ref(false)
const columnsDrawerVisible = ref(false)
const aiCreateTableVisible = ref(false)
const tableConfigVisible = ref(false)
const currentEditTable = ref(null)
const currentTable = ref(null)
const currentConfigTable = ref(null)
const importProgressRef = ref(null)

// 加载项目信息
const loadProject = async () => {
  try {
    const projectId = parseInt(route.params.id)
    const data = await invoke('db_get_project', { id: projectId })
    const projectData = JSON.parse(data)
    const datasourceData = await invoke('db_get_datasource', { id: projectData.datasource_id })
    projectData.datasource = JSON.parse(datasourceData)
    project.value = projectData
  } catch (error) { message.error('加载项目失败: ' + error) }
}

// 加载表列表
const loadTables = async () => {
  try {
    loading.value = true
    const projectId = parseInt(route.params.id)
    tables.value = await projectsApi.getProjectTables(projectId)
  } catch (error) { message.error('加载表列表失败: ' + error) } finally { loading.value = false }
}

// 搜索、筛选、排序处理
const handleSearch = () => {}
const handleFilter = (value) => { filterValue.value = value }
const handleSort = (value) => { sortValue.value = value }

// 导出 SQL DDL
const exportTables = async () => {
  try {
    let sql = ''
    for (const table of filteredTables.value) {
      const cols = await projectsApi.getTableColumns(table.id)
      sql += '-- ' + (table.comment || table.name) + '\n'
      sql += 'CREATE TABLE `' + table.name + '` (\n'
      const columnDefs = cols.map(col => {
        let def = '  `' + col.name + '` ' + col.data_type
        if (col.length) def += '(' + col.length + ')'
        if (!col.is_nullable) def += ' NOT NULL'
        if (col.default_value) def += ' DEFAULT ' + col.default_value
        if (col.comment) def += " COMMENT '" + col.comment + "'"
        return def
      })
      const primaryKeys = cols.filter(c => c.is_primary_key).map(c => '`' + c.name + '`')
      if (primaryKeys.length > 0) columnDefs.push('  PRIMARY KEY (' + primaryKeys.join(', ') + ')')
      sql += columnDefs.join(',\n') + '\n)'
      if (table.engine) sql += ' ENGINE=' + table.engine
      if (table.comment) sql += " COMMENT='" + table.comment + "'"
      sql += ';\n\n'
    }
    const filePath = await save({ defaultPath: (project.value?.name || 'tables') + '.sql', filters: [{ name: 'SQL', extensions: ['sql'] }] })
    if (filePath) { await invoke('write_file', { path: filePath, content: sql }); message.success('SQL 文件导出成功') }
  } catch (error) { message.error('导出失败: ' + error) }
}

// 查看列详情
const viewColumns = async (table) => {
  currentTable.value = table
  columnsDrawerVisible.value = true
}

// 列更新后刷新
const onColumnsUpdated = async () => { await loadTables() }

// 删除表
const deleteTable = async (table) => {
  try {
    await projectsApi.deleteTable(table.id)
    message.success(`表 "${table.name}" 删除成功`)
    await loadTables()
  } catch (error) { message.error('删除表失败: ' + error) }
}

// 批量删除表
const batchDeleteTables = async () => {
  if (selectedRowKeys.value.length === 0) { message.warning('请先选择要删除的表'); return }
  Modal.confirm({
    title: '确认删除', content: `确定要删除选中的 ${selectedRowKeys.value.length} 张表吗？此操作不可恢复！`, okText: '确定', cancelText: '取消', okType: 'danger',
    onOk: async () => {
      try {
        await Promise.all(selectedRowKeys.value.map(id => projectsApi.deleteTable(id)))
        message.success(`成功删除 ${selectedRowKeys.value.length} 张表`)
        selectedRowKeys.value = []
        await loadTables()
      } catch (error) { message.error('批量删除失败: ' + error) }
    }
  })
}

// 新增表
const showAddTableDialog = () => { addTableDialogVisible.value = true }

// 编辑表
const editTable = (table) => {
  currentEditTable.value = table
  editTableDialogVisible.value = true
}

// 表配置
const openTableConfig = (table) => {
  currentConfigTable.value = table
  tableConfigVisible.value = true
}
const onTableConfigSaved = () => {}

// 导入表结构
const importTables = () => {
  if (!project.value?.datasource) { message.error('项目未关联数据源'); return }
  importDialogVisible.value = true
}

// 头部下拉菜单处理
const handleHeaderMenu = ({ key }) => {
  switch (key) {
    case 'sql-import': sqlImportDialogVisible.value = true; break
    case 'import-structure': importTables(); break
    case 'export': exportTables(); break
    case 'refresh': loadTables(); break
  }
}

// 数据库标签
const getDatabaseColor = (type) => ({ mysql: 'blue', postgresql: 'cyan', sqlite: 'green' }[type] || 'default')
const getDatabaseLabel = (type) => ({ mysql: 'MySQL', postgresql: 'PostgreSQL', sqlite: 'SQLite' }[type] || type)

// 格式化日期
const formatDate = (dateStr) => dateStr ? new Date(dateStr).toLocaleString('zh-CN') : '-'

// 生命周期
onMounted(async () => {
  await loadProject()
  await loadTables()
  layoutStore.showFooterPagination(filteredTables.value.length, 1, 20, ['10', '20', '50', '100'])
})

watch(filteredTables, (newVal) => {
  layoutStore.showFooterPagination(newVal.length, 1, layoutStore.footerPagination.pageSize, ['10', '20', '50', '100'])
})

onUnmounted(() => { layoutStore.hideFooter() })
</script>

<style scoped>
.tables-view { padding: var(--spacing-lg); }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--spacing-lg); }
.header-left { display: flex; align-items: center; gap: var(--spacing-sm); }
.page-title { margin: 0; font-size: 24px; font-weight: 600; color: var(--color-text); }
.database-name { color: var(--color-text-secondary); font-size: 14px; font-family: 'Courier New', 'Consolas', monospace; }
.header-right { display: flex; gap: var(--spacing-sm); }
.table-card { margin-top: var(--spacing-md); }
.toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--spacing-md); padding: var(--spacing-sm) 0; }
.toolbar-left { display: flex; align-items: center; gap: var(--spacing-sm); }
.toolbar-right { display: flex; align-items: center; gap: var(--spacing-md); }
.result-count { color: var(--color-text-secondary); font-size: 14px; }
:deep(.ant-table) { font-size: 14px; }
:deep(.ant-table-thead > tr > th) { font-weight: 600; background: var(--color-surface); }
:deep(.ant-table-tbody > tr:hover > td) { background: var(--color-surface); }
:deep(.ant-table-thead > tr > th.ant-table-selection-column .ant-checkbox-wrapper) { display: none; }
.comment-text { display: inline-block; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--color-text-secondary); cursor: default; }
.comment-empty { color: var(--color-text-muted); }
</style>
