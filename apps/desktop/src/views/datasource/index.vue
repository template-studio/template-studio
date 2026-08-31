<template>
  <div class="datasource-view">
    <div class="toolbar">
      <div class="toolbar-left">
        <h2 class="page-title">数据源管理</h2>
        <span class="result-count">共 {{ filteredDatasources.length }} 个数据源</span>
      </div>
      <div class="toolbar-right">
        <SearchBar
          v-model="searchQuery"
          placeholder="搜索数据源名称..."
          :filters="databaseFilters"
          :sort-options="sortOptions"
          @search="handleSearch"
          @filter="handleFilter"
          @sort="handleSort"
        />
        <a-button type="primary" size="large" @click="openCreateDialog">
          <template #icon><PlusOutlined /></template>
          新建数据源
        </a-button>
      </div>
    </div>

    <div class="datasources-content">
      <a-spin :spinning="loading">
        <div v-if="paginatedDatasources.length > 0" class="datasources-grid">
          <div
            v-for="datasource in paginatedDatasources"
            :key="datasource.id"
            class="datasource-card"
            @click="openEditDialog(datasource)"
          >
            <div class="card-visual" :class="`datasource-${datasource.type_}`">
              <div class="visual-bg">
                <div class="code-preview" v-html="getPythonCode(datasource)"></div>
              </div>
            </div>
            <div class="card-content">
              <h3 class="datasource-name">{{ datasource.name }}</h3>
              <div class="datasource-details">
                <div class="detail-row">
                  <CheckCircleOutlined v-if="datasource.is_active" class="status-icon status-active" />
                  <CloseCircleOutlined v-else class="status-icon status-inactive" />
                  <span class="detail-text">{{ datasource.is_active ? '连接正常' : '未连接' }}</span>
                </div>
                <div v-if="datasource.type_ !== 'sqlite'" class="detail-row">
                  <UserOutlined class="detail-icon" />
                  <span class="detail-text">{{ datasource.username || '-' }}</span>
                </div>
                <div v-else class="detail-row">
                  <FileOutlined class="detail-icon" />
                  <span class="detail-text file-path">{{ datasource.sqlite_file || '-' }}</span>
                </div>
                <div class="detail-row">
                  <ClockCircleOutlined class="detail-icon" />
                  <span class="detail-text">{{ formatDate(datasource.created_at) }}</span>
                </div>
              </div>
            </div>
            <div class="card-actions">
              <a-button type="text" size="small" @click.stop="openBrowser(datasource)" class="action-btn" title="浏览数据库">
                <EyeOutlined />
              </a-button>
              <a-button type="text" size="small" @click.stop="openStatus(datasource)" class="action-btn" title="连接状态">
                <DashboardOutlined />
              </a-button>
              <a-button type="text" size="small" @click.stop="openEditDialog(datasource)" class="action-btn" title="编辑">
                <EditOutlined />
              </a-button>
              <a-button type="text" size="small" @click.stop="testConnection(datasource)" class="action-btn" title="测试连接">
                <ApiOutlined />
              </a-button>
              <a-button type="text" size="small" danger @click.stop="confirmDelete(datasource)" class="action-btn" title="删除">
                <DeleteOutlined />
              </a-button>
            </div>
          </div>
        </div>
        <a-empty
          v-else-if="!loading && filteredDatasources.length === 0"
          :description="searchQuery ? '没有找到匹配的数据源' : '暂无数据源'"
          :image="Empty.PRESENTED_IMAGE_SIMPLE"
          class="empty-state"
        >
          <a-button type="primary" size="large" @click="openCreateDialog">
            <template #icon><PlusOutlined /></template>
            创建第一个数据源
          </a-button>
        </a-empty>
      </a-spin>
    </div>

    <DatasourceDialog
      v-model:open="dialogVisible"
      :mode="dialogMode"
      :datasource="editingDatasource"
      @saved="onDialogSaved"
    />
    <ConnectionStatusModal
      v-model:open="statusVisible"
      :datasource="statusDatasource"
    />
  </div>
</template>

