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

    <!-- 全局 Footer -->
    <div v-if="layoutStore.footerType" class="content-footer">
      <!-- 分页 footer -->
      <a-pagination
        v-if="layoutStore.footerType === 'pagination'"
        v-model:current="layoutStore.footerPagination.current"
        v-model:pageSize="layoutStore.footerPagination.pageSize"
        :total="layoutStore.footerPagination.total"
        :show-size-changer="true"
        :show-quick-jumper="true"
        :show-total="(total, range) => `共 ${total} 条，当前 ${range[0]}-${range[1]}`"
        :page-size-options="layoutStore.footerPageSizeOptions"
        @change="handlePageChange"
        @showSizeChange="handleSizeChange"
      />
      <!-- 概览 footer -->
      <div v-else-if="layoutStore.footerType === 'overview'" class="footer-overview">
        <div
          v-for="(item, index) in layoutStore.footerOverview.items"
          :key="index"
          class="overview-item"
        >
          <span class="overview-label">{{ item.label }}</span>
          <span class="overview-value">{{ item.value }}</span>
        </div>
      </div>
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

const handlePageChange = (page, size) => {
  layoutStore.updateFooterPagination({ current: page, pageSize: size })
}

const handleSizeChange = (current, size) => {
  layoutStore.updateFooterPagination({ current: 1, pageSize: size })
}

onMounted(() => {
  console.log('MainContent mounted with layout:', {
    isMobile: layoutStore.isMobile,
    sidebarCollapsed: layoutStore.sidebarCollapsed
  })
})
</script>

<style scoped>
.main-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
  background: var(--color-background);
}

.content-wrapper {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

/* Page transitions */
.page-enter-active,
.page-leave-active {
  transition: opacity var(--transition-normal);
}

.page-enter-from {
  opacity: 0;
}

.page-leave-to {
  opacity: 0;
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

/* 全局 Footer */
.content-footer {
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-top: 1px solid var(--color-border-light);
  background: transparent;
  flex-shrink: 0;
  padding: 0 var(--spacing-lg);
}

.footer-overview {
  display: flex;
  align-items: center;
  gap: var(--spacing-xl);
  width: 100%;
}

.overview-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.overview-label {
  font-size: 13px;
  color: var(--color-text-secondary);
}

.overview-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .content-footer {
    padding: 0 var(--spacing-md);
    height: auto;
    min-height: 56px;
  }

  .content-footer :deep(.ant-pagination) {
    flex-wrap: wrap;
    justify-content: center;
  }

  .content-footer :deep(.ant-pagination-options) {
    margin-top: var(--spacing-sm);
  }
}
</style>