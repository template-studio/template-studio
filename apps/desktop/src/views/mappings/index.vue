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
        <div class="db-tabs-header"><span>数据库类型</span></div>
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
            <a-input-search v-model:value="searchText" placeholder="搜索数据库类型..." style="width: 240px" allow-clear />
            <div class="toolbar-actions">
              <a-button @click="templateDialogVisible = true">
                <template #icon><AppstoreOutlined /></template>
                模板
              </a-button>
              <a-dropdown>
                <a-button>
                  <template #icon><ExportOutlined /></template>
                  导出
                </a-button>
                <template #overlay>
                  <a-menu @click="handleExport">
                    <a-menu-item key="current">导出当前配置</a-menu-item>
                    <a-menu-item key="all">导出全部映射</a-menu-item>
                  </a-menu>
                </template>
              </a-dropdown>
              <a-button @click="triggerImport">
                <template #icon><ImportOutlined /></template>
                导入
              </a-button>
              <a-button type="primary" @click="showAddMappingDialog">
                <template #icon><PlusOutlined /></template>
                添加映射
              </a-button>
            </div>
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
    <MappingDialog
      v-model:open="mappingDialogVisible"
      :mapping="editingMapping"
      :target-type-options="targetTypeOptions"
      :current-language-name="currentLanguageName"
      :saving="saving"
      @saved="handleMappingSaved"
    />

    <!-- 模板选择对话框 -->
    <MappingTemplateDialog
      v-model:open="templateDialogVisible"
      @apply="applyTemplate"
    />

    <!-- 隐藏的文件输入 -->
    <input ref="fileInputRef" type="file" accept=".json" style="display: none" @change="handleImportFile" />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { message, Modal } from 'ant-design-vue'
import { PlusOutlined, ExportOutlined, ImportOutlined, AppstoreOutlined } from '@ant-design/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { getAllLanguages } from '@/api/languages'
import { useLayoutStore } from '@/stores/layout'
import MappingDialog from './components/MappingDialog.vue'
import MappingTemplateDialog from './components/MappingTemplateDialog.vue'

const layoutStore = useLayoutStore()

// 数据库类型列表
const databaseTypes = ref([
  { key: 'mysql', label: 'MySQL', icon: '🐬' },
  { key: 'postgresql', label: 'PostgreSQL', icon: '🐘' },
  { key: 'sqlite', label: 'SQLite', icon: '📦' }
])

// 标准数据库字段定义
const p = (pattern, priority = 10) => ({ pattern, priority })
const standardDbFields = {
  mysql: [p('VARCHAR(%)'), p('CHAR(%)'), p('TEXT'), p('LONGTEXT'), p('INT'), p('BIGINT'), p('SMALLINT'), p('TINYINT(1)', 20), p('TINYINT(%)'), p('DECIMAL(%,%)'), p('FLOAT'), p('DOUBLE'), p('BOOLEAN'), p('DATE'), p('TIMESTAMP'), p('DATETIME'), p('TIME'), p('BLOB'), p('JSON')],
  postgresql: [p('VARCHAR(%)'), p('CHAR(%)'), p('TEXT'), p('INTEGER'), p('BIGINT'), p('SMALLINT'), p('BOOLEAN'), p('DECIMAL(%,%)'), p('NUMERIC(%,%)'), p('REAL'), p('DOUBLE PRECISION'), p('DATE'), p('TIMESTAMP'), p('TIMESTAMPTZ'), p('TIME'), p('BYTEA'), p('JSON'), p('JSONB'), p('UUID'), p('SERIAL'), p('BIGSERIAL')],
  sqlite: [p('INTEGER'), p('TEXT'), p('BLOB'), p('REAL'), p('NUMERIC'), p('BOOLEAN'), p('DATE'), p('DATETIME')]
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
const templateDialogVisible = ref(false)

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
  if (!lang || !lang.id) { targetTypeOptions.value = []; return }
  try {
    const { getLanguageFieldTypes } = await import('@/api/languages')
    const fieldTypes = await getLanguageFieldTypes(lang.id)
    targetTypeOptions.value = fieldTypes.map(f => ({ value: f.name, label: f.name, description: f.description }))
  } catch { targetTypeOptions.value = [] }
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
  return currentMappings.value.filter(m => m.pattern.toLowerCase().includes(search) || m.targetType.toLowerCase().includes(search))
})

