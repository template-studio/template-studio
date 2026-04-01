<template>
  <div class="setting-container">
    <!-- 提供商信息卡片 -->
    <div class="setting-group">
      <div class="provider-header">
        <div class="provider-info">
          <div class="provider-name">{{ displayName }}</div>
          <a
            v-if="providerWebsite"
            :href="providerWebsite"
            target="_blank"
            rel="noopener noreferrer"
            class="provider-link"
          >
            <GlobalOutlined /> 官方网站
          </a>
        </div>
        <a-switch
          :checked="localConfig.enabled"
          @change="handleToggle"
          checked-children="已启用"
          un-checked-children="已禁用"
        />
      </div>
    </div>

    <!-- API 配置卡片 -->
    <div class="setting-group">
      <div class="setting-title">API 配置</div>

      <!-- API 密钥 -->
      <div class="setting-row">
        <div class="setting-row-title">
          <div>API 密钥</div>
          <div class="setting-description">用于身份验证的密钥</div>
        </div>
        <div class="setting-actions">
          <a-input-password
            v-model:value="localConfig.apiKey"
            :placeholder="apiKeyPlaceholder"
            style="width: 280px"
          />
          <a-button size="small" @click="handleSaveApiKey">
            <SaveOutlined /> 保存
          </a-button>
          <a-button size="small" @click="handleTestConnection">
            <ApiOutlined /> 测试
          </a-button>
          <CheckCircleOutlined v-if="localConfig.apiKey" class="status-icon success" />
        </div>
      </div>

      <div class="setting-divider"></div>

      <!-- API 地址 -->
      <div class="setting-row">
        <div class="setting-row-title">
          <div>API 地址</div>
          <div class="setting-description">自定义 API 端点地址（可选）</div>
        </div>
        <div class="setting-actions">
          <a-input
            v-model:value="localConfig.apiEndpoint"
            :placeholder="apiEndpointPlaceholder"
            style="width: 280px"
          />
          <a-button size="small" @click="handleSaveEndpoint">
            <SaveOutlined /> 保存
          </a-button>
        </div>
      </div>
    </div>

    <!-- 高级设置卡片 -->
    <div class="setting-group">
      <div class="setting-title">高级设置</div>

      <!-- Temperature -->
      <div class="setting-row">
        <div class="setting-row-title">
          <div>Temperature</div>
          <div class="setting-description">控制输出的随机性，值越大越随机</div>
        </div>
        <a-slider
          v-model:value="localConfig.temperature"
          :min="0"
          :max="2"
          :step="0.1"
          :marks="{ 0: '0', 1: '1', 2: '2' }"
          style="width: 200px"
          @change="handleSaveAdvanced"
        />
      </div>

      <div class="setting-divider"></div>

      <!-- Max Tokens -->
      <div class="setting-row">
        <div class="setting-row-title">
          <div>Max Tokens</div>
          <div class="setting-description">最大生成的 token 数量</div>
        </div>
        <a-input-number
          v-model:value="localConfig.maxTokens"
          :min="1"
          :max="128000"
          :step="1000"
          style="width: 200px"
          @change="handleSaveAdvanced"
        />
      </div>
    </div>

    <!-- 模型管理卡片 -->
    <div class="setting-group">
      <div class="setting-title">
        <span>模型管理</span>
        <a-tag class="model-count">{{ totalModelCount }} 个</a-tag>
      </div>

      <div class="add-model-row">
        <a-button type="primary" size="small" @click="showAddModelDialog">
          <PlusOutlined /> 添加模型
        </a-button>
      </div>

      <div class="models-content">
        <a-spin :spinning="loadingModels">
          <div v-if="modelGroups.length === 0" class="empty-models">
            <a-empty description="暂无模型">
              <a-button type="primary" @click="showAddModelDialog">
                添加第一个模型
              </a-button>
            </a-empty>
          </div>
          <div v-else class="model-groups">
            <div
              v-for="group in modelGroups"
              :key="group.groupId"
              class="model-group"
            >
              <div class="group-header" @click="toggleGroup(group.groupId)">
                <CaretRightOutlined
                  :class="['group-icon', { rotated: expandedGroups.has(group.groupId) }]"
                />
                <span class="group-name">{{ group.groupName }}</span>
                <a-tag class="group-count">{{ group.count }}</a-tag>
              </div>
              <div v-show="expandedGroups.has(group.groupId)" class="group-content">
                <a-list
                  :data-source="group.models"
                  size="small"
                  :split="false"
                >
                  <template #renderItem="{ item }">
                    <a-list-item class="model-item">
                      <div class="model-main">
                        <div class="model-id">{{ item.modelId }}</div>
                        <div class="model-name">{{ item.modelName }}</div>
                        <div v-if="item.description" class="model-description">
                          {{ item.description }}
                        </div>
                      </div>
                      <template #actions>
                        <a-button type="text" size="small" @click="editModel(item)">
                          <EditOutlined />
                        </a-button>
                        <a-button type="text" size="small" danger @click="deleteModel(item.id)">
                          <DeleteOutlined />
                        </a-button>
                      </template>
                    </a-list-item>
                  </template>
                </a-list>
              </div>
            </div>
          </div>
        </a-spin>
      </div>
    </div>

    <!-- 添加/编辑模型对话框 -->
    <a-modal
      v-model:open="modelDialogVisible"
      :title="editingModel ? '编辑模型' : '添加模型'"
      width="500px"
      ok-text="确定"
      cancel-text="取消"
      @ok="saveModel"
    >
      <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
        <a-form-item label="模型 ID" required>
          <a-input
            v-model:value="modelForm.modelId"
            placeholder="如: gpt-4"
          />
        </a-form-item>
        <a-form-item label="模型名称" required>
          <a-input
            v-model:value="modelForm.modelName"
            placeholder="如: GPT-4"
          />
        </a-form-item>
        <a-form-item label="分组">
          <a-select v-model:value="modelForm.groupId">
            <a-select-option value="chat">对话模型</a-select-option>
            <a-select-option value="code">代码模型</a-select-option>
            <a-select-option value="image">图像模型</a-select-option>
            <a-select-option value="embedding">嵌入模型</a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="描述">
          <a-textarea
            v-model:value="modelForm.description"
            :rows="3"
            placeholder="模型描述（可选）"
          />
        </a-form-item>
        <a-form-item label="最大 Tokens">
          <a-input-number
            v-model:value="modelForm.maxTokens"
            :min="1"
            :max="128000"
            :step="1000"
            style="width: 100%"
          />
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, computed, watch } from 'vue'
import { message } from 'ant-design-vue'
import {
  CheckCircleOutlined,
  SaveOutlined,
  ApiOutlined,
  DeleteOutlined,
  PlusOutlined,
  QuestionCircleOutlined,
  CaretRightOutlined,
  EditOutlined,
  DatabaseOutlined,
  GlobalOutlined
} from '@ant-design/icons-vue'
import { useAIConfigStore } from '@/stores/ai-config'

