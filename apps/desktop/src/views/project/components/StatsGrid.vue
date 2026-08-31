<template>
  <div class="stats-grid">
    <div class="stat-card" @click="$emit('navigate', 'tables')">
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
        <span class="stat-value">{{ stats.lastUpdate }}</span>
        <span class="stat-label">最后更新</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  TableOutlined,
  ColumnHeightOutlined,
  SwapOutlined,
  ClockCircleOutlined
} from '@ant-design/icons-vue'

defineProps({
  stats: Object
})

defineEmits(['navigate'])
</script>

<style scoped>
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
  border-color: var(--color-border-strong);
  box-shadow: none;
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
</style>