// 概览数据
const overviewData = computed(() => {
  const langKey = activeLang.value, dbKey = activeDbType.value
  const dbLabel = databaseTypes.value.find(d => d.key === dbKey)?.label || dbKey
  const langName = currentLanguageName.value || '-'
  const all = mappings.value[langKey]?.[dbKey]
  if (!all) return [{ label: '当前语言', value: langName }, { label: '当前数据库', value: dbLabel || '-' }, { label: '总映射数', value: '0' }, { label: '已配置', value: '0' }, { label: '未配置', value: '0' }]
  const configured = all.filter(m => m.targetType).length
  return [{ label: '当前语言', value: langName }, { label: '当前数据库', value: dbLabel }, { label: '总映射数', value: String(all.length) }, { label: '已配置', value: String(configured) }, { label: '未配置', value: String(all.length - configured) }]
})

const updateFooterOverview = () => { layoutStore.showFooterOverview(overviewData.value) }
watch([activeLang, activeDbType, mappings], updateFooterOverview, { deep: true })

const loadMappings = async () => {
  loading.value = true
  try {
    const langData = await getAllLanguages()
    availableLanguages.value = langData.map(lang => ({ key: lang.name.toLowerCase().replace(/\s+/g, '_'), label: lang.name, id: lang.id, icon: lang.icon }))
    if (availableLanguages.value.length === 0) { availableLanguages.value = [{ key: 'empty', label: '请先添加语言', disabled: true }]; loading.value = false; return }
    if (!activeLang.value && availableLanguages.value.length > 0) activeLang.value = availableLanguages.value[0].key
    await loadLanguageFieldTypes()

    const result = await invoke('db_get_system_type_mappings')
    const loadedMappings = {}
    if (result) {
      const data = typeof result === 'string' ? JSON.parse(result) : result
      for (const m of data) {
        const langKey = m.language_name?.toLowerCase().replace(/\s+/g, '_') || m.langType
        const dbKey = m.db_type || m.dbType
        if (!langKey || !dbKey) continue
        if (!loadedMappings[langKey]) loadedMappings[langKey] = {}
        if (!loadedMappings[langKey][dbKey]) loadedMappings[langKey][dbKey] = []
        loadedMappings[langKey][dbKey].push({ id: m.id, pattern: m.pattern, targetType: m.target_type, priority: m.priority, language_id: m.language_id, dbType: dbKey, langType: langKey })
      }
    }

    for (const lang of availableLanguages.value) {
      if (lang.disabled) continue
      if (!loadedMappings[lang.key]) loadedMappings[lang.key] = {}
      for (const db of databaseTypes.value) {
        if (!loadedMappings[lang.key][db.key]) {
          loadedMappings[lang.key][db.key] = standardDbFields[db.key].map((field, i) => ({ id: `${lang.key}_${db.key}_${i}`, pattern: field.pattern, targetType: '', priority: field.priority, sort_order: i, dbType: db.key, langType: lang.key }))
        }
      }
    }
    mappings.value = loadedMappings
  } catch (error) {
    console.error('加载映射数据失败:', error)
    initDefaultMappings()
  } finally { loading.value = false }
}

const initDefaultMappings = () => {
  mappings.value = {}
  for (const lang of availableLanguages.value) {
    if (lang.disabled) continue
    mappings.value[lang.key] = {}
    for (const db of databaseTypes.value) {
      mappings.value[lang.key][db.key] = standardDbFields[db.key].map((field, i) => ({ id: `${lang.key}_${db.key}_${i}`, pattern: field.pattern, targetType: '', priority: field.priority, sort_order: i, dbType: db.key, langType: lang.key }))
    }
  }
}

// 显示添加映射对话框
const showAddMappingDialog = () => { message.info('请点击表格中的"配置"按钮来设置映射') }

// 编辑映射
const editMapping = (record) => {
  editingMapping.value = record
  mappingDialogVisible.value = true
}

// 保存映射（从 MappingDialog 回调）
const handleMappingSaved = async (targetType, priority) => {
  saving.value = true
  try {
    const langKey = activeLang.value
    const dbKey = editingMapping.value?.dbKey
    if (!langKey || !dbKey || !mappings.value[langKey] || !mappings.value[langKey][dbKey]) {
      message.error('映射数据异常'); return
    }
    const index = mappings.value[langKey][dbKey].findIndex(m => m.id === editingMapping.value.id)
    if (index !== -1) {
      mappings.value[langKey][dbKey][index] = { ...editingMapping.value, targetType, priority }
      message.success('映射已更新')
      await saveMappingsToBackend()
    }
    mappingDialogVisible.value = false
    editingMapping.value = null
  } catch (error) {
    console.error('保存映射失败:', error)
    message.error('保存失败: ' + error)
  } finally {
    saving.value = false
  }
}

