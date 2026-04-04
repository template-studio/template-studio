<template>
  <div class="sub-sidebar" v-if="showSubSidebar">
    <ul class="sub-menu">
      <!-- 常规设置二级菜单 -->
      <template v-if="currentMainTab === 'general'">
        <li>
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === 'basic' }"
            @click="switchSubTab('basic')"
          >
            <span>基础设置</span>
          </a>
        </li>
        <li>
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === 'behavior' }"
            @click="switchSubTab('behavior')"
          >
            <span>行为设置</span>
          </a>
        </li>
      </template>

      <!-- AI 服务二级菜单 -->
      <template v-if="currentMainTab === 'ai-services'">
        <!-- 默认服务（最上方） -->
        <li>
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === 'default-service' }"
            @click="switchSubTab('default-service')"
          >
            <div class="menu-content">
              <StarOutlined />
              <span>默认服务</span>
            </div>
          </a>
        </li>

        <li v-if="currentSubTab === 'default-service'" class="menu-divider"></li>

        <!-- 各个提供商 -->
        <li v-for="provider in aiProviders" :key="provider.providerName">
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === provider.providerName }"
            @click="switchSubTab(provider.providerName)"
          >
            <div class="menu-content">
              <component :is="getProviderIcon(provider.providerType)" />
              <span>{{ provider.displayName }}</span>
            </div>
          </a>
        </li>
      </template>

      <!-- 高级设置二级菜单 -->
      <template v-if="currentMainTab === 'advanced'">
        <li>
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === 'security' }"
            @click="switchSubTab('security')"
          >
            <span>安全设置</span>
          </a>
        </li>
        <li>
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === 'network' }"
            @click="switchSubTab('network')"
          >
            <span>网络设置</span>
          </a>
        </li>
        <li>
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === 'developer' }"
            @click="switchSubTab('developer')"
          >
            <span>开发者选项</span>
          </a>
        </li>
      </template>
    </ul>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import {
  CloudServerOutlined,
  DatabaseOutlined,
  RobotOutlined,
  FireOutlined,
  StarOutlined
} from '@ant-design/icons-vue'
import { useAIConfigStore } from '@/stores/ai-config'

const props = defineProps({
  currentMainTab: {
    type: String,
    default: 'general'
  },
  currentSubTab: {
    type: String,
    default: 'basic'
  },
  showSubSidebar: {
    type: Boolean,
    default: true
  }
})

const emit = defineEmits(['sub-tab-change'])

const aiConfigStore = useAIConfigStore()
const aiProviders = computed(() => aiConfigStore.providers)

const switchSubTab = (tab) => {
  emit('sub-tab-change', tab)
}

// 获取提供商标图
const getProviderIcon = (providerType) => {
  switch (providerType) {
    case 'openai':
    case 'openai_compatible':
      return CloudServerOutlined
    case 'ollama':
      return DatabaseOutlined
    case 'deepseek':
      return FireOutlined
    case 'glm':
      return RobotOutlined
    default:
      return RobotOutlined
  }
}

// 组件挂载时初始化
onMounted(async () => {
  if (props.currentMainTab === 'ai-services') {
    await aiConfigStore.initialize()
  }
})
</script>

<style scoped>
.sub-sidebar {
  min-width: 160px;
  border-right: 1px solid var(--color-border);
  padding: 10px;
  user-select: none;
  flex-shrink: 0;
  min-height: 100%;
}

.sub-menu {
  list-style: none;
  margin: 0;
  padding: 0;
}

.sub-menu li {
  margin-bottom: 4px;
}

.sub-menu-item {
  display: block;
  padding: 8px 12px;
  width: 100%;
  text-decoration: none;
  color: var(--color-text);
  cursor: pointer;
  border-radius: var(--border-radius-md);
  font-size: 13px;
  transition: background-color var(--transition-fast);
  border: 0.5px solid transparent;
  margin: 2px 4px;
}

.sub-menu-item:hover {
  background: var(--color-surface);
  color: var(--color-primary);
  border-color: transparent;
}

.sub-menu-item.active {
  background: var(--color-surface);
  color: var(--color-primary);
  border-color: var(--color-border);
  font-weight: 500;
}

/* AI 服务二级菜单样式 */
.menu-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* Theme adjustments now handled by global CSS variables */

.menu-divider {
  height: 1px;
  background: var(--color-border);
  margin: 8px 0;
}

</style>