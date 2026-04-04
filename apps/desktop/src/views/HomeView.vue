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

const goToProjects = () => {
  router.push('/projects')
}

const goToDataSource = () => {
  router.push('/datasource')
}

const goToTemplates = () => {
  router.push('/templates')
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

</style>
