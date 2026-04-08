<template>
  <div class="browser-sidebar" :style="{ width: sidebarWidth + 'px' }">
    <div class="sidebar-content">
      <a-spin :spinning="treeLoading" size="small">
        <div v-if="filteredDatabases.length > 0" class="tree-list">
          <div
            v-for="db in filteredDatabases"
            :key="db.name"
            class="tree-node database-node"
            :class="{ 'is-expanded': expandedDbs.has(db.name), 'is-selected': selectedDb === db.name && !selectedTable }"
          >
            <div class="tree-node-header" @click="emit('toggle-database', db.name)">
              <span class="tree-arrow">
                <RightOutlined v-if="!expandedDbs.has(db.name)" />
                <DownOutlined v-else />
              </span>
              <DatabaseOutlined class="tree-icon db-icon" />
              <span class="tree-label">{{ db.name }}</span>
              <span class="tree-count" v-if="db.tables.length">{{ db.tables.length }}</span>
            </div>
            <div v-if="expandedDbs.has(db.name)" class="tree-children">
              <div
                v-for="table in filterTables(db.tables)"
                :key="table"
                class="tree-node table-node"
                :class="{ 'is-selected': selectedDb === db.name && selectedTable === table }"
                @click="emit('select-table', db.name, table)"
              >
                <span class="tree-indent"></span>
                <TableOutlined class="tree-icon table-icon" />
                <span class="tree-label">{{ table }}</span>
              </div>
              <div v-if="filterTables(db.tables).length === 0 && treeSearch" class="tree-empty">
                无匹配表
              </div>
            </div>
          </div>
        </div>
        <div v-else-if="!treeLoading" class="tree-empty">
          暂无数据库
        </div>
      </a-spin>
    </div>
    <div class="resize-handle" @mousedown="emit('start-resize', $event)"></div>
  </div>
</template>

<script setup>
import { DatabaseOutlined, TableOutlined, RightOutlined, DownOutlined } from '@ant-design/icons-vue'

const props = defineProps({
  databases: { type: Array, default: () => [] },
  filteredDatabases: { type: Array, default: () => [] },
  expandedDbs: { type: Object, default: () => new Set() },
  selectedDb: { type: String, default: '' },
  selectedTable: { type: String, default: '' },
  treeSearch: { type: String, default: '' },
  treeLoading: { type: Boolean, default: false },
  sidebarWidth: { type: Number, default: 240 }
})

const emit = defineEmits(['toggle-database', 'select-table', 'start-resize'])

const filterTables = (tables) => {
  if (!props.treeSearch) return tables
  const q = props.treeSearch.toLowerCase()
  return tables.filter(t => t.toLowerCase().includes(q))
}
</script>

<style scoped>
.browser-sidebar {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--color-border);
  position: relative;
  flex-shrink: 0;
  overflow: hidden;
}
.sidebar-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}
.tree-list {
  display: flex;
  flex-direction: column;
}
.tree-node-header {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  cursor: pointer;
  transition: background 0.15s;
  user-select: none;
}
.tree-node-header:hover {
  background: var(--color-hover);
}
.database-node.is-selected > .tree-node-header {
  background: var(--color-primary-bg);
}
.tree-arrow {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}
.tree-icon {
  font-size: 14px;
  flex-shrink: 0;
}
.db-icon {
  color: var(--color-primary);
}
.table-icon {
  color: var(--color-success);
}
.tree-label {
  font-size: 13px;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}
.tree-count {
  font-size: 11px;
  color: var(--color-text-muted);
  background: var(--color-surface);
  padding: 0 6px;
  border-radius: 10px;
  flex-shrink: 0;
}
.tree-children {
  display: flex;
  flex-direction: column;
}
.table-node {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 12px 5px 28px;
  cursor: pointer;
  transition: background 0.15s;
  user-select: none;
}
.table-node:hover {
  background: var(--color-hover);
}
.table-node.is-selected {
  background: var(--color-primary-bg);
}
.tree-indent {
  width: 16px;
  flex-shrink: 0;
}
.tree-empty {
  padding: 12px 16px;
  color: var(--color-text-muted);
  font-size: 13px;
  text-align: center;
}
.resize-handle {
  position: absolute;
  top: 0;
  right: -2px;
  width: 4px;
  height: 100%;
  cursor: col-resize;
  z-index: 10;
}
.resize-handle:hover,
.resize-handle:active {
  background: var(--color-primary);
}
</style>
