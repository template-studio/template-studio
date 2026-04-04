<template>
  <a-drawer
    v-model:open="visible"
    :title="`表配置 - ${table?.name || ''}`"
    :width="drawerWidth"
    :closable="true"
    :maskClosable="false"
    :mask="false"
    class="table-config-drawer"
    :wrapStyle="{ position: 'absolute', left: '60px' }"
    @close="handleClose"
  >
    <a-spin :spinning="loading">
      <!-- 动态渲染各个区块 -->
      <div class="config-sections">
        <template v-for="section in sections" :key="section.type">
          <!-- 基本信息 -->
          <BasicSection
            v-if="section.type === 'basic'"
            :section="section"
            :config="config"
            @update="updateConfig"
          />

          <!-- 字段配置 -->
          <FieldsSection
            v-if="section.type === 'fields'"
            :section="section"
            v-model:data="config.fields._data"
          />

          <!-- 生成选项 -->
          <OptionsSection
            v-if="section.type === 'options'"
            :section="section"
            :config="config"
            @update="updateConfig"
          />

          <!-- 扩展配置 -->
          <ExtraSection
            v-if="section.type === 'extra'"
            :section="section"
            :config="config"
            @update="updateConfig"
          />
        </template>
      </div>
    </a-spin>

    <!-- 底部操作栏 -->
    <template #footer>
      <div class="drawer-footer">
        <a-button @click="resetConfig">
          <template #icon><ReloadOutlined /></template>
          重置
        </a-button>
        <a-button @click="saveConfig" :loading="saving">
          <template #icon><SaveOutlined /></template>
          保存
        </a-button>
        <a-button type="primary" @click="generateCode" :loading="generating">
          <template #icon><CodeOutlined /></template>
          生成代码
        </a-button>
      </div>
    </template>
  </a-drawer>
</template>

<script setup>
import { ref, reactive, computed, watch } from 'vue'
import { message } from 'ant-design-vue'
import {
  ReloadOutlined,
  SaveOutlined,
  CodeOutlined
} from '@ant-design/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { parseConfig, createDefaultConfig } from '@/utils/configParser'
import { NamingConverter } from '@/utils/naming'
import BasicSection from './BasicSection.vue'
import FieldsSection from './FieldsSection.vue'
import OptionsSection from './OptionsSection.vue'
import ExtraSection from './ExtraSection.vue'

const props = defineProps({
  open: { type: Boolean, default: false },
  table: { type: Object, default: null }
})

const emit = defineEmits(['update:open', 'saved'])

// 双向绑定 open
const visible = computed({
  get: () => props.open,
  set: (val) => emit('update:open', val)
})

// 抽屉宽度
const drawerWidth = computed(() => 'calc(100vw - 60px)')

// 状态
const loading = ref(false)
const saving = ref(false)
const generating = ref(false)

// 配置数据
const config = reactive({})

// 解析后的区块
const sections = computed(() => parseConfig(config))

// 监听表变化
watch(() => props.table, async (newTable) => {
  if (newTable && props.open) {
    await loadConfig()
  }
}, { immediate: true })

watch(() => props.open, async (isOpen) => {
  if (isOpen && props.table) {
    await loadConfig()
  }
})

// 加载配置
const loadConfig = async () => {
  if (!props.table) return

  loading.value = true
  try {
    // 获取表字段
    const rawColumns = await invoke('db_get_table_columns', { tableId: props.table.id })
    const columns = JSON.parse(rawColumns)

    // 创建默认配置
    const defaultConfig = createDefaultConfig(props.table, columns)

    // 初始化实体名等
    initEntityName(defaultConfig, columns)

    // 尝试加载保存的配置
    try {
      const savedConfig = await invoke('get_table_gen_config', { tableId: props.table.id })
      if (savedConfig) {
        const parsed = JSON.parse(savedConfig)
        // 合并配置
        Object.assign(defaultConfig, parsed)
        // 确保字段数据完整
        if (!defaultConfig.fields?._data) {
          defaultConfig.fields = {
            _columns: defaultConfig.fields?._columns || createDefaultConfig(props.table, columns).fields._columns,
            _data: columns.map(col => initFieldData(col, defaultConfig.namingCase))
          }
        }
      }
    } catch (e) {
      // 没有保存的配置
    }

    Object.assign(config, defaultConfig)
  } catch (error) {
    console.error('加载配置失败:', error)
    message.error('加载配置失败')
  } finally {
    loading.value = false
  }
}

