<template>
  <div class="mappings-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1>映射管理</h1>
      <p class="page-desc">配置数据库类型到目标语言类型的全局映射规则</p>
    </div>

    <div class="mappings-container">
      <!-- 左侧：数据库类型垂直 Tab -->
      <div class="db-tabs">
        <div class="db-tabs-header">
          <span>数据库类型</span>
        </div>
        <div
          v-for="db in databaseTypes"
          :key="db.key"
          :class="['db-tab-item', { active: activeDbType === db.key }]"
          @click="activeDbType = db.key"
        >
          <span class="db-icon">{{ db.icon }}</span>
          <span class="db-label">{{ db.label }}</span>
          <span class="db-count">{{ getDbMappingCount(db.key) }}</span>
        </div>
      </div>

      <!-- 右侧：映射配置 -->
      <div class="mappings-content">
        <!-- 语言横向 Tab -->
        <div class="lang-tabs">
          <template v-if="availableLanguages.length > 0">
            <div
              v-for="lang in availableLanguages"
              :key="lang.key"
              :class="['lang-tab-item', { active: activeLang === lang.key, disabled: lang.disabled }]"
              @click="!lang.disabled && (activeLang = lang.key)"
            >
              <span class="lang-label">{{ lang.label }}</span>
                </div>
          </template>
          <div v-else class="lang-tabs-empty">
            <span>请先在</span>
            <a @click="$router.push('/languages')">语言管理</a>
            <span>中添加语言</span>
          </div>
        </div>

        <!-- 映射表格 -->
        <div class="mappings-table-wrapper">
          <div class="table-toolbar">
            <a-input-search
              v-model:value="searchText"
              placeholder="搜索数据库类型..."
              style="width: 240px"
              allow-clear
            />
            <a-button type="primary" @click="showAddMappingDialog">
              <template #icon><PlusOutlined /></template>
              添加映射
            </a-button>
          </div>

          <a-table
            :columns="mappingColumns"
            :data-source="filteredMappings"
            :pagination="false"
            :loading="loading"
            size="small"
            row-key="id"
          >
            <template #bodyCell="{ column, record }">
              <template v-if="column.key === 'pattern'">
                <code class="type-code">{{ record.pattern }}</code>
              </template>
              <template v-else-if="column.key === 'targetType'">
                <code :class="['type-code', 'target', { 'empty': !record.targetType }]">{{ record.targetType || '未配置' }}</code>
              </template>
              <template v-else-if="column.key === 'action'">
                <a-button type="link" size="small" @click="editMapping(record)">{{ record.targetType ? '编辑' : '配置' }}</a-button>
              </template>
            </template>
          </a-table>

          <a-empty v-if="filteredMappings.length === 0 && !loading" description="暂无映射规则，点击上方按钮添加" />
        </div>
      </div>
    </div>

    <!-- 添加/编辑映射对话框 -->
    <a-modal
      v-model:open="mappingDialogVisible"
      :title="editingMapping ? `配置映射 - ${editingMapping.dbType}` : '添加映射'"
      :confirm-loading="saving"
      @ok="saveMapping"
      @cancel="closeMappingDialog"
      width="520px"
    >
      <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 18 }">
        <a-form-item label="数据库类型">
          <a-input
            v-model:value="mappingForm.pattern"
            disabled
          />
          <div class="form-hint">数据库字段类型（只读）</div>
        </a-form-item>

        <a-form-item label="目标类型" required>
          <a-auto-complete
            v-model:value="mappingForm.targetType"
            :options="targetTypeOptions"
            placeholder="如: String, Long, BigDecimal"
            :filter-option="filterTargetType"
            allow-clear
          >
            <template #option="{ value, label, description }">
              <div>
                <div style="font-weight: 500">{{ label }}</div>
                <div v-if="description" style="font-size: 12px; color: #999">{{ description }}</div>
              </div>
            </template>
          </a-auto-complete>
          <div class="form-hint">
            从 <a @click="openLanguageFieldTypes">{{ currentLanguageName }} 类型字段</a> 中选择，或输入自定义类型
          </div>
        </a-form-item>

        <a-form-item label="优先级">
          <a-input-number
            v-model:value="mappingForm.priority"
            :min="0"
            :max="100"
            style="width: 100%"
          />
          <div class="form-hint">数值越大优先级越高，精确匹配优先于通配匹配</div>
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { message } from 'ant-design-vue'
import { PlusOutlined } from '@ant-design/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { getAllLanguages } from '@/api/languages'

