<template>
  <div class="setting-container">
    <!-- 默认服务配置 -->
    <div class="setting-group">
      <div class="setting-title">默认 AI 服务</div>

      <div class="setting-row">
        <div class="setting-row-title">
          <div>默认提供商</div>
          <div class="setting-description">
            选择系统默认使用的 AI 服务提供商
          </div>
        </div>
        <a-select
          v-model:value="defaultSettings.provider"
          style="width: 200px"
          @change="handleProviderChange"
        >
          <a-select-option
            v-for="provider in enabledProviders"
            :key="provider.providerName"
            :value="provider.providerName"
          >
            {{ provider.displayName }}
          </a-select-option>
        </a-select>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">
          <div>默认模型</div>
          <div class="setting-description">
            选择默认使用的对话模型
          </div>
        </div>
        <a-select
          v-model:value="defaultSettings.model"
          style="width: 200px"
          :loading="loadingModels"
          :disabled="!defaultSettings.provider"
          @change="handleModelChange"
        >
          <a-select-option
            v-for="model in availableModels"
            :key="model.modelId"
            :value="model.modelId"
          >
            {{ model.modelName }}
          </a-select-option>
        </a-select>
      </div>
    </div>

    <!-- 保存按钮 -->
    <div class="action-buttons">
      <a-button type="primary" @click="handleSave" :loading="saving">
        <SaveOutlined /> 保存设置
      </a-button>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { message } from 'ant-design-vue'
import { SaveOutlined } from '@ant-design/icons-vue'
import { useAIConfigStore } from '@/stores/ai-config'

const aiConfigStore = useAIConfigStore()

const defaultSettings = reactive({
  provider: '',
  model: ''
})

const loadingModels = ref(false)
const availableModels = ref([])
const saving = ref(false)

// 获取已启用的提供商
const enabledProviders = computed(() => {
  return aiConfigStore.enabledProviders
})

// 监听默认提供商变化，加载对应的模型
watch(
  () => defaultSettings.provider,
  async (providerName) => {
    if (providerName) {
      await loadModelsForProvider(providerName)
    }
  }
)

// 加载提供商的模型列表
const loadModelsForProvider = async (providerName) => {
  loadingModels.value = true
  try {
    const models = await aiConfigStore.getProviderModelsGrouped(providerName)
    // 展平所有分组的模型
    availableModels.value = models.flatMap(group => group.models || [])

    // 如果当前默认模型不在列表中，选择第一个
    if (!availableModels.value.find(m => m.modelId === defaultSettings.model)) {
      defaultSettings.model = availableModels.value[0]?.modelId || ''
    }
  } catch (error) {
    console.error('加载模型失败:', error)
    message.error('加载模型失败: ' + error)
  } finally {
    loadingModels.value = false
  }
}

const handleProviderChange = (value) => {
  console.log('切换默认提供商:', value)
}

const handleModelChange = (value) => {
  console.log('切换默认模型:', value)
}

const handleSave = async () => {
  if (!defaultSettings.provider) {
    message.warning('请选择默认提供商')
    return
  }

  if (!defaultSettings.model) {
    message.warning('请选择默认模型')
    return
  }

  saving.value = true
  try {
    // 保存到 localStorage
    localStorage.setItem('ai-default-service', JSON.stringify(defaultSettings))

    // 更新 store 中的全局设置
    aiConfigStore.setDefaultProvider(defaultSettings.provider)

    message.success('默认服务已保存')
  } catch (error) {
    message.error('保存失败: ' + error)
  } finally {
    saving.value = false
  }
}

const loadSettings = () => {
  try {
    const saved = localStorage.getItem('ai-default-service')
    if (saved) {
      const parsed = JSON.parse(saved)
      defaultSettings.provider = parsed.provider || ''
      defaultSettings.model = parsed.model || ''
    }

    // 如果没有默认提供商，使用第一个启用的提供商
    if (!defaultSettings.provider && enabledProviders.value.length > 0) {
      defaultSettings.provider = enabledProviders.value[0].providerName
    }
  } catch (error) {
    console.error('加载设置失败:', error)
  }
}

onMounted(async () => {
  await aiConfigStore.initialize()
  loadSettings()
})
</script>

<style scoped>
@import '@/assets/styles/settings.css';

.action-buttons {
  display: flex;
  gap: 12px;
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--color-border);
}
</style>
