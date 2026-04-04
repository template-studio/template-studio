<template>
  <div class="languages-view">
    <!-- 页面头部 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <h2 class="page-title">语言管理</h2>
        <span class="result-count">共 {{ languages.length }} 种语言</span>
      </div>
      <div class="toolbar-right" v-if="languages.length > 0">
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
        <div v-if="languages.length > 0" class="languages-grid">
          <div
            v-for="language in languages"
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
          v-else-if="!loading"
          description="暂无语言"
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
    <a-modal
      v-model:open="dialogVisible"
      :title="isEditMode ? '编辑语言' : '添加语言'"
      width="500px"
      :confirm-loading="submitting"
      @ok="handleSubmit"
    >
      <a-form
        ref="formRef"
        :model="formData"
        :rules="formRules"
        layout="vertical"
        @finish="handleSubmit"
      >
        <a-form-item label="语言名称" name="name">
          <a-input
            v-model:value="formData.name"
            placeholder="例如：Rust, Go, TypeScript"
            size="large"
          />
        </a-form-item>

        <a-form-item label="图标（Emoji）" name="icon">
          <a-input
            v-model:value="formData.icon"
            placeholder="输入 emoji 图标，例如：🦀, 🐹, 💛"
            size="large"
          />
          <div class="emoji-hint" v-if="emojiSuggestions.length > 0">
            <span
              v-for="emoji in emojiSuggestions"
              :key="emoji"
              @click="formData.icon = emoji"
              class="emoji-suggestion"
            >
              {{ emoji }}
            </span>
          </div>
        </a-form-item>

        <a-form-item label="颜色" name="color">
          <a-select
            v-model:value="formData.color"
            placeholder="选择颜色"
            size="large"
          >
            <a-select-option value="default">
              <span class="color-option">
                <span class="color-box" style="background: #d9d9d9;"></span>
                默认灰色
              </span>
            </a-select-option>
            <a-select-option value="red">
              <span class="color-option">
                <span class="color-box" style="background: #f5222d;"></span>
                红色
              </span>
            </a-select-option>
            <a-select-option value="orange">
              <span class="color-option">
                <span class="color-box" style="background: #fa8c16;"></span>
                橙色
              </span>
            </a-select-option>
            <a-select-option value="gold">
              <span class="color-option">
                <span class="color-box" style="background: #faad14;"></span>
                金色
              </span>
            </a-select-option>
            <a-select-option value="green">
              <span class="color-option">
                <span class="color-box" style="background: #52c41a;"></span>
                绿色
              </span>
            </a-select-option>
            <a-select-option value="cyan">
              <span class="color-option">
                <span class="color-box" style="background: #13c2c2;"></span>
              青色
              </span>
            </a-select-option>
            <a-select-option value="blue">
              <span class="color-option">
                <span class="color-box" style="background: #1890ff;"></span>
                蓝色
              </span>
            </a-select-option>
            <a-select-option value="purple">
              <span class="color-option">
                <span class="color-box" style="background: #722ed1;"></span>
                紫色
              </span>
            </a-select-option>
            <a-select-option value="pink">
              <span class="color-option">
                <span class="color-box" style="background: #eb2f96;"></span>
                粉色
              </span>
            </a-select-option>
          </a-select>
        </a-form-item>

        <a-form-item label="描述" name="description">
          <a-textarea
            v-model:value="formData.description"
            placeholder="简要描述该语言的用途（可选）"
            size="large"
            :rows="3"
          />
        </a-form-item>
      </a-form>

      <!-- 对话框底部按钮 -->
      <template #footer>
        <a-button @click="dialogVisible = false">取消</a-button>
        <a-button type="primary" :loading="submitting" @click="handleSubmit">
          {{ isEditMode ? '保存' : '添加' }}
        </a-button>
      </template>
    </a-modal>

    <!-- 类型字段管理对话框 -->
    <a-modal
      v-model:open="fieldTypesDialogVisible"
      :title="currentLanguage ? `${currentLanguage.name} - 类型字段管理` : '类型字段管理'"
      width="680px"
      :footer="null"
      @cancel="closeFieldTypesDialog"
    >
      <div class="field-types-container">
        <!-- 工具栏 -->
        <div class="field-types-toolbar">
          <a-button type="primary" size="small" @click="addNewFieldTypeRow">
            <template #icon><PlusOutlined /></template>
            添加
          </a-button>
          <a-button size="small" @click="resetToDefaults" :disabled="!currentLanguage">
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
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, computed } from 'vue'
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  SettingOutlined,
  CheckOutlined,
  CloseOutlined,
  ReloadOutlined
} from '@ant-design/icons-vue'
import { useRouter } from 'vue-router'
import { Empty, message, Modal } from 'ant-design-vue'
import * as languagesApi from '../api/languages'
import {
  getLanguageFieldTypes,
  createLanguageFieldType,
  updateLanguageFieldType,
  deleteLanguageFieldType,
  batchSaveLanguageFieldTypes
} from '../api/languages'

