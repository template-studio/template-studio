<template>
  <div class="project-workspace-layout">
    <div
      class="sidebar"
      :data-collapsed="layoutStore.sidebarCollapsed"
    >
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

      <div class="sidebar-nav">
        <div v-if="!layoutStore.sidebarCollapsed" class="side-label">项目</div>
        <template v-for="item in navItems" :key="item.key">
          <a-tooltip v-if="layoutStore.sidebarCollapsed" :title="item.label" placement="right">
            <button class="nav-item collapsed-item" :class="{ active: activeKey === item.key }" @click="go(item.key)">
              <component :is="item.icon" class="nav-ic" />
            </button>
          </a-tooltip>
          <button v-else class="nav-item" :class="{ active: activeKey === item.key }" @click="go(item.key)">
            <component :is="item.icon" class="nav-ic" />
            <span class="nav-text">{{ item.label }}</span>
          </button>
        </template>
      </div>

      <div class="sidebar-bottom">
      <!-- 帮助 -->
      <a-tooltip v-if="layoutStore.sidebarCollapsed" title="帮助" placement="right">
        <button class="nav-item collapsed-item" @click="goToHelp">
          <QuestionCircleOutlined class="nav-ic" />
        </button>
      </a-tooltip>
      <button v-else class="nav-item" @click="goToHelp">
        <QuestionCircleOutlined class="nav-ic" />
        <span class="nav-text">帮助</span>
      </button>

      <!-- 设置 -->
      <a-tooltip v-if="layoutStore.sidebarCollapsed" title="设置" placement="right">
        <button class="nav-item collapsed-item" @click="goToSettings">
          <SettingOutlined class="nav-ic" />
        </button>
      </a-tooltip>
      <button v-else class="nav-item" @click="goToSettings">
        <SettingOutlined class="nav-ic" />
        <span class="nav-text">设置</span>
      </button>

      <!-- 主题切换 -->
      <a-tooltip v-if="layoutStore.sidebarCollapsed" :title="isDark ? '切换浅色' : '切换深色'" placement="right">
        <button class="nav-item collapsed-item" @click="toggleTheme">
          <StarOutlined v-if="isDark" class="nav-ic" />
          <BulbOutlined v-else class="nav-ic" />
        </button>
      </a-tooltip>
      <button v-else class="nav-item" @click="toggleTheme">
        <StarOutlined v-if="isDark" class="nav-ic" />
        <BulbOutlined v-else class="nav-ic" />
        <span class="nav-text">{{ isDark ? '切换浅色' : '切换深色' }}</span>
      </button>
    </div>
    </div>

    <div class="main-area">
      <WorkspaceHeader
        :sidebar-collapsed="layoutStore.sidebarCollapsed"
        :project-name="projectName"
        :database-type="databaseType"
        :table-count="tableCount"
        :current-page-title="currentPageTitle"
        @toggle-sidebar="layoutStore.toggleSidebar()"
        @go-back="goBack"
        @minimize="minimizeWindow"
        @maximize="maximizeWindow"
        @close="closeWindow"
      />

      <div class="content">
        <router-view />
      </div>

      <WorkspaceFooter />
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useLayoutStore } from '@/stores/layout'
import { useThemeStore } from '@/stores/theme'
import { invoke } from '@tauri-apps/api/core'
import { tauriApi } from '@/utils/tauriApi'
import * as projectsApi from '@/api/projects'
import {
  TableOutlined,
  QuestionCircleOutlined,
  SettingOutlined,
  StarOutlined,
  BulbOutlined,
  SwapOutlined,
  DashboardOutlined
} from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import WorkspaceHeader from './workspace/WorkspaceHeader.vue'
import WorkspaceFooter from './workspace/WorkspaceFooter.vue'

const router = useRouter()
const route = useRoute()
const layoutStore = useLayoutStore()
const themeStore = useThemeStore()

const projectId = computed(() => route.params.id || '')

const projectName = ref('加载中...')
const databaseType = ref('-')
const tableCount = ref(0)

const loadProjectInfo = async () => {
  if (!projectId.value) return

  try {
    const project = await projectsApi.getProject(projectId.value)

    projectName.value = project.name

    if (project.datasource_id) {
      const datasourceData = await invoke('db_get_datasource', { id: project.datasource_id })
      const datasource = JSON.parse(datasourceData)

      databaseType.value = datasource.type_ === 'postgresql' ? 'PostgreSQL' :
                         datasource.type_ === 'mysql' ? 'MySQL' :
                         datasource.type_ === 'sqlite' ? 'SQLite' : datasource.type_
    }

    tableCount.value = project.table_count || 0
  } catch (error) {
    console.error('加载项目信息失败:', error)
    message.error('加载项目信息失败')
    projectName.value = '未知项目'
  }
}

onMounted(() => {
  loadProjectInfo()
})

const isDark = computed(() => themeStore.isDark)

