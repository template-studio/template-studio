<template>
  <div class="projects-view">
    <div class="toolbar">
      <div class="toolbar-left">
        <h2 class="page-title">项目列表</h2>
        <span class="result-count">共 {{ filteredProjects.length }} 个项目</span>
      </div>
      <div class="toolbar-right">
        <SearchBar
          v-model="searchQuery"
          placeholder="搜索项目名称或描述..."
          :filters="databaseFilters"
          :sort-options="sortOptions"
          @search="handleSearch"
          @filter="handleFilter"
          @sort="handleSort"
        />
        <a-button type="primary" size="large" @click="openCreateDialog">
          <template #icon><PlusOutlined /></template>
          新建项目
        </a-button>
      </div>
    </div>

    <div class="projects-content">
      <a-spin :spinning="loading">
        <div v-if="paginatedProjects.length > 0" class="projects-grid">
          <div
            v-for="project in paginatedProjects"
            :key="project.id"
            class="project-card"
            @click="openProject(project.id)"
          >
            <div class="card-visual" :class="`project-${getDatabaseType(project)}`">
              <div class="visual-bg">
                <div class="geometric-pattern">
                  <div class="circle circle-1"></div>
                  <div class="circle circle-2"></div>
                  <div class="circle circle-3"></div>
                  <div class="wave wave-1"></div>
                  <div class="wave wave-2"></div>
                </div>
                <div class="glow-effect"></div>
              </div>
            </div>
            <div class="card-content">
              <h3 class="project-name">{{ project.name }}</h3>
              <div class="project-details">
                <div class="detail-row">
                  <DatabaseOutlined class="detail-icon" />
                  <a-tag :color="getDatabaseColor(getDatabaseType(project))">{{ getDatabaseLabel(getDatabaseType(project)) }}</a-tag>
                  <span class="detail-text datasource-name">{{ project.datasource?.name || '未关联数据源' }}</span>
                </div>
                <div v-if="getProjectLanguages(project).length > 0" class="detail-row">
                  <CodeOutlined class="detail-icon" />
                  <a-space :size="4">
                    <a-tag v-for="lang in getProjectLanguages(project)" :key="lang.id" :color="getLanguageColor(lang.color)">
                      {{ lang.name }}<span v-if="lang.is_primary" style="margin-left: 4px; font-weight: 600;">★</span>
                    </a-tag>
                  </a-space>
                </div>
                <div class="detail-row">
                  <ApiOutlined class="detail-icon" />
                  <span class="detail-text database-name">{{ project.database_name }}</span>
                </div>
                <div class="detail-row">
                  <TableOutlined class="detail-icon" />
                  <span class="detail-text">{{ project.table_count || 0 }} 张表</span>
                </div>
                <div class="detail-row">
                  <ClockCircleOutlined class="detail-icon" />
                  <span class="detail-text">{{ formatDate(project.created_at) }}</span>
                </div>
              </div>
            </div>
            <div class="card-actions">
              <a-button type="text" size="small" @click.stop="openProject(project.id)" class="action-btn" title="打开">
                <FolderOpenOutlined />
              </a-button>
              <a-button type="text" size="small" @click.stop="openEditDialog(project)" class="action-btn" title="编辑">
                <EditOutlined />
              </a-button>
              <a-button type="text" size="small" danger @click.stop="confirmDelete(project)" class="action-btn" title="删除">
                <DeleteOutlined />
              </a-button>
            </div>
          </div>
        </div>
        <a-empty
          v-else-if="!loading && filteredProjects.length === 0"
          :description="searchQuery ? '没有找到匹配的项目' : '暂无项目'"
          :image="Empty.PRESENTED_IMAGE_SIMPLE"
          class="empty-state"
        >
          <a-button type="primary" size="large" @click="openCreateDialog">
            <template #icon><PlusOutlined /></template>
            创建第一个项目
          </a-button>
        </a-empty>
      </a-spin>
    </div>

    <ProjectDialog
      v-model:open="dialogVisible"
      :mode="dialogMode"
      :project="editingProject"
      @saved="onDialogSaved"
    />
  </div>
