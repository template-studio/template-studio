<template>
  <div v-if="layoutStore.footerType" class="content-footer">
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
</template>

<script setup>
import { useLayoutStore } from '@/stores/layout'

const layoutStore = useLayoutStore()

const handlePageChange = (page, size) => {
  layoutStore.updateFooterPagination({ current: page, pageSize: size })
}

const handleSizeChange = (current, size) => {
  layoutStore.updateFooterPagination({ current: 1, pageSize: size })
}
</script>

<style scoped>
.content-footer {
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
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
</style>
