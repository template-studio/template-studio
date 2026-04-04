<template>
  <div class="preferences-manager">
    <!-- 左侧导航 -->
    <div class="sidebar-nav">
      <div
        v-for="tab in mainTabs"
        :key="tab.key"
        :class="['nav-item', { active: activeMainTab === tab.key }]"
        @click="activeMainTab = tab.key"
      >
        <component :is="tab.icon" class="nav-icon" />
        <span class="nav-label">{{ tab.label }}</span>
      </div>
    </div>

    <!-- 右侧内容区 -->
    <div class="content-area">
      <!-- 字段规范 -->
      <div v-show="activeMainTab === 'fields'" class="setting-container">
        <!-- 内层 Tab -->
        <div class="inner-tabs">
          <div
            v-for="tab in fieldTabs"
            :key="tab.key"
            :class="['inner-tab-item', { active: activeFieldTab === tab.key }]"
            @click="activeFieldTab = tab.key"
          >
            {{ tab.label }}
          </div>
        </div>

        <!-- 主键规范 -->
        <div v-show="activeFieldTab === 'pk'" class="config-section">
          <div class="section-header">
            <h3>主键规范</h3>
            <a-switch v-model:checked="tableConfig.pkEnabled" size="small" />
          </div>
          <a-divider />
          <template v-if="tableConfig.pkEnabled">
            <div class="setting-row">
              <div class="setting-row-title">字段名</div>
              <a-input v-model:value="tableConfig.pkFieldName" style="width: 180px" />
            </div>
            <div class="setting-divider"></div>
            <div class="setting-row">
              <div class="setting-row-title">字段类型</div>
              <a-select v-model:value="tableConfig.pkFieldType" style="width: 180px">
                <a-select-option value="INT">INT</a-select-option>
                <a-select-option value="BIGINT">BIGINT（推荐）</a-select-option>
                <a-select-option value="CHAR(36)">CHAR(36) - UUID</a-select-option>
                <a-select-option value="VARCHAR(32)">VARCHAR(32) - 雪花ID</a-select-option>
              </a-select>
            </div>
            <div class="setting-divider"></div>
            <div class="setting-row">
              <div class="setting-row-title">
                <div>自增 (AUTO_INCREMENT)</div>
                <div class="setting-description">主键自动递增</div>
              </div>
              <a-switch v-model:checked="tableConfig.pkAutoIncrement" size="small" />
            </div>
            <div class="setting-divider"></div>
            <div class="setting-row">
              <div class="setting-row-title">字段注释</div>
              <a-input v-model:value="tableConfig.pkComment" style="width: 180px" />
            </div>
          </template>
          <div v-else class="empty-tip">主键规范已禁用，新建表时不会自动添加主键字段</div>
        </div>

        <!-- 审计字段 -->
        <div v-show="activeFieldTab === 'audit'" class="config-section">
          <div class="section-header">
            <h3>审计字段</h3>
            <a-switch v-model:checked="tableConfig.auditEnabled" size="small" />
          </div>
          <a-divider />
          <template v-if="tableConfig.auditEnabled">
            <a-table
              :columns="auditColumns"
              :data-source="auditData"
              :pagination="false"
              size="small"
              bordered
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.key === 'enabled'">
                  <a-switch v-model:checked="record.enabled" size="small" />
                </template>
                <template v-else-if="column.key === 'fieldName'">
                  <a-input v-model:value="record.fieldName" size="small" />
                </template>
                <template v-else-if="column.key === 'fieldType'">
                  <a-select v-model:value="record.fieldType" size="small" style="width: 100%">
                    <a-select-option v-for="t in getTypeOptions(record.key)" :key="t" :value="t">{{ t }}</a-select-option>
                  </a-select>
                </template>
                <template v-else-if="column.key === 'comment'">
                  <a-input v-model:value="record.comment" size="small" />
                </template>
              </template>
            </a-table>
          </template>
          <div v-else class="empty-tip">审计字段已禁用，新建表时不会自动添加创建时间、更新时间等字段</div>
        </div>

        <!-- 软删除 -->
        <div v-show="activeFieldTab === 'softDelete'" class="config-section">
          <div class="section-header">
            <h3>软删除字段</h3>
            <a-switch v-model:checked="tableConfig.softDeleteEnabled" size="small" />
          </div>
          <a-divider />
          <template v-if="tableConfig.softDeleteEnabled">
            <div class="setting-row">
              <div class="setting-row-title">
                <div>启用软删除</div>
                <div class="setting-description">删除操作会更新此字段而非物理删除</div>
              </div>
            </div>
            <div class="setting-divider"></div>
            <div class="setting-row">
              <div class="setting-row-title">字段名</div>
              <a-input v-model:value="tableConfig.softDeleteField" style="width: 180px" />
            </div>
            <div class="setting-divider"></div>
            <div class="setting-row">
              <div class="setting-row-title">字段类型</div>
              <a-select v-model:value="tableConfig.softDeleteFieldType" style="width: 180px">
                <a-select-option value="TIMESTAMP">TIMESTAMP</a-select-option>
                <a-select-option value="DATETIME">DATETIME</a-select-option>
                <a-select-option value="BIGINT">BIGINT</a-select-option>
              </a-select>
            </div>
            <div class="setting-divider"></div>
            <div class="setting-row">
              <div class="setting-row-title">
                <div>允许空值</div>
                <div class="setting-description">NULL 表示未删除（建议勾选）</div>
              </div>
              <a-switch v-model:checked="tableConfig.softDeleteNullable" size="small" />
            </div>
            <div class="setting-divider"></div>
            <div class="setting-row">
              <div class="setting-row-title">字段注释</div>
              <a-input v-model:value="tableConfig.softDeleteComment" style="width: 180px" />
            </div>
          </template>
          <div v-else class="empty-tip">软删除已禁用，删除操作将物理删除数据</div>
        </div>

        <!-- 底部操作栏 -->
        <div class="action-bar">
          <a-button @click="resetFieldPreferences">重置为默认</a-button>
          <a-button type="primary" @click="savePreferences" :loading="saving">
            <template #icon><SaveOutlined /></template>
            保存配置
          </a-button>
        </div>
      </div>

      <!-- 命名规范 -->
      <div v-show="activeMainTab === 'naming'" class="setting-container">
        <div class="page-header">
          <h2>命名规范</h2>
          <p class="page-desc">配置字段命名的前缀和后缀规范，保持代码风格统一</p>
        </div>

        <div class="setting-group">
          <div class="setting-title">命名约定</div>
          <div class="setting-row">
            <div class="setting-row-title">
              <div>布尔字段前缀</div>
              <div class="setting-description">示例: is_active, is_deleted, is_verified</div>
            </div>
            <a-input v-model:value="tableConfig.booleanPrefix" style="width: 120px" />
          </div>
          <div class="setting-divider"></div>
          <div class="setting-row">
            <div class="setting-row-title">
              <div>时间字段后缀</div>
              <div class="setting-description">示例: created_at, updated_at, deleted_at</div>
            </div>
            <a-input v-model:value="tableConfig.datetimeSuffix" style="width: 120px" />
          </div>
        </div>

        <div class="action-bar">
          <a-button @click="resetNamingPreferences">重置为默认</a-button>
          <a-button type="primary" @click="savePreferences" :loading="saving">
            <template #icon><SaveOutlined /></template>
            保存配置
          </a-button>
        </div>
      </div>

      <!-- 存储配置 -->
      <div v-show="activeMainTab === 'storage'" class="setting-container">
        <div class="page-header">
          <h2>存储配置</h2>
          <p class="page-desc">配置数据库存储引擎、字符集等，新建表时自动应用</p>
        </div>

        <div class="setting-group">
          <div class="setting-title">数据库配置</div>
          <div class="setting-row">
            <div class="setting-row-title">存储引擎</div>
            <a-select v-model:value="tableConfig.engineType" style="width: 200px">
              <a-select-option value="InnoDB">InnoDB（推荐，支持事务）</a-select-option>
              <a-select-option value="MyISAM">MyISAM（不支持事务）</a-select-option>
            </a-select>
          </div>
          <div class="setting-divider"></div>
          <div class="setting-row">
            <div class="setting-row-title">字符集</div>
            <a-select v-model:value="tableConfig.charset" style="width: 200px">
              <a-select-option value="utf8mb4">utf8mb4（推荐，支持 emoji）</a-select-option>
              <a-select-option value="utf8">utf8</a-select-option>
            </a-select>
          </div>
          <div class="setting-divider"></div>
          <div class="setting-row">
            <div class="setting-row-title">排序规则</div>
            <a-select v-model:value="tableConfig.collation" style="width: 200px">
              <a-select-option value="utf8mb4_unicode_ci">utf8mb4_unicode_ci</a-select-option>
              <a-select-option value="utf8mb4_general_ci">utf8mb4_general_ci</a-select-option>
            </a-select>
          </div>
          <div class="setting-divider"></div>
          <div class="setting-row">
            <div class="setting-row-title">行格式</div>
            <a-select v-model:value="tableConfig.rowFormat" style="width: 200px">
              <a-select-option value="DYNAMIC">DYNAMIC</a-select-option>
              <a-select-option value="COMPRESSED">COMPRESSED</a-select-option>
            </a-select>
          </div>
        </div>

        <div class="action-bar">
          <a-button @click="resetStoragePreferences">重置为默认</a-button>
          <a-button type="primary" @click="savePreferences" :loading="saving">
            <template #icon><SaveOutlined /></template>
            保存配置
          </a-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { SaveOutlined, TableOutlined, FontSizeOutlined, DatabaseOutlined } from '@ant-design/icons-vue'

