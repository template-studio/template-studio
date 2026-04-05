<template>
  <a-modal
    v-model:open="modalVisible"
    :title="language ? `${language.name} - 类型字段管理` : '类型字段管理'"
    width="680px"
    :footer="null"
    @cancel="closeDialog"
  >
    <div class="field-types-container">
      <!-- 工具栏 -->
      <div class="field-types-toolbar">
        <a-button type="primary" size="small" @click="addNewFieldTypeRow">
          <template #icon><PlusOutlined /></template>
          添加
        </a-button>
        <a-button size="small" @click="resetToDefaults" :disabled="!language">
          <template #icon><ReloadOutlined /></template>
          重置
        </a-button>
      </div>

      <!-- 类型字段列表（行内编辑） -->
      <a-table
        :columns="fieldTypeColumns"
        :data-source="editableFieldTypes"
        :pagination="{ pageSize: 10 }"
        :loading="fieldTypeLoading"
        size="small"
        row-key="id"
        class="field-types-table"
        :row-class-name="(record) => record.editing ? 'editing-row' : ''"
      >
        <template #bodyCell="{ column, record }">
          <!-- 类型名称列 -->
          <template v-if="column.key === 'name'">
            <div v-if="record.editing" class="editable-cell">
              <a-input
                v-model:value="record.editName"
                placeholder="类型名称"
                size="small"
                @keyup.enter="saveFieldTypeRow(record)"
              />
            </div>
            <div v-else class="cell-content">
              <code class="field-type-name">{{ record.name }}</code>
            </div>
          </template>

          <!-- 描述列 -->
          <template v-else-if="column.key === 'description'">
            <div v-if="record.editing" class="editable-cell">
              <a-input
                v-model:value="record.editDescription"
                placeholder="描述（可选）"
                size="small"
                @keyup.enter="saveFieldTypeRow(record)"
              />
            </div>
            <span v-else class="field-type-desc">{{ record.description || '-' }}</span>
          </template>

          <!-- 操作列 -->
          <template v-else-if="column.key === 'action'">
            <div v-if="record.editing" class="action-buttons">
              <a-button type="link" size="small" @click="saveFieldTypeRow(record)">
                <CheckOutlined />
              </a-button>
              <a-button type="link" size="small" @click="cancelEditFieldTypeRow(record)">
                <CloseOutlined />
              </a-button>
            </div>
            <div v-else class="action-buttons">
              <a-button type="link" size="small" @click="startEditFieldTypeRow(record)">
                <EditOutlined />
              </a-button>
              <a-popconfirm
                title="确定删除此类型字段？"
                ok-text="确定"
                cancel-text="取消"
                @confirm="deleteFieldType(record)"
              >
                <a-button type="link" size="small" danger>
                  <DeleteOutlined />
                </a-button>
              </a-popconfirm>
            </div>
          </template>
        </template>
      </a-table>
    </div>
  </a-modal>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  CheckOutlined,
  CloseOutlined,
  ReloadOutlined
} from '@ant-design/icons-vue'
import { message, Modal } from 'ant-design-vue'
import {
  getLanguageFieldTypes,
  createLanguageFieldType,
  updateLanguageFieldType,
  deleteLanguageFieldType,
  batchSaveLanguageFieldTypes
} from '@/api/languages'

const props = defineProps({
  open: { type: Boolean, default: false },
  language: { type: Object, default: null }
})

const emit = defineEmits(['update:open'])

// 对话框可见性
const modalVisible = computed({
  get: () => props.open,
  set: (val) => emit('update:open', val)
})

// 状态
const fieldTypes = ref([])
const fieldTypeLoading = ref(false)

// 类型字段表格列定义
const fieldTypeColumns = [
  { title: '类型名称', key: 'name', width: 200 },
  { title: '描述', key: 'description' },
  { title: '操作', key: 'action', width: 100, align: 'center' }
]

// 可编辑的类型字段列表
const editableFieldTypes = computed(() => fieldTypes.value)

// 监听打开事件
watch(() => props.open, (val) => {
  if (val && props.language) {
    loadFieldTypes(props.language.id)
  }
})