const props = defineProps({
  providerType: {
    type: String,
    required: true
  },
  providerName: {
    type: String,
    required: true
  },
  apiKeyPlaceholder: {
    type: String,
    default: '请输入 API 密钥'
  },
  apiEndpointPlaceholder: {
    type: String,
    default: '请输入 API 地址'
  },
  initialConfig: {
    type: Object,
    default: () => ({})
  }
})

const emit = defineEmits(['configChange', 'providerToggle', 'connectionTest'])

const aiConfigStore = useAIConfigStore()

// 本地配置
const localConfig = reactive({
  apiKey: '',
  apiEndpoint: '',
  enabled: false,
  temperature: 0.7,
  maxTokens: 4096,
  ...props.initialConfig
})

// 模型数据
const modelGroups = ref([])
const loadingModels = ref(false)
const expandedGroups = ref(new Set(['chat', 'code']))

// 模型对话框
const modelDialogVisible = ref(false)
const editingModel = ref(null)
const modelForm = reactive({
  modelId: '',
  modelName: '',
  groupId: 'chat',
  description: '',
  maxTokens: 4096
})

// 计算属性
const displayName = computed(() => {
  const provider = aiConfigStore.getProviderByName(props.providerName)
  return provider?.displayName || props.providerName
})

const providerWebsite = computed(() => {
  const websites = {
    deepseek: 'https://www.deepseek.com',
    glm: 'https://open.bigmodel.cn',
    openai: 'https://platform.openai.com',
    ollama: 'https://ollama.ai'
  }
  return websites[props.providerType] || null
})

const totalModelCount = computed(() => {
  return modelGroups.value.reduce((sum, group) => sum + group.count, 0)
})

// 监听 provider 状态变化
watch(
  () => {
    const provider = aiConfigStore.getProviderByName(props.providerName)
    return provider?.isEnabled
  },
  (isEnabled) => {
    if (isEnabled !== undefined) {
      localConfig.enabled = isEnabled
    }
  },
  { immediate: false }
)