</template>

<script setup>
import { ref, onMounted, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  PlusOutlined, DatabaseOutlined, ApiOutlined, FileOutlined,
  TableOutlined, ClockCircleOutlined, FolderOpenOutlined,
  DeleteOutlined, EditOutlined, CodeOutlined
} from '@ant-design/icons-vue'
import { Empty, message, Modal } from 'ant-design-vue'
import { notify } from '@/utils/notify'
import { invoke } from '@tauri-apps/api/core'
import * as projectsApi from '@/api/projects'
import * as languagesApi from '@/api/languages'
import { SearchBar } from '@/components/common'
import { useLayoutStore } from '@/stores/layout'
import ProjectDialog from './components/ProjectDialog.vue'

const router = useRouter()
const layoutStore = useLayoutStore()

const loading = ref(false)
const projects = ref([])
const languages = ref([])
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
  { label: '名称 Z-A', value: 'name:desc' },
  { label: '表数量最多', value: 'table_count:desc' },
  { label: '表数量最少', value: 'table_count:asc' }
]

const dialogVisible = ref(false)
const dialogMode = ref('create')
const editingProject = ref(null)

const filteredProjects = computed(() => {
  let result = [...projects.value]
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(p =>
      p.name.toLowerCase().includes(query) ||
      (p.description && p.description.toLowerCase().includes(query)) ||
      (p.database_name && p.database_name.toLowerCase().includes(query))
    )
  }
  if (filterValue.value) {
    result = result.filter(p => (p.datasource?.type_ || 'default') === filterValue.value)
  }
  if (sortValue.value) {
    const [field, order] = sortValue.value.split(':')
    result.sort((a, b) => {
      let valueA, valueB
      switch (field) {
        case 'name': valueA = a.name.toLowerCase(); valueB = b.name.toLowerCase(); break
        case 'created_at': valueA = new Date(a.created_at).getTime(); valueB = new Date(b.created_at).getTime(); break
        case 'table_count': valueA = a.table_count || 0; valueB = b.table_count || 0; break
        default: return 0
      }
      return order === 'asc' ? (valueA > valueB ? 1 : -1) : (valueA < valueB ? 1 : -1)
    })
  }
  return result
})

const paginatedProjects = computed(() => {
  const { current, pageSize: size } = layoutStore.footerPagination
  return filteredProjects.value.slice((current - 1) * size, current * size)
})

const loadProjects = async () => {
  try {
    loading.value = true
    const data = await projectsApi.getAllProjects()
    projects.value = await Promise.all(data.map(async (project) => {
      await Promise.all([
        (async () => {
          try {
            const datasourceData = await invoke('db_get_datasource', { id: project.datasource_id })
            project.datasource = JSON.parse(datasourceData)
          } catch { project.datasource = null }
        })(),
        (async () => {
          try { project.languages = await languagesApi.getProjectLanguages(project.id) }
          catch { project.languages = [] }
        })()
      ])
      return project
    }))
  } catch (error) {
    message.error('加载项目失败: ' + error)
  } finally {
    loading.value = false
  }
}

const loadLanguages = async () => {
  try { languages.value = (await languagesApi.getAllLanguages()).filter(l => l.is_active) }
  catch (error) { message.error('加载语言失败: ' + error) }
}

const getDatabaseColor = (type) => ({ mysql: 'blue', postgresql: 'cyan', sqlite: 'green', default: 'default' }[type] || 'default')
const getDatabaseLabel = (type) => ({ mysql: 'MySQL', postgresql: 'PostgreSQL', sqlite: 'SQLite', default: 'Database' }[type] || 'Database')
const getDatabaseType = (project) => project.datasource?.type_ || 'default'

