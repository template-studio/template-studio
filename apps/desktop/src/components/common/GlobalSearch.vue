<template>
  <Teleport to="body">
    <Transition name="search">
      <div v-if="visible" class="global-search-overlay" @mousedown.self="close">
        <div class="global-search-modal" @keydown="handleKeydown">
          <!-- 搜索头部 -->
          <div class="search-header">
            <SearchOutlined class="search-icon" />
            <input
              ref="searchInputRef"
              v-model="searchQuery"
              class="search-input"
              placeholder="搜索页面、项目、数据源、模板..."
              spellcheck="false"
              autocomplete="off"
            />
            <kbd class="esc-hint">ESC</kbd>
          </div>

          <!-- 搜索结果 -->
          <div class="search-body" ref="searchBodyRef">
            <template v-if="hasResults">
              <div
                v-for="category in filteredCategories"
                :key="category.key"
                class="category-group"
              >
                <div class="category-header">
                  <component :is="category.icon" />
                  <span>{{ category.label }}</span>
                  <span class="category-count">{{ category.items.length }}</span>
                </div>
                <div
                  v-for="item in category.items"
                  :key="item.id"
                  class="result-item"
                  :class="{ active: getGlobalIndex(item) === activeIndex }"
                  @click="selectItem(item)"
                  @mouseenter="activeIndex = getGlobalIndex(item)"
                >
                  <component :is="item.icon" class="item-icon" />
                  <div class="item-info">
                    <span class="item-name">{{ item.name }}</span>
                    <span v-if="item.description" class="item-desc">{{ item.description }}</span>
                  </div>
                  <a-tag v-if="item.tag" size="small" color="default">{{ item.tag }}</a-tag>
                </div>
              </div>
            </template>
            <div v-else class="empty-state">
              <SearchOutlined />
              <span>没有找到匹配的结果</span>
            </div>
          </div>

          <!-- 底部提示 -->
          <div class="search-footer">
            <span><kbd>↑</kbd><kbd>↓</kbd> 导航</span>
            <span><kbd>↵</kbd> 打开</span>
            <span><kbd>ESC</kbd> 关闭</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, computed, nextTick, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  SearchOutlined, HomeOutlined, FileTextOutlined, DatabaseOutlined,
  FolderOutlined, CodeOutlined, SwapOutlined, AppstoreOutlined,
  SettingOutlined, QuestionCircleOutlined
} from '@ant-design/icons-vue'
import { getAllProjects } from '@/api/projects'
import { getAllDatasources } from '@/api/datasources'
import { getTemplates } from '@/api/templates'

const router = useRouter()

// --- 状态 ---
const visible = ref(false)
const searchQuery = ref('')
const searchInputRef = ref(null)
const searchBodyRef = ref(null)
const activeIndex = ref(0)

// --- 数据 ---
const projects = ref([])
const datasources = ref([])
const templates = ref([])

// --- 静态页面列表 ---
const pages = [
  { id: 'page-home', name: '首页', description: '应用首页', path: '/home', icon: HomeOutlined, tag: '页面' },
  { id: 'page-templates', name: '脚手架', description: '模板管理', path: '/templates', icon: FileTextOutlined, tag: '页面' },
  { id: 'page-languages', name: '语言管理', description: '编程语言配置', path: '/languages', icon: CodeOutlined, tag: '页面' },
  { id: 'page-datasource', name: '数据源', description: '数据库连接管理', path: '/datasource', icon: DatabaseOutlined, tag: '页面' },
  { id: 'page-projects', name: '项目', description: '项目列表', path: '/projects', icon: FolderOutlined, tag: '页面' },
  { id: 'page-mappings', name: '映射管理', description: '全局类型映射', path: '/mappings', icon: SwapOutlined, tag: '页面' },
  { id: 'page-render', name: '模板渲染', description: '渲染模板生成代码', path: '/template-render', icon: AppstoreOutlined, tag: '页面' },
  { id: 'page-settings', name: '设置', description: '应用设置', path: '/settings', icon: SettingOutlined, tag: '页面' },
  { id: 'page-help', name: '帮助', description: '帮助中心', path: '/help', icon: QuestionCircleOutlined, tag: '页面' }
]

// --- 分类结果 ---
const filteredCategories = computed(() => {
  const q = searchQuery.value.toLowerCase().trim()
  const limit = q ? 999 : 5

  const cats = []

  // 页面
  const filteredPages = pages.filter(p => matchesQuery(p.name + p.description, q))
  if (filteredPages.length) {
    cats.push({ key: 'pages', label: '页面', icon: HomeOutlined, items: filteredPages.slice(0, limit) })
  }

  // 项目
  const filteredProjects = projects.value.filter(p => matchesQuery(p.name + (p.description || ''), q))
  if (filteredProjects.length) {
    cats.push({ key: 'projects', label: '项目', icon: FolderOutlined, items: filteredProjects.slice(0, limit) })
  }

  // 数据源
  const filteredDatasources = datasources.value.filter(d => matchesQuery(d.name + (d.type_ || d.type || ''), q))
  if (filteredDatasources.length) {
    cats.push({ key: 'datasources', label: '数据源', icon: DatabaseOutlined, items: filteredDatasources.slice(0, limit) })
  }

  // 模板
  const filteredTemplates = templates.value.filter(t => matchesQuery(t.name + (t.description || ''), q))
  if (filteredTemplates.length) {
    cats.push({ key: 'templates', label: '模板', icon: FileTextOutlined, items: filteredTemplates.slice(0, limit) })
  }

  return cats
})