const props = defineProps({
  projectId: {
    type: Number,
    required: true
  }
})

const emit = defineEmits(['save'])

// Tab 状态
const activeMainTab = ref('fields')
const activeFieldTab = ref('pk')
const saving = ref(false)

// 外层 Tab 配置
const mainTabs = [
  { key: 'fields', icon: TableOutlined, label: '字段规范' },
  { key: 'naming', icon: FontSizeOutlined, label: '命名规范' },
  { key: 'storage', icon: DatabaseOutlined, label: '存储配置' }
]

// 内层 Tab 配置（字段规范）
const fieldTabs = [
  { key: 'pk', label: '主键规范' },
  { key: 'audit', label: '审计字段' },
  { key: 'softDelete', label: '软删除' }
]

// 表规范配置
const tableConfig = reactive({
  // 主键规范
  pkEnabled: true,
  pkFieldName: 'id',
  pkFieldType: 'BIGINT',
  pkAutoIncrement: true,
  pkComment: '主键ID',

  // 审计字段
  auditEnabled: true,
  auditFields: {
    createdAt: { enabled: true, fieldName: 'created_at', fieldType: 'TIMESTAMP', comment: '创建时间' },
    updatedAt: { enabled: true, fieldName: 'updated_at', fieldType: 'TIMESTAMP', comment: '更新时间' },
    createdBy: { enabled: false, fieldName: 'created_by', fieldType: 'BIGINT', comment: '创建人ID' },
    updatedBy: { enabled: false, fieldName: 'updated_by', fieldType: 'BIGINT', comment: '更新人ID' }
  },

  // 软删除字段
  softDeleteEnabled: false,
  softDeleteField: 'deleted_at',
  softDeleteFieldType: 'TIMESTAMP',
  softDeleteNullable: true,
  softDeleteComment: '删除时间，NULL表示未删除',

  // 命名规范
  booleanPrefix: 'is_',
  datetimeSuffix: '_at',

  // 存储配置
  engineType: 'InnoDB',
  charset: 'utf8mb4',
  collation: 'utf8mb4_unicode_ci',
  rowFormat: 'DYNAMIC'
})

