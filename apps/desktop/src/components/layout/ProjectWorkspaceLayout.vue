<template>
  <div class="project-workspace-layout">
    <!-- 左侧菜单栏 -->
    <div
      class="sidebar"
      :data-collapsed="layoutStore.sidebarCollapsed"
    >
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
              <span class="logo-main">Template <span class="brand-accent">Studio</span></span>
              <span class="logo-shadow">Template <span class="brand-accent">Studio</span></span>
            </div>
          </transition>
        </div>
      </div>

      <!-- 工作区导航菜单 -->
      <div class="sidebar-nav">
        <a-menu
          v-model:selectedKeys="selectedKeys"
          mode="inline"
          class="navigation-menu"
          @click="handleMenuClick"
        >
          <a-menu-item key="tables">
            <template #icon>
              <TableOutlined />
            </template>
            <span>表管理</span>
          </a-menu-item>
          <a-menu-item key="preferences">
            <template #icon>
              <SettingOutlined />
            </template>
            <span>规范管理</span>
          </a-menu-item>
          <a-menu-item key="mappings">
            <template #icon>
              <SwapOutlined />
            </template>
            <span>映射管理</span>
          </a-menu-item>
        </a-menu>
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
              @click="goToHelp"
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
              @click="goToSettings"
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
              @click="toggleTheme"
              :title="layoutStore.sidebarCollapsed ? 'Toggle Theme' : ''"
            >
              <template #icon>
                <StarOutlined v-if="isDark" />
                <BulbOutlined v-else />
              </template>
            </a-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 主内容区 -->
    <a-layout :style="{ marginLeft: layoutStore.sidebarCollapsed ? '60px' : '240px', minHeight: '100vh' }">
      <!-- 顶部标题栏 -->
      <a-layout-header class="header titlebar-drag-region">
        <div class="header-left">
          <!-- 侧边栏收缩按钮 -->
          <a-button
            type="text"
            class="sidebar-toggle titlebar-no-drag"
            @click="layoutStore.toggleSidebar()"
          >
            <template #icon>
              <MenuFoldOutlined v-if="!layoutStore.sidebarCollapsed" />
              <MenuUnfoldOutlined v-else />
            </template>
          </a-button>
          <a-breadcrumb>
            <a-breadcrumb-item>
              <a @click.prevent="goBack" class="titlebar-no-drag">项目列表</a>
            </a-breadcrumb-item>
            <a-breadcrumb-item>{{ projectName }}</a-breadcrumb-item>
            <a-breadcrumb-item>{{ currentPageTitle }}</a-breadcrumb-item>
          </a-breadcrumb>
        </div>
        <div class="header-right">
          <!-- 项目信息展示 -->
          <div class="project-info-display">
            <FolderOutlined class="info-icon project-icon" />
            <span class="project-name-text">{{ projectName }}</span>
            <a-divider type="vertical" />
            <DatabaseOutlined class="info-icon" />
            <span class="info-text">{{ databaseType }}</span>
            <a-divider type="vertical" />
            <TableOutlined class="info-icon" />
            <span class="info-text">{{ tableCount }} 张表</span>
          </div>

          <!-- 返回按钮 -->
          <a-button
            type="text"
            size="small"
            class="window-control back-btn"
            @click="goBack"
          >
            <template #icon>
              <ArrowLeftOutlined />
            </template>
          </a-button>

          <!-- 窗口控制按钮 -->
          <div class="window-controls titlebar-no-drag">
            <a-button
              type="text"
              size="small"
              class="window-control"
              @click="minimizeWindow"
            >
              <template #icon>
                <MinusOutlined />
              </template>
            </a-button>
            <a-button
              type="text"
              size="small"
              class="window-control"
              @click="maximizeWindow"
            >
              <template #icon>
                <BorderOutlined />
              </template>
            </a-button>
            <a-button
              type="text"
              size="small"
              class="window-control close"
              @click="closeWindow"
            >
              <template #icon>
                <CloseOutlined />
              </template>
            </a-button>
          </div>
        </div>
      </a-layout-header>

      <!-- 内容区 -->
      <a-layout-content class="content">
        <router-view />
      </a-layout-content>
    </a-layout>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useLayoutStore } from '@/stores/layout'
import { useThemeStore } from '@/stores/theme'
import { invoke } from '@tauri-apps/api/core'
import { tauriApi } from '@/utils/tauriApi'
import * as projectsApi from '@/api/projects'
import {
  TableOutlined,
  DatabaseOutlined,
  FolderOutlined,
  ArrowLeftOutlined,
  PlaySquareOutlined,
  MinusOutlined,
  BorderOutlined,
  CloseOutlined,
  MenuFoldOutlined,
  MenuUnfoldOutlined,
  QuestionCircleOutlined,
  SettingOutlined,
  StarOutlined,
  BulbOutlined,
  SwapOutlined
} from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'

const router = useRouter()
const route = useRoute()
const layoutStore = useLayoutStore()
const themeStore = useThemeStore()

// 从路由参数获取项目ID
const projectId = computed(() => route.params.id || '')