const flatResults = computed(() => {
  return filteredCategories.value.flatMap(c => c.items)
})

const hasResults = computed(() => flatResults.value.length > 0)

// --- 匹配函数 ---
function matchesQuery(text, query) {
  if (!query) return true
  return (text || '').toLowerCase().includes(query)
}

// --- 全局索引计算 ---
function getGlobalIndex(item) {
  return flatResults.value.findIndex(r => r.id === item.id)
}

// --- 数据加载 ---
async function loadLocalData() {
  const [proj, ds] = await Promise.all([
    getAllProjects().catch(() => []),
    getAllDatasources().catch(() => [])
  ])
  projects.value = (proj || []).map(p => ({
    id: 'proj-' + p.id,
    name: p.name,
    description: p.description || '',
    path: '/project/' + p.id,
    icon: FolderOutlined,
    tag: '项目'
  }))
  datasources.value = (ds || []).map(d => ({
    id: 'ds-' + d.id,
    name: d.name,
    description: (d.type_ || d.type || '').toUpperCase(),
    path: '/datasource',
    icon: DatabaseOutlined,
    tag: '数据源'
  }))
}

async function loadTemplates() {
  if (templates.value.length > 0) return
  try {
    const res = await getTemplates({})
    const list = res?.data?.templatesList || res?.templatesList || []
    templates.value = list.map(t => ({
      id: 'tpl-' + t.id,
      name: t.name,
      description: t.description || '',
      path: '/templates',
      icon: FileTextOutlined,
      tag: '模板'
    }))
  } catch (e) {
    // 模板加载失败不影响其他搜索
  }
}

// --- 打开/关闭 ---
function open() {
  visible.value = true
  searchQuery.value = ''
  activeIndex.value = 0
  nextTick(() => searchInputRef.value?.focus())
  loadLocalData()
  loadTemplates()
}

function close() {
  visible.value = false
  searchQuery.value = ''
  activeIndex.value = 0
}

// --- 键盘事件 ---
function handleKeydown(e) {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (flatResults.value.length) {
      activeIndex.value = (activeIndex.value + 1) % flatResults.value.length
      scrollToActive()
    }
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (flatResults.value.length) {
      activeIndex.value = (activeIndex.value - 1 + flatResults.value.length) % flatResults.value.length
      scrollToActive()
    }
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const item = flatResults.value[activeIndex.value]
    if (item) selectItem(item)
  } else if (e.key === 'Escape') {
    e.preventDefault()
    close()
  }
}

function scrollToActive() {
  nextTick(() => {
    const el = searchBodyRef.value?.querySelector('.result-item.active')
    el?.scrollIntoView({ block: 'nearest' })
  })
}

// --- 选择跳转 ---
function selectItem(item) {
  close()
  router.push(item.path)
}

// --- 搜索词变化时重置索引 ---
watch(searchQuery, () => {
  activeIndex.value = 0
})

defineExpose({ open, close })
</script>

<style scoped>
.global-search-overlay {
  position: fixed;
  inset: 0;
  z-index: 1070;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  justify-content: center;
  padding-top: 18vh;
}

[data-theme="dark"] .global-search-overlay {
  background: rgba(0, 0, 0, 0.6);
}

.global-search-modal {
  width: 100%;
  max-width: 640px;
  max-height: 480px;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 搜索头部 */
.search-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
}

.search-icon {
  font-size: 16px;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-size: 15px;
  color: var(--color-text);
  line-height: 1.5;
}

.search-input::placeholder {
  color: var(--color-text-muted);
}

.esc-hint {
  display: inline-block;
  padding: 2px 6px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  font-size: 11px;
  font-family: 'Consolas', 'Monaco', monospace;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

/* 搜索结果 */
.search-body {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
}

.category-group {
  margin-bottom: 2px;
}

.category-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 16px 3px;
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.category-count {
  font-size: 10px;
  color: var(--color-text-muted);
  font-weight: 400;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;
  cursor: pointer;
  transition: background var(--transition-fast);
  border-left: 3px solid transparent;
}

.result-item:hover,
.result-item.active {
  background: var(--color-hover);
}

.result-item.active {
  border-left-color: var(--color-primary);
}

.item-icon {
  font-size: 15px;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.item-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.item-name {
  font-size: 13px;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-desc {
  font-size: 11px;
  color: var(--color-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 40px 16px;
  color: var(--color-text-muted);
  font-size: 13px;
}

.empty-state .anticon {
  font-size: 24px;
}

/* 底部提示 */
.search-footer {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 16px;
  border-top: 1px solid var(--color-border);
  font-size: 11px;
  color: var(--color-text-muted);
}

.search-footer kbd {
  display: inline-block;
  padding: 1px 5px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 3px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 10px;
  margin-right: 3px;
}

/* 动画 */
.search-enter-active,
.search-leave-active {
  transition: opacity 150ms ease;
}

.search-enter-active .global-search-modal,
.search-leave-active .global-search-modal {
  transition: transform 150ms ease, opacity 150ms ease;
}

.search-enter-from,
.search-leave-to {
  opacity: 0;
}

.search-enter-from .global-search-modal,
.search-leave-to .global-search-modal {
  transform: translateY(-8px);
  opacity: 0;
}
</style>
