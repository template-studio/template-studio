<template>
  <div class="project-workspace">
    <ProjectInfoCard :project="project" :datasource="project?.datasource" />

    <StatsGrid :stats="stats" @navigate="navigateTo" />

    <div class="workspace-body">
      <QuickActionsGrid :project-id="projectId" @navigate="handleActionNavigate" />

      <RecentTablesList :tables="recentTables" @navigate-table="navigateTo('tables')" />
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { message } from 'ant-design-vue'
import ProjectInfoCard from './components/ProjectInfoCard.vue'
import StatsGrid from './components/StatsGrid.vue'
import QuickActionsGrid from './components/QuickActionsGrid.vue'
import RecentTablesList from './components/RecentTablesList.vue'
import * as projectsApi from '@/api/projects'

const route = useRoute()
const router = useRouter()
const projectId = route.params.id

const project = ref(null)
const recentTables = ref([])
const stats = reactive({
  tableCount: 0,
  columnCount: 0,
  mappingCount: 0,
  lastUpdate: '-'
})

const formatTime = (dateStr) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now - date
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))

  if (days === 0) return '今天'
  if (days === 1) return '昨天'
  if (days < 7) return `${days}天前`
  return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}

const loadProject = async () => {
  try {
    const data = await projectsApi.getProject(projectId)
    project.value = data
    stats.lastUpdate = formatTime(data?.updated_at)

    try {
      const tables = await projectsApi.getProjectTables(projectId)
      recentTables.value = (tables || []).slice(0, 8)
      stats.tableCount = tables?.length || 0
      stats.columnCount = tables?.reduce((sum, t) => sum + (t.column_count || 0), 0) || 0
    } catch {
      recentTables.value = []
    }

    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const mappings = await invoke('db_get_system_type_mappings')
      const data = typeof mappings === 'string' ? JSON.parse(mappings) : mappings
      stats.mappingCount = data?.length || 0
    } catch {
      stats.mappingCount = 0
    }
  } catch (error) {
    message.error('加载项目失败: ' + error)
    router.push('/projects')
  }
}

const navigateTo = (page) => {
  router.push(`/project/${projectId}/${page}`)
}

const handleActionNavigate = (page) => {
  if (page === 'datasource') {
    if (project.value?.datasource_id) {
      router.push('/datasource')
    }
  } else {
    navigateTo(page)
  }
}

onMounted(() => {
  loadProject()
})
</script>

<style scoped>
.project-workspace {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  background: var(--color-background);
  padding: 20px;
}

.workspace-body {
  display: flex;
  flex-direction: column;
  gap: 28px;
}
</style>
