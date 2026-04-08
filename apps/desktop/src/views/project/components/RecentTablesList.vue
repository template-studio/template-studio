<template>
  <div class="section" v-if="tables.length > 0">
    <div class="section-header">
      <h3 class="section-title">数据表</h3>
      <a-button type="link" size="small" @click="$emit('navigate-table')">
        查看全部 <RightOutlined />
      </a-button>
    </div>
    <div class="tables-list">
      <div
        v-for="table in tables"
        :key="table.id"
        class="table-item"
        @click="$emit('navigate-table')"
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
</template>

<script setup>
import {
  TableOutlined,
  RightOutlined
} from '@ant-design/icons-vue'

defineProps({
  tables: {
    type: Array,
    default: () => []
  }
})

defineEmits(['navigate-table'])
</script>

<style scoped>
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
</style>
