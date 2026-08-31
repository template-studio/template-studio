<template>
  <div class="sidebar">
    <!-- Logo -->
    <div class="sidebar-logo">
      <div class="logo-content">
        <div class="logo-icon" :class="{ collapsed: layoutStore.sidebarCollapsed }">
          <svg
            width="24"
            height="24"
            viewBox="0 0 32 32"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <rect width="32" height="32" rx="7" fill="url(#brandGradient)" />
            <rect x="8" y="6" width="12" height="16" rx="1" fill="#ffffff" />
            <path d="M18 6 L18 10 L22 10 Z" fill="#e6f7ff" />
            <rect x="10" y="10" width="6" height="1" fill="#52c41a" />
            <rect x="10" y="12" width="4" height="1" fill="#16a34a" />
            <rect x="10" y="14" width="5" height="1" fill="#722ed1" />
            <circle cx="11" cy="17" r="0.5" fill="#ff4d4f" />
            <circle cx="13" cy="17" r="0.5" fill="#ff4d4f" />
            <rect x="14.5" y="16.5" width="2" height="1" fill="#ff4d4f" />
            <path
              d="M22 20 L26 24 L22 28"
              stroke="#52c41a"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              fill="none"
            />
            <defs>
              <linearGradient id="brandGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style="stop-color: #16a34a; stop-opacity: 1" />
                <stop offset="100%" style="stop-color: #18a058; stop-opacity: 1" />
              </linearGradient>
            </defs>
          </svg>
        </div>
        <div v-show="!layoutStore.sidebarCollapsed" class="logo-text">
          Template <span class="brand-accent">Studio</span>
        </div>
      </div>
    </div>

    <!-- Navigation Menu -->
    <div class="sidebar-nav">
      <NavigationMenu />
    </div>

    <!-- Bottom Section -->
    <div class="sidebar-bottom">
      <!-- Settings Section -->
      <div class="settings-section">
        <!-- Help -->
        <div class="action-item">
          <a-button
            type="text"
            class="sidebar-action-button"
            @click="openHelp"
            :title="layoutStore.sidebarCollapsed ? 'Help' : ''"
          >
            <template #icon>
              <QuestionCircleOutlined />
            </template>
          </a-button>
        </div>

        <!-- Theme Toggle -->
        <div class="action-item">
          <a-button
            type="text"
            class="sidebar-action-button"
            @click="themeStore.toggleTheme()"
            :title="layoutStore.sidebarCollapsed ? 'Toggle Theme' : ''"
          >
            <template #icon>
              <StarOutlined v-if="themeStore.isDark" />
              <BulbOutlined v-else />
            </template>
          </a-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useLayoutStore } from '@/stores/layout'
import {
  QuestionCircleOutlined,
  StarOutlined,
  BulbOutlined
} from '@ant-design/icons-vue'
import NavigationMenu from './NavigationMenu.vue'
import { useThemeStore } from '@/stores/theme'

const router = useRouter()
const layoutStore = useLayoutStore()
const themeStore = useThemeStore()

// Computed properties
const isCollapsed = computed(() => layoutStore.sidebarCollapsed)

// Methods
const openHelp = () => {
  router.push('/help')
}

const openSettings = () => {
  router.push('/settings')
}
</script>

<style scoped>
.sidebar {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-sidebar);
}

/* Logo 区：小标 + 文字（单层，无动画） */
/* Logo 行：与顶栏严格等高的标题行（跨卡顶部视觉带），底部细分隔线 */
.sidebar-logo {
  height: 45px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  padding: 0 14px;
  box-sizing: border-box;
  border-bottom: 1px solid var(--color-border-light);
}

.logo-content {
  display: flex;
  align-items: center;
  gap: 10px;
}

.logo-icon {
  display: flex;
  align-items: center;
  flex: none;
}

.logo-icon.collapsed {
  margin: 0 auto;
}

.logo-text {
  font-size: 14px;
  font-weight: 650;
  letter-spacing: 0.2px;
  color: var(--color-text);
  white-space: nowrap;
}

.logo-text .brand-accent {
  color: var(--color-brand);
}

.sidebar-nav {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

/* 底部动作条：帮助 + 主题切换 */
.sidebar-bottom {
  flex-shrink: 0;
  padding: 8px 10px 10px;
  border-top: 1px solid var(--color-border-light);
  display: flex;
  gap: 6px;
}

.sidebar-bottom .action-item {
  flex: 1;
  min-width: 0;
}

.sidebar-bottom :deep(.sidebar-action-button) {
  width: 100%;
  height: 30px;
  border-radius: 7px;
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
}

.sidebar-bottom :deep(.sidebar-action-button:hover) {
  background: var(--color-hover) !important;
  color: var(--color-text) !important;
}
</style>