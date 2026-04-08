<template>
  <div class="section">
    <h3 class="section-title">快速操作</h3>
    <div class="actions-grid">
      <div class="action-card" @click="$emit('navigate', 'tables')">
        <TableOutlined class="action-icon" />
        <span class="action-label">表管理</span>
        <span class="action-desc">管理数据表和列结构</span>
      </div>
      <div class="action-card" @click="$emit('navigate', 'preferences')">
        <ControlOutlined class="action-icon" />
        <span class="action-label">规范配置</span>
        <span class="action-desc">字段规范、命名规范</span>
      </div>
      <div class="action-card" @click="$emit('navigate', 'mappings')">
        <SwapOutlined class="action-icon" />
        <span class="action-label">类型映射</span>
        <span class="action-desc">数据库到语言的类型映射</span>
      </div>
      <div class="action-card" @click="$emit('navigate', 'datasource')">
        <DatabaseOutlined class="action-icon" />
        <span class="action-label">数据源</span>
        <span class="action-desc">查看关联的数据源</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  TableOutlined,
  ControlOutlined,
  SwapOutlined,
  DatabaseOutlined
} from '@ant-design/icons-vue'

defineProps({
  projectId: [String, Number]
})

defineEmits(['navigate'])
</script>

<style scoped>
.section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  padding: 20px;
}

.section-title {
  margin: 0 0 16px;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

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
</style>
