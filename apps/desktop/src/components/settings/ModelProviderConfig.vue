<template>
  <div class="setting-container">
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

    <div class="setting-group">
      <div class="setting-title">API 配置</div>

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
            @blur="handleConfigChange"
          />
          <a-tooltip title="测试连接">
            <a-button @click="handleTestConnection">
              <ApiOutlined />
            </a-button>
          </a-tooltip>
          <CheckCircleOutlined v-if="localConfig.apiKey" class="status-icon success" />
        </div>
      </div>

      <div class="setting-divider"></div>

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
            @blur="handleConfigChange"
          />
        </div>
      </div>
    </div>

    <div class="setting-group">
      <div class="setting-title">高级设置</div>

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
          @change="handleAdvancedChange"
        />
      </div>

      <div class="setting-divider"></div>

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
          @change="handleAdvancedChange"
        />
      </div>
    </div>

    <div class="setting-group">
      <div class="setting-title">
        <span>模型管理</span>
        <a-tag class="model-count">{{ totalModelCount }} 个</a-tag>
      </div>

      <ModelGroupList
        :provider="providerName"
        :model-groups="modelGroups"
        :expanded-groups="expandedGroups"
        :loading-models="loadingModels"
        @toggle-group="toggleGroup"
        @edit-model="editModel"
        @delete-model="deleteModel"
        @add-model="showAddModelDialog"
        @refresh-models="openManagePopup"
      />
    </div>

    <ManageModelsPopup
      v-model:visible="managePopupVisible"
      :provider-name="providerName"
      :provider-display-name="displayName"
      @refresh="loadModels"
    />

    <ModelEditDialog
      v-model:open="modelDialogVisible"
      :model-form="modelForm"
      :editing-model="editingModel"
      @submit="saveModel"
      @cancel="() => {}"
    />
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, computed, watch } from 'vue'
import { message } from 'ant-design-vue'
import {
  CheckCircleOutlined,
  ApiOutlined,
  GlobalOutlined
} from '@ant-design/icons-vue'
import { useAIConfigStore } from '@/stores/ai-config'
import ManageModelsPopup from './ManageModelsPopup.vue'
import ModelGroupList from './model/ModelGroupList.vue'
import ModelEditDialog from './model/ModelEditDialog.vue'

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

const localConfig = reactive({
  apiKey: '',
  apiEndpoint: '',
  enabled: false,
  temperature: 0.7,
  maxTokens: 4096,
  ...props.initialConfig
})

watch(
  () => props.initialConfig,
  (newConfig) => {
    if (newConfig) {
      Object.assign(localConfig, {
        apiKey: newConfig.apiKey || '',
        apiEndpoint: newConfig.apiEndpoint || '',
        enabled: newConfig.enabled || false,
        temperature: newConfig.temperature || 0.7,
        maxTokens: newConfig.maxTokens || 4096
      })
    }
  },
  { deep: true, immediate: false }
)

const modelGroups = ref([])
const loadingModels = ref(false)
const expandedGroups = ref(new Set(['chat', 'code']))

const managePopupVisible = ref(false)

const modelDialogVisible = ref(false)
const editingModel = ref(null)
const modelForm = reactive({
  modelId: '',
  modelName: '',
  groupId: 'chat',
  description: '',
  maxTokens: 4096
})

const displayName = computed(() => {
  const provider = aiConfigStore.getProviderByName(props.providerName)
  return provider?.displayName || props.providerName
})

const providerWebsite = computed(() => {
  const websites = {
    deepseek: 'https://www.deepseek.com',
    glm: 'https://open.bigmodel.cn',
    openai: 'https://platform.openai.com',
    ollama: 'https://ollama.ai',
    mimo: 'https://mimo.xiaomi.com/zh'
  }
  return websites[props.providerType] || null
})

const totalModelCount = computed(() => {
  return modelGroups.value.reduce((sum, group) => sum + group.count, 0)
})

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

const handleToggle = async (enabled) => {
  localConfig.enabled = enabled
  emit('providerToggle', props.providerName, enabled)
}

const handleConfigChange = async () => {
  if (localConfig.apiKey || localConfig.apiEndpoint) {
    emit('configChange', props.providerName, { ...localConfig })
  }
}

const handleAdvancedChange = async () => {
  emit('configChange', props.providerName, { ...localConfig })
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

const openManagePopup = () => {
  managePopupVisible.value = true
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

.setting-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.status-icon.success {
  font-size: 16px;
  color: #52c41a;
}

.model-count {
  font-size: 12px;
  margin-left: auto;
}
</style>
