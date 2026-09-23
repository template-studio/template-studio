<script setup>
import { onMounted, onBeforeUnmount, computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import { theme } from 'ant-design-vue'
import { useThemeStore } from '@/stores/theme'
import { useConfigStore } from '@/stores/config'
import { useServerConnection } from '@/composables/useServerConnection'
import AppLayout from '@/components/layout/AppLayout.vue'
import ProjectWorkspaceLayout from '@/components/layout/ProjectWorkspaceLayout.vue'
import GlobalSearch from '@/components/common/GlobalSearch.vue'
import zhCN from 'ant-design-vue/es/locale/zh_CN'

const route = useRoute()
const themeStore = useThemeStore()
const globalSearchRef = ref(null)
const configStore = useConfigStore()

// 服务端连接感知(#716):离线横幅数据源
const { offline: serverOffline, start: startConnWatch, probeNow } = useServerConnection()

// Ant Design 主题配置（视觉语言：AgentHub/HiFox——单色主操作 + 品牌绿强调）
const antTheme = computed(() => ({
  algorithm: themeStore.isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
  components: {
    // 开关是状态指示器：开启=品牌绿（惯例彩色语义），不用单色主操作（黑/白开关难以辨认）
    Switch: {
      colorPrimary: themeStore.isDark ? '#22c55e' : '#16a34a',
    },
  },
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
    borderRadiusSM: 8,
    controlItemBgActive: themeStore.isDark ? '#32343b' : '#e8e8e6',
    controlItemBgHover: themeStore.isDark ? '#2a2c32' : '#f1f1ef',
  }
}))

// 根据路由判断使用哪个布局
const isProjectWorkspace = computed(() => {
  return route.path.startsWith('/project/')
})

// 独立全屏页（模板编辑器、项目转模板工作台等）：不套布局，直接渲染路由组件
// 注意 /convert 本身是主布局内的引导页，只有工作台子路径全屏
const isStandalonePage = computed(() => {
  return route.path.startsWith('/editor/') || route.path.startsWith('/convert/workbench')
})

// 全局右键菜单禁用
const handleGlobalContextMenu = (event) => {
  // 检查是否在编辑器区域内
  const target = event.target
  const isInEditor = target.closest('.cm-editor') ||
                     target.closest('.codemirror-container') ||
                     target.closest('.code-preview') ||
                     target.closest('.cw-code') ||          // 转换工作台代码区(自定义右键)
                     target.closest('.cw-ctx') ||           // 右键菜单浮层自身
                     target.closest('.ant-tree')            // 转换工作台文件树(自定义右键)

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

  // 服务端连接感知:配置就绪后再起(探活需 baseURL)
  startConnWatch()

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
      <!-- 离线横幅(#716):服务端不可达时全局提示,点击可立即重试 -->
      <transition name="offline-slide">
        <div v-if="serverOffline" class="offline-banner">
          <span class="offline-dot"></span>
          <span>无法连接服务器——在线功能不可用。请检查 Web 服务端是否运行,或到 设置→Web服务器 检查地址。</span>
          <button class="offline-retry" @click="probeNow">重试</button>
        </div>
      </transition>

      <!-- 主应用布局 -->
      <AppLayout v-if="!isProjectWorkspace && !isStandalonePage" />

      <!-- 项目工作区布局 -->
      <ProjectWorkspaceLayout v-else-if="isProjectWorkspace" />

      <!-- 独立全屏页（编辑器/转换工作台） -->
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

/* 离线横幅(#716) */
.offline-banner {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 3000;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 16px;
  background: #b45309;
  color: #fff;
  font-size: 12.5px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
}

.offline-dot {
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #fecaca;
  animation: offline-pulse 1.4s ease-in-out infinite;
}

@keyframes offline-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.35; }
}

.offline-retry {
  margin-left: auto;
  flex-shrink: 0;
  border: 1px solid rgba(255, 255, 255, 0.7);
  background: transparent;
  color: #fff;
  border-radius: 5px;
  padding: 2px 12px;
  font-size: 12px;
  cursor: pointer;
}

.offline-retry:hover { background: rgba(255, 255, 255, 0.15); }

.offline-slide-enter-active, .offline-slide-leave-active { transition: transform 0.25s ease, opacity 0.25s ease; }
.offline-slide-enter-from, .offline-slide-leave-to { transform: translateY(-100%); opacity: 0; }
</style>
