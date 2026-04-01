<template>
  <a-menu
    :selectedKeys="selectedKeys"
    mode="inline"
    class="navigation-menu"
    @click="handleMenuClick"
  >
    <a-menu-item key="/">
      <template #icon>
        <HomeOutlined />
      </template>
      <span>首页</span>
    </a-menu-item>

    <a-menu-item key="/templates">
      <template #icon>
        <FileTextOutlined />
      </template>
      <span>模板</span>
    </a-menu-item>

    <a-menu-item key="/settings">
      <template #icon>
        <SettingOutlined />
      </template>
      <span>设置</span>
    </a-menu-item>
  </a-menu>
</template>

<script setup>
import { ref, watch, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useLayoutStore } from '@/stores/layout'
import {
  HomeOutlined,
  FileTextOutlined,
  SettingOutlined
} from '@ant-design/icons-vue'

const router = useRouter()
const route = useRoute()
const layoutStore = useLayoutStore()

// 使用计算属性直接从路由获取选中状态，避免状态不同步
const selectedKeys = computed(() => {
  return [route.path]
})

const handleMenuClick = ({ key }) => {
  if (key !== route.path) {
    router.push(key)
  }
}
</script>

<style scoped>
.navigation-menu {
  border: none !important;
  border-right: none !important;
  background: transparent;
}

/* 强制移除 Ant Design 菜单的右边框 */
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

/* 使用流畅的过渡动画，参考设置界面的体验 */
.navigation-menu :deep(.ant-menu-item) {
  transition: background-color var(--transition-fast) ease, color var(--transition-fast) ease;
}

.navigation-menu :deep(.ant-menu-item::after) {
  transition: none !important;
}

.navigation-menu :deep(.ant-menu-item-selected::after) {
  transition: none !important;
}

/* 悬浮状态 - 增强CSS优先级，覆盖Ant Design默认样式 */
.navigation-menu :deep(.ant-menu-item:hover),
.navigation-menu :deep(.ant-menu-item.ant-menu-item:hover),
.navigation-menu :deep(.ant-menu-inline .ant-menu-item:hover) {
  background: var(--color-surface) !important;
  color: var(--color-primary) !important;
  border-color: transparent !important;
}

/* 选中状态 - 与悬浮状态颜色完全相同，只有边框不同 */
.navigation-menu :deep(.ant-menu-item-selected),
.navigation-menu :deep(.ant-menu-item.ant-menu-item-selected),
.navigation-menu :deep(.ant-menu-inline .ant-menu-item-selected) {
  background: var(--color-surface) !important;
  color: var(--color-primary) !important;
  border-color: var(--color-border) !important;
}

/* 选中状态再悬浮 - 保持相同的颜色，避免任何视觉变化 */
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
    margin: 2px 14px;
    width: 32px !important;
    height: 32px !important;
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
    border-radius: 6px !important;
    border: 1px solid transparent !important;
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
    border-color: transparent !important;
    color: var(--color-primary) !important;
  }

  /* 选中状态 */
  .ant-menu-item-selected,
  .ant-menu-item-selected:hover {
    background: var(--color-hover) !important;
    border-color: transparent !important;
    color: var(--color-primary) !important;
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

/* Removed dark theme adjustments - now handled by global CSS variables */
</style>