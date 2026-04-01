<template>
  <div class="content-area">
    <transition name="fade" mode="out-in">
      <!-- 常规设置页面 -->
      <div v-if="currentMainTab === 'general' && currentSubTab === 'basic'" key="general-basic">
        <GeneralBasicSettings />
      </div>
      <div v-else-if="currentMainTab === 'general' && currentSubTab === 'behavior'" key="general-behavior">
        <GeneralBehaviorSettings />
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

      <!-- 关于页面 -->
      <div v-else-if="currentMainTab === 'about'" key="about">
        <AboutSettings />
      </div>

      <!-- 默认页面 -->
      <div v-else key="default">
        <GeneralBasicSettings />
      </div>
    </transition>
  </div>
</template>

<script setup>
import {
  ref,
  computed
} from 'vue'

// 导入所有设置组件
import GeneralBasicSettings from '@/views/settings/GeneralBasicSettings.vue'
import GeneralBehaviorSettings from '@/views/settings/GeneralBehaviorSettings.vue'
import WebServerSettings from '@/views/settings/WebServerSettings.vue'
import AdvancedSecuritySettings from '@/views/settings/AdvancedSecuritySettings.vue'
import AdvancedNetworkSettings from '@/views/settings/AdvancedNetworkSettings.vue'
import AdvancedDeveloperDebugSettings from '@/views/settings/AdvancedDeveloperDebugSettings.vue'
import AdvancedDeveloperExperimentalSettings from '@/views/settings/AdvancedDeveloperExperimentalSettings.vue'
import AdvancedDeveloperConsoleSettings from '@/views/settings/AdvancedDeveloperConsoleSettings.vue'
import AboutSettings from '@/views/settings/AboutSettings.vue'

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