<script setup>
import { ref, onMounted, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  PlusOutlined, DatabaseOutlined, ApiOutlined, FileOutlined,
  UserOutlined, ClockCircleOutlined, CheckCircleOutlined,
  CloseCircleOutlined, DeleteOutlined, EditOutlined,
  EyeOutlined, DashboardOutlined
} from '@ant-design/icons-vue'
import { Empty, message, Modal } from 'ant-design-vue'
import { notify } from '@/utils/notify'
import * as datasourcesApi from '@/api/datasources'
import { SearchBar } from '@/components/common'
import { useLayoutStore } from '@/stores/layout'
import DatasourceDialog from './components/DatasourceDialog.vue'
import ConnectionStatusModal from './components/ConnectionStatusModal.vue'

const router = useRouter()
const layoutStore = useLayoutStore()

const loading = ref(false)
const datasources = ref([])
const searchQuery = ref('')
const filterValue = ref(undefined)
const sortValue = ref('created_at:desc')

const databaseFilters = [
  { label: 'MySQL', value: 'mysql' },
  { label: 'PostgreSQL', value: 'postgresql' },
  { label: 'SQLite', value: 'sqlite' }
]

const sortOptions = [
  { label: '最新创建', value: 'created_at:desc' },
  { label: '最早创建', value: 'created_at:asc' },
  { label: '名称 A-Z', value: 'name:asc' },
  { label: '名称 Z-A', value: 'name:desc' }
]

const dialogVisible = ref(false)
const dialogMode = ref('create')
const editingDatasource = ref(null)
const statusVisible = ref(false)
const statusDatasource = ref(null)

const filteredDatasources = computed(() => {
  let result = [...datasources.value]
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(ds =>
      ds.name.toLowerCase().includes(query) ||
      (ds.host && ds.host.toLowerCase().includes(query)) ||
      (ds.username && ds.username.toLowerCase().includes(query))
    )
  }
  if (filterValue.value) {
    result = result.filter(ds => ds.type_ === filterValue.value)
  }
  if (sortValue.value) {
    const [field, order] = sortValue.value.split(':')
    result.sort((a, b) => {
      let valueA, valueB
      switch (field) {
        case 'name': valueA = a.name.toLowerCase(); valueB = b.name.toLowerCase(); break
        case 'created_at': valueA = new Date(a.created_at).getTime(); valueB = new Date(b.created_at).getTime(); break
        default: return 0
      }
      return order === 'asc' ? (valueA > valueB ? 1 : -1) : (valueA < valueB ? 1 : -1)
    })
  }
  return result
})

const paginatedDatasources = computed(() => {
  const { current, pageSize: size } = layoutStore.footerPagination
  const start = (current - 1) * size
  return filteredDatasources.value.slice(start, start + size)
})

const loadDatasources = async () => {
  try {
    loading.value = true
    datasources.value = await datasourcesApi.getAllDatasources()
  } catch (error) {
    message.error('加载数据源失败: ' + error)
  } finally {
    loading.value = false
  }
}

const getDatabaseColor = (type) => ({ mysql: 'blue', postgresql: 'cyan', sqlite: 'green' }[type] || 'default')
const getDatabaseLabel = (type) => ({ mysql: 'MySQL', postgresql: 'PostgreSQL', sqlite: 'SQLite' }[type] || type)

const formatDate = (dateStr) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  const diff = Date.now() - date
  const days = Math.floor(diff / 86400000)
  if (days === 0) {
    const hours = Math.floor(diff / 3600000)
    if (hours === 0) { const m = Math.floor(diff / 60000); return m <= 0 ? '刚刚' : `${m} 分钟前` }
    return `${hours} 小时前`
  }
  if (days === 1) return '昨天'
  if (days < 7) return `${days} 天前`
  return date.toLocaleDateString('zh-CN')
}

const getFileName = (path) => {
  if (!path) return '-'
  const parts = path.split(/[/\\]/)
  return parts[parts.length - 1] || path
}

