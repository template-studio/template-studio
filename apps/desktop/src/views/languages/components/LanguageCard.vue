<template>
  <div class="language-card" @click="$emit('edit', language)">
    <div class="card-content">
      <div class="language-icon" :style="{ color: getLanguageColor(language.color) }">
        {{ language.icon || '\u{1F4BB}' }}
      </div>

      <h3 class="language-name">{{ language.name }}</h3>

      <p class="language-description">
        {{ language.description || '暂无描述' }}
      </p>

      <div class="language-tags">
        <a-tag v-if="language.is_builtin" color="blue">内置</a-tag>
        <a-tag v-else color="green">自定义</a-tag>
      </div>
    </div>

    <div class="card-actions">
      <a-button
        type="text"
        size="small"
        @click.stop="$emit('settings', language)"
        class="action-btn"
        title="管理类型字段"
      >
        <SettingOutlined />
      </a-button>
      <a-button
        type="text"
        size="small"
        @click.stop="$emit('edit', language)"
        class="action-btn"
        title="编辑"
      >
        <EditOutlined />
      </a-button>
      <a-button
        type="text"
        size="small"
        danger
        @click.stop="$emit('delete', language)"
        class="action-btn"
        title="删除"
        :disabled="language.is_builtin"
      >
        <DeleteOutlined />
      </a-button>
    </div>
  </div>
</template>

<script setup>
import { EditOutlined, DeleteOutlined, SettingOutlined } from '@ant-design/icons-vue'

defineProps({
  language: {
    type: Object,
    required: true
  }
})

defineEmits(['settings', 'edit', 'delete'])

const colorMap = {
  red: '#f5222d',
  orange: '#fa8c16',
  gold: '#faad14',
  green: '#52c41a',
  cyan: '#13c2c2',
  blue: '#3e7bfa',
  purple: '#722ed1',
  pink: '#eb2f96'
}

const getLanguageColor = (color) => {
  if (!color) return '#d9d9d9'
  return colorMap[color] || color
}
</script>

<style scoped>
.language-card {
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  overflow: hidden;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.language-card:hover {
  transform: translateY(-3px);
  box-shadow: none;
}

.card-content {
  padding: var(--spacing-md);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.language-icon {
  font-size: 42px;
  line-height: 1;
  margin-bottom: 4px;
}

.language-name {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

.language-description {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-secondary);
  text-align: center;
  min-height: 32px;
  display: flex;
  align-items: center;
}

.language-tags {
  display: flex;
  gap: var(--spacing-xs);
}

.card-actions {
  display: flex;
  justify-content: flex-end;
  gap: 4px;
  padding: 8px var(--spacing-sm);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

.card-actions .ant-btn {
  font-size: 14px;
  padding: 4px 6px;
  height: auto;
  min-width: auto;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.card-actions .ant-btn:hover:not(:disabled) {
  transform: scale(1.1);
  background: var(--color-hover);
}
</style>
