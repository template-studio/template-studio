# AI 模型服务三栏设置设计文档

## 文档概述

本文档专门说明项目中 **AI 模型服务设置**的三栏架构设计、实现方式和技术细节。AI 模型服务是设置页面中的一个独立功能模块，采用动态二级菜单和统一配置组件的设计。

---

## 整体架构

### 三栏布局结构

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI 模型服务设置页面                          │
├──────────────┬──────────────────┬──────────────────────────────┤
│  一级菜单    │    二级菜单       │         内容区域             │
│ (固定)       │  (动态生成)       │      (动态加载)              │
├──────────────┼──────────────────┼──────────────────────────────┤
│ 常规设置     │                  │                              │
│ 高级设置     │                  │                              │
│ 模型服务 ◀───┼─▶ OpenAI        │  [提供商配置界面]             │
│ API 服务器   │   Ollama        │  - 提供商信息                 │
│ 关于         │   GLM           │  - API 配置                  │
│              │   DeepSeek      │  - 模型管理                   │
│              │   Claude        │  - 高级设置                   │
│              │   ...           │                              │
└──────────────┴──────────────────┴──────────────────────────────┘
```

**关键特性**:
- **一级菜单**: 固定选项，包含"模型服务"入口
- **二级菜单**: 动态生成，从数据库加载所有已配置的 AI 提供商
- **内容区域**: 统一的 `ModelProviderConfig.vue` 组件，根据选中提供商动态渲染

---

## 核心组件

### 1. 一级菜单 (SettingsSidebar.vue)

**职责**: 显示"模型服务"入口

**实现**:

```vue
<template>
  <div class="settings-sidebar">
    <ul class="settings-menu">
      <!-- 其他菜单项... -->
      <li>
        <a
          class="menu-item"
          :class="{ active: currentMainTab === 'models' }"
          @click="switchMainTab('models')"
        >
          <RobotOutlined />
          <span>模型服务</span>
        </a>
      </li>
    </ul>
  </div>
</template>
```

**关键点**:
- 使用 `RobotOutlined` 图标标识 AI 服务
- 点击 `models` 切换到模型服务设置

---

### 2. 二级菜单 (SettingsSubSidebar.vue)

**职责**: 动态显示所有 AI 提供商列表

**关键特性**:
- **动态生成**: 从 `aiConfigStore.providers` 加载提供商列表
- **状态标签**: 显示每个提供商的启用状态（ON 标签）
- **右键菜单**: 支持模型备注和添加新供应商
- **图标识别**: 根据提供商类型显示不同图标

**实现代码**:

```vue
<template>
  <div class="sub-sidebar" v-show="showSubSidebar">
    <ul class="sub-menu">
      <!-- 模型服务二级菜单 - 动态生成 -->
      <template v-if="currentMainTab === 'models'">
        <li v-for="provider in aiConfigStore.providers" :key="provider.providerName">
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === provider.providerName }"
            @click="switchSubTab(provider.providerName)"
            @contextmenu="handleMenuItemRightClick($event, provider.providerName)"
          >
            <div class="menu-content">
              <component :is="getProviderIcon(provider.providerType)" />
              <span>{{ provider.displayName }}</span>
            </div>
            <!-- 状态标签 -->
            <div
              v-if="getProviderStatus(provider).class"
              class="status-tag"
              :class="getProviderStatus(provider).class"
            >
              {{ getProviderStatus(provider).text }}
            </div>
          </a>
        </li>
      </template>
    </ul>
  </div>
</template>

<script setup>
import { computed, onMounted } from 'vue'
import {
  CloudServerOutlined,
  DatabaseOutlined,
  RobotOutlined
} from '@ant-design/icons-vue'
import { useAIConfigStore } from '@/stores/ai-config'

const aiConfigStore = useAIConfigStore()

// 获取提供商图标
const getProviderIcon = (providerType) => {
  switch (providerType) {
    case 'openai':
    case 'openai_compatible':
      return CloudServerOutlined
    case 'ollama':
      return DatabaseOutlined
    default:
      return RobotOutlined
  }
}

// 获取提供商状态
const getProviderStatus = (provider) => {
  if (!provider.isEnabled) {
    return { class: '', text: '' }
  }
  return { class: 'on', text: 'ON' }
}

