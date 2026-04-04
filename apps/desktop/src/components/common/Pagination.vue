<template>
  <div class="pagination-wrapper" v-if="total > 0">
    <a-pagination
      v-model:current="currentPage"
      v-model:pageSize="pageSize"
      :total="total"
      :show-size-changer="showSizeChanger"
      :show-quick-jumper="showQuickJumper"
      :show-total="showTotal ? (total, range) => `共 ${total} 条，当前 ${range[0]}-${range[1]}` : undefined"
      :page-size-options="pageSizeOptions"
      @change="handleChange"
      @showSizeChange="handleSizeChange"
    />
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'

const props = defineProps({
  total: {
    type: Number,
    required: true
  },
  current: {
    type: Number,
    default: 1
  },
  pageSize: {
    type: Number,
    default: 12
  },
  showSizeChanger: {
    type: Boolean,
    default: true
  },
  showQuickJumper: {
    type: Boolean,
    default: true
  },
  showTotal: {
    type: Boolean,
    default: true
  },
  pageSizeOptions: {
    type: Array,
    default: () => ['12', '24', '36', '48']
  }
})

const emit = defineEmits(['update:current', 'update:pageSize', 'change', 'sizeChange'])

const currentPage = ref(props.current)
const pageSize = ref(props.pageSize)

// 监听外部值变化
watch(() => props.current, (newVal) => {
  currentPage.value = newVal
})

watch(() => props.pageSize, (newVal) => {
  pageSize.value = newVal
})

// 页码变化
const handleChange = (page, size) => {
  emit('update:current', page)
  emit('update:pageSize', size)
  emit('change', page, size)
}

// 每页条数变化
const handleSizeChange = (current, size) => {
  emit('update:current', 1)
  emit('update:pageSize', size)
  emit('sizeChange', 1, size)
}
</script>

<style scoped>
.pagination-wrapper {
  display: flex;
  justify-content: center;
  padding: var(--spacing-lg) 0;
  margin-top: var(--spacing-lg);
  border-top: 1px solid var(--color-border);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .pagination-wrapper {
    padding: var(--spacing-md) 0;
  }

  :deep(.ant-pagination) {
    flex-wrap: wrap;
    justify-content: center;
  }

  :deep(.ant-pagination-options) {
    margin-top: var(--spacing-sm);
  }
}
</style>
