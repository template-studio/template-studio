<template>
  <div class="search-bar">
    <!-- 搜索输入框 -->
    <a-input
      v-model:value="searchValue"
      :placeholder="placeholder"
      allow-clear
      @input="handleSearch"
      @clear="handleClear"
      class="search-input"
    >
      <template #prefix>
        <SearchOutlined />
      </template>
    </a-input>

    <!-- 筛选下拉框 -->
    <a-select
      v-if="filters && filters.length > 0"
      v-model:value="filterValue"
      placeholder="筛选"
      allow-clear
      @change="handleFilterChange"
      class="filter-select"
    >
      <a-select-option v-for="filter in filters" :key="filter.value" :value="filter.value">
        {{ filter.label }}
      </a-select-option>
    </a-select>

    <!-- 排序选择 -->
    <a-select
      v-if="sortOptions && sortOptions.length > 0"
      v-model:value="sortValue"
      placeholder="排序"
      @change="handleSortChange"
      class="sort-select"
    >
      <a-select-option v-for="option in sortOptions" :key="option.value" :value="option.value">
        {{ option.label }}
      </a-select-option>
    </a-select>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'
import { SearchOutlined } from '@ant-design/icons-vue'

const props = defineProps({
  placeholder: {
    type: String,
    default: '搜索...'
  },
  filters: {
    type: Array,
    default: () => []
    // 格式: [{ label: '筛选项', value: 'filter_value' }]
  },
  sortOptions: {
    type: Array,
    default: () => []
    // 格式: [{ label: '排序方式', value: 'sort_field:asc' }]
  },
  modelValue: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['update:modelValue', 'search', 'filter', 'sort'])

const searchValue = ref(props.modelValue)
const filterValue = ref(undefined)
const sortValue = ref(props.sortOptions?.[0]?.value || undefined)

// 监听外部值变化
watch(() => props.modelValue, (newVal) => {
  searchValue.value = newVal
})

// 搜索处理
const handleSearch = () => {
  emit('update:modelValue', searchValue.value)
  emit('search', searchValue.value)
}

// 清空搜索
const handleClear = () => {
  searchValue.value = ''
  emit('update:modelValue', '')
  emit('search', '')
}

// 筛选变化
const handleFilterChange = (value) => {
  emit('filter', value)
}

// 排序变化
const handleSortChange = (value) => {
  emit('sort', value)
}
</script>

<style scoped>
.search-bar {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  flex-wrap: wrap;
}

.search-input {
  flex: 1;
  min-width: 200px;
  max-width: 400px;
}

.filter-select {
  min-width: 120px;
}

.sort-select {
  min-width: 120px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .search-bar {
    flex-direction: column;
    align-items: stretch;
  }

  .search-input {
    max-width: none;
  }

  .filter-select,
  .sort-select {
    min-width: auto;
  }
}
</style>