// 组件挂载时初始化
onMounted(async () => {
  await aiConfigStore.initialize()
})
</script>
```

**状态标签样式**:

```css
.status-tag {
  padding: 2px 6px;
  border-radius: 10px;
  font-size: 10px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  line-height: 1;
}

.status-tag.on {
  background-color: #52c41a;
  color: white;
  box-shadow: 0 1px 3px rgba(82, 196, 26, 0.3);
}
```

---

### 3. 内容区域 (SettingsContent.vue + ModelProviderConfig.vue)

**职责**: 渲染选中提供商的配置界面

**实现方式**: 使用 Vue 动态组件和渲染函数

```vue
<script setup>
import { computed, h } from 'vue'
import ModelProviderConfig from '@/components/settings/ModelProviderConfig.vue'
import { useAIConfigStore } from '@/stores/ai-config'

const aiConfigStore = useAIConfigStore()

// 计算当前要显示的组件
const currentComponent = computed(() => {
  const { currentMainTab, currentSubTab } = props

  // 模型服务页面 - 动态生成
  if (currentMainTab === 'models') {
    const provider = aiConfigStore.providers.find(p => p.providerName === currentSubTab)
    if (provider) {
      return {
        render() {
          return h(ModelProviderConfig, {
            providerType: provider.providerType,
            providerName: provider.providerName,
            apiKeyPlaceholder: getApiKeyPlaceholder(provider.providerType),
            apiUrlPlaceholder: getApiUrlPlaceholder(provider.providerType),
            initialConfig: getProviderConfig(provider.providerName),
            onConfigChange: (config) => handleProviderConfigChange(provider.providerName, config),
            onProviderToggle: (enabled) => handleProviderToggle(provider.providerName, enabled),
            onConnectionTest: () => handleConnectionTest(provider.providerName)
          })
        }
      }
    }
  }

  // 其他设置页面...
})

// 根据提供商类型获取 API 密钥占位符
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
    case 'claude':
      return '请输入Anthropic API密钥'
    default:
      return '请输入 API 密钥'
  }
}

