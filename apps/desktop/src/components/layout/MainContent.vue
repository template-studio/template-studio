<template>
  <div class="main-content">
    <!-- Page Content with Router -->
    <div class="content-wrapper">
      <router-view v-slot="{ Component, route }">
        <transition name="page" mode="out-in">
          <component :is="Component" :key="route.path" />
        </transition>
      </router-view>
    </div>

    <!-- Loading Overlay -->
    <div v-if="isLoading" class="loading-overlay">
      <a-spin size="large" />
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useLayoutStore } from '@/stores/layout'

const layoutStore = useLayoutStore()
const isLoading = ref(false)

onMounted(() => {
  // Initialize layout
  console.log('MainContent mounted with layout:', {
    isMobile: layoutStore.isMobile,
    sidebarCollapsed: layoutStore.sidebarCollapsed
  })
})
</script>

<style scoped>
.main-content {
  height: 100%;
  position: relative;
  background: var(--color-background);
}

.content-wrapper {
  height: 100%;
  overflow-y: auto;
  padding: var(--spacing-lg);
}

/* Page transitions */
.page-enter-active,
.page-leave-active {
  transition: all var(--transition-normal);
}

.page-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

.page-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}

/* Loading overlay */
.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-overlay);
  /* Using semi-transparent white for both themes, could be enhanced with CSS variables if needed */
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(2px);
}

/* Loading overlay theme handled by global CSS variables */

/* Content wrapper scrollbar - override global hide */
.content-wrapper {
  scrollbar-width: thin; /* Firefox */
  scrollbar-color: var(--color-border) var(--color-surface);
}

.content-wrapper::-webkit-scrollbar {
  display: block; /* Override global hide */
  width: 8px;
}

.content-wrapper::-webkit-scrollbar-track {
  background: var(--color-surface);
}

.content-wrapper::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 4px;
  transition: background var(--transition-fast);
}

.content-wrapper::-webkit-scrollbar-thumb:hover {
  background: var(--color-hover);
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .content-wrapper {
    padding: var(--spacing-md);
  }
}

@media (max-width: 480px) {
  .content-wrapper {
    padding: var(--spacing-sm);
  }
}

/* Scrollbar theme adjustments now handled by global CSS variables */
</style>