<template>
  <div class="browser-content">
    <template v-if="selectedTable">
      <div class="content-header">
        <div class="header-left">
          <h3 class="table-title">
            <TableOutlined />
            {{ selectedTable }}
          </h3>
          <span class="table-meta">
            <span v-if="tableData">{{ tableData.total }} 行</span>
            <span v-if="columns.length"> · {{ columns.length }} 列</span>
          </span>
        </div>
        <div class="header-right">
          <a-segmented :value="viewMode" @update:value="emit('update:viewMode', $event)" :options="viewOptions" size="small" />
        </div>
      </div>

      <div class="content-body" v-if="viewMode === 'data'">
        <a-spin :spinning="dataLoading">
          <div class="data-table-wrapper">
            <table class="data-table" v-if="tableData && tableData.columns.length > 0">
              <thead>
                <tr>
                  <th class="row-num-header">#</th>
                  <th v-for="col in tableData.columns" :key="col" class="data-header">
                    {{ col }}
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(row, idx) in tableData.rows" :key="idx">
                  <td class="row-num">{{ dataOffset + idx + 1 }}</td>
                  <td v-for="(val, colIdx) in row" :key="colIdx" class="data-cell">
                    <span v-if="val === null" class="null-value">NULL</span>
                    <span v-else>{{ val }}</span>
                  </td>
                </tr>
              </tbody>
            </table>
            <a-empty v-else-if="!dataLoading" description="表中暂无数据" />
          </div>
        </a-spin>
      </div>

      <div class="content-body" v-else>
        <a-spin :spinning="columnsLoading">
          <div class="columns-table-wrapper">
            <table class="columns-table" v-if="columns.length > 0">
              <thead>
                <tr>
                  <th>列名</th>
                  <th>类型</th>
                  <th>可空</th>
                  <th>键</th>
                  <th>默认值</th>
                  <th>注释</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="col in columns" :key="col.name">
                  <td class="col-name">
                    <span class="col-key-icon" v-if="col.key === 'PRI'"><KeyOutlined /></span>
                    {{ col.name }}
                  </td>
                  <td><a-tag>{{ col.type }}</a-tag></td>
                  <td>{{ col.nullable ? 'YES' : 'NO' }}</td>
                  <td>
                    <a-tag v-if="col.key === 'PRI'" color="orange">PK</a-tag>
                    <span v-else>-</span>
                  </td>
                  <td class="col-default">{{ col.default || '-' }}</td>
                  <td class="col-comment">{{ col.comment || '-' }}</td>
                </tr>
              </tbody>
            </table>
            <a-empty v-else-if="!columnsLoading" description="暂无列信息" />
          </div>
        </a-spin>
      </div>
    </template>

    <div v-else class="content-empty">
      <div class="empty-inner">
        <TableOutlined class="empty-icon" />
        <p class="empty-text">选择左侧的表以浏览数据</p>
        <p class="empty-hint">点击数据库展开表列表，然后选择要查看的表</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { TableOutlined, KeyOutlined } from '@ant-design/icons-vue'

defineProps({
  selectedTable: { type: String, default: '' },
  tableData: { type: Object, default: null },
  columns: { type: Array, default: () => [] },
  viewMode: { type: String, default: 'data' },
  viewOptions: { type: Array, default: () => [] },
  dataLoading: { type: Boolean, default: false },
  columnsLoading: { type: Boolean, default: false },
  dataOffset: { type: Number, default: 0 }
})

const emit = defineEmits(['update:viewMode'])
</script>

<style scoped>
.browser-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}
.content-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
  flex-shrink: 0;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}
.table-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  display: flex;
  align-items: center;
  gap: 6px;
}
.table-meta {
  font-size: 13px;
  color: var(--color-text-secondary);
}
.content-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.content-body :deep(.ant-spin-nested-loading),
.content-body :deep(.ant-spin-container) {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.data-table-wrapper {
  flex: 1;
  overflow: auto;
  min-height: 0;
  height: 100%;
}
.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
  table-layout: auto;
  color: var(--color-text);
}
.data-table th,
.data-table td {
  padding: 6px 12px;
  border-bottom: 1px solid var(--color-border);
  border-right: 1px solid var(--color-border-light);
  text-align: left;
  white-space: nowrap;
  color: var(--color-text);
}
.data-table thead {
  position: sticky;
  top: 0;
  z-index: 1;
}
.data-header {
  background: var(--color-surface);
  font-weight: 600;
  color: var(--color-text);
  border-right: 1px solid var(--color-border);
}
.row-num-header {
  background: var(--color-surface);
  width: 50px;
  text-align: center;
  color: var(--color-text-muted);
  border-right: 1px solid var(--color-border);
}
.row-num {
  width: 50px;
  text-align: center;
  color: var(--color-text-muted);
  font-size: 11px;
  background: var(--color-surface);
  border-right: 1px solid var(--color-border);
}
.data-cell {
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-text);
}
.null-value {
  color: var(--color-text-muted);
  font-style: italic;
  font-size: 12px;
}
.columns-table-wrapper {
  flex: 1;
  overflow: auto;
  padding: 0;
  height: 100%;
}
.columns-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
  color: var(--color-text);
}
.columns-table th {
  padding: 8px 16px;
  background: var(--color-surface);
  font-weight: 600;
  color: var(--color-text);
  border-bottom: 1px solid var(--color-border);
  text-align: left;
  position: sticky;
  top: 0;
  z-index: 1;
}
.columns-table td {
  padding: 8px 16px;
  border-bottom: 1px solid var(--color-border);
  color: var(--color-text);
}
.col-name {
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 4px;
}
.col-key-icon {
  color: var(--color-warning);
  font-size: 12px;
}
.col-default {
  color: var(--color-text-secondary);
  font-family: 'Courier New', monospace;
  font-size: 12px;
}
.col-comment {
  color: var(--color-text-secondary);
}
.content-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
.empty-inner {
  text-align: center;
}
.empty-icon {
  font-size: 48px;
  color: var(--color-text-muted);
  margin-bottom: 16px;
}
.empty-text {
  font-size: 16px;
  color: var(--color-text);
  margin: 0 0 8px;
}
.empty-hint {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin: 0;
}
</style>
