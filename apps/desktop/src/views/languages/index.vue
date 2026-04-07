<template>
  <div class="languages-view">
    <!-- 页面头部 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <h2 class="page-title">语言管理</h2>
        <span class="result-count">共 {{ filteredLanguages.length }} 种语言</span>
      </div>
      <div class="toolbar-right">
        <SearchBar
          v-model="searchQuery"
          placeholder="搜索语言名称..."
          :filters="languageFilters"
          :sort-options="sortOptions"
          @search="handleSearch"
          @filter="handleFilter"
          @sort="handleSort"
        />
        <a-button type="primary" size="large" @click="openCreateDialog">
          <template #icon>
            <PlusOutlined />
          </template>
          添加语言
        </a-button>
      </div>
    </div>

    <!-- 语言卡片列表 -->
    <div class="languages-content">
      <a-spin :spinning="loading">
        <div v-if="paginatedLanguages.length > 0" class="languages-grid">
          <div
            v-for="language in paginatedLanguages"
            :key="language.id"
            class="language-card"
            @click="openEditDialog(language)"
          >
            <!-- 内容区域 -->
            <div class="card-content">
              <!-- 图标 -->
              <div class="language-icon" :style="{ color: getLanguageColor(language.color) }">
                {{ language.icon || '💻' }}
              </div>

              <!-- 语言名称 -->
              <h3 class="language-name">{{ language.name }}</h3>

              <!-- 描述 -->
              <p class="language-description">
                {{ language.description || '暂无描述' }}
              </p>

              <!-- 标签 -->
              <div class="language-tags">
                <a-tag v-if="language.is_builtin" color="blue">内置</a-tag>
                <a-tag v-else color="green">自定义</a-tag>
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="card-actions">
              <a-button
                type="text"
                size="small"
                @click.stop="openFieldTypesDialog(language)"
                class="action-btn"
                title="管理类型字段"
              >
                <SettingOutlined />
              </a-button>
              <a-button
                type="text"
                size="small"
                @click.stop="openEditDialog(language)"
                class="action-btn"
                title="编辑"
              >
                <EditOutlined />
              </a-button>
              <a-button
                type="text"
                size="small"
                danger
                @click.stop="confirmDelete(language)"
                class="action-btn"
                title="删除"
                :disabled="language.is_builtin"
              >
                <DeleteOutlined />
              </a-button>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <a-empty
          v-else-if="!loading && filteredLanguages.length === 0"
          :description="searchQuery ? '没有找到匹配的语言' : '暂无语言'"
          :image="Empty.PRESENTED_IMAGE_SIMPLE"
          class="empty-state"
        >
          <a-button type="primary" size="large" @click="openCreateDialog">
            <template #icon>
              <PlusOutlined />
            </template>
            添加第一个语言
          </a-button>
        </a-empty>
      </a-spin>
    </div>

    <!-- 创建/编辑对话框 -->
    <LanguageDialog
      v-model:open="dialogVisible"
      :mode="dialogMode"
      :language="editingLanguage"
      @saved="onDialogSaved"
    />

    <!-- 类型字段管理对话框 -->
    <FieldTypesModal
      v-model:open="fieldTypesDialogVisible"
      :language="currentLanguage"
    />
  </div>
</template>

<script setup>
import { ref, onMounted, computed, watch } from 'vue'
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  SettingOutlined
} from '@ant-design/icons-vue'
import { Empty, message, Modal } from 'ant-design-vue'
import { notify } from '@/utils/notify'
import * as languagesApi from '@/api/languages'
import { SearchBar } from '@/components/common'
import { useLayoutStore } from '@/stores/layout'
import LanguageDialog from './components/LanguageDialog.vue'
import FieldTypesModal from './components/FieldTypesModal.vue'

const layoutStore = useLayoutStore()

// 状态
const loading = ref(false)
const languages = ref([])

// 搜索、筛选、排序状态
const searchQuery = ref('')
const filterValue = ref(undefined)
const sortValue = ref('name:asc')

// 筛选选项
const languageFilters = [
  { label: '内置语言', value: 'builtin' },
  { label: '自定义语言', value: 'custom' }
]

// 排序选项
const sortOptions = [
  { label: '名称 A-Z', value: 'name:asc' },
  { label: '名称 Z-A', value: 'name:desc' },
  { label: '最新添加', value: 'created_at:desc' },
  { label: '最早添加', value: 'created_at:asc' }
]

// 对话框状态
const dialogVisible = ref(false)
const dialogMode = ref('create')
const editingLanguage = ref(null)

// 类型字段管理对话框状态
const fieldTypesDialogVisible = ref(false)
const currentLanguage = ref(null)

