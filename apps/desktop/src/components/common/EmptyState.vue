<template>
  <div class="empty-state">
    <a-empty :description="description" :image="image">
      <template v-if="$slots.default" #default>
        <slot />
      </template>
    </a-empty>
  </div>
</template>

<script setup>
import { Empty } from 'ant-design-vue'

defineProps({
  description: {
    type: String,
    default: '暂无数据'
  },
  image: {
    type: Object,
    default: () => Empty.PRESENTED_IMAGE_SIMPLE
  }
})
</script>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  padding: var(--spacing-xl);
}
</style>