// 加载类型字段列表
const loadFieldTypes = async (languageId) => {
  fieldTypeLoading.value = true
  try {
    const data = await getLanguageFieldTypes(languageId)
    if (data && data.length > 0) {
      fieldTypes.value = data.map(item => ({
        ...item,
        editing: false,
        editName: item.name,
        editDescription: item.description || ''
      }))
    } else {
      const defaults = getDefaultFieldTypes()
      await batchSaveLanguageFieldTypes(languageId, defaults)
      fieldTypes.value = defaults
    }
  } catch (error) {
    message.error('加载类型字段失败: ' + error)
    fieldTypes.value = getDefaultFieldTypes()
  } finally {
    fieldTypeLoading.value = false
  }
}

// 获取默认类型字段
const getDefaultFieldTypes = () => {
  const langName = props.language?.name?.toLowerCase() || ''

  const createField = (id, name, description, isBuiltin = true) => ({
    id, name, description, isBuiltin,
    editing: false, editName: name, editDescription: description || ''
  })

  if (langName.includes('java')) {
    return [
      createField(1, 'String', '字符串类型'),
      createField(2, 'Integer', '整型'),
      createField(3, 'Long', '长整型'),
      createField(4, 'Short', '短整型'),
      createField(5, 'Boolean', '布尔类型'),
      createField(6, 'BigDecimal', '高精度小数'),
      createField(7, 'Float', '单精度浮点'),
      createField(8, 'Double', '双精度浮点'),
      createField(9, 'LocalDate', '日期类型'),
      createField(10, 'LocalDateTime', '日期时间类型'),
      createField(11, 'LocalTime', '时间类型'),
      createField(12, 'byte[]', '字节数组')
    ]
  } else if (langName.includes('go') || langName.includes('golang')) {
    return [
      createField(1, 'string', '字符串类型'),
      createField(2, 'int', '整型'),
      createField(3, 'int8', '8位整型'),
      createField(4, 'int16', '16位整型'),
      createField(5, 'int32', '32位整型'),
      createField(6, 'int64', '64位整型'),
      createField(7, 'uint', '无符号整型'),
      createField(8, 'bool', '布尔类型'),
      createField(9, 'float32', '单精度浮点'),
      createField(10, 'float64', '双精度浮点'),
      createField(11, 'time.Time', '时间类型'),
      createField(12, '[]byte', '字节切片')
    ]
  } else if (langName.includes('typescript') || langName.includes('javascript')) {
    return [
      createField(1, 'string', '字符串类型'),
      createField(2, 'number', '数字类型'),
      createField(3, 'boolean', '布尔类型'),
      createField(4, 'Date', '日期类型'),
      createField(5, 'Array', '数组类型'),
      createField(6, 'object', '对象类型'),
      createField(7, 'any', '任意类型'),
      createField(8, 'Buffer', '缓冲区类型')
    ]
  } else if (langName.includes('rust')) {
    return [
      createField(1, 'String', '字符串类型'),
      createField(2, 'i8', '8位有符号整数'),
      createField(3, 'i16', '16位有符号整数'),
      createField(4, 'i32', '32位有符号整数'),
      createField(5, 'i64', '64位有符号整数'),
      createField(6, 'u8', '8位无符号整数'),
      createField(7, 'u16', '16位无符号整数'),
      createField(8, 'u32', '32位无符号整数'),
      createField(9, 'u64', '64位无符号整数'),
      createField(10, 'f32', '32位浮点数'),
      createField(11, 'f64', '64位浮点数'),
      createField(12, 'bool', '布尔类型'),
      createField(13, 'Vec<u8>', '字节向量'),
      createField(14, 'chrono::NaiveDate', '日期类型'),
      createField(15, 'chrono::NaiveDateTime', '日期时间类型')
    ]
  } else if (langName.includes('python')) {
    return [
      createField(1, 'str', '字符串类型'),
      createField(2, 'int', '整型'),
      createField(3, 'float', '浮点型'),
      createField(4, 'bool', '布尔类型'),
      createField(5, 'bytes', '字节类型'),
      createField(6, 'datetime', '日期时间类型'),
      createField(7, 'date', '日期类型'),
      createField(8, 'Decimal', '高精度小数'),
      createField(9, 'List', '列表类型'),
      createField(10, 'Dict', '字典类型'),
      createField(11, 'Optional', '可选类型')
    ]
  }

  return [
    createField(1, 'String', '字符串类型'),
    createField(2, 'Integer', '整型'),
    createField(3, 'Long', '长整型'),
    createField(4, 'Boolean', '布尔类型'),
    createField(5, 'Float', '浮点型'),
    createField(6, 'Double', '双精度浮点'),
    createField(7, 'Date', '日期类型'),
    createField(8, 'DateTime', '日期时间类型')
  ]
}

