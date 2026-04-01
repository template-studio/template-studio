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

// Simple debounce function
const debounce = (func, wait) => {
  let timeout
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout)
      func(...args)
    }
    clearTimeout(timeout)
    timeout = setTimeout(later, wait)
  }
}
</script>

<style scoped>
.app-layout {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.app-sidebar {
  background: var(--color-sidebar);
  border-right: 1px solid var(--color-border);
  box-shadow: 1px 0 4px rgba(0, 0, 0, 0.1);
}

.app-navbar {
  background: var(--color-navbar);
  border-bottom: 1px solid var(--color-border);
  padding: 0;
  height: var(--navbar-height);
  line-height: var(--navbar-height);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
  width: 100%;
}

.app-content {
  background: var(--color-background);
  overflow: hidden;
  height: calc(100vh - var(--navbar-height));
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

/* Smooth transitions */
.app-layout :deep(.ant-layout-sider-trigger) {
  background: var(--color-surface);
  border-top: 1px solid var(--color-border);
  color: var(--color-text);
}

.app-layout :deep(.ant-layout-sider-trigger:hover) {
  background: var(--color-hover);
}

/* Theme adjustments now handled by global CSS variables */
</style>