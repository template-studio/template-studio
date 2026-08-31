<template>
  <div class="sidebar">
    <!-- Logo -->
    <div class="sidebar-logo">
      <div class="logo-content">
        <div class="logo-icon" :class="{ collapsed: layoutStore.sidebarCollapsed }">
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <rect width="24" height="24" rx="6.5" fill="url(#brandGradient)" />
            <path
              d="M9.2 8.4 L6 12 L9.2 15.6"
              stroke="#ffffff"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              fill="none"
            />
            <path
              d="M13.8 7.2 L10.8 16.8"
              stroke="#ffffff"
              stroke-width="2"
              stroke-linecap="round"
              fill="none"
            />
            <path
              d="M14.8 8.4 L18 12 L14.8 15.6"
              stroke="#ffffff"
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
      <!-- 帮助 -->
      <a-tooltip v-if="layoutStore.sidebarCollapsed" title="帮助" placement="right">
        <button class="bottom-item collapsed-item" @click="openHelp">
          <QuestionCircleOutlined class="bottom-ic" />
        </button>
      </a-tooltip>
      <button v-else class="bottom-item" @click="openHelp">
        <QuestionCircleOutlined class="bottom-ic" />
        <span class="bottom-text">帮助</span>
      </button>

      <!-- 主题切换 -->
      <a-tooltip v-if="layoutStore.sidebarCollapsed" :title="themeStore.isDark ? '切换浅色' : '切换深色'" placement="right">
        <button class="bottom-item collapsed-item" @click="themeStore.toggleTheme()">
          <StarOutlined v-if="themeStore.isDark" class="bottom-ic" />
          <BulbOutlined v-else class="bottom-ic" />
        </button>
      </a-tooltip>
      <button v-else class="bottom-item" @click="themeStore.toggleTheme()">
        <StarOutlined v-if="themeStore.isDark" class="bottom-ic" />
        <BulbOutlined v-else class="bottom-ic" />
        <span class="bottom-text">{{ themeStore.isDark ? '切换浅色' : '切换深色' }}</span>
      </button>

      <!-- 登录态身份锚点（配置 API Token 后显示） -->
      <SidebarUserCard v-if="configStore.hasApiKey" />
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useLayoutStore } from '@/stores/layout'
import { useConfigStore } from '@/stores/config'
import {
  QuestionCircleOutlined,
  StarOutlined,
  BulbOutlined
} from '@ant-design/icons-vue'
import NavigationMenu from './NavigationMenu.vue'
import SidebarUserCard from './SidebarUserCard.vue'
import { useThemeStore } from '@/stores/theme'

const router = useRouter()
const layoutStore = useLayoutStore()
const configStore = useConfigStore()
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

/* 底部动作条：与导航条目同款规格（图标+文字，折叠态图标） */
.sidebar-bottom {
  flex-shrink: 0;
  padding: 6px 10px 10px;
  border-top: 1px solid var(--color-border-light);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.bottom-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  height: 30px;
  padding: 0 8px;
  border: none;
  border-radius: 7px;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 13px;
  text-align: left;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease;
}

.bottom-item:hover {
  background: var(--color-nav-hover);
  color: var(--color-text);
}

.bottom-ic {
  font-size: 15px;
  color: var(--color-text-muted);
  flex: none;
}

.bottom-item:hover .bottom-ic {
  color: var(--color-text);
}

.bottom-text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  position: relative;
  top: 0.5px;
}

.collapsed-item {
  justify-content: center;
  padding: 0;
}
</style>