// 项目信息
const projectName = ref('加载中...')
const databaseType = ref('-')
const tableCount = ref(0)

// 加载项目信息
const loadProjectInfo = async () => {
  if (!projectId.value) return

  try {
    const project = await projectsApi.getProject(projectId.value)

    // 设置项目名称
    projectName.value = project.name

    // 加载数据源信息
    if (project.datasource_id) {
      const datasourceData = await invoke('db_get_datasource', { id: project.datasource_id })
      const datasource = JSON.parse(datasourceData)

      // 设置数据库类型
      databaseType.value = datasource.type_ === 'postgresql' ? 'PostgreSQL' :
                         datasource.type_ === 'mysql' ? 'MySQL' :
                         datasource.type_ === 'sqlite' ? 'SQLite' : datasource.type_
    }

    // 设置表数量
    tableCount.value = project.table_count || 0
  } catch (error) {
    console.error('加载项目信息失败:', error)
    message.error('加载项目信息失败')
    projectName.value = '未知项目'
  }
}

// 组件挂载时加载项目信息
onMounted(() => {
  loadProjectInfo()
})

// 主题状态
const isDark = computed(() => themeStore.isDark)

// 当前选中的菜单
const selectedKeys = ref(['tables'])

// 当前页面标题
const currentPageTitle = computed(() => {
  const titleMap = {
    tables: '表管理',
    preferences: '规范管理',
    mappings: '映射管理'
  }
  return titleMap[selectedKeys.value[0]] || ''
})

// 监听路由变化，更新菜单选中状态
watch(
  () => route.path,
  (newPath) => {
    // 提取当前的工作区子路由
    const match = newPath.match(/\/project\/[^/]+\/(.+)/)
    if (match) {
      const subRoute = match[1]
      const pathToKey = {
        'tables': 'tables',
        'preferences': 'preferences',
        'mappings': 'mappings'
      }
      selectedKeys.value = [pathToKey[subRoute] || 'tables']
    }
  },
  { immediate: true }
)

// 菜单点击处理
const handleMenuClick = ({ key }) => {
  router.push(`/project/${projectId.value}/${key}`)
}

// 返回项目列表
const goBack = () => {
  router.push('/projects')
}

// 主题切换
const toggleTheme = () => {
  themeStore.toggleTheme()
}

// 跳转到设置
const goToSettings = () => {
  router.push('/settings')
}

// 跳转到帮助
const goToHelp = () => {
  router.push('/help')
}

// 窗口控制
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
.project-workspace-layout {
  width: 100%;
  min-height: 100vh;
}

/* 侧边栏 */
.sidebar {
  height: 100vh;
  width: 240px;
  display: flex;
  flex-direction: column;
  background: var(--color-sidebar);
  position: fixed;
  left: 0;
  top: 0;
  bottom: 0;
  overflow: hidden;
  transition: width 0.2s;
  border-right: 1px solid var(--color-border);
  box-shadow: 1px 0 4px rgba(0, 0, 0, 0.1);
}

.sidebar[data-collapsed="true"] {
  width: 60px;
}

/* Logo 区域 */
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
  color: var(--color-text);
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
  color: var(--color-text-muted);
  opacity: 0.3;
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

/* 导航菜单区域 */
.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-sm) 12px;
}

/* 导航菜单样式 */
.navigation-menu {
  border: none !important;
  border-right: none !important;
  background: transparent;
}

.navigation-menu :deep(.ant-menu-inline),
.navigation-menu :deep(.ant-menu-root) {
  border-right: none !important;
}

.navigation-menu :deep(.ant-menu-item) {
  margin: 2px 0;
  border-radius: var(--border-radius-md);
  color: var(--color-text);
  border: 0.5px solid transparent;
  transition: background-color var(--transition-fast) ease;
  position: relative;
}

.navigation-menu :deep(.ant-menu-item) {
  transition: background-color var(--transition-fast) ease, color var(--transition-fast) ease;
}

.navigation-menu :deep(.ant-menu-item::after) {
  transition: none !important;
}

.navigation-menu :deep(.ant-menu-item-selected::after) {
  transition: none !important;
}

/* 悬浮状态 */
.navigation-menu :deep(.ant-menu-item:hover),
.navigation-menu :deep(.ant-menu-item.ant-menu-item:hover),
.navigation-menu :deep(.ant-menu-inline .ant-menu-item:hover) {
  background: var(--color-surface) !important;
  color: var(--color-primary) !important;
  border-color: transparent !important;
}

/* 选中状态 */
.navigation-menu :deep(.ant-menu-item-selected),
.navigation-menu :deep(.ant-menu-item.ant-menu-item-selected),
.navigation-menu :deep(.ant-menu-inline .ant-menu-item-selected) {
  background: var(--color-surface) !important;
  color: var(--color-primary) !important;
  border-color: var(--color-border) !important;
}

/* 选中状态再悬浮 */
.navigation-menu :deep(.ant-menu-item-selected:hover),
.navigation-menu :deep(.ant-menu-item.ant-menu-item-selected:hover),
.navigation-menu :deep(.ant-menu-inline .ant-menu-item-selected:hover) {
  background: var(--color-surface) !important;
  color: var(--color-primary) !important;
  border-color: var(--color-border) !important;
}