// 方法
const handleToggle = async (enabled) => {
  localConfig.enabled = enabled
  emit('providerToggle', props.providerName, enabled)
}

const handleSaveApiKey = async () => {
  await emit('configChange', props.providerName, {
    ...localConfig,
    apiKey: localConfig.apiKey
  })
  message.success('API 密钥已保存')
}

const handleDeleteApiKey = async () => {
  localConfig.apiKey = ''
  await emit('configChange', props.providerName, {
    ...localConfig,
    apiKey: null
  })
  message.success('API 密钥已删除')
}

const handleSaveEndpoint = async () => {
  await emit('configChange', props.providerName, {
    ...localConfig,
    apiEndpoint: localConfig.apiEndpoint
  })
  message.success('API 地址已保存')
}

const handleSaveAdvanced = async () => {
  await emit('configChange', props.providerName, {
    ...localConfig,
    temperature: localConfig.temperature,
    maxTokens: localConfig.maxTokens
  })
}

const handleTestConnection = () => {
  emit('connectionTest', props.providerName)
}

const toggleGroup = (groupId) => {
  if (expandedGroups.value.has(groupId)) {
    expandedGroups.value.delete(groupId)
  } else {
    expandedGroups.value.add(groupId)
  }
}

const showAddModelDialog = () => {
  editingModel.value = null
  modelForm.modelId = ''
  modelForm.modelName = ''
  modelForm.groupId = 'chat'
  modelForm.description = ''
  modelForm.maxTokens = 4096
  modelDialogVisible.value = true
}

const editModel = (model) => {
  editingModel.value = model
  modelForm.modelId = model.modelId
  modelForm.modelName = model.modelName
  modelForm.groupId = model.groupId || 'chat'
  modelForm.description = model.description || ''
  modelForm.maxTokens = model.maxTokens || 4096
  modelDialogVisible.value = true
}

const saveModel = async () => {
  if (!modelForm.modelId || !modelForm.modelName) {
    message.warning('请填写模型 ID 和模型名称')
    return
  }

  try {
    if (editingModel.value) {
      await aiConfigStore.updateModel(editingModel.value.id, {
        modelId: modelForm.modelId,
        modelName: modelForm.modelName,
        groupId: modelForm.groupId,
        description: modelForm.description,
        maxTokens: modelForm.maxTokens
      })
    } else {
      await aiConfigStore.addModel({
        modelId: modelForm.modelId,
        modelName: modelForm.modelName,
        providerName: props.providerName,
        groupId: modelForm.groupId,
        description: modelForm.description,
        maxTokens: modelForm.maxTokens
      })
    }

    modelDialogVisible.value = false
    await loadModels()
    message.success(editingModel.value ? '模型已更新' : '模型已添加')
  } catch (error) {
    console.error('保存模型失败:', error)
  }
}

const deleteModel = async (modelId) => {
  try {
    await aiConfigStore.deleteModel(modelId)
    await loadModels()
    message.success('模型已删除')
  } catch (error) {
    console.error('删除模型失败:', error)
  }
}

const loadModels = async () => {
  loadingModels.value = true
  try {
    const groups = await aiConfigStore.getProviderModelsGrouped(props.providerName)
    modelGroups.value = groups
  } catch (error) {
    console.error('加载模型失败:', error)
  } finally {
    loadingModels.value = false
  }
}

onMounted(async () => {
  await loadModels()
})
</script>

<style scoped>
@import '@/assets/styles/settings.css';

/* 提供商头部样式 */
.provider-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.provider-info {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.provider-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.provider-link {
  font-size: 12px;
  color: var(--color-primary);
  text-decoration: none;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.provider-link:hover {
  text-decoration: underline;
}

/* 设置操作区域 */
.setting-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-icon.success {
  font-size: 16px;
  color: #52c41a;
}

/* 模型数量标签 */
.model-count {
  font-size: 12px;
  margin-left: auto;
}

/* 添加模型按钮行 */
.add-model-row {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 10px;
}

/* 模型内容区域 */
.models-content {
  max-height: 400px;
  overflow-y: auto;
  margin-top: 10px;
}

.empty-models {
  padding: 40px 20px;
  text-align: center;
}

.model-groups {
  display: flex;
  flex-direction: column;
  gap: 8px;
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
  padding: 10px 12px;
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
  padding: 10px 12px;
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
  margin-bottom: 4px;
}

.model-description {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

/* 滚动条样式 */
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
