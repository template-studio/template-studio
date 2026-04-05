<template>
  <div class="project-workspace">
    <!-- 统计卡片 -->
    <div class="stats-grid">
      <div class="stat-card" @click="navigateTo('tables')">
        <div class="stat-icon table-icon">
          <TableOutlined />
        </div>
        <div class="stat-content">
          <span class="stat-value">{{ stats.tableCount }}</span>
          <span class="stat-label">数据表</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon column-icon">
          <ColumnHeightOutlined />
        </div>
        <div class="stat-content">
          <span class="stat-value">{{ stats.columnCount }}</span>
          <span class="stat-label">总列数</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon mapping-icon">
          <SwapOutlined />
        </div>
        <div class="stat-content">
          <span class="stat-value">{{ stats.mappingCount }}</span>
          <span class="stat-label">映射配置</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon time-icon">
          <ClockCircleOutlined />
        </div>
        <div class="stat-content">
          <span class="stat-value">{{ formatTime(project?.updated_at) }}</span>
          <span class="stat-label">最后更新</span>
        </div>
      </div>
    </div>

    <!-- 主体内容 -->
    <div class="workspace-body">
      <!-- 快速操作 -->
      <div class="section">
        <h3 class="section-title">快速操作</h3>
        <div class="actions-grid">
          <div class="action-card" @click="navigateTo('tables')">
            <TableOutlined class="action-icon" />
            <span class="action-label">表管理</span>
            <span class="action-desc">管理数据表和列结构</span>
          </div>
          <div class="action-card" @click="navigateTo('preferences')">
            <ControlOutlined class="action-icon" />
            <span class="action-label">规范配置</span>
            <span class="action-desc">字段规范、命名规范</span>
          </div>
          <div class="action-card" @click="navigateTo('mappings')">
            <SwapOutlined class="action-icon" />
            <span class="action-label">类型映射</span>
            <span class="action-desc">数据库到语言的类型映射</span>
          </div>
          <div class="action-card" @click="goToDataSource">
            <DatabaseOutlined class="action-icon" />
            <span class="action-label">数据源</span>
            <span class="action-desc">查看关联的数据源</span>
          </div>
        </div>
      </div>

      <!-- 数据表列表 -->
      <div class="section" v-if="recentTables.length > 0">
        <div class="section-header">
          <h3 class="section-title">数据表</h3>
          <a-button type="link" size="small" @click="navigateTo('tables')">
            查看全部 <RightOutlined />
          </a-button>
        </div>
        <div class="tables-list">
          <div
            v-for="table in recentTables"
            :key="table.id"
            class="table-item"
            @click="navigateTo('tables')"
          >
            <div class="table-main">
              <TableOutlined class="table-icon" />
              <span class="table-name">{{ table.name }}</span>
              <a-tag v-if="table.engine" size="small">{{ table.engine }}</a-tag>
            </div>
            <div class="table-meta">
              <span v-if="table.comment" class="table-comment">{{ table.comment }}</span>
              <span class="table-columns">{{ table.column_count || 0 }} 列</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 数据源信息 -->
      <div class="section" v-if="project?.datasource">
        <h3 class="section-title">数据源信息</h3>
        <div class="datasource-info">
          <div class="info-row">
            <span class="info-label">名称</span>
            <span class="info-value">{{ project.datasource.name }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">类型</span>
            <a-tag :color="getDatabaseColor(project.datasource.type_)">
              {{ project.datasource.type_?.toUpperCase() }}
            </a-tag>
          </div>
          <div class="info-row" v-if="project.datasource.host">
            <span class="info-label">主机</span>
            <span class="info-value">{{ project.datasource.host }}:{{ project.datasource.port }}</span>
          </div>
          <div class="info-row" v-if="project.database_name">
            <span class="info-label">数据库</span>
            <span class="info-value font-mono">{{ project.database_name }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { message } from 'ant-design-vue'
import {
  ArrowLeftOutlined,
  TableOutlined,
  ColumnHeightOutlined,
  SwapOutlined,
  ClockCircleOutlined,
  ControlOutlined,
  DatabaseOutlined,
  RightOutlined
} from '@ant-design/icons-vue'
import * as projectsApi from '@/api/projects'

const route = useRoute()
const router = useRouter()
const projectId = route.params.id

const project = ref(null)
const recentTables = ref([])
const stats = reactive({
  tableCount: 0,
  columnCount: 0,
  mappingCount: 0
})

// 加载项目数据
const loadProject = async () => {
  try {
    const data = await projectsApi.getProject(projectId)
    project.value = data

    // 加载表列表
    try {
      const tables = await projectsApi.getProjectTables(projectId)
      recentTables.value = (tables || []).slice(0, 8)
      stats.tableCount = tables?.length || 0
      stats.columnCount = tables?.reduce((sum, t) => sum + (t.column_count || 0), 0) || 0
    } catch {
      recentTables.value = []
    }

    // 加载映射数量（简单估算）
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

// 导航到子页面
const navigateTo = (page) => {
  router.push(`/project/${projectId}/${page}`)
}

// 返回项目列表
const goBack = () => {
  router.push('/projects')
}

// 跳转到数据源
const goToDataSource = () => {
  if (project.value?.datasource_id) {
    router.push(`/datasource`)
  }
}

// 格式化时间
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

// 获取数据库颜色
const getDatabaseColor = (type) => {
  const colors = { mysql: 'blue', postgresql: 'cyan', sqlite: 'green' }
  return colors[type] || 'default'
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

/* 头部 */
.workspace-header {
  margin-bottom: 24px;
}

.header-left {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.back-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  margin-top: 2px;
}

.project-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.project-name {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  color: var(--color-text);
  line-height: 1.3;
}

.project-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.database-name {
  font-size: 13px;
  color: var(--color-text-secondary);
  font-family: 'Courier New', monospace;
}

.project-desc {
  font-size: 13px;
  color: var(--color-text-muted);
}

/* 统计卡片 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 28px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  cursor: pointer;
  transition: all 0.2s;
}

.stat-card:hover {
  border-color: var(--color-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.stat-icon {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--border-radius-md);
  font-size: 20px;
}

.table-icon {
  background: rgba(25, 118, 210, 0.1);
  color: var(--color-primary);
}

.column-icon {
  background: rgba(82, 196, 26, 0.1);
  color: var(--color-success);
}

.mapping-icon {
  background: rgba(250, 173, 20, 0.1);
  color: var(--color-warning);
}

.time-icon {
  background: rgba(117, 117, 117, 0.1);
  color: var(--color-text-secondary);
}

.stat-content {
  display: flex;
  flex-direction: column;
}

.stat-value {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text);
  line-height: 1.2;
}

.stat-label {
  font-size: 13px;
  color: var(--color-text-secondary);
}

/* 主体内容 */
.workspace-body {
  display: flex;
  flex-direction: column;
  gap: 28px;
}

.section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  padding: 20px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-title {
  margin: 0 0 16px;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

.section-header .section-title {
  margin-bottom: 0;
}

/* 快速操作 */
.actions-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}

.action-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 20px 16px;
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-md);
  cursor: pointer;
  transition: all 0.2s;
  text-align: center;
}

.action-card:hover {
  border-color: var(--color-primary);
  background: var(--color-hover);
}

.action-icon {
  font-size: 24px;
  color: var(--color-primary);
}

.action-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text);
}

.action-desc {
  font-size: 12px;
  color: var(--color-text-muted);
}

/* 表列表 */
.tables-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
  background: var(--color-border);
  border-radius: var(--border-radius-md);
  overflow: hidden;
}

.table-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--color-background);
  cursor: pointer;
  transition: background 0.15s;
}

.table-item:hover {
  background: var(--color-hover);
}

.table-main {
  display: flex;
  align-items: center;
  gap: 10px;
}

.table-icon {
  font-size: 14px;
  color: var(--color-primary);
}

.table-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text);
}

.table-meta {
  display: flex;
  align-items: center;
  gap: 12px;
}

.table-comment {
  font-size: 13px;
  color: var(--color-text-muted);
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.table-columns {
  font-size: 12px;
  color: var(--color-text-secondary);
  background: var(--color-bg-secondary);
  padding: 2px 8px;
  border-radius: 10px;
}

/* 数据源信息 */
.datasource-info {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.info-label {
  width: 60px;
  font-size: 13px;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.info-value {
  font-size: 14px;
  color: var(--color-text);
}

.font-mono {
  font-family: 'Courier New', monospace;
}
</style>