// 筛选后的语言列表
const filteredLanguages = computed(() => {
  let result = [...languages.value]

  // 搜索筛选
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(lang =>
      lang.name.toLowerCase().includes(query) ||
      (lang.description && lang.description.toLowerCase().includes(query))
    )
  }

  // 类型筛选
  if (filterValue.value) {
    if (filterValue.value === 'builtin') {
      result = result.filter(lang => lang.is_builtin)
    } else if (filterValue.value === 'custom') {
      result = result.filter(lang => !lang.is_builtin)
    }
  }

  // 排序
  if (sortValue.value) {
    const [field, order] = sortValue.value.split(':')
    result.sort((a, b) => {
      let valueA, valueB

      switch (field) {
        case 'name':
          valueA = a.name.toLowerCase()
          valueB = b.name.toLowerCase()
          break
        case 'created_at':
          valueA = new Date(a.created_at).getTime()
          valueB = new Date(b.created_at).getTime()
          break
        default:
          return 0
      }

      if (order === 'asc') {
        return valueA > valueB ? 1 : -1
      } else {
        return valueA < valueB ? 1 : -1
      }
    })
  }

  return result
})

// 分页后的语言列表
const paginatedLanguages = computed(() => {
  const { current, pageSize: size } = layoutStore.footerPagination
  const start = (current - 1) * size
  const end = start + size
  return filteredLanguages.value.slice(start, end)
})

// 加载语言列表
const loadLanguages = async () => {
  try {
    loading.value = true
    const data = await languagesApi.getAllLanguages()
    languages.value = data
  } catch (error) {
    message.error('加载语言失败: ' + error)
  } finally {
    loading.value = false
  }
}

// 获取语言颜色
const getLanguageColor = (color) => {
  if (!color) return '#d9d9d9'
  const colorMap = {
    red: '#f5222d',
    orange: '#fa8c16',
    gold: '#faad14',
    green: '#52c41a',
    cyan: '#13c2c2',
    blue: '#1890ff',
    purple: '#722ed1',
    pink: '#eb2f96'
  }
  return colorMap[color] || color
}

// 打开创建对话框
const openCreateDialog = () => {
  dialogMode.value = 'create'
  editingLanguage.value = null
  dialogVisible.value = true
}

// 打开编辑对话框
const openEditDialog = (language) => {
  dialogMode.value = 'edit'
  editingLanguage.value = language
  dialogVisible.value = true
}

// 对话框保存回调
const onDialogSaved = async () => {
  await loadLanguages()
}

// 打开类型字段管理对话框
const openFieldTypesDialog = (language) => {
  currentLanguage.value = language
  fieldTypesDialogVisible.value = true
}

// 确认删除
const confirmDelete = (language) => {
  if (language.is_builtin) {
    message.warning('内置语言不能删除')
    return
  }

  Modal.confirm({
    title: '确认删除',
    content: `确定要删除语言 "${language.name}" 吗？此操作不可恢复。`,
    okText: '删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      try {
        await languagesApi.deleteLanguage(language.id)
        notify({ type: 'success', title: '语言删除成功', content: `语言 "${language.name}" 已删除` })
        await loadLanguages()
      } catch (error) {
        notify({ type: 'error', title: '删除失败', content: String(error) })
      }
    }
  })
}

// 搜索处理
const handleSearch = () => {
  layoutStore.updateFooterPagination({ current: 1 })
}

// 筛选处理
const handleFilter = (value) => {
  filterValue.value = value
  layoutStore.updateFooterPagination({ current: 1 })
}

// 排序处理
const handleSort = (value) => {
  sortValue.value = value
  layoutStore.updateFooterPagination({ current: 1 })
}

// 同步分页状态到 store
watch(filteredLanguages, (newVal) => {
  layoutStore.showFooterPagination(newVal.length, layoutStore.footerPagination.current, layoutStore.footerPagination.pageSize)
})

// 组件挂载时加载数据
onMounted(async () => {
  await loadLanguages()
  layoutStore.showFooterPagination(filteredLanguages.value.length, layoutStore.footerPagination.current, layoutStore.footerPagination.pageSize)
})
</script>

<style scoped>
.languages-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 顶部工具栏 */
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-lg);
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  align-items: baseline;
  gap: var(--spacing-md);
}

.page-title {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text);
}

.result-count {
  color: var(--color-text-secondary);
  font-size: 14px;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}

/* 内容区域 */
.languages-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  min-height: 0;
  padding: 0 var(--spacing-lg);
}

.languages-content > :deep(.ant-spin-container) {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  min-height: 400px;
}

/* 语言网格 */
.languages-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: var(--spacing-sm);
}

/* 语言卡片 */
.language-card {
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  overflow: hidden;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.language-card:hover {
  transform: translateY(-3px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.1);
}

/* 内容区域 */
.card-content {
  padding: var(--spacing-md);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.language-icon {
  font-size: 42px;
  line-height: 1;
  margin-bottom: 4px;
}

.language-name {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

.language-description {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-secondary);
  text-align: center;
  min-height: 32px;
  display: flex;
  align-items: center;
}

.language-tags {
  display: flex;
  gap: var(--spacing-xs);
}

/* 操作按钮 */
.card-actions {
  display: flex;
  justify-content: flex-end;
  gap: 4px;
  padding: 8px var(--spacing-sm);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

.card-actions .ant-btn {
  font-size: 14px;
  padding: 4px 6px;
  height: auto;
  min-width: auto;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.card-actions .ant-btn:hover:not(:disabled) {
  transform: scale(1.1);
  background: var(--color-hover);
}
</style>
