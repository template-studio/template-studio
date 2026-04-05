<template>
  <div class="home-view">
    <!-- 欢迎区域 -->
    <div class="welcome-section">
      <h1>欢迎使用 Template Studio</h1>
      <p>强大的模板脚手架与代码生成工具，支持多种编程语言和框架</p>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-section">
      <a-row :gutter="16">
        <a-col :span="6">
          <div class="stat-card" @click="goToProjects">
            <div class="stat-icon projects">
              <FolderOutlined />
            </div>
            <div class="stat-content">
              <div class="stat-value">{{ statistics.total_projects }}</div>
              <div class="stat-label">项目总数</div>
            </div>
          </div>
        </a-col>
        <a-col :span="6">
          <div class="stat-card" @click="goToDataSource">
            <div class="stat-icon datasources">
              <DatabaseOutlined />
            </div>
            <div class="stat-content">
              <div class="stat-value">{{ statistics.total_datasources }}</div>
              <div class="stat-label">数据源</div>
            </div>
          </div>
        </a-col>
        <a-col :span="6">
          <div class="stat-card" @click="goToLanguages">
            <div class="stat-icon languages">
              <CodeOutlined />
            </div>
            <div class="stat-content">
              <div class="stat-value">{{ statistics.total_languages }}</div>
              <div class="stat-label">编程语言</div>
            </div>
          </div>
        </a-col>
        <a-col :span="6">
          <div class="stat-card" @click="goToProjects">
            <div class="stat-icon tables">
              <TableOutlined />
            </div>
            <div class="stat-content">
              <div class="stat-value">{{ statistics.total_tables }}</div>
              <div class="stat-label">数据表</div>
            </div>
          </div>
        </a-col>
      </a-row>
    </div>

    <!-- 快速操作 -->
    <div class="quick-actions">
      <a-card title="快速开始" :bordered="false">
        <a-row :gutter="16">
          <a-col :span="8">
            <div class="action-card" @click="goToProjects">
              <FolderOutlined class="action-icon" />
              <div class="action-title">我的项目</div>
              <div class="action-desc">管理您的生成项目</div>
            </div>
          </a-col>
          <a-col :span="8">
            <div class="action-card" @click="goToDataSource">
              <DatabaseOutlined class="action-icon" />
              <div class="action-title">数据源管理</div>
              <div class="action-desc">配置数据库连接</div>
            </div>
          </a-col>
          <a-col :span="8">
            <div class="action-card" @click="goToTemplates">
              <FileTextOutlined class="action-icon" />
              <div class="action-title">模板中心</div>
              <div class="action-desc">浏览代码模板</div>
            </div>
          </a-col>
        </a-row>
      </a-card>
    </div>

    <!-- 最近项目 -->
    <div class="recent-projects">
      <a-card title="最近项目" :bordered="false">
        <template #extra>
          <a-button type="link" @click="goToProjects">
            查看全部
            <RightOutlined />
          </a-button>
        </template>

        <a-spin :spinning="loading">
          <a-empty v-if="!loading && recentProjects.length === 0" description="暂无项目" />
          <div v-else class="project-list">
            <div
              v-for="project in recentProjects"
              :key="project.id"
              class="project-item"
              @click="openProject(project.id)"
            >
              <div class="project-icon">
                <FolderOutlined />
              </div>
              <div class="project-info">
                <div class="project-name">{{ project.name }}</div>
                <div class="project-meta">
                  <span v-if="project.database_name" class="meta-item">
                    <DatabaseOutlined />
                    {{ project.database_name }}
                  </span>
                  <span class="meta-item">
                    <TableOutlined />
                    {{ project.table_count }} 张表
                  </span>
                  <span class="meta-item">
                    <ClockCircleOutlined />
                    {{ formatDate(project.created_at) }}
                  </span>
                </div>
              </div>
              <RightOutlined class="project-arrow" />
            </div>
          </div>
        </a-spin>
      </a-card>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  FolderOutlined,
  DatabaseOutlined,
  FileTextOutlined,
  CodeOutlined,
  TableOutlined,
  RightOutlined,
  ClockCircleOutlined
} from '@ant-design/icons-vue'
import * as statisticsApi from '@/api/statistics'

const router = useRouter()
const loading = ref(false)

// 统计数据
const statistics = reactive({
  total_projects: 0,
  total_datasources: 0,
  total_languages: 0,
  total_tables: 0
})

// 最近项目
const recentProjects = ref([])

// 加载统计数据
const loadStatistics = async () => {
  try {
    const stats = await statisticsApi.getStatistics()
    Object.assign(statistics, stats)
  } catch (error) {
    console.error('加载统计数据失败:', error)
  }
}