// 审计字段表格列定义
const auditColumns = [
  { title: '启用', key: 'enabled', width: 60, align: 'center' },
  { title: '字段', dataIndex: 'label', width: 100 },
  { title: '字段名', key: 'fieldName', width: 120 },
  { title: '类型', key: 'fieldType', width: 120 },
  { title: '注释', key: 'comment' }
]

// 审计字段表格数据（使用 computed 实现双向绑定）
const auditData = computed(() => [
  { key: 'createdAt', label: '创建时间', get enabled() { return tableConfig.auditFields.createdAt.enabled }, set enabled(v) { tableConfig.auditFields.createdAt.enabled = v }, get fieldName() { return tableConfig.auditFields.createdAt.fieldName }, set fieldName(v) { tableConfig.auditFields.createdAt.fieldName = v }, get fieldType() { return tableConfig.auditFields.createdAt.fieldType }, set fieldType(v) { tableConfig.auditFields.createdAt.fieldType = v }, get comment() { return tableConfig.auditFields.createdAt.comment }, set comment(v) { tableConfig.auditFields.createdAt.comment = v } },
  { key: 'updatedAt', label: '更新时间', get enabled() { return tableConfig.auditFields.updatedAt.enabled }, set enabled(v) { tableConfig.auditFields.updatedAt.enabled = v }, get fieldName() { return tableConfig.auditFields.updatedAt.fieldName }, set fieldName(v) { tableConfig.auditFields.updatedAt.fieldName = v }, get fieldType() { return tableConfig.auditFields.updatedAt.fieldType }, set fieldType(v) { tableConfig.auditFields.updatedAt.fieldType = v }, get comment() { return tableConfig.auditFields.updatedAt.comment }, set comment(v) { tableConfig.auditFields.updatedAt.comment = v } },
  { key: 'createdBy', label: '创建人', get enabled() { return tableConfig.auditFields.createdBy.enabled }, set enabled(v) { tableConfig.auditFields.createdBy.enabled = v }, get fieldName() { return tableConfig.auditFields.createdBy.fieldName }, set fieldName(v) { tableConfig.auditFields.createdBy.fieldName = v }, get fieldType() { return tableConfig.auditFields.createdBy.fieldType }, set fieldType(v) { tableConfig.auditFields.createdBy.fieldType = v }, get comment() { return tableConfig.auditFields.createdBy.comment }, set comment(v) { tableConfig.auditFields.createdBy.comment = v } },
  { key: 'updatedBy', label: '更新人', get enabled() { return tableConfig.auditFields.updatedBy.enabled }, set enabled(v) { tableConfig.auditFields.updatedBy.enabled = v }, get fieldName() { return tableConfig.auditFields.updatedBy.fieldName }, set fieldName(v) { tableConfig.auditFields.updatedBy.fieldName = v }, get fieldType() { return tableConfig.auditFields.updatedBy.fieldType }, set fieldType(v) { tableConfig.auditFields.updatedBy.fieldType = v }, get comment() { return tableConfig.auditFields.updatedBy.comment }, set comment(v) { tableConfig.auditFields.updatedBy.comment = v } }
])

