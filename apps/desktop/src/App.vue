<script setup>
import { onMounted, onBeforeUnmount, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useThemeStore } from '@/stores/theme'
import { useConfigStore } from '@/stores/config'
import AppLayout from '@/components/layout/AppLayout.vue'
import ProjectWorkspaceLayout from '@/components/layout/ProjectWorkspaceLayout.vue'
import zhCN from 'ant-design-vue/es/locale/zh_CN'

const route = useRoute()
const themeStore = useThemeStore()
const configStore = useConfigStore()

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
  <a-config-provider :locale="zhCN">
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