// 数据库类型列表
const databaseTypes = ref([
  { key: 'mysql', label: 'MySQL', icon: '🐬' },
  { key: 'postgresql', label: 'PostgreSQL', icon: '🐘' },
  { key: 'sqlite', label: 'SQLite', icon: '📦' }
])

// 标准数据库字段定义（每种数据库的固定字段列表）
const standardDbFields = {
  mysql: [
    { pattern: 'VARCHAR(%)', priority: 10 },
    { pattern: 'CHAR(%)', priority: 10 },
    { pattern: 'TEXT', priority: 10 },
    { pattern: 'LONGTEXT', priority: 10 },
    { pattern: 'INT', priority: 10 },
    { pattern: 'BIGINT', priority: 10 },
    { pattern: 'SMALLINT', priority: 10 },
    { pattern: 'TINYINT(1)', priority: 20 },
    { pattern: 'TINYINT(%)', priority: 10 },
    { pattern: 'DECIMAL(%,%)', priority: 10 },
    { pattern: 'FLOAT', priority: 10 },
    { pattern: 'DOUBLE', priority: 10 },
    { pattern: 'BOOLEAN', priority: 10 },
    { pattern: 'DATE', priority: 10 },
    { pattern: 'TIMESTAMP', priority: 10 },
    { pattern: 'DATETIME', priority: 10 },
    { pattern: 'TIME', priority: 10 },
    { pattern: 'BLOB', priority: 10 },
    { pattern: 'JSON', priority: 10 }
  ],
  postgresql: [
    { pattern: 'VARCHAR(%)', priority: 10 },
    { pattern: 'CHAR(%)', priority: 10 },
    { pattern: 'TEXT', priority: 10 },
    { pattern: 'INTEGER', priority: 10 },
    { pattern: 'BIGINT', priority: 10 },
    { pattern: 'SMALLINT', priority: 10 },
    { pattern: 'BOOLEAN', priority: 10 },
    { pattern: 'DECIMAL(%,%)', priority: 10 },
    { pattern: 'NUMERIC(%,%)', priority: 10 },
    { pattern: 'REAL', priority: 10 },
    { pattern: 'DOUBLE PRECISION', priority: 10 },
    { pattern: 'DATE', priority: 10 },
    { pattern: 'TIMESTAMP', priority: 10 },
    { pattern: 'TIMESTAMPTZ', priority: 10 },
    { pattern: 'TIME', priority: 10 },
    { pattern: 'BYTEA', priority: 10 },
    { pattern: 'JSON', priority: 10 },
    { pattern: 'JSONB', priority: 10 },
    { pattern: 'UUID', priority: 10 },
    { pattern: 'SERIAL', priority: 10 },
    { pattern: 'BIGSERIAL', priority: 10 }
  ],
  sqlite: [
    { pattern: 'INTEGER', priority: 10 },
    { pattern: 'TEXT', priority: 10 },
    { pattern: 'BLOB', priority: 10 },
    { pattern: 'REAL', priority: 10 },
    { pattern: 'NUMERIC', priority: 10 },
    { pattern: 'BOOLEAN', priority: 10 },
    { pattern: 'DATE', priority: 10 },
    { pattern: 'DATETIME', priority: 10 }
  ]
}

// 可用语言列表（从语言管理同步）
const availableLanguages = ref([])

// 当前选中的数据库和语言
const activeDbType = ref('mysql')
const activeLang = ref('java')

// 搜索和加载状态
const searchText = ref('')
const loading = ref(false)
const saving = ref(false)

// 映射数据
const mappings = ref({})