// 路由
const router = useRouter()

// 状态
const loading = ref(false)
const languages = ref([])

// 类型字段管理对话框状态
const fieldTypesDialogVisible = ref(false)
const currentLanguage = ref(null)
const fieldTypes = ref([])
const fieldTypeLoading = ref(false)

// 对话框状态
const dialogVisible = ref(false)
const isEditMode = ref(false)
const editingId = ref(null)
const submitting = ref(false)

// 表单引用
const formRef = ref()

// 表单数据
const formData = reactive({
  name: '',
  icon: '',
  color: 'blue',
  description: ''
})

// 表单验证规则
const formRules = {
  name: [
    { required: true, message: '请输入语言名称', trigger: 'blur' },
    { min: 2, max: 20, message: '语言名称长度应在 2-20 个字符', trigger: 'blur' }
  ],
  icon: [
    { required: true, message: '请选择图标', trigger: 'change' }
  ]
}

// 类型字段表格列定义
const fieldTypeColumns = [
  { title: '类型名称', key: 'name', width: 200 },
  { title: '描述', key: 'description' },
  { title: '操作', key: 'action', width: 100, align: 'center' }
]

// Emoji 建议
const emojiSuggestions = computed(() => {
  const name = formData.name.toLowerCase()
  const suggestions = []

  if (name.includes('rust') || name.includes('系统')) suggestions.push('🦀')
  if (name.includes('go') || name.includes('golang')) suggestions.push('🐹')
  if (name.includes('python')) suggestions.push('🐍')
  if (name.includes('java')) suggestions.push('☕')
  if (name.includes('javascript') || name.includes('js')) suggestions.push('💛')
  if (name.includes('typescript') || name.includes('ts')) suggestions.push('💠')
  if (name.includes('c++')) suggestions.push('⚡')
  if (name.includes('c#')) suggestions.push('🔷')
  if (name.includes('swift')) suggestions.push('🍎')
  if (name.includes('kotlin')) suggestions.push('🤖')
  if (name.includes('dart')) suggestions.push('🎯')
  if (name.includes('php')) suggestions.push('🐘')
  if (name.includes('ruby')) suggestions.push('💎')

  return suggestions
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

// 打开类型字段管理对话框
const openFieldTypesDialog = (language) => {
  currentLanguage.value = language
  fieldTypesDialogVisible.value = true
  loadFieldTypes(language.id)
}

// 加载类型字段列表
const loadFieldTypes = async (languageId) => {
  fieldTypeLoading.value = true
  try {
    const data = await getLanguageFieldTypes(languageId)
    if (data && data.length > 0) {
      // 添加编辑状态标记
      fieldTypes.value = data.map(item => ({
        ...item,
        editing: false,
        editName: item.name,
        editDescription: item.description || ''
      }))
    } else {
      // 如果没有数据，初始化默认类型并保存到数据库
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
  const langName = currentLanguage.value?.name?.toLowerCase() || ''

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

// 可编辑的类型字段列表（带编辑状态）
const editableFieldTypes = computed(() => fieldTypes.value)

// 开始编辑行
const startEditFieldTypeRow = (record) => {
  // 先取消其他行的编辑状态
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

  // 检查是否重复（排除自己）
  const duplicate = fieldTypes.value.find(
    f => f.id !== record.id && f.name.toLowerCase() === record.editName.trim().toLowerCase()
  )
  if (duplicate) {
    message.warning('类型名称已存在')
    return
  }

  try {
    if (record.isNew) {
      // 创建新字段
      const newId = await createLanguageFieldType(currentLanguage.value.id, {
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
      // 更新现有字段
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
    // 如果是新增的行，直接删除
    const index = fieldTypes.value.findIndex(f => f.id === record.id)
    if (index !== -1) {
      fieldTypes.value.splice(index, 1)
    }
  } else {
    // 恢复原始值
    record.editing = false
    record.editName = record.name
    record.editDescription = record.description || ''
  }
}

// 添加新类型行
const addNewFieldTypeRow = () => {
  // 先取消其他行的编辑状态
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
        await batchSaveLanguageFieldTypes(currentLanguage.value.id, defaults)
        fieldTypes.value = defaults
        message.success('已重置为默认类型字段')
      } catch (error) {
        message.error('重置失败: ' + error)
      }
    }
  })
}

// 关闭类型字段对话框
const closeFieldTypesDialog = () => {
  fieldTypesDialogVisible.value = false
  currentLanguage.value = null
  fieldTypes.value = []
}

// 打开创建对话框
const openCreateDialog = () => {
  isEditMode.value = false
  editingId.value = null
  Object.assign(formData, {
    name: '',
    icon: '',
    color: 'blue',
    description: ''
  })
  dialogVisible.value = true
}

// 打开编辑对话框
const openEditDialog = (language) => {
  isEditMode.value = true
  editingId.value = language.id
  Object.assign(formData, {
    name: language.name,
    icon: language.icon || '',
    color: language.color || 'blue',
    description: language.description || ''
  })
  dialogVisible.value = true
}

// 提交表单
const handleSubmit = async () => {
  try {
    await formRef.value.validate()
  } catch (error) {
    return
  }

  submitting.value = true
  try {
    const data = {
      name: formData.name,
      icon: formData.icon,
      color: formData.color,
      description: formData.description
    }

    if (isEditMode.value) {
      await languagesApi.updateLanguage(editingId.value, data)
      message.success('语言更新成功')
    } else {
      await languagesApi.createLanguage(data)
      message.success('语言添加成功')
    }
    dialogVisible.value = false
    await loadLanguages()
  } catch (error) {
    message.error('操作失败: ' + error)
  } finally {
    submitting.value = false
  }
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
        message.success('语言删除成功')
        await loadLanguages()
      } catch (error) {
        message.error('删除失败: ' + error)
      }
    }
  })
}