// 获取类型选项
const getTypeOptions = (key) => {
  if (key === 'createdAt' || key === 'updatedAt') {
    return ['TIMESTAMP', 'DATETIME']
  }
  return ['BIGINT', 'INT', 'VARCHAR(64)']
}

// 默认配置
const defaultConfig = {
  pkEnabled: true,
  pkFieldName: 'id',
  pkFieldType: 'BIGINT',
  pkAutoIncrement: true,
  pkComment: '主键ID',
  auditEnabled: true,
  auditFields: {
    createdAt: { enabled: true, fieldName: 'created_at', fieldType: 'TIMESTAMP', comment: '创建时间' },
    updatedAt: { enabled: true, fieldName: 'updated_at', fieldType: 'TIMESTAMP', comment: '更新时间' },
    createdBy: { enabled: false, fieldName: 'created_by', fieldType: 'BIGINT', comment: '创建人ID' },
    updatedBy: { enabled: false, fieldName: 'updated_by', fieldType: 'BIGINT', comment: '更新人ID' }
  },
  softDeleteEnabled: false,
  softDeleteField: 'deleted_at',
  softDeleteFieldType: 'TIMESTAMP',
  softDeleteNullable: true,
  softDeleteComment: '删除时间，NULL表示未删除',
  booleanPrefix: 'is_',
  datetimeSuffix: '_at',
  engineType: 'InnoDB',
  charset: 'utf8mb4',
  collation: 'utf8mb4_unicode_ci',
  rowFormat: 'DYNAMIC'
}