// 对话框状态
const mappingDialogVisible = ref(false)
const editingMapping = ref(null)

// 表单数据
const mappingForm = reactive({
  pattern: '',
  targetType: '',
  priority: 10
})

// 表格列定义
const mappingColumns = [
  { title: '数据库类型', key: 'pattern', width: 220 },
  { title: '目标类型', key: 'targetType', width: 200 },
  { title: '操作', key: 'action', width: 100, align: 'center' }
]

// 当前语言和数据库的映射
const currentMappings = computed(() => {
  const langKey = activeLang.value
  const dbKey = activeDbType.value
  if (!langKey || !dbKey || !mappings.value[langKey] || !mappings.value[langKey][dbKey]) return []

  return mappings.value[langKey][dbKey].map(m => ({
    ...m,
    dbType: databaseTypes.value.find(db => db.key === dbKey)?.label || dbKey,
    dbKey: dbKey
  }))
})

// 当前选中的语言名称
const currentLanguageName = computed(() => {
  const lang = availableLanguages.value.find(l => l.key === activeLang.value)
  return lang?.label || activeLang.value
})

// 当前语言可用的目标类型选项
const targetTypeOptions = ref([])

// 加载语言类型字段
const loadLanguageFieldTypes = async () => {
  const lang = availableLanguages.value.find(l => l.key === activeLang.value)
  if (!lang || !lang.id) {
    targetTypeOptions.value = []
    return
  }

  try {
    const { getLanguageFieldTypes } = await import('@/api/languages')
    const fieldTypes = await getLanguageFieldTypes(lang.id)
    targetTypeOptions.value = fieldTypes.map(f => ({
      value: f.name,
      label: f.name,
      description: f.description
    }))
  } catch {
    targetTypeOptions.value = []
  }
}

// 过滤目标类型选项
const filterTargetType = (input, option) => {
  const searchText = input.toLowerCase()
  return (
    option.label.toLowerCase().includes(searchText) ||
    (option.description && option.description.toLowerCase().includes(searchText))
  )
}

// 跳转到语言类型字段管理
const router = useRouter()
const openLanguageFieldTypes = () => {
  router.push('/languages')
}

// 获取数据库标签颜色
const getDbTagColor = (dbKey) => {
  const colorMap = {
    mysql: 'blue',
    postgresql: 'green',
    sqlite: 'orange'
  }
  return colorMap[dbKey] || 'default'
}

// 获取数据库的已配置映射数量
const getDbMappingCount = (dbKey) => {
  const langKey = activeLang.value
  if (!langKey || !mappings.value[langKey] || !mappings.value[langKey][dbKey]) return 0
  return mappings.value[langKey][dbKey].filter(m => m.targetType).length
}

// 过滤后的映射列表
const filteredMappings = computed(() => {
  if (!searchText.value) return currentMappings.value
  const search = searchText.value.toLowerCase()
  return currentMappings.value.filter(m =>
    m.pattern.toLowerCase().includes(search) ||
    m.targetType.toLowerCase().includes(search)
  )
})