const formatDate = (dateStr) => {
  if (!dateStr) return '-'
  const diff = Date.now() - new Date(dateStr)
  const days = Math.floor(diff / 86400000)
  if (days === 0) {
    const hours = Math.floor(diff / 3600000)
    if (hours === 0) { const m = Math.floor(diff / 60000); return m <= 0 ? '刚刚' : `${m} 分钟前` }
    return `${hours} 小时前`
  }
  if (days === 1) return '昨天'
  if (days < 7) return `${days} 天前`
  return new Date(dateStr).toLocaleDateString('zh-CN')
}

const getProjectLanguages = (project) => {
  const result = []
  if (project.primary_language_id) {
    const primaryLang = project.primary_language || languages.value.find(l => l.id === project.primary_language_id)
    if (primaryLang) result.push({ ...primaryLang, is_primary: true })
  }
  if (project.languages?.length > 0) {
    for (const lang of project.languages) {
      if (lang.id !== project.primary_language_id) result.push({ ...lang, is_primary: false })
    }
  }
  return result
}

const getLanguageColor = (color) => {
  const map = { red: 'red', orange: 'orange', gold: 'gold', green: 'green', cyan: 'cyan', blue: 'blue', purple: 'purple', pink: 'pink' }
  return map[color] || 'default'
}

const openCreateDialog = () => {
  dialogMode.value = 'create'
  editingProject.value = null
  dialogVisible.value = true
}

const openEditDialog = (project) => {
  dialogMode.value = 'edit'
  editingProject.value = project
  dialogVisible.value = true
}

const openProject = (id) => router.push(`/project/${id}`)

const onDialogSaved = async () => { await loadProjects() }