// 初始化实体名
const initEntityName = (cfg, columns) => {
  const tableName = props.table.name
  // 提取表前缀
  const prefixMatch = tableName.match(/^(sys_|t_|tb_|app_|biz_)/i)
  const prefix = prefixMatch ? prefixMatch[1] : ''

  cfg.tablePrefix = prefix
  cfg.entityName = NamingConverter.convert(
    NamingConverter.removePrefix(tableName, prefix),
    'PascalCase'
  )
  cfg.entityComment = props.table.comment || ''
  cfg.businessName = NamingConverter.convert(
    NamingConverter.removePrefix(tableName, prefix),
    'camelCase'
  )

  // 初始化字段
  cfg.fields._data = columns.map(col => initFieldData(col, cfg.namingCase))
}

// 初始化单个字段
const initFieldData = (col, namingCase) => {
  return {
    name: col.name,
    field: NamingConverter.convert(col.name, namingCase),
    type: guessFieldType(col.data_type),
    label: col.comment || col.name,
    input: guessInputType(col.data_type),
    list: !col.is_primary_key,
    form: !col.is_primary_key && !isTimestampField(col.name),
    query: false,
    required: !col.is_nullable,
    dict: ''
  }
}

// 猜测字段类型
const guessFieldType = (dbType) => {
  const baseType = (dbType || '').replace(/\([^)]*\)/, '').toUpperCase()
  const typeMap = {
    'VARCHAR': 'String', 'CHAR': 'String', 'TEXT': 'String',
    'INT': 'Integer', 'BIGINT': 'Long', 'TINYINT': 'Integer',
    'FLOAT': 'Float', 'DOUBLE': 'Double', 'DECIMAL': 'BigDecimal',
    'DATE': 'Date', 'DATETIME': 'DateTime', 'TIMESTAMP': 'DateTime',
    'BOOLEAN': 'Boolean'
  }
  return typeMap[baseType] || 'String'
}

// 猜测输入类型
const guessInputType = (dbType) => {
  const baseType = (dbType || '').replace(/\([^)]*\)/, '').toUpperCase()
  if (baseType.includes('TEXT')) return 'textarea'
  if (baseType.includes('DATE') || baseType.includes('TIME')) return 'datetime'
  return 'text'
}

// 是否时间戳字段
const isTimestampField = (name) => {
  const lowerName = name.toLowerCase()
  return ['created_at', 'updated_at', 'deleted_at', 'create_time', 'update_time'].includes(lowerName)
}

// 更新配置
const updateConfig = (key, value) => {
  if (key.includes('.')) {
    // 嵌套路径
    const parts = key.split('.')
    let obj = config
    for (let i = 0; i < parts.length - 1; i++) {
      obj = obj[parts[i]]
    }
    obj[parts[parts.length - 1]] = value
  } else {
    config[key] = value
  }
}

// 保存配置
const saveConfig = async () => {
  saving.value = true
  try {
    await invoke('save_table_gen_config', {
      tableId: props.table.id,
      config: JSON.stringify(config)
    })
    message.success('配置保存成功')
    emit('saved')
  } catch (error) {
    message.error('保存失败: ' + error)
  } finally {
    saving.value = false
  }
}

// 重置配置
const resetConfig = async () => {
  await loadConfig()
  message.info('配置已重置')
}

// 生成代码
const generateCode = async () => {
  await saveConfig()
  generating.value = true
  try {
    await invoke('generate_code', { tableId: props.table.id })
    message.success('代码生成成功')
  } catch (error) {
    message.error('生成失败: ' + error)
  } finally {
    generating.value = false
  }
}

// 关闭
const handleClose = () => {
  emit('update:open', false)
}
</script>

<style scoped>
.config-sections {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.drawer-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
</style>

<style>
/* 全局样式 */
.table-config-drawer .ant-drawer-content-wrapper {
  position: absolute !important;
  left: 60px !important;
  height: 100vh !important;
}

.table-config-drawer .ant-drawer-body {
  padding: 16px 24px;
  height: calc(100vh - 110px);
  overflow-y: auto;
}
</style>
