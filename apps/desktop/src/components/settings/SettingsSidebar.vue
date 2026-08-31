<template>
  <div class="settings-sidebar">
    <!-- 一级菜单 -->
    <ul class="settings-menu">
      <li>
        <a
          class="menu-item"
          :class="{ active: currentMainTab === 'general' }"
          @click="switchMainTab('general')"
        >
          <SettingOutlined />
          <span>常规设置</span>
        </a>
      </li>
      <li>
        <a
          class="menu-item"
          :class="{ active: currentMainTab === 'web-server' }"
          @click="switchMainTab('web-server')"
        >
          <CloudServerOutlined />
          <span>Web服务器</span>
        </a>
      </li>
      <li>
        <a
          class="menu-item"
          :class="{ active: currentMainTab === 'advanced' }"
          @click="switchMainTab('advanced')"
        >
          <ApiOutlined />
          <span>高级设置</span>
        </a>
      </li>
      <li>
        <a
          class="menu-item"
          :class="{ active: currentMainTab === 'ai-services' }"
          @click="switchMainTab('ai-services')"
        >
          <AiIcon :size="14" />
          <span>AI 服务</span>
        </a>
      </li>
      <li>
        <a
          class="menu-item"
          :class="{ active: currentMainTab === 'about' }"
          @click="switchMainTab('about')"
        >
          <InfoCircleOutlined />
          <span>关于</span>
        </a>
      </li>
    </ul>
  </div>
</template>

<script setup>
import AiIcon from '@/components/icons/AiIcon.vue'
import {
  SettingOutlined,
  CloudServerOutlined,
  ApiOutlined,
  InfoCircleOutlined,
} from '@ant-design/icons-vue'

const props = defineProps({
  currentMainTab: {
    type: String,
    default: 'general'
  }
})

const emit = defineEmits(['main-tab-change'])

const switchMainTab = (tab) => {
  // 只有当值真正改变时才更新
  if (props.currentMainTab !== tab) {
    emit('main-tab-change', tab)
  }
}
</script>

<style scoped>
.settings-sidebar {
  display: flex;
  flex-direction: column;
  min-width: 200px;
  border-right: 1px solid var(--color-border);
  padding: 10px;
  user-select: none;
  flex-shrink: 0;
}

.settings-menu {
  display: flex;
  flex-direction: column;
  list-style: none;
  margin: 0;
  padding: 0;
}

.settings-menu li {
  margin-bottom: 5px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  width: 100%;
  text-decoration: none;
  color: var(--color-text);
  cursor: pointer;
  border-radius: var(--border-radius-md);
  font-weight: 500;
  transition: background-color var(--transition-fast);
  border: 0.5px solid transparent;
  margin: 2px 4px;
}

.menu-item .anticon {
  font-size: 16px;
  opacity: 0.8;
}

.menu-item:hover {
  background: var(--color-surface);
  color: var(--color-primary);
  border-color: transparent;
}

.menu-item.active {
  background: var(--color-surface);
  color: var(--color-primary);
  border-color: var(--color-border);
}

/* Theme adjustments now handled by global CSS variables */
</style>