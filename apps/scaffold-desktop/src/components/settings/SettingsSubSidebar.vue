<template>
  <div class="sub-sidebar" v-if="showSubSidebar">
    <ul class="sub-menu">
      <!-- 常规设置二级菜单 -->
      <template v-if="currentMainTab === 'general'">
        <li>
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === 'basic' }"
            @click="switchSubTab('basic')"
          >
            <span>基础设置</span>
          </a>
        </li>
        <li>
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === 'behavior' }"
            @click="switchSubTab('behavior')"
          >
            <span>行为设置</span>
          </a>
        </li>
      </template>

      <!-- 高级设置二级菜单 -->
      <template v-if="currentMainTab === 'advanced'">
        <li>
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === 'security' }"
            @click="switchSubTab('security')"
          >
            <span>安全设置</span>
          </a>
        </li>
        <li>
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === 'network' }"
            @click="switchSubTab('network')"
          >
            <span>网络设置</span>
          </a>
        </li>
        <li>
          <a
            class="sub-menu-item"
            :class="{ active: currentSubTab === 'developer' }"
            @click="switchSubTab('developer')"
          >
            <span>开发者选项</span>
          </a>
        </li>
      </template>
    </ul>
  </div>
</template>

<script setup>
const props = defineProps({
  currentMainTab: {
    type: String,
    default: 'general'
  },
  currentSubTab: {
    type: String,
    default: 'basic'
  },
  showSubSidebar: {
    type: Boolean,
    default: true
  }
})

const emit = defineEmits(['sub-tab-change'])

const switchSubTab = (tab) => {
  emit('sub-tab-change', tab)
}
</script>

<style scoped>
.sub-sidebar {
  min-width: 160px;
  border-right: 1px solid var(--color-border);
  padding: 10px;
  user-select: none;
  flex-shrink: 0;
  min-height: 100%;
}

.sub-menu {
  list-style: none;
  margin: 0;
  padding: 0;
}

.sub-menu li {
  margin-bottom: 4px;
}

.sub-menu-item {
  display: block;
  padding: 8px 12px;
  width: 100%;
  text-decoration: none;
  color: var(--color-text);
  cursor: pointer;
  border-radius: var(--border-radius-md);
  font-size: 13px;
  transition: background-color var(--transition-fast);
  border: 0.5px solid transparent;
  margin: 2px 4px;
}

.sub-menu-item:hover {
  background: var(--color-surface);
  color: var(--color-primary);
  border-color: transparent;
}

.sub-menu-item.active {
  background: var(--color-surface);
  color: var(--color-primary);
  border-color: var(--color-border);
  font-weight: 500;
}

/* Theme adjustments now handled by global CSS variables */
</style>