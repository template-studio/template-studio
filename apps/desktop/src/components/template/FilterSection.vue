<template>
  <div class="filters-section">
    <div class="filter-block search-block">
      <n-input
        :value="searchQuery"
        placeholder="搜索模板名称或描述..."
        size="large"
        clearable
        @update:value="$emit('update:searchQuery', $event)"
      >
        <template #prefix>
          <n-icon><SearchOutline /></n-icon>
        </template>
      </n-input>
    </div>

    <div class="filter-block">
      <h2>分类</h2>
      <div class="tiles">
        <div
          v-for="cat in categories"
          :key="cat.id"
          class="tile"
          :class="{ active: selectedCategory === cat.id }"
          @click="$emit('update:selectedCategory', cat.id)"
        >
          {{ cat.name }}
        </div>
      </div>
    </div>

    <div class="filter-block">
      <h2>语言</h2>
      <div class="tiles">
        <div
          v-for="lang in languages"
          :key="lang.id"
          class="tile"
          :class="{ active: selectedLanguage === lang.id }"
          @click="$emit('update:selectedLanguage', lang.id)"
        >
          {{ lang.name }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { NInput, NIcon } from 'naive-ui'
import { SearchOutline } from '@vicons/ionicons5'

defineProps({
  searchQuery: {
    type: String,
    default: ''
  },
  categories: {
    type: Array,
    default: () => []
  },
  languages: {
    type: Array,
    default: () => []
  },
  selectedCategory: {
    type: String,
    default: 'all'
  },
  selectedLanguage: {
    type: String,
    default: 'all'
  }
})

defineEmits(['update:searchQuery', 'update:selectedCategory', 'update:selectedLanguage'])
</script>

<style scoped>
.filters-section {
  background: #fff;
  padding: 20px;
  border-radius: 12px;
  margin-bottom: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.filter-block {
  margin-bottom: 20px;
}

.filter-block:last-child {
  margin-bottom: 0;
}

.search-block {
  margin-bottom: 24px;
}

.search-block :deep(.n-input) {
  border-radius: 12px;
}

.search-block :deep(.n-input__input) {
  font-size: 15px;
}

.filter-block h2 {
  font-size: 1rem;
  font-weight: 600;
  color: #333;
  margin: 0 0 12px 0;
}

.tiles {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.tile {
  background: #fff;
  border-radius: 12px;
  padding: 8px 16px;
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  transition: all 0.25s ease;
  display: inline-flex;
  align-items: center;
  font-size: 0.875rem;
  color: #333;
  font-weight: 500;
  border: 1px solid transparent;
}

.tile:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.1);
  color: #4285f4;
  border-color: rgba(66, 133, 244, 0.1);
}

.tile.active {
  background: linear-gradient(135deg, #4285f4 0%, #34a853 100%);
  color: #fff;
  box-shadow: 0 4px 14px rgba(66, 133, 244, 0.3);
}

.tile.active:hover {
  background: linear-gradient(135deg, #3b78e7 0%, #2d9249 100%);
  color: #fff;
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(66, 133, 244, 0.4);
}
</style>
