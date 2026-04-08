<template>
  <div class="project-card">
    <div class="project-card-header">
      <div class="project-card-icon">
        <FolderOutlined />
      </div>
      <div class="project-card-info">
        <h2 class="project-card-name">{{ project?.name || '加载中...' }}</h2>
        <span v-if="project?.description" class="project-card-desc">{{ project.description }}</span>
      </div>
    </div>
    <div class="project-card-meta">
      <div class="meta-item" v-if="project?.datasource">
        <DatabaseOutlined class="meta-icon" />
        <a-tag :color="getDatabaseColor(project.datasource.type_)" size="small">
          {{ project.datasource.type_?.toUpperCase() }}
        </a-tag>
        <span class="meta-text">{{ project.datasource.name }}</span>
      </div>
      <div class="meta-item" v-if="project?.database_name">
        <span class="meta-label">数据库</span>
        <span class="meta-value font-mono">{{ project.database_name }}</span>
      </div>
      <div class="meta-item">
        <ClockCircleOutlined class="meta-icon" />
        <span class="meta-text">更新于 {{ formatTime(project?.updated_at) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  FolderOutlined,
  DatabaseOutlined,
  ClockCircleOutlined
} from '@ant-design/icons-vue'

defineProps({
  project: Object,
  datasource: Object
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

const getDatabaseColor = (type) => {
  const colors = { mysql: 'blue', postgresql: 'cyan', sqlite: 'green' }
  return colors[type] || 'default'
}
</script>

<style scoped>
.project-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  padding: 20px;
  margin-bottom: 16px;
}

.project-card-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}

.project-card-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(25, 118, 210, 0.1);
  border-radius: var(--border-radius-md);
  font-size: 24px;
  color: var(--color-primary);
}

.project-card-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.project-card-name {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text);
  line-height: 1.3;
}

.project-card-desc {
  font-size: 14px;
  color: var(--color-text-secondary);
}

.project-card-meta {
  display: flex;
  align-items: center;
  gap: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--color-border);
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.meta-icon {
  font-size: 14px;
  color: var(--color-text-muted);
}

.meta-text {
  color: var(--color-text-secondary);
}

.meta-label {
  color: var(--color-text-muted);
}

.meta-value {
  color: var(--color-text);
  font-weight: 500;
}

.font-mono {
  font-family: 'Courier New', monospace;
}
</style>
