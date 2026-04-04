<template>
  <div class="home-view">
    <div class="welcome-section">
      <h1>欢迎使用 Template Studio</h1>
      <p>强大的模板脚手架与代码生成工具，支持多种编程语言和框架</p>
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

    <!-- 统计信息 -->
    <div class="stats-section">
      <a-row :gutter="16">
        <a-col :span="6">
          <a-card :bordered="false" class="stat-card">
            <a-statistic
              title="项目总数"
              :value="8"
              :value-style="{ color: '#1890ff' }"
              prefix="📁"
            />
          </a-card>
        </a-col>
        <a-col :span="6">
          <a-card :bordered="false" class="stat-card">
            <a-statistic
              title="数据表总数"
              :value="289"
              :value-style="{ color: '#52c41a' }"
              prefix="📊"
            />
          </a-card>
        </a-col>
        <a-col :span="6">
          <a-card :bordered="false" class="stat-card">
            <a-statistic
              title="生成次数"
              :value="156"
              :value-style="{ color: '#faad14' }"
              prefix="⚡"
            />
          </a-card>
        </a-col>
        <a-col :span="6">
          <a-card :bordered="false" class="stat-card">
            <a-statistic
              title="可用模板"
              :value="24"
              :value-style="{ color: '#722ed1' }"
              prefix="📝"
            />
          </a-card>
        </a-col>
      </a-row>
    </div>

    <!-- 最近项目 -->
    <div class="recent-projects">
      <a-card title="最近项目" :bordered="false">
        <a-list :data-source="recentProjects" :grid="{ gutter: 16, xs: 1, sm: 2, md: 3, lg: 3, xl: 3, xxl: 3 }">
          <template #renderItem="{ item }">
            <a-list-item>
              <a-card
                hoverable
                class="project-card"
                @click="openProject(item.id)"
              >
                <template #title>
                  <div class="project-name">{{ item.name }}</div>
                </template>
                <div class="project-meta">
                  <a-tag :color="getDatabaseColor(item.database)">{{ item.database }}</a-tag>
                  <span>{{ item.tables }} 张表</span>
                </div>
                <div class="project-time">{{ item.time }}</div>
              </a-card>
            </a-list-item>
          </template>
        </a-list>
      </a-card>
    </div>
  </div>
</template>

<script setup>
import { useRouter } from 'vue-router'
import {
  FolderOutlined,
  DatabaseOutlined,
  FileTextOutlined
} from '@ant-design/icons-vue'

const router = useRouter()

// 最近项目数据
const recentProjects = [
  {
    id: '1',
    name: '电商后台系统',
    database: 'mysql',
    tables: 24,
    time: '2 小时前'
  },
  {
    id: '2',
    name: '博客系统',
    database: 'postgresql',
    tables: 12,
    time: '1 天前'
  },
  {
    id: '3',
    name: '企业管理系统',
    database: 'mysql',
    tables: 45,
    time: '3 天前'
  }
]

// 导航方法
const goToProjects = () => {
  router.push('/projects')
}

const goToDataSource = () => {
  router.push('/datasource')
}

const goToTemplates = () => {
  router.push('/templates')
}

const openProject = (id) => {
  router.push(`/project/${id}`)
}

// 获取数据库颜色
const getDatabaseColor = (database) => {
  const colors = {
    mysql: 'blue',
    postgresql: 'cyan',
    sqlite: 'green',
    oracle: 'orange',
    sqlserver: 'purple'
  }
  return colors[database] || 'default'
}
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

.quick-actions {
  margin-bottom: var(--spacing-lg);
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

.stats-section {
  margin-bottom: var(--spacing-lg);
}

.stat-card {
  text-align: center;
  border: 1px solid var(--color-border);
  transition: all 0.3s ease;
}

.stat-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.recent-projects :deep(.ant-list-item) {
  padding: 0;
}

.project-card {
  cursor: pointer;
  height: 100%;
  transition: all 0.3s ease;
}

.project-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.project-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.project-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  margin: var(--spacing-sm) 0;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.project-time {
  font-size: 12px;
  color: var(--color-text-secondary);
}
</style>
