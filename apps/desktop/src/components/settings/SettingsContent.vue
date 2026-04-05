<template>
  <div class="content-area">
    <transition name="fade" mode="out-in">
      <!-- 常规设置页面 -->
      <div v-if="currentMainTab === 'general' && currentSubTab === 'basic'" key="general-basic">
        <GeneralBasicSettings />
      </div>
      <div v-else-if="currentMainTab === 'general' && currentSubTab === 'display'" key="general-display">
        <DisplaySettings />
      </div>
      <div v-else-if="currentMainTab === 'general' && currentSubTab === 'behavior'" key="general-behavior">
        <GeneralBehaviorSettings />
      </div>
      <div v-else-if="currentMainTab === 'general' && currentSubTab === 'shortcuts'" key="general-shortcuts">
        <KeyboardShortcutsSettings />
      </div>
      <div v-else-if="currentMainTab === 'general' && currentSubTab === 'backup'" key="general-backup">
        <BackupSettings />
      </div>

      <!-- 高级设置页面 -->
      <div v-else-if="currentMainTab === 'web-server'" key="web-server">
        <WebServerSettings />
      </div>
      <div v-else-if="currentMainTab === 'advanced' && currentSubTab === 'security'" key="advanced-security">
        <AdvancedSecuritySettings />
      </div>
      <div v-else-if="currentMainTab === 'advanced' && currentSubTab === 'network'" key="advanced-network">
        <AdvancedNetworkSettings />
      </div>
      <div v-else-if="currentMainTab === 'advanced' && currentSubTab === 'developer'" key="advanced-developer">
        <!-- 开发者选项页面 -->
        <div v-if="currentThirdTab === 'debug'" key="developer-debug">
          <AdvancedDeveloperDebugSettings />
        </div>
        <div v-else-if="currentThirdTab === 'experimental'" key="developer-experimental">
          <AdvancedDeveloperExperimentalSettings />
        </div>
        <div v-else-if="currentThirdTab === 'console'" key="developer-console">
          <AdvancedDeveloperConsoleSettings />
        </div>
        <div v-else key="developer-default">
          <AdvancedDeveloperDebugSettings />
        </div>
      </div>

      <!-- AI 服务页面 -->
      <div v-else-if="currentMainTab === 'ai-services'" key="ai-services">
        <!-- 默认服务配置 -->
        <DefaultAIService
          v-if="currentSubTab === 'default-service'"
        />

        <!-- 提供商配置 -->
        <ModelProviderConfig
          v-else-if="currentSubTab"
          :provider-type="getProviderType(currentSubTab)"
          :provider-name="currentSubTab"
          :api-key-placeholder="getApiKeyPlaceholder(getProviderType(currentSubTab))"
          :api-endpoint-placeholder="getApiUrlPlaceholder(currentSubTab)"
          :initial-config="getProviderConfig(currentSubTab)"
          @config-change="handleProviderConfigChange"
          @provider-toggle="handleProviderToggle"
          @connection-test="handleConnectionTest"
        />
      </div>

      <!-- 关于页面 -->
      <div v-else-if="currentMainTab === 'about'" key="about">
        <AboutSettings />
      </div>

      <!-- 默认页面 -->
      <div v-else key="default">
        <GeneralBasicSettings />
      </div>
    </transition>

    <!-- 模型选择对话框 -->
    <a-modal
      v-model:open="testModelDialogVisible"
      title="选择测试模型"
      @ok="executeConnectionTest"
      ok-text="测试连接"
      cancel-text="取消"
      width="400px"
    >
      <div style="margin-bottom: 12px; color: var(--color-text-secondary);">
        请选择一个模型进行连接测试：
      </div>
      <a-select
        v-model:value="testSelectedModel"
        style="width: 100%"
        placeholder="选择模型"
        show-search
        :filter-option="(input, option) => option.label.toLowerCase().includes(input.toLowerCase())"
      >
        <a-select-option
          v-for="model in testModelList"
          :key="model.modelId"
          :value="model.modelId"
          :label="model.modelName || model.modelId"
        >
          <div style="display: flex; justify-content: space-between;">
            <span>{{ model.modelName || model.modelId }}</span>
            <span style="color: var(--color-text-muted); font-size: 12px;">{{ model.modelId }}</span>
          </div>
        </a-select-option>
      </a-select>
    </a-modal>
  </div>
</template>

<script setup>
import {
  ref,
  computed,
  onMounted
} from 'vue'
import { message } from 'ant-design-vue'

// 导入所有设置组件
import GeneralBasicSettings from '@/views/settings/GeneralBasicSettings.vue'
import GeneralBehaviorSettings from '@/views/settings/GeneralBehaviorSettings.vue'
import DisplaySettings from '@/views/settings/DisplaySettings.vue'
import KeyboardShortcutsSettings from '@/views/settings/KeyboardShortcutsSettings.vue'
import BackupSettings from '@/views/settings/BackupSettings.vue'
import WebServerSettings from '@/views/settings/WebServerSettings.vue'
import AdvancedSecuritySettings from '@/views/settings/AdvancedSecuritySettings.vue'
import AdvancedNetworkSettings from '@/views/settings/AdvancedNetworkSettings.vue'
import AdvancedDeveloperDebugSettings from '@/views/settings/AdvancedDeveloperDebugSettings.vue'
import AdvancedDeveloperExperimentalSettings from '@/views/settings/AdvancedDeveloperExperimentalSettings.vue'
import AdvancedDeveloperConsoleSettings from '@/views/settings/AdvancedDeveloperConsoleSettings.vue'
import AboutSettings from '@/views/settings/AboutSettings.vue'
import ModelProviderConfig from '@/components/settings/ModelProviderConfig.vue'
import DefaultAIService from '@/components/settings/DefaultAIService.vue'
import { useAIConfigStore } from '@/stores/ai-config'