const getPythonCode = (datasource) => {
  if (datasource.type_ === 'mysql') {
    return `<span class="code-kw">import</span> <span class="code-mod">pymysql</span>\n<span class="code-var">conn</span> = <span class="code-mod">pymysql</span>.<span class="code-fn">connect</span>(\n    <span class="code-par">host</span>=<span class="code-str">'${datasource.host || 'localhost'}'</span>,\n    <span class="code-par">port</span>=<span class="code-num">${datasource.port || 3306}</span>,\n    <span class="code-par">user</span>=<span class="code-str">'${datasource.username || 'root'}'</span>\n)`
  } else if (datasource.type_ === 'postgresql') {
    return `<span class="code-kw">import</span> <span class="code-mod">psycopg2</span>\n<span class="code-var">conn</span> = <span class="code-mod">psycopg2</span>.<span class="code-fn">connect</span>(\n    <span class="code-par">host</span>=<span class="code-str">'${datasource.host || 'localhost'}'</span>,\n    <span class="code-par">port</span>=<span class="code-num">${datasource.port || 5432}</span>,\n    <span class="code-par">user</span>=<span class="code-str">'${datasource.username || 'postgres'}'</span>\n)`
  } else if (datasource.type_ === 'sqlite') {
    return `<span class="code-kw">import</span> <span class="code-mod">sqlite3</span>\n<span class="code-var">conn</span> = <span class="code-mod">sqlite3</span>.<span class="code-fn">connect</span>(\n    <span class="code-str">'${getFileName(datasource.sqlite_file)}'</span>\n)`
  }
  return ''
}

const openCreateDialog = () => {
  dialogMode.value = 'create'
  editingDatasource.value = null
  dialogVisible.value = true
}

const openEditDialog = (datasource) => {
  dialogMode.value = 'edit'
  editingDatasource.value = datasource
  dialogVisible.value = true
}

const openBrowser = (datasource) => router.push(`/datasource/${datasource.id}/browse`)

const openStatus = (datasource) => {
  statusDatasource.value = datasource
  statusVisible.value = true
}

const testConnection = async (datasource) => {
  try {
    const params = { type: datasource.type_, host: datasource.host, port: datasource.port, username: datasource.username, password: datasource.password }
    if (datasource.type_ === 'postgresql') params.database = datasource.database || 'postgres'
    if (datasource.type_ === 'sqlite') params.sqliteFile = datasource.sqlite_file
    const result = await datasourcesApi.testConnection(params)
    notify({ type: 'success', title: '连接测试成功', content: `${datasource.name}: ${result}` })
  } catch (error) {
    notify({ type: 'error', title: '连接测试失败', content: `${datasource.name}: ${error}` })
  }
}

const onDialogSaved = async () => { await loadDatasources() }

