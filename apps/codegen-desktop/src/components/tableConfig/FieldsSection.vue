<template>
  <div class="section fields-section">
    <div class="section-header">
      <h4>
        <TableOutlined /> {{ section.title }}
        <span class="stats">
          共 {{ data.length }} 个字段，
          列表 {{ listCount }} 个，
          表单 {{ formCount }} 个
        </span>
      </h4>
      <div class="section-actions">
        <a-dropdown>
          <a-button size="small">
            快速配置 <DownOutlined />
          </a-button>
          <template #overlay>
            <a-menu @click="handleQuickAction">
              <a-menu-item key="allList">全部列表显示</a-menu-item>
              <a-menu-item key="noneList">全部列表隐藏</a-menu-item>
              <a-menu-divider />
              <a-menu-item key="allForm">全部表单显示</a-menu-item>
              <a-menu-item key="noneForm">全部表单隐藏</a-menu-item>
              <a-menu-divider />
              <a-menu-item key="smart">智能配置</a-menu-item>
            </a-menu>
          </template>
        </a-dropdown>
      </div>
    </div>
    <div class="section-content">
      <a-table
        :columns="tableColumns"
        :data-source="data"
        :pagination="false"
        :scroll="{ x: 900, y: 350 }"
        size="small"
        row-key="name"
      >
        <template #bodyCell="{ column, record }">
          <!-- 字段名（只读） -->
          <template v-if="column.key === 'name'">
            <span class="mono field-name">
              <KeyOutlined v-if="isPrimaryKey(record)" class="pk-icon" />
              {{ record.name }}
            </span>
          </template>

          <!-- 可编辑文本 -->
          <template v-else-if="isEditableText(column)">
            <a-input
              v-model:value="record[column.key]"
              size="small"
              class="table-input"
            />
          </template>

          <!-- 下拉选择 -->
          <template v-else-if="column.type === 'select'">
            <a-select
              v-model:value="record[column.key]"
              size="small"
              class="table-select"
            >
              <a-select-option
                v-for="opt in column.options"
                :key="opt.value"
                :value="opt.value"
              >
                {{ opt.label }}
              </a-select-option>
            </a-select>
          </template>

          <!-- 开关 -->
          <template v-else-if="column.type === 'switch'">
            <a-switch
              v-model:checked="record[column.key]"
              size="small"
            />
          </template>
        </template>
      </a-table>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { TableOutlined, DownOutlined, KeyOutlined } from '@ant-design/icons-vue'

const props = defineProps({
  section: { type: Object, required: true },
  data: { type: Array, default: () => [] }
})

const emit = defineEmits(['update:data'])

// 表格列配置
const tableColumns = computed(() => {
  return (props.section.columns || []).map(col => ({
    key: col.key,
    title: col.title,
    dataIndex: col.key,
    width: col.width,
    fixed: col.fixed,
    align: col.align || 'left'
  }))
})

// 统计
const listCount = computed(() => props.data.filter(f => f.list).length)
const formCount = computed(() => props.data.filter(f => f.form).length)

// 是否主键
const isPrimaryKey = (record) => {
  return record.name === 'id' || record.name.endsWith('_id') && record.name === 'id'
}

// 是否可编辑文本
const isEditableText = (column) => {
  return !column.readonly && column.type !== 'select' && column.type !== 'switch'
}

// 快速操作
const handleQuickAction = ({ key }) => {
  const newData = [...props.data]

  switch (key) {
    case 'allList':
      newData.forEach(f => f.list = true)
      break
    case 'noneList':
      newData.forEach(f => f.list = false)
      break
    case 'allForm':
      newData.forEach(f => f.form = true)
      break
    case 'noneForm':
      newData.forEach(f => f.form = false)
      break
    case 'smart':
      smartConfig(newData)
      break
  }

  emit('update:data', newData)
}

// 智能配置
const smartConfig = (data) => {
  data.forEach(f => {
    const name = f.name.toLowerCase()

    // 主键
    if (name === 'id') {
      f.list = false
      f.form = false
      f.query = false
    }
    // 时间戳
    else if (['created_at', 'updated_at', 'deleted_at'].includes(name)) {
      f.form = false
      f.query = false
    }
    // 名称字段
    else if (name.includes('name')) {
      f.query = true
    }
    // 状态字段
    else if (name.includes('status')) {
      f.query = true
      f.input = 'select'
    }
  })
}
</script>

<style scoped>
.section {
  background: var(--color-bg-container);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  overflow: hidden;
}

.section-header {
  padding: 12px 16px;
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.section-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.stats {
  font-weight: 400;
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-left: 12px;
}

.section-content {
  padding: 0;
}

.field-name {
  display: flex;
  align-items: center;
  gap: 4px;
}

.pk-icon {
  color: #faad14;
}

.table-input {
  font-size: 12px;
}

.table-select {
  width: 100%;
  font-size: 12px;
}

:deep(.ant-table-thead > tr > th) {
  background: var(--color-bg-secondary);
  font-size: 13px;
  padding: 8px 12px;
}

:deep(.ant-table-tbody > tr > td) {
  padding: 6px 12px;
}
</style>