/* 强制覆盖Ant Design的悬浮文字颜色样式 */
.navigation-menu :deep(.ant-menu-item:hover .ant-menu-title-content),
.navigation-menu :deep(.ant-menu-item:hover .anticon),
.navigation-menu :deep(.ant-menu-item:hover span) {
  color: var(--color-primary) !important;
}

/* 选中状态下的文字颜色也要强制覆盖 */
.navigation-menu :deep(.ant-menu-item-selected .ant-menu-title-content),
.navigation-menu :deep(.ant-menu-item-selected .anticon),
.navigation-menu :deep(.ant-menu-item-selected span) {
  color: var(--color-primary) !important;
}

/* 选中指示器样式 */
.navigation-menu :deep(.ant-menu-item-selected::after) {
  display: none !important;
}

.navigation-menu :deep(.ant-menu-item-icon) {
  font-size: 16px;
}

/* Collapsed state adjustments - 完全复制 a-button type="text" 的样式 */
.navigation-menu :deep(.ant-menu-inline-collapsed) {
  .ant-menu-item {
    padding: 0 !important;
    padding-inline-start: 0 !important;
    padding-inline-end: 0 !important;
    margin: 2px auto !important;
    width: 32px !important;
    max-width: calc(100% - 16px) !important;
    height: 32px !important;
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
    border-radius: 6px !important;
    border: none !important;
    background-color: transparent !important;
    color: var(--color-text-secondary) !important;
    line-height: 1.5714285714285714 !important;
    font-size: 14px !important;
    transition: all 0.2s cubic-bezier(0.645, 0.045, 0.355, 1) !important;
    cursor: pointer !important;
  }

  /* 悬浮状态 */
  .ant-menu-item:hover {
    background: var(--color-hover) !important;
    color: var(--color-primary) !important;
  }

  /* 选中状态 */
  .ant-menu-item-selected,
  .ant-menu-item-selected:hover {
    background: var(--color-hover) !important;
    color: var(--color-primary) !important;
    box-shadow: 0 0 0 1px var(--color-border) inset !important;
  }

  /* 图标样式 */
  .ant-menu-item-icon {
    font-size: 16px !important;
    margin: 0 !important;
    display: inline-flex !important;
    align-items: center !important;
    justify-content: center !important;
  }

  .ant-menu-title-content {
    display: none !important;
  }

  /* 隐藏所有指示器 */
  .ant-menu-item-selected::after,
  .ant-menu-item::before,
  .ant-menu-item::after {
    display: none !important;
  }
}

/* 底部区域 */
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
  outline: none;
}

.sidebar-action-button:hover {
  color: var(--color-primary);
  background: var(--color-hover);
}

/* 收缩状态下的底部区域 */
.sidebar[data-collapsed="true"] .sidebar-bottom {
  padding: var(--spacing-md) 0;
}

.sidebar[data-collapsed="true"] .settings-section {
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
}

/* 滚动条样式 */
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

/* Collapsed state adjustments */
.sidebar[data-collapsed="true"] .sidebar-nav {
  padding: var(--spacing-sm) 0;
}

.header {
  background: var(--color-navbar);
  border-bottom: 1px solid var(--color-border);
  padding: 0 var(--spacing-lg);
  display: flex;
  align-items: center;
  justify-content: space-between;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
  height: var(--navbar-height);
  line-height: var(--navbar-height);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.sidebar-toggle {
  width: 32px;
  height: 32px;
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

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

/* 项目信息展示 */
.project-info-display {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 16px;
  background: transparent;
  height: 32px;
  line-height: 20px;
}

/* 返回按钮 */
.header-right :deep(.ant-btn) {
  margin: 0;
  cursor: pointer !important;
}

.header-right :deep(.ant-btn:hover) {
  cursor: pointer !important;
}

.header-right :deep(.ant-btn .anticon) {
  cursor: pointer !important;
}

.project-icon {
  font-size: 16px;
  color: var(--color-primary);
}

.project-name-text {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  line-height: 20px;
}

.info-icon {
  font-size: 14px;
  color: var(--color-text-secondary);
}

.info-text {
  font-size: 13px;
  color: var(--color-text);
  font-weight: 500;
  line-height: 20px;
}

/* 窗口拖动区域 */
.titlebar-drag-region {
  -webkit-app-region: drag;
  user-select: none;
}

/* 排除拖动的交互元素 */
.titlebar-no-drag {
  -webkit-app-region: no-drag;
}

/* 窗口控制按钮 */
.window-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 12px;
}

.window-control {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: all 0.2s;
}

.window-control:hover {
  color: var(--color-primary);
  background: var(--color-hover);
}

.window-control.close:hover {
  background: #ff4757;
  color: white;
}

/* 内容区 */
.content {
  background: var(--color-background);
  padding: var(--spacing-lg);
  min-height: calc(100vh - var(--navbar-height));
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
</style>