const confirmDelete = (datasource) => {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除数据源 "${datasource.name}" 吗？此操作不可恢复。`,
    okText: '删除', okType: 'danger', cancelText: '取消',
    onOk: async () => {
      try {
        await datasourcesApi.deleteDatasource(datasource.id)
        notify({ type: 'success', title: '数据源删除成功', content: `数据源 "${datasource.name}" 已删除` })
        await loadDatasources()
      } catch (error) {
        notify({ type: 'error', title: '删除失败', content: String(error) })
      }
    }
  })
}

const handleSearch = () => layoutStore.updateFooterPagination({ current: 1 })
const handleFilter = (value) => { filterValue.value = value; layoutStore.updateFooterPagination({ current: 1 }) }
const handleSort = (value) => { sortValue.value = value; layoutStore.updateFooterPagination({ current: 1 }) }

watch(filteredDatasources, (newVal) => {
  layoutStore.showFooterPagination(newVal.length, layoutStore.footerPagination.current, layoutStore.footerPagination.pageSize)
})

onMounted(async () => {
  await loadDatasources()
  layoutStore.showFooterPagination(filteredDatasources.value.length, layoutStore.footerPagination.current, layoutStore.footerPagination.pageSize)
})
</script>

<style scoped>
.datasource-view { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--spacing-md); padding: var(--spacing-sm) var(--spacing-lg); flex-shrink: 0; }
.toolbar-left { display: flex; align-items: baseline; gap: var(--spacing-md); }
.page-title { margin: 0; font-size: 24px; font-weight: 600; color: var(--color-text); }
.result-count { color: var(--color-text-secondary); font-size: 14px; }
.toolbar-right { display: flex; align-items: center; gap: var(--spacing-md); }
.datasources-content { flex: 1; display: flex; flex-direction: column; overflow-y: auto; min-height: 0; padding: 0 var(--spacing-lg); }
.datasources-content > :deep(.ant-spin-container) { display: flex; flex-direction: column; flex: 1; min-height: 0; }
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; flex: 1; min-height: 400px; }
.datasources-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: var(--spacing-md); }
.datasource-card { background: var(--color-background); border: 1px solid var(--color-border); border-radius: var(--border-radius-lg); overflow: hidden; cursor: pointer; transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); }
.datasource-card:hover { transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(15, 23, 42, 0.12);
  border-color: var(--color-border-strong); }
.card-visual { height: 120px; position: relative; overflow: hidden; }
.visual-bg { width: 100%; height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; position: relative; }
.visual-bg::before { content: ''; position: absolute; top: -50%; left: -50%; width: 200%; height: 200%; background: linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.1) 50%, transparent 70%); animation: shimmer 3s infinite; }
@keyframes shimmer { 0% { transform: translateX(-100%); } 100% { transform: translateX(100%); } }
@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.8; } }
.datasource-mysql .visual-bg { background: linear-gradient(135deg, #00758f 0%, #f29111 100%); }
.datasource-mysql .visual-bg::after { content: ''; position: absolute; inset: 0; background: linear-gradient(180deg, transparent 0%, rgba(0,0,0,0.2) 100%); }
.datasource-postgresql .visual-bg { background: linear-gradient(135deg, #336791 0%, #0064a5 100%); }
.datasource-postgresql .visual-bg::after { content: ''; position: absolute; inset: 0; background: linear-gradient(180deg, transparent 0%, rgba(0,0,0,0.2) 100%); }
.datasource-sqlite .visual-bg { background: linear-gradient(135deg, #0f8044 0%, #003b2e 100%); }
.datasource-sqlite .visual-bg::after { content: ''; position: absolute; inset: 0; background: linear-gradient(180deg, transparent 0%, rgba(0,0,0,0.2) 100%); }
.code-preview { font-family: 'Courier New', 'Consolas', monospace; font-size: 9px; line-height: 1.5; color: rgba(255,255,255,0.7); white-space: pre; text-align: left; padding: var(--spacing-md); position: relative; z-index: 1; width: 100%; box-sizing: border-box; }
.code-kw { color: rgba(255,138,101,0.95); font-weight: 600; }
.code-mod { color: rgba(102,153,204,0.95); }
.code-var { color: rgba(152,195,121,0.95); }
.code-fn { color: rgba(86,156,214,0.95); }
.code-par { color: rgba(207,138,221,0.9); }
.code-str { color: rgba(173,186,199,0.95); }
.code-num { color: rgba(189,147,249,0.95); }
.card-content { padding: var(--spacing-sm) var(--spacing-md); }
.datasource-name { margin: 0 0 8px 0; font-size: 15px; font-weight: 600; color: var(--color-text); transition: color 0.2s ease; }
.datasource-card:hover .datasource-name { color: var(--color-primary); }
.datasource-details { display: flex; flex-direction: column; gap: 6px; }
.detail-row { display: flex; align-items: center; gap: 8px; }
.detail-icon { font-size: 14px; color: var(--color-text-secondary); flex-shrink: 0; }
.detail-text { font-size: 13px; color: var(--color-text-secondary); flex: 1; }
.detail-text.file-path { font-family: 'Courier New', 'Consolas', monospace; font-size: 11px; word-break: break-all; }
.status-icon { font-size: 16px; }
.status-active { color: var(--color-success); animation: pulse 2s ease-in-out infinite; }
.status-inactive { color: var(--color-text-muted); }
.card-actions { display: flex; justify-content: flex-end; gap: 4px; padding: 8px var(--spacing-md); border-top: 1px solid var(--color-border); background: var(--color-surface); }
.card-actions .ant-btn { font-size: 16px; padding: 4px 8px; height: auto; min-width: auto; transition: all 0.2s ease; display: inline-flex; align-items: center; justify-content: center; }
.card-actions .ant-btn:hover { transform: scale(1.1); background: var(--color-hover); }
</style>
