<template>
  <div class="model-toolbar">
    <a-tooltip title="获取模型列表">
      <a-button type="primary" size="small" @click="emit('refresh-models')">
        <SettingOutlined />
      </a-button>
    </a-tooltip>
    <a-tooltip title="手动添加模型">
      <a-button size="small" @click="emit('add-model')">
        <PlusOutlined />
      </a-button>
    </a-tooltip>
  </div>

  <div class="models-content">
    <a-spin :spinning="loadingModels">
      <div v-if="modelGroups.length === 0" class="empty-models">
        <a-empty description="暂无模型">
          <a-button type="primary" @click="emit('refresh-models')">
            <SettingOutlined /> 获取模型列表
          </a-button>
        </a-empty>
      </div>
      <div v-else class="model-groups">
        <div
          v-for="group in modelGroups"
          :key="group.groupId"
          class="model-group"
        >
          <div class="group-header" @click="emit('toggle-group', group.groupId)">
            <CaretRightOutlined
              :class="['group-icon', { rotated: expandedGroups.has(group.groupId) }]"
            />
            <span class="group-name">{{ group.groupName }}</span>
            <a-tag class="group-count">{{ group.count }}</a-tag>
          </div>
          <div v-show="expandedGroups.has(group.groupId)" class="group-content">
            <div
              v-for="item in group.models"
              :key="item.id"
              class="model-item"
            >
              <div class="model-main">
                <div class="model-id">{{ item.modelId }}</div>
                <div class="model-name">{{ item.modelName }}</div>
                <div v-if="item.description" class="model-description">
                  {{ item.description }}
                </div>
              </div>
              <div class="model-actions">
                <a-button type="text" size="small" @click="emit('edit-model', item)">
                  <EditOutlined />
                </a-button>
                <a-button type="text" size="small" danger @click="emit('delete-model', item.id)">
                  <DeleteOutlined />
                </a-button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </a-spin>
  </div>
</template>

<script setup>
import {
  DeleteOutlined,
  PlusOutlined,
  CaretRightOutlined,
  EditOutlined,
  SettingOutlined
} from '@ant-design/icons-vue'

defineProps({
  provider: {
    type: Object,
    default: null
  },
  modelGroups: {
    type: Array,
    default: () => []
  },
  expandedGroups: {
    type: Object,
    required: true
  },
  loadingModels: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['toggle-group', 'edit-model', 'delete-model', 'add-model', 'refresh-models'])
</script>

<style scoped>
.model-toolbar {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-bottom: 10px;
}

.models-content {
  max-height: 400px;
  overflow-y: auto;
}

.empty-models {
  padding: 40px 20px;
  text-align: center;
}

.model-groups {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.model-group {
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--color-surface);
  cursor: pointer;
  transition: background-color 0.2s;
  user-select: none;
}

.group-header:hover {
  background: var(--color-background);
}

.group-icon {
  font-size: 12px;
  color: var(--color-text-secondary);
  transition: transform 0.25s ease;
}

.group-icon.rotated {
  transform: rotate(90deg);
}

.group-name {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
}

.group-count {
  font-size: 11px;
  padding: 0 6px;
}

.group-content {
  border-top: 1px solid var(--color-border);
}

.model-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  transition: background-color 0.15s;
}

.model-item:hover {
  background: var(--color-surface);
}

.model-main {
  flex: 1;
  min-width: 0;
}

.model-id {
  font-weight: 500;
  font-size: 13px;
  color: var(--color-text);
  margin-bottom: 2px;
}

.model-name {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-bottom: 2px;
}

.model-description {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.model-actions {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
}

.models-content::-webkit-scrollbar {
  width: 6px;
}

.models-content::-webkit-scrollbar-track {
  background: transparent;
}

.models-content::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 3px;
}

.models-content::-webkit-scrollbar-thumb:hover {
  background: var(--color-hover);
}
</style>
