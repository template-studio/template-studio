<template>
  <a-layout class="app-layout">
    <!-- Sidebar -->
    <a-layout-sider
      v-model:collapsed="layoutStore.sidebarCollapsed"
      :width="layoutStore.sidebarWidth"
      :collapsed-width="60"
      breakpoint="lg"
      class="app-sidebar"
    >
      <Sidebar />
    </a-layout-sider>

    <!-- Main Content Area -->
    <a-layout>
      <!-- Navbar -->
      <a-layout-header class="app-navbar">
        <Navbar />
      </a-layout-header>

      <!-- Content -->
      <a-layout-content class="app-content">
        <MainContent />
      </a-layout-content>
    </a-layout>
  </a-layout>
</template>

<script setup>
import { onMounted, onUnmounted } from 'vue'
import { useLayoutStore } from '@/stores/layout'
import { useThemeStore } from '@/stores/theme'
import { debounce } from '@/utils/debounce'
import Sidebar from './Sidebar.vue'
import Navbar from './Navbar.vue'
import MainContent from './MainContent.vue'

const layoutStore = useLayoutStore()
const themeStore = useThemeStore()

let resizeObserver = null

onMounted(() => {
  // Initialize window size tracking
  updateWindowSize()

  // Setup resize observer with debouncing
  resizeObserver = new ResizeObserver(debounce(updateWindowSize, 200))
  resizeObserver.observe(document.body)

  // Initialize theme if not already done
  if (!themeStore.currentTheme) {
    themeStore.initializeTheme()
  }
})

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
  }
})


const updateWindowSize = () => {
  layoutStore.updateWindowSize({
    width: window.innerWidth,
    height: window.innerHeight
  })
}
</script>

<style scoped>
/* ============================================================
 * 壳层：画布 + 悬浮面板（AgentHub/HiFox 视觉语言）
 * 画布灰底上：侧栏卡片 | (顶栏卡片 + 内容卡片)，栏间 8px 细缝
 * ============================================================ */
.app-layout {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: var(--color-canvas);
  column-gap: 8px;
  padding: 8px 10px 10px 10px;
  box-sizing: border-box;
}

/* 内层布局透明，让画布透出 */
.app-layout :deep(.ant-layout) {
  background: transparent;
}

/* 悬浮面板通用：圆角卡片 + 细边框 + 呼吸阴影 */
.app-navbar,
.app-content {
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-panel);
  box-shadow: var(--shadow-panel);
  transition: box-shadow 0.35s ease;
  animation: panelBreathe 8s ease-in-out infinite;
}

.app-sidebar {
  background: var(--color-sidebar);
}

.app-sidebar:hover,
.app-navbar:hover,
.app-content:hover {
  animation: none;
  box-shadow: var(--shadow-panel-hover);
}

/* 悬浮栏呼吸：阴影缓慢涨落，像面板在轻轻起伏 */
@keyframes panelBreathe {
  /* 用令牌驱动，暗色主题下阴影同步翻转（写死值在暗色下不可见） */
  0%, 100% { box-shadow: var(--shadow-panel); }
  50%      { box-shadow: var(--shadow-panel-breathe); }
}

.app-sidebar {
  overflow: hidden;
}

.app-navbar {
  padding: 0;
  height: var(--navbar-height);
  line-height: var(--navbar-height);
  width: 100%;
  flex: none;
  margin-bottom: 8px;
}

.app-content {
  overflow: hidden;
  flex: 1;
  min-height: 0;
  width: 100%;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .app-sidebar {
    position: fixed !important;
    z-index: 1001;
    height: 100vh;
  }

  .app-content {
    margin-left: 0 !important;
  }
}

/* 折叠触发条随面板底色 */
.app-layout :deep(.ant-layout-sider-trigger) {
  background: var(--color-surface);
  border-top: 1px solid var(--color-border);
  color: var(--color-text);
}

.app-layout :deep(.ant-layout-sider-trigger:hover) {
  background: var(--color-hover);
}
</style>