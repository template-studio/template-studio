<template>
  <div class="stats-section">
    <a-row :gutter="16">
      <a-col :span="6">
        <div class="stat-card" @click="$emit('navigate', 'projects')">
          <div class="stat-icon projects">
            <FolderOutlined />
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.total_projects }}</div>
            <div class="stat-label">项目总数</div>
          </div>
        </div>
      </a-col>
      <a-col :span="6">
        <div class="stat-card" @click="$emit('navigate', 'datasource')">
          <div class="stat-icon datasources">
            <DatabaseOutlined />
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.total_datasources }}</div>
            <div class="stat-label">数据源</div>
          </div>
        </div>
      </a-col>
      <a-col :span="6">
        <div class="stat-card" @click="$emit('navigate', 'languages')">
          <div class="stat-icon languages">
            <CodeOutlined />
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.total_languages }}</div>
            <div class="stat-label">编程语言</div>
          </div>
        </div>
      </a-col>
      <a-col :span="6">
        <div class="stat-card" @click="$emit('navigate', 'projects')">
          <div class="stat-icon tables">
            <TableOutlined />
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.total_tables }}</div>
            <div class="stat-label">数据表</div>
          </div>
        </div>
      </a-col>
    </a-row>
  </div>
</template>

<script setup>
import {
  FolderOutlined,
  DatabaseOutlined,
  CodeOutlined,
  TableOutlined
} from '@ant-design/icons-vue'

defineProps({
  stats: Object
})

defineEmits(['navigate'])
</script>

<style scoped>
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
  border-color: var(--color-border-strong);
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
}
</style>
