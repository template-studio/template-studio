<template>
  <div class="sidebar">
    <!-- Logo Section -->
    <div class="sidebar-logo">
      <div class="logo-content">
        <div class="logo-icon" :class="{ collapsed: layoutStore.sidebarCollapsed }">
          <svg
            width="32"
            height="32"
            viewBox="0 0 32 32"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <rect width="32" height="32" rx="6" fill="url(#brandGradient)" />
            <rect x="8" y="6" width="12" height="16" rx="1" fill="#ffffff" />
            <path d="M18 6 L18 10 L22 10 Z" fill="#e6f7ff" />
            <rect x="10" y="10" width="6" height="1" fill="#52c41a" />
            <rect x="10" y="12" width="4" height="1" fill="#1890ff" />
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
                <stop offset="0%" style="stop-color: #1890ff; stop-opacity: 1" />
                <stop offset="100%" style="stop-color: #18a058; stop-opacity: 1" />
              </linearGradient>
            </defs>
          </svg>
        </div>
        <transition name="fade">
          <div v-show="!layoutStore.sidebarCollapsed" class="logo-text">
            <span class="logo-main">CodeGen <span class="brand-accent">Studio</span></span>
            <span class="logo-shadow">CodeGen <span class="brand-accent">Studio</span></span>
          </div>
        </transition>
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

        <!-- Settings -->
        <div class="action-item">
          <a-button
            type="text"
            class="sidebar-action-button"
            @click="openSettings"
            :title="layoutStore.sidebarCollapsed ? 'Settings' : ''"
          >
            <template #icon>
              <SettingOutlined />
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
  SettingOutlined,
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

.sidebar-logo {
  padding: var(--spacing-md);
  flex-shrink: 0;
}

.logo-content {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.logo-icon {
  margin-right: 12px;
  display: flex;
  align-items: center;
  transition: all 0.3s ease;
}

.logo-icon.collapsed {
  margin-right: 0;
}

.logo-icon svg {
  transition: all 0.3s ease;
}

.logo-content:hover .logo-icon svg {
  transform: scale(1.1) rotate(5deg);
}

.logo-text {
  position: relative;
  display: flex;
  align-items: center;
}

.logo-main {
  font-size: 1.1rem;
  font-weight: 800;
  letter-spacing: 1px;
  color: #333;
  font-family: 'Fira Code', 'Lato', 'Segoe UI', 'Arial', sans-serif;
  background: linear-gradient(90deg, #18a058 0%, #2196f3 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  position: relative;
  z-index: 2;
  animation: float 3s ease-in-out infinite;
  transition: all 0.3s ease;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  white-space: nowrap;
}

.logo-shadow {
  font-size: 1.1rem;
  font-weight: 800;
  letter-spacing: 1px;
  color: rgba(51, 51, 51, 0.2);
  font-family: 'Fira Code', 'Lato', 'Segoe UI', 'Arial', sans-serif;
  position: absolute;
  top: 2px;
  left: 0;
  right: 0;
  z-index: 1;
  animation: float-shadow 3s ease-in-out infinite;
  transition: all 0.3s ease;
  filter: blur(1px);
  white-space: nowrap;
}

.brand-accent {
  color: #18a058;
  -webkit-text-fill-color: #18a058;
  background: none;
  font-weight: 900;
}

@keyframes float {
  0%,
  100% {
    transform: translateY(0px);
  }

  50% {
    transform: translateY(-3px);
  }
}

@keyframes float-shadow {
  0%,
  100% {
    transform: translateY(2px);
    opacity: 0.2;
  }

  50% {
    transform: translateY(5px);
    opacity: 0.3;
  }
}

.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-sm) 12px;
}

.sidebar-bottom {
  padding: var(--spacing-md);
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

/* Settings Section */
.settings-section {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-sm);
  flex-wrap: wrap;
}

/* Divider */
.sidebar-divider {
  height: 1px;
  background: var(--color-border);
  margin: var(--spacing-xs) 0;
}

.action-item {
  display: flex;
  align-items: center;
}

.sidebar-action-button {
  color: var(--color-text-secondary);
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  outline: none; /* 移除默认焦点轮廓 */
}

.sidebar-action-button:hover {
  color: var(--color-primary);
  background: var(--color-hover);
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity var(--transition-fast);
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* Scrollbar styling */
.sidebar-nav::-webkit-scrollbar {
  width: 4px;
}

.sidebar-nav::-webkit-scrollbar-track {
  background: transparent;
}

.sidebar-nav::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 2px;
}

.sidebar-nav::-webkit-scrollbar-thumb:hover {
  background: var(--color-hover);
}

/* Dark theme adjustments now handled by global CSS variables */

/* Responsive behavior */
@media (max-width: 768px) {
  .sidebar {
    position: relative;
  }

  .settings-section {
    justify-content: center;
    flex-wrap: wrap;
  }

  .action-item {
    flex: 1;
    justify-content: center;
  }
}

/* Collapsed state adjustments */
.sidebar[data-collapsed="true"] .settings-section {
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
}

.sidebar[data-collapsed="true"] .sidebar-divider {
  width: 80%;
  margin: var(--spacing-xs) auto;
}
</style>