// 获取提供商配置
const getProviderConfig = (providerName) => {
  const provider = aiConfigStore.getProviderByName(providerName)
  if (!provider) return {}

  return {
    apiKey: provider.apiKey || '',
    baseUrl: provider.apiEndpoint || '',
    enabled: provider.isEnabled || false,
    temperature: provider.temperature || 0.7,
    maxTokens: provider.maxTokens || 4096
  }
}
</script>
```

---

## ModelProviderConfig 组件详解

**文件**: `src/renderer/src/components/settings/ModelProviderConfig.vue`

### 组件结构

```
┌─────────────────────────────────────────┐
│  提供商配置组件                          │
├─────────────────────────────────────────┤
│  [提供商头部]                            │
│  - 名称 + 官网链接                       │
│  - 启用开关                             │
├─────────────────────────────────────────┤
│  [API 配置区域]                         │
│  - API 密钥（密码输入框）                │
│    ├─ 添加配置按钮                      │
│    ├─ 测试连接按钮                      │
│    └─ 删除密钥按钮                      │
│  - API 地址（文本输入框）                │
│    └─ User-Agent 配置按钮               │
├─────────────────────────────────────────┤
│  [模型管理区域]                         │
│  - 分组列表（可折叠）                   │
│    ├─ 分组名称 + 数量                   │
│    └─ 模型列表                          │
│       ├─ 模型 ID                        │
│       ├─ 模型名称                       │
│       └─ 操作按钮（编辑/删除）          │
│  - 添加模型按钮                         │
├─────────────────────────────────────────┤
│  [自定义区域插槽]                       │
│  (用于提供商特定配置)                    │
└─────────────────────────────────────────┘
```

### 组件 Props

```javascript
const props = defineProps({
  // 基础配置
  providerType: {
    type: String,
    required: true
  },
  providerName: {
    type: String,
    required: true
  },

  // API 配置
  apiKeyPlaceholder: {
    type: String,
    default: '请输入 API 密钥'
  },
  apiUrlPlaceholder: {
    type: String,
    default: '请输入 API 地址'
  },

  // 初始数据
  initialConfig: {
    type: Object,
    default: () => ({})
  }
})
```

### 组件 Events

```javascript
const emit = defineEmits([
  'configChange',      // 配置变更
  'providerToggle',    // 启用状态切换
  'connectionTest'     // 连接测试
])
```

---

## AI 配置 Store

**文件**: `src/renderer/src/stores/ai-config.js`

### State 结构

```javascript
state: () => ({
  providers: [],              // AI 提供商列表
  selectedProvider: null,      // 当前选中的提供商
  loading: false,              // 加载状态
  defaultProvider: null,       // 默认提供商
  globalSettings: {            // 全局设置
    defaultProvider: '',
    autoRetry: true,
    maxRetries: 3
  }
})
```

### 核心 Actions

#### 1. 加载所有提供商

```javascript
async loadAllProviders() {
  this.loading = true
  try {
    const result = await window.electronAPI.ai.getAllAiProviders()
    if (result.success) {
      this.providers = result.data
    }
  } finally {
    this.loading = false
  }
}
```

#### 2. 保存提供商配置

```javascript
async saveProviderConfig(providerConfig) {
  try {
    const result = await window.electronAPI.ai.saveAiProvider(providerConfig)

    if (result.success) {
      // 更新本地状态
      const index = this.providers.findIndex(
        p => p.providerName === providerConfig.providerName
      )
      if (index !== -1) {
        this.providers[index] = providerConfig
      }

      message.success(result.message || '配置已保存')
      return true
    }
  } catch (error) {
    message.error('保存失败: ' + error.message)
    return false
  }
}
```

#### 3. 切换启用状态

```javascript
async toggleProvider(providerName, enabled) {
  try {
    const result = await window.electronAPI.ai.toggleAiProvider(providerName, enabled)

    if (result.success) {
      const provider = this.getProviderByName(providerName)
      if (provider) {
        provider.isEnabled = enabled
      }

      message.success(`提供商已${enabled ? '启用' : '禁用'}`)
      return true
    }
  } catch (error) {
    message.error('切换失败: ' + error.message)
    return false
  }
}
```

#### 4. 获取模型分组

```javascript
async getProviderModelsGrouped(providerName) {
  try {
    const result = await window.electronAPI.ai.getAiProviderModelsGrouped(providerName)

    if (result.success) {
      return result.data
    }
  } catch (error) {
    console.error('获取模型分组失败:', error)
    return []
  }
}
```

---

## 数据库表结构

### AI 提供商表 (ai_providers)

```sql
CREATE TABLE ai_providers (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  provider_name TEXT UNIQUE NOT NULL,        -- 提供商名称（如 openai, ollama）
  display_name TEXT NOT NULL,                -- 显示名称（如 OpenAI, Ollama）
  provider_type TEXT NOT NULL,               -- 提供商类型
  api_key TEXT,                              -- API 密钥
  api_endpoint TEXT,                         -- API 地址
  is_enabled INTEGER DEFAULT 0,              -- 是否启用
  is_default INTEGER DEFAULT 0,              -- 是否为默认提供商
  temperature REAL DEFAULT 0.7,              -- 温度参数
  max_tokens INTEGER DEFAULT 4096,           -- 最大 tokens
  timeout_seconds INTEGER DEFAULT 30,        -- 超时时间
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
)
```

### 模型表 (ai_models)

```sql
CREATE TABLE ai_models (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  model_id TEXT NOT NULL,                   -- 模型 ID（如 gpt-4）
  model_name TEXT NOT NULL,                 -- 模型名称
  provider_name TEXT NOT NULL,              -- 所属提供商
  group_id TEXT DEFAULT 'chat',             -- 分组 ID
  description TEXT,                         -- 描述
  max_tokens INTEGER DEFAULT 4096,          -- 最大 tokens
  supports_functions INTEGER DEFAULT 0,     -- 是否支持函数调用
  supports_vision INTEGER DEFAULT 0,        -- 是否支持视觉
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (provider_name) REFERENCES ai_providers(provider_name)
)
```

---

## 实现流程

### 初始化流程

```
1. 用户点击"模型服务"
   ↓
