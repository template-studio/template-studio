<script setup>
import { onMounted, onBeforeUnmount, computed } from 'vue'
import { useRoute } from 'vue-router'
import { theme } from 'ant-design-vue'
import { useThemeStore } from '@/stores/theme'
import { useConfigStore } from '@/stores/config'
import AppLayout from '@/components/layout/AppLayout.vue'
import ProjectWorkspaceLayout from '@/components/layout/ProjectWorkspaceLayout.vue'
import zhCN from 'ant-design-vue/es/locale/zh_CN'

const route = useRoute()
const themeStore = useThemeStore()
const configStore = useConfigStore()

// Ant Design 主题配置
const antTheme = computed(() => ({
  algorithm: themeStore.isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
  token: {
    colorBgContainer: themeStore.isDark ? '#1e1e1e' : '#ffffff',
    colorBgElevated: themeStore.isDark ? '#252525' : '#ffffff',
    colorBgLayout: themeStore.isDark ? '#121212' : '#f5f5f5',
    colorText: themeStore.isDark ? '#ffffff' : '#212121',
    colorTextSecondary: themeStore.isDark ? '#a0a0a0' : '#757575',
    colorBorder: themeStore.isDark ? '#333333' : '#e0e0e0',
    colorBorderSecondary: themeStore.isDark ? '#2a2a2a' : '#f0f0f0',
    colorPrimary: themeStore.isDark ? '#90caf9' : '#1976d2',
    controlItemBgActive: themeStore.isDark ? 'rgba(144, 202, 249, 0.16)' : 'rgba(25, 118, 210, 0.08)',
    controlItemBgHover: themeStore.isDark ? 'rgba(144, 202, 249, 0.08)' : 'rgba(25, 118, 210, 0.04)',
  }
}))

// 根据路由判断使用哪个布局
const isProjectWorkspace = computed(() => {
  return route.path.startsWith('/project/')
})

// 全局右键菜单禁用
const handleGlobalContextMenu = (event) => {
  // 检查是否在编辑器区域内
  const target = event.target
  const isInEditor = target.closest('.cm-editor') ||
                     target.closest('.codemirror-container') ||
                     target.closest('.code-preview')

  if (isInEditor) {
    // 编辑器内不阻止，使用自定义右键菜单
    return
  }

  // 其他区域阻止系统右键菜单
  event.preventDefault()
}

onMounted(async () => {
  // 先加载配置（包含 API URL）
  await configStore.loadConfig()

  // 再初始化主题
  await themeStore.initializeTheme()

  // 全局禁用右键菜单
  document.addEventListener('contextmenu', handleGlobalContextMenu)
})

onBeforeUnmount(() => {
  // 清理事件监听
  document.removeEventListener('contextmenu', handleGlobalContextMenu)
})
</script>

<template>
  <a-config-provider :locale="zhCN" :theme="antTheme">
    <div id="app">
      <!-- 主应用布局 -->
      <AppLayout v-if="!isProjectWorkspace" />

      <!-- 项目工作区布局 -->
      <ProjectWorkspaceLayout v-else />
    </div>
  </a-config-provider>
</template>

<style>
html, body {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

#app {
  width: 100vw;
  height: 100vh;
  margin: 0;
  padding: 0;
  overflow: hidden;
}
</style>