// 保存映射到后端
const saveMappingsToBackend = async () => {
  try {
    const allMappings = []
    for (const langKey in mappings.value) {
      const lang = availableLanguages.value.find(l => l.key === langKey)
      const languageId = lang?.id || 0
      for (const dbKey in mappings.value[langKey]) {
        for (const mapping of mappings.value[langKey][dbKey]) {
          if (mapping.targetType) {
            allMappings.push({
              language_id: languageId, db_type: dbKey, pattern: mapping.pattern,
              target_type: mapping.targetType, priority: mapping.priority || 10
            })
          }
        }
      }
    }
    await invoke('db_batch_save_system_type_mappings', { mappings: JSON.stringify(allMappings) })
  } catch (error) {
    console.error('保存映射到后端失败:', error)
  }
}

// 应用模板
const applyTemplate = async (template) => {
  const lk = template.langType, dk = template.dbType
  if (!mappings.value[lk]) mappings.value[lk] = {}
  const tMap = Object.fromEntries(template.mappings.map(t => [t.pattern, t]))
  if (mappings.value[lk][dk]) {
    for (let i = 0; i < mappings.value[lk][dk].length; i++) {
      const existing = mappings.value[lk][dk][i]
      if (tMap[existing.pattern]) mappings.value[lk][dk][i] = { ...existing, targetType: tMap[existing.pattern].targetType, priority: tMap[existing.pattern].priority }
    }
  }
  activeDbType.value = dk; activeLang.value = lk
  await saveMappingsToBackend()
  templateDialogVisible.value = false
  message.success(`已应用模板: ${template.name}`)
}

// ===== 导出/导入 =====

const fileInputRef = ref(null)

const handleExport = async ({ key }) => {
  try {
    let exportData
    if (key === 'current') {
      const langKey = activeLang.value, dbKey = activeDbType.value
      const configured = (mappings.value[langKey]?.[dbKey] || []).filter(m => m.targetType)
      exportData = { version: '1.0', exportedAt: new Date().toISOString(), scope: 'single', langType: langKey, dbType: dbKey, mappings: configured.map(m => ({ pattern: m.pattern, targetType: m.targetType, priority: m.priority })) }
    } else {
      const allMappings = []
      for (const langKey in mappings.value) { for (const dbKey in mappings.value[langKey]) { for (const m of mappings.value[langKey][dbKey]) { if (m.targetType) allMappings.push({ langType: langKey, dbType: dbKey, pattern: m.pattern, targetType: m.targetType, priority: m.priority }) } } }
      exportData = { version: '1.0', exportedAt: new Date().toISOString(), scope: 'all', mappings: allMappings }
    }
    const filePath = await save({ defaultPath: `mappings-${key === 'current' ? `${activeLang.value}-${activeDbType.value}` : 'all'}.json`, filters: [{ name: 'JSON', extensions: ['json'] }] })
    if (filePath) { await invoke('write_text_file', { path: filePath, content: JSON.stringify(exportData, null, 2) }); message.success(`已导出 ${exportData.mappings.length} 条映射`) }
  } catch (error) { if (error !== 'cancelled') message.error('导出失败: ' + error) }
}

const triggerImport = () => { fileInputRef.value?.click() }

const handleImportFile = async (event) => {
  const file = event.target.files?.[0]
  if (!file) return
  event.target.value = ''
  try {
    const data = JSON.parse(await file.text())
    if (!data.mappings || !Array.isArray(data.mappings)) { message.error('无效的映射文件格式'); return }
    const count = data.mappings.length
    Modal.confirm({
      title: '确认导入', content: `即将导入 ${count} 条映射规则，已有相同类型的映射将被覆盖。是否继续？`, okText: '导入', cancelText: '取消',
      onOk: async () => {
        try {
          for (const item of data.mappings) {
            const lk = item.langType, dk = item.dbType
            if (!lk || !dk || !item.pattern) continue
            if (!mappings.value[lk]) mappings.value[lk] = {}
            if (!mappings.value[lk][dk]) {
              mappings.value[lk][dk] = standardDbFields[dk]?.map((f, i) => ({ id: `${lk}_${dk}_${i}`, pattern: f.pattern, targetType: '', priority: f.priority, sort_order: i, dbType: dk, langType: lk })) || []
            }
            const idx = mappings.value[lk][dk].findIndex(m => m.pattern === item.pattern)
            if (idx !== -1) mappings.value[lk][dk][idx] = { ...mappings.value[lk][dk][idx], targetType: item.targetType || '', priority: item.priority || 10 }
          }
          await saveMappingsToBackend(); message.success(`成功导入 ${count} 条映射规则`)
        } catch (e) { message.error('导入失败: ' + e) }
      }
    })
  } catch (e) { message.error('文件解析失败: ' + e) }
}