2. SettingsView 切换 currentMainTab = 'models'
   ↓
3. SettingsSubSidebar 显示二级菜单
   ↓
4. aiConfigStore.loadAllProviders()
   ↓
5. IPC: ai:getAllAiProviders
   ↓
6. 主进程从数据库加载提供商列表
   ↓
7. 返回数据到渲染进程
   ↓
8. 更新 aiConfigStore.providers
   ↓
9. SettingsSubSidebar 渲染提供商列表
```

### 选择提供商流程

```
1. 用户点击二级菜单中的提供商（如 OpenAI）
   ↓
2. switchSubTab('openai')
   ↓
3. currentSubTab = 'openai'
   ↓
4. SettingsContent 检测到 currentMainTab === 'models'
   ↓
5. 从 aiConfigStore.providers 查找 'openai'
   ↓
6. 使用 h() 函数渲染 ModelProviderConfig
   ↓
7. 传入提供商配置（apiKey, baseUrl, etc.）
   ↓
8. ModelProviderConfig 加载模型分组
   ↓
9. IPC: ai:getAiProviderModelsGrouped('openai')
   ↓
10. 渲染完整的配置界面
```

### 保存配置流程

```
1. 用户修改 API 密钥
   ↓
2. handleApiKeyChange() 触发
   ↓
3. emit('configChange', { ...providerData })
   ↓
4. SettingsContent 接收事件
   ↓
5. handleProviderConfigChange('openai', config)
   ↓
6. aiConfigStore.saveProviderConfig(config)
   ↓
7. IPC: ai:saveAiProvider
   ↓
8. 主进程更新数据库
   ↓
9. 返回成功/失败
   ↓
10. 更新本地状态
   ↓
11. 显示成功消息
```

---

## 关键技术实现

### 1. 动态二级菜单

**挑战**: 二级菜单需要从数据库动态加载

**解决方案**:
- 在 `SettingsSubSidebar.vue` 中监听 `aiConfigStore.providers`
- 使用 `v-for` 遍历渲染提供商列表
- 在组件挂载时调用 `aiConfigStore.initialize()`

```vue
<template>
  <template v-if="currentMainTab === 'models'">
    <li v-for="provider in aiConfigStore.providers" :key="provider.providerName">
      <a @click="switchSubTab(provider.providerName)">
        {{ provider.displayName }}
      </a>
    </li>
  </template>
</template>

<script setup>
onMounted(async () => {
  await aiConfigStore.initialize()
})
</script>
```

### 2. 动态组件渲染

**挑战**: 需要根据选中的提供商动态渲染配置组件

**解决方案**:
- 使用 Vue 的 `h()` 渲染函数
- 在 `computed` 中返回组件配置
- 动态传递 props

```javascript
const currentComponent = computed(() => {
  if (currentMainTab === 'models') {
    const provider = aiConfigStore.providers.find(
      p => p.providerName === currentSubTab
    )

    return {
      render() {
        return h(ModelProviderConfig, {
          providerType: provider.providerType,
          providerName: provider.providerName,
          // ... 其他 props
        })
      }
    }
  }
})
```

### 3. 状态标签显示

**挑战**: 需要实时显示提供商的启用状态

**解决方案**:
- 在二级菜单中使用计算属性获取状态
- 根据状态渲染不同样式的标签

```javascript
const getProviderStatus = (provider) => {
  if (!provider.isEnabled) {
    return { class: '', text: '' }
  }
  return { class: 'on', text: 'ON' }
}
```

```vue
<div
  v-if="getProviderStatus(provider).class"
  class="status-tag"
  :class="getProviderStatus(provider).class"
>
  {{ getProviderStatus(provider).text }}