// 加载最近项目
const loadRecentProjects = async () => {
  loading.value = true
  try {
    const projects = await statisticsApi.getRecentProjects(5)
    recentProjects.value = projects
  } catch (error) {
    console.error('加载最近项目失败:', error)
  } finally {
    loading.value = false
  }
}

// 格式化日期
const formatDate = (dateStr) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now - date
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))

  if (days === 0) {
    const hours = Math.floor(diff / (1000 * 60 * 60))
    if (hours === 0) {
      const minutes = Math.floor(diff / (1000 * 60))
      return minutes <= 0 ? '刚刚' : `${minutes} 分钟前`
    }
    return `${hours} 小时前`
  } else if (days === 1) {
    return '昨天'
  } else if (days < 7) {
    return `${days} 天前`
  } else {
    return date.toLocaleDateString('zh-CN')
  }
}

// 导航函数
const goToProjects = () => {
  router.push('/projects')
}

const goToDataSource = () => {
  router.push('/datasource')
}

const goToTemplates = () => {
  router.push('/templates')
}

const goToLanguages = () => {
  router.push('/languages')
}

const openProject = (id) => {
  router.push(`/project/${id}`)
}

// 初始化
onMounted(async () => {
  await Promise.all([
    loadStatistics(),
    loadRecentProjects()
  ])
})
</script>

<style scoped>
.home-view {
  padding: var(--spacing-lg);
  width: 100%;
  min-height: calc(100vh - var(--navbar-height));
  background: var(--color-background);
  color: var(--color-text);
}

.welcome-section {
  text-align: center;
  margin-bottom: var(--spacing-xl);
  padding: var(--spacing-xl) 0;
}

.welcome-section h1 {
  font-size: 2.5rem;
  margin-bottom: var(--spacing-md);
  background: linear-gradient(90deg, #18a058 0%, #2196f3 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  font-weight: 800;
}

.welcome-section p {
  font-size: 1.2rem;
  color: var(--color-text-secondary);
}

/* 统计卡片 */
.stats-section {
  margin-bottom: var(--spacing-xl);
}

.stat-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  padding: var(--spacing-lg);
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  cursor: pointer;
  transition: all 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  border-color: var(--color-primary);
}

.stat-icon {
  width: 60px;
  height: 60px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  color: white;
}

.stat-icon.projects {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.stat-icon.datasources {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.stat-icon.languages {
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
}

.stat-icon.tables {
  background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%);
}

.stat-content {
  flex: 1;
}

.stat-value {
  font-size: 32px;
  font-weight: 700;
  color: var(--color-text);
  line-height: 1;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 14px;
  color: var(--color-text-secondary);
}

/* 快速操作 */
.quick-actions {
  margin-bottom: var(--spacing-xl);
}

.action-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  padding: var(--spacing-lg);
  text-align: center;
  cursor: pointer;
  transition: all 0.3s ease;
  height: 100%;
}

.action-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  border-color: var(--color-primary);
}

.action-icon {
  font-size: 48px;
  color: var(--color-primary);
  margin-bottom: var(--spacing-sm);
}

.action-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--spacing-xs);
}

.action-desc {
  font-size: 14px;
  color: var(--color-text-secondary);
}

/* 最近项目 */
.recent-projects {
  margin-bottom: var(--spacing-xl);
}

.project-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.project-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-md);
  cursor: pointer;
  transition: all 0.2s ease;
}

.project-item:hover {
  border-color: var(--color-primary);
  background: var(--color-hover);
}

.project-icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 20px;
  flex-shrink: 0;
}

.project-info {
  flex: 1;
  min-width: 0;
}

.project-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  font-size: 13px;
  color: var(--color-text-secondary);
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.project-arrow {
  color: var(--color-text-secondary);
  font-size: 14px;
  flex-shrink: 0;
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .stat-icon {
    width: 50px;
    height: 50px;
    font-size: 24px;
  }

  .stat-value {
    font-size: 28px;
  }
}

@media (max-width: 768px) {
  .welcome-section h1 {
    font-size: 2rem;
  }

  .welcome-section p {
    font-size: 1rem;
  }

  .stat-card {
    padding: var(--spacing-md);
  }

  .stat-icon {
    width: 44px;
    height: 44px;
    font-size: 20px;
  }

  .stat-value {
    font-size: 24px;
  }

  .action-card {
    padding: var(--spacing-md);
  }

  .action-icon {
    font-size: 36px;
  }

  .project-meta {
    flex-wrap: wrap;
    gap: var(--spacing-sm);
  }
}
</style>
