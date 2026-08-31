<template>
  <div class="recent-projects">
    <a-card title="最近项目" :bordered="false">
      <template #extra>
        <a-button type="link" @click="$emit('navigate-project')">
          查看全部
          <RightOutlined />
        </a-button>
      </template>

      <a-spin :spinning="loading">
        <a-empty v-if="!loading && projects.length === 0" description="暂无项目" />
        <div v-else class="project-list">
          <div
            v-for="project in projects"
            :key="project.id"
            class="project-item"
            @click="$emit('navigate-project', project.id)"
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
</template>

<script setup>
import {
  FolderOutlined,
  DatabaseOutlined,
  TableOutlined,
  RightOutlined,
  ClockCircleOutlined
} from '@ant-design/icons-vue'

defineProps({
  projects: {
    type: Array,
    default: () => []
  },
  loading: {
    type: Boolean,
    default: false
  }
})

defineEmits(['navigate-project'])

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
</script>

<style scoped>
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
  border-color: var(--color-border-strong);
  background: var(--color-hover);
}

.project-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--border-radius-lg);
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

@media (max-width: 768px) {
  .project-meta {
    flex-wrap: wrap;
    gap: var(--spacing-sm);
  }
}
</style>