const confirmDelete = (project) => {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除项目 "${project.name}" 吗？此操作将同时删除该项目关联的所有表和字段数据，且不可恢复。`,
    okText: '删除', okType: 'danger', cancelText: '取消',
    onOk: async () => {
      try {
        await projectsApi.deleteProject(project.id)
        notify({ type: 'success', title: '项目删除成功', content: `项目 "${project.name}" 已删除` })
        await loadProjects()
      } catch (error) { notify({ type: 'error', title: '删除失败', content: String(error) }) }
    }
  })
}

const handleSearch = () => layoutStore.updateFooterPagination({ current: 1 })
const handleFilter = (value) => { filterValue.value = value; layoutStore.updateFooterPagination({ current: 1 }) }
const handleSort = (value) => { sortValue.value = value; layoutStore.updateFooterPagination({ current: 1 }) }

watch(filteredProjects, (newVal) => {
  layoutStore.showFooterPagination(newVal.length, layoutStore.footerPagination.current, layoutStore.footerPagination.pageSize)
})

onMounted(async () => {
  await Promise.all([loadProjects(), loadLanguages()])
  layoutStore.showFooterPagination(filteredProjects.value.length, layoutStore.footerPagination.current, layoutStore.footerPagination.pageSize)
})
</script>

<style scoped>
.projects-view { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--spacing-md); padding: var(--spacing-sm) var(--spacing-lg); flex-shrink: 0; }
.toolbar-left { display: flex; align-items: baseline; gap: var(--spacing-md); }
.page-title { margin: 0; font-size: 24px; font-weight: 600; color: var(--color-text); }
.result-count { color: var(--color-text-secondary); font-size: 14px; }
.toolbar-right { display: flex; align-items: center; gap: var(--spacing-md); }
.projects-content { flex: 1; display: flex; flex-direction: column; overflow-y: auto; min-height: 0; padding: 0 var(--spacing-lg); }
.projects-content > :deep(.ant-spin-container) { display: flex; flex-direction: column; flex: 1; min-height: 0; }
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; flex: 1; min-height: 400px; }
.projects-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: var(--spacing-md); }
.project-card { background: var(--color-background); border: 1px solid var(--color-border); border-radius: var(--border-radius-lg); overflow: hidden; cursor: pointer; transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); }
.project-card:hover { transform: translateY(-6px); box-shadow: 0 12px 24px rgba(0, 0, 0, 0.15); }
.card-visual { height: 120px; position: relative; overflow: hidden; }
.visual-bg { width: 100%; height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; position: relative; }
.visual-bg::before { content: ''; position: absolute; top: -50%; left: -50%; width: 200%; height: 200%; background: linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.1) 50%, transparent 70%); animation: shimmer 3s infinite; }
@keyframes shimmer { 0% { transform: translateX(-100%); } 100% { transform: translateX(100%); } }
.project-mysql .visual-bg { background: linear-gradient(135deg, #00758f 0%, #00a0e4 50%, #f29111 100%); overflow: hidden; }
.project-postgresql .visual-bg { background: linear-gradient(135deg, #336791 0%, #0064a5 50%, #008bfc 100%); overflow: hidden; }
.project-sqlite .visual-bg { background: linear-gradient(135deg, #0f8044 0%, #00a86b 50%, #98fb98 100%); overflow: hidden; }
.project-default .visual-bg { background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #f093fb 100%); overflow: hidden; }
.geometric-pattern { position: absolute; top: 0; left: 0; width: 100%; height: 100%; opacity: 0.15; }
.circle { position: absolute; border: 2px solid rgba(255,255,255,0.3); border-radius: 50%; animation: rotate 20s linear infinite; }
.circle-1 { width: 200px; height: 200px; top: -50px; right: -50px; animation-duration: 25s; }
.circle-2 { width: 150px; height: 150px; bottom: -30px; left: -30px; animation-duration: 20s; animation-direction: reverse; }
.circle-3 { width: 100px; height: 100px; top: 50%; left: 50%; transform: translate(-50%, -50%); animation-duration: 15s; }
@keyframes rotate { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.wave { position: absolute; width: 200%; height: 200%; background: radial-gradient(circle, rgba(255,255,255,0.1) 0%, transparent 70%); animation: wave 8s ease-in-out infinite; }
.wave-1 { top: -50%; left: -50%; }
.wave-2 { bottom: -50%; right: -50%; animation-delay: -4s; }
@keyframes wave { 0%, 100% { transform: translate(0,0) scale(1); } 50% { transform: translate(20px,20px) scale(1.1); } }
.glow-effect { position: absolute; top: 50%; left: 50%; transform: translate(-50%,-50%); width: 300px; height: 300px; background: radial-gradient(circle, rgba(255,255,255,0.2) 0%, transparent 70%); animation: glow 4s ease-in-out infinite; pointer-events: none; }
@keyframes glow { 0%, 100% { opacity: 0.3; transform: translate(-50%,-50%) scale(1); } 50% { opacity: 0.6; transform: translate(-50%,-50%) scale(1.2); } }
.card-content { padding: var(--spacing-sm) var(--spacing-md); }
.project-name { margin: 0 0 8px 0; font-size: 15px; font-weight: 600; color: var(--color-text); transition: color 0.2s ease; }
.project-card:hover .project-name { color: var(--color-primary); }
.project-details { display: flex; flex-direction: column; gap: 6px; }
.detail-row { display: flex; align-items: center; gap: 8px; }
.detail-icon { font-size: 14px; color: var(--color-text-secondary); flex-shrink: 0; }
.detail-text { font-size: 13px; color: var(--color-text-secondary); flex: 1; }
.detail-text.datasource-name { font-weight: 500; }
.detail-text.database-name { font-family: 'Courier New', 'Consolas', monospace; font-size: 11px; }
.card-actions { display: flex; justify-content: flex-end; gap: 4px; padding: 8px var(--spacing-md); border-top: 1px solid var(--color-border); background: var(--color-surface); }
.card-actions .ant-btn { font-size: 16px; padding: 4px 8px; height: auto; min-width: auto; transition: all 0.2s ease; display: inline-flex; align-items: center; justify-content: center; }
.card-actions .ant-btn:hover { transform: scale(1.1); background: var(--color-hover); }
</style>
