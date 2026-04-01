<template>
  <div class="theme-toggle">
    <a-button
      type="text"
      class="toggle-button"
      @click="toggleTheme"
      :title="`Switch to ${themeStore.isDark ? 'light' : 'dark'} theme`"
    >
      <template #icon>
        <StarOutlined v-if="themeStore.isDark" />
        <BulbOutlined v-else />
      </template>
    </a-button>
  </div>
</template>

<script setup>
import { useThemeStore } from '@/stores/theme'
import { StarOutlined, BulbOutlined } from '@ant-design/icons-vue'

const themeStore = useThemeStore()

const toggleTheme = () => {
  themeStore.toggleTheme()
}
</script>

<style scoped>
.theme-toggle {
  display: flex;
  align-items: center;
}

.toggle-button {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  width: 32px;
  height: 32px;
  outline: none; /* 移除默认焦点轮廓 */
}

.toggle-button:hover {
  color: var(--color-primary);
  background: var(--color-hover);
}

/* Animation for icon switch */
.toggle-button .anticon {
  transition: transform var(--transition-fast);
}

.toggle-button:hover .anticon {
  transform: scale(1.1);
}

/* Theme adjustments now handled by global CSS variables */
</style>