// 组件挂载时加载数据
onMounted(() => {
  loadLanguages()
})
</script>

<style scoped>
.languages-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: var(--spacing-lg);
  overflow-y: auto;
}

/* 顶部工具栏 */
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
  padding: var(--spacing-sm) 0;
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

/* Emoji 建议提示 */
.emoji-hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.emoji-suggestion {
  display: inline-block;
  font-size: 24px;
  margin: 0 4px;
  cursor: pointer;
  transition: transform 0.2s;
  padding: 4px;
  border-radius: 4px;
}

.emoji-suggestion:hover {
  transform: scale(1.2);
  background: var(--color-hover);
}

/* 颜色选择器 */
.color-option {
  display: flex;
  align-items: center;
  gap: 8px;
}

.color-box {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  border: 1px solid var(--color-border);
}

/* 类型字段管理 */
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

/* 行内编辑样式 */
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

/* 编辑行高亮 */
:deep(.ant-table-row.editing-row) {
  background: var(--color-primary-bg);
}

/* 对话框样式 */
:deep(.ant-modal-content) {
  border-radius: var(--border-radius-lg);
}

:deep(.ant-modal-header) {
  border-bottom: 1px solid var(--color-border);
}

:deep(.ant-form-item-label > label) {
  font-size: 14px;
  font-weight: 500;
}

:deep(.ant-input),
:deep(.ant-textarea),
:deep(.ant-select) {
  font-size: 14px;
}
</style>