defineProps({
  currentMainTab: {
    type: String,
    default: 'general'
  },
  currentSubTab: {
    type: String,
    default: 'basic'
  },
  currentThirdTab: {
    type: String,
    default: 'debug'
  }
})

const aiConfigStore = useAIConfigStore()

// 组件挂载时初始化
onMounted(async () => {
  await aiConfigStore.initialize()
})

// 获取提供商类型
const getProviderType = (providerName) => {
  const provider = aiConfigStore.getProviderByName(providerName)
  return provider?.providerType || 'unknown'
}

// 获取 API 密钥占位符
const getApiKeyPlaceholder = (providerType) => {
  switch (providerType) {
    case 'openai':
    case 'openai_compatible':
      return 'sk-...'
    case 'ollama':
      return '本地服务通常不需要密钥'
    case 'glm':
      return '请输入智谱AI API密钥'
    case 'deepseek':
      return '请输入DeepSeek API密钥'
    default:
      return '请输入 API 密钥'
  }
}

// 获取 API 地址占位符（基于提供商名称）
const getApiUrlPlaceholder = (providerName) => {
  const defaultEndpoints = {
    'deepseek': 'https://api.deepseek.com/v1',
    'glm': 'https://open.bigmodel.cn/api/paas/v4',
    'openai': 'https://api.openai.com/v1',
    'ollama': 'http://localhost:11434/v1',
    'mimo': 'https://api.xiaomimimo.com/v1',
    'cherry-studio': 'http://127.0.0.1:23333/v1'
  }
  return defaultEndpoints[providerName] || '请输入 API 地址'
}

// 获取提供商配置
const getProviderConfig = (providerName) => {
  const provider = aiConfigStore.getProviderByName(providerName)
  if (!provider) return {}

  return {
    apiKey: provider.apiKey || '',
    apiEndpoint: provider.apiEndpoint || '',
    enabled: provider.isEnabled || false,
    temperature: provider.temperature || 0.7,
    maxTokens: provider.maxTokens || 4096
  }
}

// 处理提供商配置变更
const handleProviderConfigChange = async (providerName, config) => {
  const provider = aiConfigStore.getProviderByName(providerName)
  if (!provider) return

  const updatedConfig = {
    providerName: provider.providerName,
    displayName: provider.displayName,
    providerType: provider.providerType,
    ...config
  }

  await aiConfigStore.saveProviderConfig(updatedConfig)
}

// 处理提供商启用状态切换
const handleProviderToggle = async (providerName, enabled) => {
  await aiConfigStore.toggleProvider(providerName, enabled)
}

// 连接测试相关状态
const testModelDialogVisible = ref(false)
const testModelList = ref([])
const testSelectedModel = ref('')
const testProviderName = ref('')
const testLoading = ref(false)

// 处理连接测试
const handleConnectionTest = async (providerName) => {
  const provider = aiConfigStore.getProviderByName(providerName)
  if (!provider) {
    message.error('未找到提供商配置')
    return
  }

  if (!provider.apiKey && provider.providerType !== 'ollama') {
    message.warning('请先配置 API 密钥')
    return
  }

  // 获取已添加的模型列表
  try {
    const modelsGrouped = await aiConfigStore.getProviderModelsGrouped(providerName)
    const allModels = []
    if (Array.isArray(modelsGrouped)) {
      modelsGrouped.forEach(group => {
        if (group.models) {
          group.models.forEach(m => allModels.push(m))
        }
      })
    }

    if (allModels.length === 0) {
      message.warning('请先添加模型后再测试连接')
      return
    }

    // 显示模型选择对话框
    testModelList.value = allModels
    testSelectedModel.value = allModels[0]?.modelId || ''
    testProviderName.value = providerName
    testModelDialogVisible.value = true
  } catch (error) {
    message.error('获取模型列表失败: ' + error)
  }
}

// 执行连接测试
const executeConnectionTest = async () => {
  if (!testSelectedModel.value) {
    message.warning('请选择一个模型')
    return
  }

  testModelDialogVisible.value = false

  try {
    message.loading({ content: '正在测试连接...', key: 'connectionTest', duration: 0 })

    const { invoke } = await import('@tauri-apps/api/core')
    const result = await invoke('ai_test_connection', {
      providerName: testProviderName.value,
      providerType: aiConfigStore.getProviderByName(testProviderName.value)?.providerType || 'openai',
      apiKey: aiConfigStore.getProviderByName(testProviderName.value)?.apiKey || '',
      apiEndpoint: aiConfigStore.getProviderByName(testProviderName.value)?.apiEndpoint || '',
      model: testSelectedModel.value
    })

    message.destroy('connectionTest')
    message.success(`连接成功: ${result}`)
  } catch (error) {
    message.destroy('connectionTest')
    message.error(`连接失败: ${error}`)
  }
}
</script>

<style scoped>
.content-area {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  background: var(--color-background);
  padding: 0;
  height: 100%;
}

/* 过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* Settings content scrollbar - override global hide */
.content-area {
  scrollbar-width: thin; /* Firefox */
  scrollbar-color: var(--color-border) var(--color-background);
}

.content-area::-webkit-scrollbar {
  display: block; /* Override global hide */
  width: 8px;
}

.content-area::-webkit-scrollbar-track {
  background: var(--color-background);
}

.content-area::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 4px;
  transition: background var(--transition-fast);
}

.content-area::-webkit-scrollbar-thumb:hover {
  background: var(--color-hover);
}

/* Scrollbar theme adjustments now handled by global CSS variables */
</style>