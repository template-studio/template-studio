<template>
  <div class="preferences-manager">
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

    <div class="content-area">
      <div v-show="activeMainTab === 'fields'" class="setting-container">
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

        <PrimaryKeyConfig v-show="activeFieldTab === 'pk'" :table-config="tableConfig" />
        <AuditFieldsConfig v-show="activeFieldTab === 'audit'" :table-config="tableConfig" />
        <SoftDeleteConfig v-show="activeFieldTab === 'softDelete'" :table-config="tableConfig" />

        <div class="action-bar">
          <a-button @click="resetFieldPreferences">重置为默认</a-button>
          <a-button type="primary" @click="savePreferences" :loading="saving">
            <template #icon><SaveOutlined /></template>
            保存配置
          </a-button>
        </div>
      </div>

      <NamingConventionConfig
        v-show="activeMainTab === 'naming'"
        :table-config="tableConfig"
        :saving="saving"
        @reset="resetNamingPreferences"
        @save="savePreferences"
      />

      <StorageConfig
        v-show="activeMainTab === 'storage'"
        :table-config="tableConfig"
        :saving="saving"
        @reset="resetStoragePreferences"
        @save="savePreferences"
      />
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { SaveOutlined, TableOutlined, FontSizeOutlined, DatabaseOutlined } from '@ant-design/icons-vue'
import PrimaryKeyConfig from './preferences/PrimaryKeyConfig.vue'
import AuditFieldsConfig from './preferences/AuditFieldsConfig.vue'
import SoftDeleteConfig from './preferences/SoftDeleteConfig.vue'
import NamingConventionConfig from './preferences/NamingConventionConfig.vue'
import StorageConfig from './preferences/StorageConfig.vue'

const props = defineProps({
  projectId: {
    type: Number,
    required: true
  }
})

const emit = defineEmits(['save'])

const activeMainTab = ref('fields')
const activeFieldTab = ref('pk')
const saving = ref(false)

const mainTabs = [
  { key: 'fields', icon: TableOutlined, label: '字段规范' },
  { key: 'naming', icon: FontSizeOutlined, label: '命名规范' },
  { key: 'storage', icon: DatabaseOutlined, label: '存储配置' }
]

const fieldTabs = [
  { key: 'pk', label: '主键规范' },
  { key: 'audit', label: '审计字段' },
  { key: 'softDelete', label: '软删除' }
]

const tableConfig = reactive({
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
})

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

const loadPreferences = async () => {
  try {
    const result = await invoke('db_get_table_preferences', { projectId: props.projectId })
    if (result) {
      const parsed = typeof result === 'string' ? JSON.parse(result) : result
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

const resetNamingPreferences = () => {
  tableConfig.booleanPrefix = defaultConfig.booleanPrefix
  tableConfig.datetimeSuffix = defaultConfig.datetimeSuffix
  message.info('已重置为默认配置')
}

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

.action-bar {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 0;
  margin-top: 16px;
  border-top: 1px solid var(--color-border);
}
</style>