// 加载配置
const loadPreferences = async () => {
  try {
    const result = await invoke('db_get_table_preferences', { projectId: props.projectId })
    if (result) {
      const parsed = typeof result === 'string' ? JSON.parse(result) : result
      // 只更新存在的字段，保留默认值
      if (parsed.pkEnabled !== undefined) tableConfig.pkEnabled = parsed.pkEnabled
      if (parsed.pkFieldName) tableConfig.pkFieldName = parsed.pkFieldName
      if (parsed.pkFieldType) tableConfig.pkFieldType = parsed.pkFieldType
      if (parsed.pkAutoIncrement !== undefined) tableConfig.pkAutoIncrement = parsed.pkAutoIncrement
      if (parsed.pkComment) tableConfig.pkComment = parsed.pkComment
      if (parsed.auditEnabled !== undefined) tableConfig.auditEnabled = parsed.auditEnabled
      if (parsed.softDeleteEnabled !== undefined) tableConfig.softDeleteEnabled = parsed.softDeleteEnabled
      if (parsed.softDeleteField) tableConfig.softDeleteField = parsed.softDeleteField
      if (parsed.softDeleteFieldType) tableConfig.softDeleteFieldType = parsed.softDeleteFieldType
      if (parsed.softDeleteNullable !== undefined) tableConfig.softDeleteNullable = parsed.softDeleteNullable
      if (parsed.softDeleteComment) tableConfig.softDeleteComment = parsed.softDeleteComment
      if (parsed.booleanPrefix) tableConfig.booleanPrefix = parsed.booleanPrefix
      if (parsed.datetimeSuffix) tableConfig.datetimeSuffix = parsed.datetimeSuffix
      if (parsed.engineType) tableConfig.engineType = parsed.engineType
      if (parsed.charset) tableConfig.charset = parsed.charset
      if (parsed.collation) tableConfig.collation = parsed.collation
      if (parsed.rowFormat) tableConfig.rowFormat = parsed.rowFormat
      // 处理审计字段（如果是对象格式）
      if (parsed.auditFields && typeof parsed.auditFields === 'object') {
        if (parsed.auditFields.createdAt) {
          Object.assign(tableConfig.auditFields.createdAt, parsed.auditFields.createdAt)
        }
        if (parsed.auditFields.updatedAt) {
          Object.assign(tableConfig.auditFields.updatedAt, parsed.auditFields.updatedAt)
        }
        if (parsed.auditFields.createdBy) {
          Object.assign(tableConfig.auditFields.createdBy, parsed.auditFields.createdBy)
        }
        if (parsed.auditFields.updatedBy) {
          Object.assign(tableConfig.auditFields.updatedBy, parsed.auditFields.updatedBy)
        }
      }
    }
  } catch (error) {
    console.error('加载配置失败:', error)
  }
}

// 保存配置
const savePreferences = async () => {
  saving.value = true
  try {
    await invoke('db_save_table_preferences', {
      projectId: props.projectId,
      preferences: JSON.parse(JSON.stringify(tableConfig))
    })
    message.success('配置保存成功')
    emit('save')
  } catch (error) {
    console.error('保存配置失败:', error)
    message.error('保存失败: ' + error)
  } finally {
    saving.value = false
  }
}

