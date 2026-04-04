<template>
  <a-modal
    :open="visible"
    :title="`${providerDisplayName} 模型管理`"
    width="720px"
    :footer="null"
    centered
    @cancel="handleClose"
    :body-styles="{ padding: '0', overflow: 'hidden' }"
  >
    <!-- 搜索和操作栏 -->
    <div class="popup-toolbar">
      <a-input
        v-model:value="searchText"
        placeholder="搜索模型..."
        allow-clear
        class="search-input"
      >
        <template #prefix><SearchOutlined /></template>
      </a-input>
      <div class="toolbar-actions">
        <a-tooltip title="刷新模型列表">
          <a-button
            :icon="h(SyncOutlined)"
            :loading="loadingModels"
            @click="fetchModels"
          />
        </a-tooltip>
        <a-tooltip :title="isAllFilteredAdded ? '移除列表中的模型' : '添加列表中的模型'">
          <a-button
            :icon="h(isAllFilteredAdded ? MinusOutlined : PlusOutlined)"
            @click="isAllFilteredAdded ? removeAllFiltered() : addAllFiltered()"
            :disabled="filteredModels.length === 0"
          />
        </a-tooltip>
      </div>
    </div>

    <!-- 模型列表 -->
    <a-spin :spinning="loadingModels">
      <div class="popup-model-list">
        <a-empty
          v-if="!loadingModels && filteredModels.length === 0"
          description="暂无模型"
          :style="{ padding: '60px 0' }"
        />
        <div v-else class="model-grid">
          <div
            v-for="group in modelGroups"
            :key="group.name"
            class="model-group-section"
          >
            <div class="group-section-header">
              <span class="group-section-name">{{ group.name }}</span>
              <a-tag>{{ group.models.length }}</a-tag>
            </div>
            <div class="group-models">
              <div
                v-for="model in group.models"
                :key="model.modelId"
                class="popup-model-item"
              >
                <div class="model-item-info">
                  <div class="model-item-id">{{ model.modelId }}</div>
                  <div v-if="model.description" class="model-item-desc">
                    {{ model.description }}
                  </div>
                </div>
                <a-button
                  :type="model.isAdded ? 'default' : 'primary'"
                  size="small"
                  :danger="model.isAdded"
                  @click="toggleModel(model)"
                >
                  {{ model.isAdded ? '移除' : '添加' }}
                </a-button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </a-spin>
  </a-modal>
</template>

<script setup>
import { ref, computed, watch, h } from 'vue'
import { message } from 'ant-design-vue'
import {
  SearchOutlined,
  SyncOutlined,
  PlusOutlined,
  MinusOutlined
} from '@ant-design/icons-vue'
import { useAIConfigStore } from '@/stores/ai-config'

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  providerName: {
    type: String,
    required: true
  },
  providerDisplayName: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['update:visible', 'refresh'])

const aiConfigStore = useAIConfigStore()

const searchText = ref('')
const loadingModels = ref(false)
const apiModels = ref([])
const dbModels = ref([])

// 合并后的所有模型（去重）
const allModels = computed(() => {
  const map = new Map()
  // 先加入 DB 模型
  for (const m of dbModels.value) {
    map.set(m.modelId, { ...m, isAdded: true })
  }
  // 再加入 API 模型（不覆盖已有的）
  for (const m of apiModels.value) {
    if (!map.has(m.modelId)) {
      map.set(m.modelId, { ...m, isAdded: false })
    }
  }
  return Array.from(map.values())
})

// 搜索过滤
const filteredModels = computed(() => {
  if (!searchText.value) return allModels.value
  const kw = searchText.value.toLowerCase()
  return allModels.value.filter(
    m => m.modelId.toLowerCase().includes(kw) || (m.modelName && m.modelName.toLowerCase().includes(kw))
  )
})

// 按 ownedBy 分组
const modelGroups = computed(() => {
  const groups = {}
  for (const m of filteredModels.value) {
    const groupName = m.ownedBy || '其他'
    if (!groups[groupName]) {
      groups[groupName] = { name: groupName, models: [] }
    }
    groups[groupName].models.push(m)
  }
  return Object.values(groups)
})

