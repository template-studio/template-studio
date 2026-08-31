<script setup>
import { onMounted, onBeforeUnmount, computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import { theme } from 'ant-design-vue'
import { useThemeStore } from '@/stores/theme'
import { useConfigStore } from '@/stores/config'
import AppLayout from '@/components/layout/AppLayout.vue'
import ProjectWorkspaceLayout from '@/components/layout/ProjectWorkspaceLayout.vue'
import GlobalSearch from '@/components/common/GlobalSearch.vue'
import zhCN from 'ant-design-vue/es/locale/zh_CN'

const route = useRoute()
const themeStore = useThemeStore()
const globalSearchRef = ref(null)
const configStore = useConfigStore()

// Ant Design 主题配置（视觉语言：AgentHub/HiFox——单色主操作 + 品牌绿强调）
const antTheme = computed(() => ({
  algorithm: themeStore.isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
  token: {
    colorBgContainer: themeStore.isDark ? '#1d1e23' : '#ffffff',
    colorBgElevated: themeStore.isDark ? '#24262b' : '#ffffff',
    colorBgLayout: themeStore.isDark ? '#141519' : '#f1f1ee',
    colorText: themeStore.isDark ? '#e9e9e6' : '#1c1d21',
    colorTextSecondary: themeStore.isDark ? '#a6a8ad' : '#5c6167',
    colorBorder: themeStore.isDark ? '#2c2e35' : '#e9e9e7',
    colorBorderSecondary: themeStore.isDark ? '#262830' : '#f0f0ee',
    // 主操作单色：浅色黑底白字 / 暗色反转白底黑字（colorTextLightSolid 需同步反转）
    colorPrimary: themeStore.isDark ? '#f2f2ef' : '#1b1c1f',
    colorTextLightSolid: themeStore.isDark ? '#17181c' : '#ffffff',
    colorLink: themeStore.isDark ? '#f2f2ef' : '#1b1c1f',
    colorSuccess: themeStore.isDark ? '#22c55e' : '#16a34a',
    colorError: '#e5484d',
    colorWarning: '#f5a623',
    colorInfo: '#3e7bfa',
    borderRadius: 8,
    controlItemBgActive: themeStore.isDark ? '#32343b' : '#e8e8e6',
    controlItemBgHover: themeStore.isDark ? '#2a2c32' : '#f1f1ef',
  }
}))

// 根据路由判断使用哪个布局
const isProjectWorkspace = computed(() => {
  return route.path.startsWith('/project/')
})

// 独立全屏页（模板编辑器等）：不套布局，直接渲染路由组件
const isStandalonePage = computed(() => {
  return route.path.startsWith('/editor/')
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

// 全局搜索快捷键处理
const handleGlobalKeydown = (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault()
    globalSearchRef.value?.open()
  }
}

onMounted(async () => {
  // 先加载配置（包含 API URL）
  await configStore.loadConfig()

  // 再初始化主题
  await themeStore.initializeTheme()

  // 全局禁用右键菜单
  document.addEventListener('contextmenu', handleGlobalContextMenu)

  // 全局搜索快捷键 Ctrl+K / Cmd+K
  document.addEventListener('keydown', handleGlobalKeydown)
})

onBeforeUnmount(() => {
  // 清理事件监听
  document.removeEventListener('contextmenu', handleGlobalContextMenu)
  document.removeEventListener('keydown', handleGlobalKeydown)
})
</script>

<template>
  <a-config-provider :locale="zhCN" :theme="antTheme">
    <div id="app">
      <!-- 主应用布局 -->
      <AppLayout v-if="!isProjectWorkspace && !isStandalonePage" />

      <!-- 项目工作区布局 -->
      <ProjectWorkspaceLayout v-else-if="isProjectWorkspace" />

      <!-- 独立全屏页（编辑器） -->
      <router-view v-else />

      <!-- 全局搜索 -->
      <GlobalSearch ref="globalSearchRef" />
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