// 监听语言切换，加载对应的类型字段
watch(activeLang, () => { loadLanguageFieldTypes() })

onMounted(async () => {
  await loadMappings()
  updateFooterOverview()
})
</script>

<style scoped>
.mappings-view { padding: var(--spacing-lg); height: 100%; display: flex; flex-direction: column; background: var(--color-background); overflow: hidden; }
.page-header { margin-bottom: 24px; }
.page-header h1 { margin: 0 0 8px 0; font-size: 24px; font-weight: 600; color: var(--color-text); }
.page-desc { margin: 0; color: var(--color-text-secondary); font-size: 14px; }
.mappings-container { flex: 1; display: flex; gap: 16px; overflow: hidden; }

/* 左侧数据库 Tab */
.db-tabs { width: 180px; flex-shrink: 0; background: var(--color-surface); border-radius: 8px; border: 1px solid var(--color-border); overflow: hidden; display: flex; flex-direction: column; }
.db-tabs-header { padding: 12px 16px; font-size: 12px; font-weight: 500; color: var(--color-text-secondary); border-bottom: 1px solid var(--color-border); text-transform: uppercase; letter-spacing: 0.5px; }
.db-tab-item { display: flex; align-items: center; padding: 12px 16px; cursor: pointer; transition: all 0.2s; border-left: 3px solid transparent; }
.db-tab-item:hover { background: var(--color-hover); }
.db-tab-item.active { background: var(--color-primary-bg); border-left-color: var(--color-primary); }
.db-icon { margin-right: 10px; font-size: 16px; }
.db-label { flex: 1; font-size: 14px; color: var(--color-text); }
.db-count { font-size: 12px; color: var(--color-text-secondary); background: var(--color-bg-secondary); padding: 2px 6px; border-radius: 10px; }

/* 右侧映射内容 */
.mappings-content { flex: 1; display: flex; flex-direction: column; background: var(--color-surface); border-radius: 8px; border: 1px solid var(--color-border); overflow: hidden; }

/* 语言横向 Tab */
.lang-tabs { display: flex; align-items: center; padding: 12px 16px; gap: 8px; border-bottom: 1px solid var(--color-border); flex-wrap: wrap; }
.lang-tab-item { display: flex; align-items: center; gap: 6px; padding: 6px 14px; border-radius: 16px; cursor: pointer; font-size: 13px; color: var(--color-text-secondary); background: var(--color-bg-secondary); transition: all 0.2s; }
.lang-tab-item:hover { color: var(--color-text); background: var(--color-hover); }
.lang-tab-item.active { color: var(--color-primary); background: var(--color-primary-bg); font-weight: 500; }
.lang-count { font-size: 11px; background: var(--color-bg-base); padding: 1px 5px; border-radius: 8px; }
.lang-tab-item.active .lang-count { background: var(--color-primary); color: var(--color-canvas); }
.lang-tab-item.disabled { cursor: not-allowed; opacity: 0.6; color: var(--color-text-secondary); }
.lang-tab-item.disabled:hover { background: var(--color-bg-secondary); }
.lang-tabs-empty { color: var(--color-text-secondary); font-size: 13px; }
.lang-tabs-empty a { color: var(--color-primary); cursor: pointer; }
.lang-tabs-empty a:hover { text-decoration: underline; }

/* 映射表格 */
.mappings-table-wrapper { flex: 1; padding: 16px; overflow-y: auto; }
.table-toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.toolbar-actions { display: flex; gap: 8px; }
.type-code { font-family: 'Fira Code', 'Consolas', monospace; font-size: 13px; padding: 2px 6px; background: var(--color-bg-secondary); border-radius: 4px; color: var(--color-text); }
.type-code.target { color: var(--color-primary); background: var(--color-primary-bg); }
.type-code.target.empty { color: var(--color-text-secondary); background: var(--color-bg-secondary); font-style: italic; }

/* 表格样式 */
:deep(.ant-table) { font-size: 13px; }
:deep(.ant-table-thead > tr > th) { background: var(--color-bg-secondary); font-weight: 500; }
:deep(.ant-table-tbody > tr:hover > td) { background: var(--color-hover); }
</style>