// 开始编辑行
const startEditFieldTypeRow = (record) => {
  fieldTypes.value.forEach(item => {
    if (item.editing) {
      cancelEditFieldTypeRow(item)
    }
  })
  record.editing = true
  record.editName = record.name
  record.editDescription = record.description || ''
}

// 保存行编辑
const saveFieldTypeRow = async (record) => {
  if (!record.editName.trim()) {
    message.warning('请输入类型名称')
    return
  }

  const duplicate = fieldTypes.value.find(
    f => f.id !== record.id && f.name.toLowerCase() === record.editName.trim().toLowerCase()
  )
  if (duplicate) {
    message.warning('类型名称已存在')
    return
  }

  try {
    if (record.isNew) {
      const newId = await createLanguageFieldType(props.language.id, {
        name: record.editName.trim(),
        description: record.editDescription.trim(),
        sortOrder: fieldTypes.value.length
      })
      record.id = newId
      record.isNew = false
      record.isBuiltin = false
      record.name = record.editName.trim()
      record.description = record.editDescription.trim()
      record.editing = false
      message.success('类型字段已创建')
    } else {
      await updateLanguageFieldType(record.id, {
        name: record.editName.trim(),
        description: record.editDescription.trim(),
        sortOrder: fieldTypes.value.indexOf(record)
      })
      record.name = record.editName.trim()
      record.description = record.editDescription.trim()
      record.editing = false
      message.success('类型字段已更新')
    }
  } catch (error) {
    message.error('保存失败: ' + error)
  }
}

// 取消行编辑
const cancelEditFieldTypeRow = (record) => {
  if (record.isNew) {
    const index = fieldTypes.value.findIndex(f => f.id === record.id)
    if (index !== -1) {
      fieldTypes.value.splice(index, 1)
    }
  } else {
    record.editing = false
    record.editName = record.name
    record.editDescription = record.description || ''
  }
}

// 添加新类型行
const addNewFieldTypeRow = () => {
  fieldTypes.value.forEach(item => {
    if (item.editing) {
      cancelEditFieldTypeRow(item)
    }
  })

  const newId = Date.now()
  const newRow = {
    id: newId,
    name: '',
    description: '',
    isBuiltin: false,
    isNew: true,
    editing: true,
    editName: '',
    editDescription: ''
  }
  fieldTypes.value.unshift(newRow)
}

// 删除类型字段
const deleteFieldType = async (record) => {
  try {
    await deleteLanguageFieldType(record.id)
    const index = fieldTypes.value.findIndex(f => f.id === record.id)
    if (index !== -1) {
      fieldTypes.value.splice(index, 1)
    }
    message.success('类型字段已删除')
  } catch (error) {
    message.error('删除失败: ' + error)
  }
}

// 重置为默认
const resetToDefaults = () => {
  Modal.confirm({
    title: '确认重置',
    content: '确定要重置为默认类型字段吗？所有自定义类型将丢失。',
    okText: '重置',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      try {
        const defaults = getDefaultFieldTypes()
        await batchSaveLanguageFieldTypes(props.language.id, defaults)
        fieldTypes.value = defaults
        message.success('已重置为默认类型字段')
      } catch (error) {
        message.error('重置失败: ' + error)
      }
    }
  })
}

// 关闭对话框
const closeDialog = () => {
  fieldTypes.value = []
}
</script>

<style scoped>
.field-types-container {
  padding: 8px;
}

.field-types-toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--color-border);
}

.field-types-table {
  margin-bottom: 16px;
}

.field-types-table :deep(.ant-table-cell) {
  padding: 8px 12px;
}

.editable-cell {
  width: 100%;
}

.editable-cell .ant-input {
  font-family: 'Fira Code', 'Consolas', monospace;
}

.cell-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.field-type-name {
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  padding: 2px 8px;
  background: var(--color-primary-bg);
  color: var(--color-primary);
  border-radius: 4px;
}

.field-type-desc {
  color: var(--color-text-secondary);
  font-size: 13px;
}

.action-buttons {
  display: flex;
  gap: 4px;
  align-items: center;
}

.action-buttons .ant-btn {
  padding: 0 4px;
}

:deep(.ant-table-row.editing-row) {
  background: var(--color-primary-bg);
}
</style>
