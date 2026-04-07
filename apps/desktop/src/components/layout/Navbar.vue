<template>
  <div class="navbar titlebar-drag-region">
    <div class="navbar-left">
      <!-- Sidebar Toggle -->
      <a-button
        type="text"
        class="sidebar-toggle titlebar-no-drag"
        @click="toggleSidebar"
        v-show="!layoutStore.isMobile"
      >
        <template #icon>
          <MenuFoldOutlined v-if="!layoutStore.sidebarCollapsed" />
          <MenuUnfoldOutlined v-else />
        </template>
      </a-button>

      <!-- Mobile Menu Toggle -->
      <a-button
        type="text"
        class="menu-toggle titlebar-no-drag"
        @click="toggleMobileMenu"
        v-show="layoutStore.isMobile"
      >
        <template #icon>
          <MenuOutlined />
        </template>
      </a-button>

      <!-- Page Title -->
      <div class="page-title">
        <h1>{{ currentPageTitle }}</h1>
      </div>
    </div>

    <div class="navbar-center">
      <!-- Search Bar (placeholder) -->
      <a-input-search
        v-if="showSearch"
        placeholder="Search..."
        class="search-bar titlebar-no-drag"
        style="width: 300px"
        @search="handleSearch"
      />
    </div>

    <div class="navbar-right">
      <!-- Notification Center -->
      <NotificationCenter />

      <!-- Window Controls -->
      <div class="window-controls">
        <a-button
          type="text"
          size="small"
          class="window-control titlebar-no-drag"
          @click="minimizeWindow"
        >
          <template #icon>
            <MinusOutlined />
          </template>
        </a-button>
        <a-button
          type="text"
          size="small"
          class="window-control titlebar-no-drag"
          @click="maximizeWindow"
        >
          <template #icon>
            <BorderOutlined />
          </template>
        </a-button>
        <a-button
          type="text"
          size="small"
          class="window-control close titlebar-no-drag"
          @click="closeWindow"
        >
          <template #icon>
            <CloseOutlined />
          </template>
        </a-button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useLayoutStore } from '@/stores/layout'
import { tauriApi } from '@/utils/tauriApi'
import NotificationCenter from '@/components/common/NotificationCenter.vue'
import {
  MenuOutlined,
  MenuFoldOutlined,
  MenuUnfoldOutlined,
  MinusOutlined,
  BorderOutlined,
  CloseOutlined
} from '@ant-design/icons-vue'

const router = useRouter()
const route = useRoute()
const layoutStore = useLayoutStore()

const showSearch = ref(false)

// Computed properties
const currentPageTitle = computed(() => {
  const titleMap = {
    '/': '首页',
    '/settings': '设置',
    '/help': '帮助'
  }
  return titleMap[route.path] || '模板'
})

// Methods
const toggleSidebar = () => {
  layoutStore.toggleSidebar()
}

const toggleMobileMenu = () => {
  layoutStore.toggleSidebar()
}

const handleSearch = (value) => {
  console.log('Search:', value)
  // TODO: Implement search functionality
}

const minimizeWindow = async () => {
  try {
    await tauriApi.window.minimize()
    document.activeElement.blur()
  } catch (error) {
    console.error('Failed to minimize window:', error)
  }
}

const maximizeWindow = async () => {
  try {
    await tauriApi.window.maximize()
    document.activeElement.blur()
  } catch (error) {
    console.error('Failed to maximize window:', error)
  }
}

const closeWindow = async () => {
  try {
    await tauriApi.window.close()
  } catch (error) {
    console.error('Failed to close window:', error)
  }
}

</script>

<style scoped>
.navbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--spacing-md);
  height: 100%;
  background: var(--color-navbar);
  user-select: none; /* Prevent text selection in title bar */
}

/* Title bar drag region */
.titlebar-drag-region {
  -webkit-app-region: drag;
  width: 100%;
  height: 100%;
}

/* No drag region for interactive elements */
.titlebar-no-drag {
  -webkit-app-region: no-drag;
}

.navbar-left {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  flex: 1;
}

.navbar-center {
  display: flex;
  justify-content: center;
  flex: 2;
}

.navbar-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  flex: 1;
  justify-content: flex-end;
}

.sidebar-toggle {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
}

.sidebar-toggle:hover {
  color: var(--color-primary);
  background: var(--color-hover);
}

.menu-toggle {
  color: var(--color-text);
}

.page-title h1 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 500;
  color: var(--color-text);
  line-height: 1; /* Ensure vertical alignment */
}

.search-bar {
  border-radius: var(--border-radius-md);
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 18px;
}

.window-control {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  outline: none;
}

.window-control:hover {
  color: var(--color-primary);
  background: var(--color-hover);
}

.window-control.close:hover {
  background: #ff4757;
  color: white;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .navbar {
    padding: 0 var(--spacing-sm);
  }

  .navbar-center {
    display: none;
  }

  .page-title h1 {
    font-size: 1rem;
  }

  }

@media (max-width: 480px) {
  .page-title {
    display: none;
  }
}

/* Dark theme adjustments now handled by global CSS variables */

</style>