// 重置字段规范
const resetFieldPreferences = () => {
  tableConfig.pkEnabled = defaultConfig.pkEnabled
  tableConfig.pkFieldName = defaultConfig.pkFieldName
  tableConfig.pkFieldType = defaultConfig.pkFieldType
  tableConfig.pkAutoIncrement = defaultConfig.pkAutoIncrement
  tableConfig.pkComment = defaultConfig.pkComment
  tableConfig.auditEnabled = defaultConfig.auditEnabled
  Object.assign(tableConfig.auditFields.createdAt, defaultConfig.auditFields.createdAt)
  Object.assign(tableConfig.auditFields.updatedAt, defaultConfig.auditFields.updatedAt)
  Object.assign(tableConfig.auditFields.createdBy, defaultConfig.auditFields.createdBy)
  Object.assign(tableConfig.auditFields.updatedBy, defaultConfig.auditFields.updatedBy)
  tableConfig.softDeleteEnabled = defaultConfig.softDeleteEnabled
  tableConfig.softDeleteField = defaultConfig.softDeleteField
  tableConfig.softDeleteFieldType = defaultConfig.softDeleteFieldType
  tableConfig.softDeleteNullable = defaultConfig.softDeleteNullable
  tableConfig.softDeleteComment = defaultConfig.softDeleteComment
  message.info('已重置为默认配置')
}

// 重置命名规范
const resetNamingPreferences = () => {
  tableConfig.booleanPrefix = defaultConfig.booleanPrefix
  tableConfig.datetimeSuffix = defaultConfig.datetimeSuffix
  message.info('已重置为默认配置')
}

// 重置存储配置
const resetStoragePreferences = () => {
  tableConfig.engineType = defaultConfig.engineType
  tableConfig.charset = defaultConfig.charset
  tableConfig.collation = defaultConfig.collation
  tableConfig.rowFormat = defaultConfig.rowFormat
  message.info('已重置为默认配置')
}

onMounted(() => {
  loadPreferences()
})

defineExpose({ savePreferences })
</script>

<style scoped>
@import '@/assets/styles/settings.css';

.preferences-manager {
  display: flex;
  height: 100%;
  overflow: hidden;
}

/* 左侧导航 */
.sidebar-nav {
  width: 160px;
  padding: 16px 8px;
  border-right: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  flex-shrink: 0;
}

.nav-item {
  display: flex;
  align-items: center;
  padding: 10px 12px;
  margin-bottom: 4px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--color-text);
}

.nav-item:hover {
  background: var(--color-surface);
}

.nav-item.active {
  background: var(--color-primary-bg);
  color: var(--color-primary);
  font-weight: 500;
}

.nav-icon {
  margin-right: 8px;
  font-size: 16px;
}

.nav-label {
  font-size: 13px;
}

/* 右侧内容区 */
.content-area {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.setting-container {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  padding-top: 15px;
}

/* 内层 Tab */
.inner-tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 16px;
  padding: 4px;
  background: var(--color-bg-secondary);
  border-radius: 6px;
}

.inner-tab-item {
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  color: var(--color-text-secondary);
  transition: all 0.2s;
}

.inner-tab-item:hover {
  color: var(--color-text);
}

.inner-tab-item.active {
  background: var(--color-surface);
  color: var(--color-primary);
  font-weight: 500;
}

/* 配置区块 */
.config-section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.section-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

.config-section .ant-divider {
  margin: 12px 0;
}

/* 空状态提示 */
.empty-tip {
  color: var(--color-text-secondary);
  font-size: 13px;
  padding: 12px 0;
}

/* 页面标题 */
.page-header {
  margin-bottom: 20px;
}

.page-header h2 {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
}

.page-desc {
  margin: 0;
  font-size: 13px;
  color: var(--color-text-secondary);
}

/* 底部操作栏 */
.action-bar {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 0;
  margin-top: 16px;
  border-top: 1px solid var(--color-border);
}

/* 表格样式 */
:deep(.ant-table) {
  font-size: 13px;
}

:deep(.ant-table-thead > tr > th) {
  background: var(--color-bg-secondary);
  font-weight: 500;
}

:deep(.ant-table-tbody > tr > td) {
  padding: 8px 12px;
}
</style>
