<template>
  <nav class="nav">
    <div class="nav-scroll">
      <template v-for="section in sections" :key="section.label">
        <!-- 折叠态不显示小节标题，只显示条目 -->
        <div v-if="!collapsed && section.items.length" class="side-label">{{ section.label }}</div>
        <template v-for="item in section.items" :key="item.path">
          <a-tooltip v-if="collapsed" :title="item.label" placement="right">
            <button
              class="nav-item collapsed-item"
              :class="{ active: isActive(item) }"
              @click="go(item.path)"
            >
              <component :is="item.icon" class="nav-ic" />
            </button>
          </a-tooltip>
          <button
            v-else
            class="nav-item"
            :class="{ active: isActive(item) }"
            @click="go(item.path)"
          >
            <component :is="item.icon" class="nav-ic" />
            <span class="nav-text">{{ item.label }}</span>
          </button>
        </template>
      </template>
    </div>
  </nav>
</template>

<script setup>
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useConfigStore } from '@/stores/config'
import { useLayoutStore } from '@/stores/layout'
import {
  HomeOutlined,
  FileTextOutlined,
  FolderOpenOutlined,
  CodeOutlined,
  DatabaseOutlined,
  FolderOutlined,
  SwapOutlined,
  AppstoreOutlined,
  SettingOutlined,
} from '@ant-design/icons-vue'

const router = useRouter()
const route = useRoute()
const configStore = useConfigStore()
const layoutStore = useLayoutStore()

const collapsed = computed(() => layoutStore.sidebarCollapsed)

// 信息架构：小节分组 + 平铺条目（不折叠——菜单总量少，展开永远可见胜过省一次点击）
const sections = computed(() => [
  {
    label: '工作台',
    items: [
      { path: '/home', label: '首页', icon: HomeOutlined },
      { path: '/templates', label: '脚手架', icon: FileTextOutlined },
      // 登录态专属：配置了 API Token 才显示
      ...(configStore.hasApiKey
        ? [{ path: '/my-templates', label: '我的模板', icon: FolderOpenOutlined }]
        : []),
    ],
  },
  {
    label: '代码生成',
    items: [
      { path: '/languages', label: '语言管理', icon: CodeOutlined },
      { path: '/datasource', label: '数据源', icon: DatabaseOutlined },
      { path: '/projects', label: '项目', icon: FolderOutlined },
      { path: '/mappings', label: '映射管理', icon: SwapOutlined },
    ],
  },
  {
    label: '系统',
    items: [
      { path: '/template-render', label: '模板渲染', icon: AppstoreOutlined },
      { path: '/settings', label: '设置', icon: SettingOutlined },
    ],
  },
])

function isActive(item) {
  return route.path === item.path || route.path.startsWith(item.path + '/')
}

function go(path) {
  if (route.path !== path) {
    router.push(path)
  }
}
</script>

<style scoped>
.nav {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.nav-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 4px 10px 12px;
}

/* 小节标题（原型 side-label） */
.side-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-muted);
  padding: 14px 8px 4px;
  user-select: none;
}

/* 导航条目（原型 nav-item）：30px 行高、7px 圆角胶囊、单色语言 */
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

/* 折叠态：图标居中的方形按钮 */
.collapsed-item {
  justify-content: center;
  padding: 0;
}
</style>
