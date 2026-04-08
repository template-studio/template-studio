<template>
  <div class="browser-toolbar">
    <div class="toolbar-left">
      <a-button type="text" @click="emit('go-back')" class="back-btn">
        <template #icon><ArrowLeftOutlined /></template>
      </a-button>
      <div class="toolbar-title">
        <DatabaseOutlined class="title-icon" />
        <span class="datasource-name">{{ datasource?.name || '数据库浏览器' }}</span>
        <a-tag v-if="datasource" :color="getTypeColor(datasource.type_)">{{ datasource.type_.toUpperCase() }}</a-tag>
      </div>
    </div>
    <div class="toolbar-right">
      <a-input-search
        :value="treeSearch"
        @update:value="emit('update:treeSearch', $event)"
        placeholder="搜索表..."
        size="small"
        style="width: 200px"
        allow-clear
      />
      <a-button size="small" @click="emit('refresh')">
        <template #icon><ReloadOutlined /></template>
      </a-button>
    </div>
  </div>
</template>

<script setup>
import { ArrowLeftOutlined, DatabaseOutlined, ReloadOutlined } from '@ant-design/icons-vue'

defineProps({
  datasource: { type: Object, default: null },
  treeSearch: { type: String, default: '' }
})

const emit = defineEmits(['go-back', 'update:treeSearch', 'refresh'])

const getTypeColor = (type) => {
  const colors = { mysql: 'blue', postgresql: 'cyan', sqlite: 'green' }
  return colors[type] || 'default'
}
</script>

<style scoped>
.browser-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
  flex-shrink: 0;
  height: 44px;
}
.toolbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}
.back-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
}
.toolbar-title {
  display: flex;
  align-items: center;
  gap: 8px;
}
.title-icon {
  font-size: 16px;
  color: var(--color-primary);
}
.datasource-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