</div>
```

### 4. 模型分组管理

**挑战**: 需要支持模型的分组、添加、编辑、删除

**解决方案**:
- 在 `ModelProviderConfig.vue` 中维护 `modelGroups` 状态
- 使用折叠/展开动画
- 通过模态对话框添加/编辑模型

```javascript
const modelGroups = ref([
  {
    id: 'chat',
    name: '对话模型',
    expanded: true,
    models: [
      { id: 'gpt-4', name: 'GPT-4', groupId: 'chat' },
      { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Turbo', groupId: 'chat' }
    ]
  }
])
```

---

## 扩展指南

### 添加新的 AI 提供商类型

#### 1. 定义提供商类型

在数据库中添加新提供商：

```sql
INSERT INTO ai_providers (
  provider_name,
  display_name,
  provider_type,
  api_endpoint,
  is_enabled
) VALUES (
  'newprovider',
  'New Provider',
  'newprovider_type',
  'https://api.newprovider.com/v1',
  0
)
```

#### 2. 添加图标映射

在 `SettingsSubSidebar.vue` 中：

```javascript
const getProviderIcon = (providerType) => {
  switch (providerType) {
    case 'openai':
      return CloudServerOutlined
    case 'ollama':
      return DatabaseOutlined
    case 'newprovider_type':  // 新增
      return RocketOutlined     // 新图标
    default:
      return RobotOutlined
  }
}
```

#### 3. 添加占位符文本

在 `SettingsContent.vue` 中：

```javascript
const getApiKeyPlaceholder = (providerType) => {
  switch (providerType) {
    case 'newprovider_type':
      return '请输入 New Provider API 密钥'  // 新增
    // ...
  }
}
```

### 自定义提供商配置

使用 `ModelProviderConfig.vue` 的插槽功能：

```vue
<ModelProviderConfig
  :provider-type="provider.providerType"
  :provider-name="provider.providerName"
>
  <template #customArea="{ provider }">
    <!-- 自定义配置区域 -->
    <div class="custom-config">
      <h4>特殊配置</h4>
      <a-input v-model:value="customSetting" />
    </div>
  </template>
</ModelProviderConfig>
```

---

## 样式定制

### 提供商头部样式

```css
.provider-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: transparent;
}

.provider-title h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
  color: var(--color-text);
}
```

### 模型分组样式

```css
.model-group {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: var(--color-background);
  cursor: pointer;
  transition: background-color 0.2s;
}

.group-header:hover {
  background: var(--color-surface);
}

.group-header .anticon {
  transition: transform 0.25s ease;
}

.group-header .anticon.rotated {
  transform: rotate(90deg);
}
```

---

## 最佳实践

### 1. 数据加载时机

- ✅ **推荐**: 在组件挂载时加载数据
```javascript
onMounted(async () => {
  await aiConfigStore.initialize()
})
```

- ❌ **避免**: 在每次渲染时加载数据
```javascript
// 不要这样做
watch(() => currentSubTab, async () => {
  await aiConfigStore.loadAllProviders()
})
```

### 2. 状态更新

- ✅ **推荐**: 使用 Store actions 更新状态
```javascript
await aiConfigStore.saveProviderConfig(config)
```

- ❌ **避免**: 直接修改 Store 状态
```javascript
// 不要这样做
aiConfigStore.providers[0].apiKey = 'new-key'
```

### 3. 错误处理

- ✅ **推荐**: 统一的错误处理和用户提示
```javascript
try {
  await aiConfigStore.saveProviderConfig(config)
  message.success('配置已保存')
} catch (error) {
  message.error('保存失败: ' + error.message)
}
```

---

## 常见问题

### Q1: 如何添加新的模型？

**A**: 在模型管理区域点击"添加模型"按钮，填写模型信息：
- 模型 ID（必填，如 `gpt-4`）
- 模型名称（可选，留空则使用模型 ID）
- 分组名称（如 `chat`、`code`、`image` 等）

### Q2: 如何测试 API 连接？

**A**: 在 API 密钥输入框右侧点击"测试连接"按钮，系统会验证配置是否正确。

### Q3: 模型分组如何管理？

**A**:
- 点击分组头部可以折叠/展开
- 鼠标悬停在分组头部会显示"删除分组"按钮
- 删除分组后，模型会移动到"未分组"

### Q4: 如何设置默认提供商？

**A**: 在全局设置中选择默认提供商，或通过 API 设置：
```javascript
await aiConfigStore.setDefaultProvider('openai')
```

---

## 参考文档

- [系统架构设计](./architecture.md)
- [状态管理设计](./state-management.md)
- [组件架构设计](./component-architecture.md)
- [IPC 通信文档](./ipc-communication.md)

---

**最后更新**: 2025-01-19