// 是否所有过滤后的模型都已添加
const isAllFilteredAdded = computed(() => {
  return filteredModels.value.length > 0 && filteredModels.value.every(m => m.isAdded)
})

// 加载 DB 中的模型
const loadDbModels = async () => {
  try {
    const groups = await aiConfigStore.getProviderModelsGrouped(props.providerName)
    const models = []
    for (const group of groups) {
      for (const m of group.models) {
        models.push({
          id: m.id,
          modelId: m.modelId,
          modelName: m.modelName,
          description: m.description,
          maxTokens: m.maxTokens
        })
      }
    }
    dbModels.value = models
  } catch (e) {
    console.error('加载 DB 模型失败:', e)
  }
}

// 从 API 获取模型列表
const fetchModels = async () => {
  loadingModels.value = true
  try {
    const models = await aiConfigStore.fetchProviderModels(props.providerName)
    apiModels.value = models
  } catch (e) {
    console.error('获取 API 模型失败:', e)
  } finally {
    loadingModels.value = false
  }
}

// 添加单个模型
const addModel = async (model) => {
  try {
    await aiConfigStore.addModel({
      modelId: model.modelId,
      modelName: model.modelName || model.modelId,
      providerName: props.providerName,
      groupId: 'chat',
      description: model.description || model.ownedBy || '',
      maxTokens: model.maxTokens || 4096
    })
    // 刷新 DB 模型列表
    await loadDbModels()
  } catch (e) {
    console.error('添加模型失败:', e)
  }
}

// 删除单个模型
const removeModel = async (model) => {
  if (!model.id) return
  try {
    await aiConfigStore.deleteModel(model.id)
    await loadDbModels()
  } catch (e) {
    console.error('删除模型失败:', e)
  }
}

// 切换模型状态
const toggleModel = async (model) => {
  if (model.isAdded) {
    await removeModel(model)
  } else {
    await addModel(model)
  }
}

// 添加所有过滤后的模型
const addAllFiltered = async () => {
  const toAdd = filteredModels.value.filter(m => !m.isAdded)
  if (toAdd.length === 0) return

  try {
    const count = await aiConfigStore.batchAddModels(
      props.providerName,
      toAdd.map(m => ({
        modelId: m.modelId,
        modelName: m.modelName || m.modelId,
        groupId: 'chat',
        description: m.description || m.ownedBy || '',
        maxTokens: m.maxTokens || 4096
      }))
    )
    await loadDbModels()
  } catch (e) {
    console.error('批量添加失败:', e)
  }
}

// 移除所有过滤后的模型
const removeAllFiltered = async () => {
  const toRemove = filteredModels.value.filter(m => m.isAdded && m.id)
  if (toRemove.length === 0) return

  for (const m of toRemove) {
    await aiConfigStore.deleteModel(m.id)
  }
  await loadDbModels()
}

const handleClose = () => {
  emit('update:visible', false)
  emit('refresh')
}

watch(() => props.visible, async (val) => {
  if (val) {
    searchText.value = ''
    apiModels.value = []
    await loadDbModels()
    await fetchModels()
  }
})
</script>

<style scoped>
.popup-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
}

.search-input {
  flex: 1;
}

.toolbar-actions {
  display: flex;
  gap: 6px;
}

.popup-model-list {
  height: 55vh;
  overflow-y: auto;
  padding: 0;
}

.popup-model-list::-webkit-scrollbar {
  width: 6px;
}

.popup-model-list::-webkit-scrollbar-track {
  background: transparent;
}

.popup-model-list::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 3px;
}

.model-grid {
  padding: 8px 16px;
}

.model-group-section {
  margin-bottom: 12px;
}

.group-section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  position: sticky;
  top: 0;
  background: var(--color-background);
  z-index: 1;
}

.group-section-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
}

.group-models {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.popup-model-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 6px;
  transition: background-color 0.15s;
}

.popup-model-item:hover {
  background: var(--color-surface);
}

.model-item-info {
  flex: 1;
  min-width: 0;
}

.model-item-id {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-item-desc {
  font-size: 11px;
  color: var(--color-text-secondary);
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