// 自绘导航：条目定义与激活态（替代 a-menu 的 selectedKeys）
const navItems = [
  { key: 'overview', label: '工作台', icon: DashboardOutlined },
  { key: 'tables', label: '表管理', icon: TableOutlined },
  { key: 'preferences', label: '规范管理', icon: SettingOutlined },
  { key: 'mappings', label: '映射管理', icon: SwapOutlined },
]

const activeKey = computed(() => {
  const base = `/project/${projectId.value}`
  const sub = route.path.slice(base.length).replace(/^\/+/, '')
  if (!sub) return 'overview'
  const first = sub.split('/')[0]
  return ['tables', 'preferences', 'mappings'].includes(first) ? first : 'tables'
})

const go = (key) => {
  if (key === 'overview') {
    router.push(`/project/${projectId.value}`)
  } else {
    router.push(`/project/${projectId.value}/${key}`)
  }
}

const currentPageTitle = computed(() => {
  const titleMap = {
    overview: '工作台',
    tables: '表管理',
    preferences: '规范管理',
    mappings: '映射管理'
  }
  return titleMap[activeKey.value] || ''
})

const goBack = () => {
  router.push('/projects')
}

const toggleTheme = () => {
  themeStore.toggleTheme()
}

const goToSettings = () => {
  router.push('/settings')
}

const goToHelp = () => {
  router.push('/help')
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
/* ============================================================
 * 项目工作区壳层：画布 + 悬浮面板（与主壳层 AppLayout 同语言）
 * ============================================================ */
.project-workspace-layout {
  width: 100%;
  height: 100vh;
  overflow: hidden;
  background: var(--color-canvas);
  display: flex;
  gap: 8px;
  padding: 8px 10px 10px;
  box-sizing: border-box;
}

/* ---------- 侧栏：悬浮卡片 ---------- */
.sidebar {
  flex: none;
  width: var(--sidebar-width-expanded);
  display: flex;
  flex-direction: column;
  background: var(--color-sidebar);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-panel);
  box-shadow: var(--shadow-panel);
  transition: width 0.2s ease, box-shadow 0.35s ease;
  animation: panelBreathe 8s ease-in-out infinite;
  overflow: hidden;
}

.sidebar:hover {
  animation: none;
  box-shadow: var(--shadow-panel-hover);
}

.sidebar[data-collapsed='true'] {
  width: 60px;
}

@keyframes panelBreathe {
  0%, 100% { box-shadow: var(--shadow-panel); }
  50%      { box-shadow: var(--shadow-panel-breathe); }
}

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

.logo-text :deep(.brand-accent),
.logo-text .brand-accent {
  color: var(--color-brand);
}

/* ---------- 自绘导航（与主侧栏同规格） ---------- */
.sidebar-nav {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 4px 10px 12px;
}

/* 小节标题（与主侧栏同规格）：给导航与 logo 带之间留出呼吸 */
.side-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-muted);
  padding: 14px 8px 4px;
  user-select: none;
}

.nav-item {
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
  font-weight: 450;
  text-align: left;
  cursor: pointer;
  /* 柔和无感：120ms 快速淡入淡出，不加位移/缩放 */
  transition: background-color 120ms ease, color 120ms ease;
}

/* 悬浮轻于选中（约 5% 灰），文字微微加深 */
.nav-item:hover {
  background: var(--color-nav-hover);
  color: var(--color-text);
}

/* 选中：低透明度底（约 9%）+ 文字加重表达位置，不与内容抢注意力 */
.nav-item.active {
  background: var(--color-nav-active);
  color: var(--color-text);
  font-weight: 550;
}

.nav-ic {
  font-size: 15px;
  color: var(--color-text-muted);
  flex: none;
  transition: color 120ms ease;
}

.nav-item:hover .nav-ic,
.nav-item.active .nav-ic {
  color: var(--color-text);
}

.nav-text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  /* 光学补偿：13px 中文字形重心在 em 盒内天然偏高，几何居中会看起来上浮，
   * 下移半像素修正（Linear/Notion 对小号 CJK 文本的常规做法） */
  position: relative;
  top: 1px;
}

.collapsed-item {
  justify-content: center;
  padding: 0;
}

/* ---------- 底部动作条：纵向导航同款条目 ---------- */
.sidebar-bottom {
  flex-shrink: 0;
  padding: 6px 10px 10px;
  border-top: 1px solid var(--color-border-light);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

/* ---------- 主区：顶栏卡 + 内容卡 ---------- */
.main-area {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.main-area :deep(.header) {
  flex: none;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-panel);
  box-shadow: var(--shadow-panel);
}

.content {
  flex: 1;
  min-height: 0;
  overflow: auto;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-panel);
  box-shadow: var(--shadow-panel);
  animation: panelBreathe 8s ease-in-out infinite;
}

.content:hover {
  animation: none;
  box-shadow: var(--shadow-panel-hover);
}

/* 底部信息条跟随画布（透明浮在画布上） */
.main-area :deep(.workspace-footer) {
  background: transparent;
}
</style>