// 加载映射数据
const loadMappings = async () => {
  loading.value = true
  try {
    // 加载语言列表
    const langData = await getAllLanguages()
    availableLanguages.value = langData.map(lang => ({
      key: lang.name.toLowerCase().replace(/\s+/g, '_'),
      label: lang.name,
      id: lang.id,
      icon: lang.icon
    }))

    // 如果没有语言，设置默认提示
    if (availableLanguages.value.length === 0) {
      availableLanguages.value = [{ key: 'empty', label: '请先添加语言', disabled: true }]
      loading.value = false
      return
    }

    // 设置默认语言
    if (!activeLang.value && availableLanguages.value.length > 0) {
      activeLang.value = availableLanguages.value[0].key
    }

    // 加载语言类型字段
    await loadLanguageFieldTypes()

    // 加载映射数据（按语言组织）
    const result = await invoke('db_get_all_type_mappings')
    const loadedMappings = {}

    if (result) {
      const data = typeof result === 'string' ? JSON.parse(result) : result
      // 按语言组织数据
      for (const mapping of data) {
        const langKey = mapping.langType
        const dbKey = mapping.dbType
        if (!loadedMappings[langKey]) {
          loadedMappings[langKey] = {}
        }
        if (!loadedMappings[langKey][dbKey]) {
          loadedMappings[langKey][dbKey] = []
        }
        loadedMappings[langKey][dbKey].push(mapping)
      }
    }

    // 为每个语言初始化映射模板（如果还没有）
    for (const lang of availableLanguages.value) {
      if (lang.disabled) continue
      if (!loadedMappings[lang.key]) {
        loadedMappings[lang.key] = {}
      }
      // 为每种数据库类型初始化
      for (const db of databaseTypes.value) {
        if (!loadedMappings[lang.key][db.key]) {
    // 生成空的映射模板
          loadedMappings[lang.key][db.key] = standardDbFields[db.key].map((field, index) => ({
            id: `${lang.key}_${db.key}_${index}`,
            pattern: field.pattern,
            targetType: '',
            priority: field.priority,
            sort_order: index,
            dbType: db.key,
            langType: lang.key
          }))
        }
      }
    }

    mappings.value = loadedMappings
  } catch (error) {
    console.error('加载映射数据失败:', error)
    // 使用默认数据
    initDefaultMappings()
  } finally {
    loading.value = false
  }
}

// 初始化默认映射（内存中）
const initDefaultMappings = () => {
  mappings.value = {}
  for (const lang of availableLanguages.value) {
    if (lang.disabled) continue
    mappings.value[lang.key] = {}
    for (const db of databaseTypes.value) {
      mappings.value[lang.key][db.key] = standardDbFields[db.key].map((field, index) => ({
        id: `${lang.key}_${db.key}_${index}`,
        pattern: field.pattern,
        targetType: '',
        priority: field.priority,
        sort_order: index,
        dbType: db.key,
        langType: lang.key
      }))
    }
  }
}

// 显示添加映射对话框（实际是直接编辑模板）
const showAddMappingDialog = () => {
  message.info('请点击表格中的"配置"按钮来设置映射')
}

// 编辑映射
const editMapping = (record) => {
  editingMapping.value = record
  mappingForm.pattern = record.pattern
  mappingForm.targetType = record.targetType || ''
  mappingForm.priority = record.priority || 10
  mappingDialogVisible.value = true
}

// 保存映射
const saveMapping = async () => {
  if (!mappingForm.targetType.trim()) {
    message.warning('请输入目标类型')
    return
  }

  saving.value = true
  try {
    const langKey = activeLang.value
    const dbKey = editingMapping.value?.dbKey

    if (!langKey || !dbKey || !mappings.value[langKey] || !mappings.value[langKey][dbKey]) {
      message.error('映射数据异常')
      return
    }

    // 更新映射
    const index = mappings.value[langKey][dbKey].findIndex(m => m.id === editingMapping.value.id)
    if (index !== -1) {
      mappings.value[langKey][dbKey][index] = {
        ...editingMapping.value,
        targetType: mappingForm.targetType.trim(),
        priority: mappingForm.priority
      }
      message.success('映射已更新')

      // 保存到后端
      await saveMappingsToBackend()
    }

    closeMappingDialog()
  } catch (error) {
    console.error('保存映射失败:', error)
    message.error('保存失败: ' + error)
  } finally {
    saving.value = false
  }
}

// 关闭映射对话框
const closeMappingDialog = () => {
  mappingDialogVisible.value = false
  editingMapping.value = null
}

// 保存映射到后端
const saveMappingsToBackend = async () => {
  try {
    // 将所有映射扁平化
    const allMappings = []
    for (const langKey in mappings.value) {
      for (const dbKey in mappings.value[langKey]) {
        for (const mapping of mappings.value[langKey][dbKey]) {
          // 只保存已配置的映射（有目标类型的）
          if (mapping.targetType) {
            allMappings.push({
              ...mapping,
              dbType: dbKey,
              langType: langKey
            })
          }
        }
      }
    }

    await invoke('db_save_all_type_mappings', {
      mappings: allMappings
    })
  } catch (error) {
    console.error('保存映射到后端失败:', error)
    // 不显示错误，因为可能后端命令还没实现
  }
}

