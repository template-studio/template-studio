<template>
  <div class="third-sidebar" v-if="showThirdSidebar">
    <ul class="third-menu">
      <!-- 开发者选项三级菜单 -->
      <template v-if="currentSubTab === 'developer'">
        <li>
          <a
            class="third-menu-item"
            :class="{ active: currentThirdTab === 'debug' }"
            @click="switchThirdTab('debug')"
          >
            <span>调试模式</span>
          </a>
        </li>
        <li>
          <a
            class="third-menu-item"
            :class="{ active: currentThirdTab === 'experimental' }"
            @click="switchThirdTab('experimental')"
          >
            <span>实验性功能</span>
          </a>
        </li>
        <li>
          <a
            class="third-menu-item"
            :class="{ active: currentThirdTab === 'console' }"
            @click="switchThirdTab('console')"
          >
            <span>控制台</span>
          </a>
        </li>
      </template>
    </ul>
  </div>
</template>

<script setup>
const props = defineProps({
  currentSubTab: {
    type: String,
    default: 'basic'
  },
  currentThirdTab: {
    type: String,
    default: 'debug'
  },
  showThirdSidebar: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['third-tab-change'])

const switchThirdTab = (tab) => {
  emit('third-tab-change', tab)
}
</script>

<style scoped>
.third-sidebar {
  min-width: 140px;
  border-right: 1px solid var(--color-border);
  padding: 8px;
  user-select: none;
  flex-shrink: 0;
  min-height: 100%;
}

.third-menu {
  list-style: none;
  margin: 0;
  padding: 0;
}

.third-menu li {
  margin-bottom: 3px;
}

.third-menu-item {
  display: block;
  padding: 6px 10px;
  width: 100%;
  text-decoration: none;
  color: var(--color-text);
  cursor: pointer;
  border-radius: var(--border-radius-md);
  font-size: 12px;
  transition: background-color var(--transition-fast);
  border: 0.5px solid transparent;
  margin: 2px 4px;
}

.third-menu-item:hover {
  background: var(--color-surface);
  color: var(--color-primary);
  border-color: transparent;
}

.third-menu-item.active {
  background: var(--color-surface);
  color: var(--color-primary);
  border-color: var(--color-border);
  font-weight: 500;
}

/* Theme adjustments now handled by global CSS variables */
</style>