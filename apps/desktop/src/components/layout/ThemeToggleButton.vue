<template>
  <!-- 主题切换：自绘太阳/月亮（细线条，currentColor），顶栏右上角 -->
  <a-tooltip :title="themeStore.isDark ? '切换浅色' : '切换深色'" placement="bottom">
    <button class="theme-toggle titlebar-no-drag" @click="themeStore.toggleTheme()">
      <!-- 浅色态显示太阳，深色态显示月亮 -->
      <svg v-if="!themeStore.isDark" width="16" height="16" viewBox="1.5 1.5 21 21" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="12" cy="12" r="4" stroke="currentColor" stroke-width="1.8" />
        <path d="M12 2.5v2.6M12 18.9v2.6M2.5 12h2.6M18.9 12h2.6M5.3 5.3l1.8 1.8M16.9 16.9l1.8 1.8M18.7 5.3l-1.8 1.8M7.1 16.9l-1.8 1.8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
      </svg>
      <svg v-else width="16" height="16" viewBox="2.5 2.5 19 19" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M20.2 14.2A8.5 8.5 0 1 1 9.8 3.8a7 7 0 0 0 10.4 10.4Z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" />
      </svg>
    </button>
  </a-tooltip>
</template>

<script setup>
import { useThemeStore } from '@/stores/theme'

const themeStore = useThemeStore()
</script>

<style scoped>
.theme-toggle {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 7px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease;
}

.theme-toggle:hover {
  background: var(--color-nav-hover);
  color: var(--color-text);
}
</style>