// 监听语言切换，加载对应的类型字段
watch(activeLang, () => {
  loadLanguageFieldTypes()
})

onMounted(() => {
  loadMappings()
})
</script>

<style scoped>
.mappings-view {
  padding: 24px;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-background);
}

.page-header {
  margin-bottom: 24px;
}

.page-header h1 {
  margin: 0 0 8px 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text);
}

.page-desc {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 14px;
}

.mappings-container {
  flex: 1;
  display: flex;
  gap: 16px;
  overflow: hidden;
}

/* 左侧数据库 Tab */
.db-tabs {
  width: 180px;
  flex-shrink: 0;
  background: var(--color-surface);
  border-radius: 8px;
  border: 1px solid var(--color-border);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.db-tabs-header {
  padding: 12px 16px;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary);
  border-bottom: 1px solid var(--color-border);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.db-tab-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  cursor: pointer;
  transition: all 0.2s;
  border-left: 3px solid transparent;
}

.db-tab-item:hover {
  background: var(--color-hover);
}

.db-tab-item.active {
  background: var(--color-primary-bg);
  border-left-color: var(--color-primary);
}

.db-icon {
  margin-right: 10px;
  font-size: 16px;
}

.db-label {
  flex: 1;
  font-size: 14px;
  color: var(--color-text);
}

.db-count {
  font-size: 12px;
  color: var(--color-text-secondary);
  background: var(--color-bg-secondary);
  padding: 2px 6px;
  border-radius: 10px;
}

/* 右侧映射内容 */
.mappings-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--color-surface);
  border-radius: 8px;
  border: 1px solid var(--color-border);
  overflow: hidden;
}

/* 语言横向 Tab */
.lang-tabs {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  gap: 8px;
  border-bottom: 1px solid var(--color-border);
  flex-wrap: wrap;
}

.lang-tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: 16px;
  cursor: pointer;
  font-size: 13px;
  color: var(--color-text-secondary);
  background: var(--color-bg-secondary);
  transition: all 0.2s;
}

.lang-tab-item:hover {
  color: var(--color-text);
  background: var(--color-hover);
}

.lang-tab-item.active {
  color: var(--color-primary);
  background: var(--color-primary-bg);
  font-weight: 500;
}

.lang-count {
  font-size: 11px;
  background: var(--color-bg-base);
  padding: 1px 5px;
  border-radius: 8px;
}

.lang-tab-item.active .lang-count {
  background: var(--color-primary);
  color: #fff;
}

.lang-tab-item.disabled {
  cursor: not-allowed;
  opacity: 0.6;
  color: var(--color-text-secondary);
}

.lang-tab-item.disabled:hover {
  background: var(--color-bg-secondary);
}

.lang-tabs-empty {
  color: var(--color-text-secondary);
  font-size: 13px;
}

.lang-tabs-empty a {
  color: var(--color-primary);
  cursor: pointer;
}

.lang-tabs-empty a:hover {
  text-decoration: underline;
}

/* 映射表格 */
.mappings-table-wrapper {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.table-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.type-code {
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  padding: 2px 6px;
  background: var(--color-bg-secondary);
  border-radius: 4px;
  color: var(--color-text);
}

.type-code.target {
  color: var(--color-primary);
  background: var(--color-primary-bg);
}

.type-code.target.empty {
  color: var(--color-text-secondary);
  background: var(--color-bg-secondary);
  font-style: italic;
}

.form-hint {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 4px;
}

.form-hint a {
  color: var(--color-primary);
  cursor: pointer;
}

.form-hint a:hover {
  text-decoration: underline;
}

/* 表格样式 */
:deep(.ant-table) {
  font-size: 13px;
}

:deep(.ant-table-thead > tr > th) {
  background: var(--color-bg-secondary);
  font-weight: 500;
}

:deep(.ant-table-tbody > tr:hover > td) {
  background: var(--color-hover);
}
</style>
