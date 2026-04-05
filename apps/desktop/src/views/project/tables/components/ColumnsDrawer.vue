<template>
  <a-drawer :open="open" title="表结构" :width="drawerWidth" placement="right" @update:open="$emit('update:open', $event)"
    :body-style="{ position: 'relative' }">
    <div class="drawer-resize-handle" @mousedown="startResize"></div>
    <template #title>
      <div style="display: flex; align-items: center; gap: 8px">
        <TableOutlined />
        <span style="font-weight: 600">{{ table?.name }}</span>
        <a-tag v-if="table" :color="table.table_type === 'table' ? 'blue' : 'purple'">
          {{ table.table_type === 'table' ? '表' : '视图' }}
        </a-tag>
      </div>
    </template>
    <div v-if="table" style="margin-bottom: 16px">
      <a-descriptions size="small" :column="2">
        <a-descriptions-item label="表名">{{ table.name }}</a-descriptions-item>
        <a-descriptions-item label="引擎">{{ table.engine || '-' }}</a-descriptions-item>
        <a-descriptions-item label="说明">{{ table.comment || '-' }}</a-descriptions-item>
        <a-descriptions-item label="列数">{{ table.column_count }}</a-descriptions-item>
      </a-descriptions>
    </div>
    <div style="margin-bottom: 16px; display: flex; gap: 8px">
      <a-button type="primary" @click="showAddColumnDialog"><PlusOutlined /> 新增字段</a-button>
      <a-button v-if="selectedColumnKeys.length > 0" danger @click="batchDeleteColumns"><DeleteOutlined /> 批量删除 ({{ selectedColumnKeys.length }})</a-button>
    </div>
    <a-table :columns="columnColumns" :data-source="currentColumns" :row-key="record => record.id" :row-selection="columnRowSelection" :pagination="false" size="small"
      :custom-row="(record, index) => ({ draggable: true, ondragstart: (e) => handleColumnDragStart(e, index), ondragover: (e) => handleColumnDragOver(e, index), ondrop: (e) => handleColumnDrop(e, index), ondragend: handleColumnDragEnd })">
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'name'">
          <a-space v-if="record.is_primary_key"><KeyOutlined style="color: var(--color-warning)" /><span style="font-weight: 600">{{ record.name }}</span></a-space>
          <span v-else>{{ record.name }}</span>
        </template>
        <template v-else-if="column.key === 'data_type'">
          <a-tag color="blue">{{ record.data_type }}</a-tag>
          <span v-if="record.length" style="color: var(--color-text-secondary); margin-left: 4px">({{ record.length }})</span>
        </template>
        <template v-else-if="column.key === 'is_nullable'">
          <a-tag :color="record.is_nullable ? 'orange' : 'green'">{{ record.is_nullable ? '可空' : '必填' }}</a-tag>
        </template>
        <template v-else-if="column.key === 'is_primary_key'">
          <a-tag v-if="record.is_primary_key" color="gold"><KeyOutlined /> 主键</a-tag>
          <span v-else style="color: var(--color-text-secondary)">-</span>
        </template>
        <template v-else-if="column.key === 'default_value'">
          <span style="color: var(--color-text-secondary)">{{ record.default_value || '-' }}</span>
        </template>
        <template v-else-if="column.key === 'comment'">
          <a-tooltip v-if="record.comment" :title="record.comment"><span class="comment-text">{{ record.comment }}</span></a-tooltip>
          <span v-else class="comment-empty">-</span>
        </template>
        <template v-else-if="column.key === 'column_action'">
          <a-space>
            <a-button type="link" size="small" @click="editColumn(record)">编辑</a-button>
            <a-popconfirm title="确定要删除这个字段吗？" ok-text="确定" cancel-text="取消" @confirm="deleteColumn(record)">
              <a-button type="link" size="small" danger>删除</a-button>
            </a-popconfirm>
          </a-space>
        </template>
      </template>
    </a-table>

    <!-- 新增/编辑列对话框 -->
    <a-modal v-model:open="columnDialogVisible" :title="columnDialogMode === 'add' ? '新增字段' : '编辑字段'" width="700px" ok-text="确定" cancel-text="取消" @ok="saveColumn" @cancel="closeColumnDialog">
      <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
        <a-form-item label="字段名" required><a-input v-model:value="columnForm.name" placeholder="请输入字段名" /></a-form-item>
        <a-form-item label="数据类型" required>
          <a-select v-model:value="columnForm.dataType" placeholder="请选择数据类型" show-search>
            <a-select-option v-for="t in dataTypes" :key="t" :value="t">{{ t.toUpperCase() }}</a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="长度"><a-input-number v-model:value="columnForm.length" :min="1" :max="65535" style="width: 100%" placeholder="长度" /></a-form-item>
        <a-form-item label="必填"><a-switch v-model:checked="columnForm.isNullable" checked-children="可空" un-checked-children="必填" /></a-form-item>
        <a-form-item label="主键"><a-switch v-model:checked="columnForm.isPrimaryKey" /></a-form-item>
        <a-form-item label="唯一"><a-switch v-model:checked="columnForm.isUnique" /></a-form-item>
        <a-form-item label="默认值"><a-input v-model:value="columnForm.defaultValue" placeholder="请输入默认值" /></a-form-item>
        <a-form-item label="说明"><a-textarea v-model:value="columnForm.comment" :rows="2" placeholder="请输入字段说明" /></a-form-item>
      </a-form>
    </a-modal>
  </a-drawer>
</template>

<script setup>
import { ref, reactive, watch } from 'vue'
import { TableOutlined, KeyOutlined, PlusOutlined, DeleteOutlined } from '@ant-design/icons-vue'
import { message, Modal } from 'ant-design-vue'
import * as projectsApi from '@/api/projects'

const props = defineProps({
  open: { type: Boolean, default: false },
  table: { type: Object, default: null }
})
const emit = defineEmits(['update:open', 'columns-updated'])

const currentColumns = ref([])
const columnDialogVisible = ref(false)
const columnDialogMode = ref('add')
const selectedColumnKeys = ref([])
let dragColumnIndex = null

// 抽屉宽度拖拽调整
const drawerWidth = ref(900)
let resizing = false
let resizeStartX = 0
let resizeStartWidth = 0

const startResize = (e) => {
  resizing = true
  resizeStartX = e.clientX
  resizeStartWidth = drawerWidth.value
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

const onResize = (e) => {
  if (!resizing) return
  const diff = resizeStartX - e.clientX
  drawerWidth.value = Math.min(Math.max(resizeStartWidth + diff, 500), 1400)
}

const stopResize = () => {
  resizing = false
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

const dataTypes = ['varchar', 'char', 'text', 'int', 'bigint', 'float', 'double', 'decimal', 'datetime', 'date', 'timestamp', 'boolean', 'json']

const columnForm = reactive({ id: null, name: '', dataType: 'varchar', length: null, isNullable: true, isPrimaryKey: false, isUnique: false, defaultValue: '', comment: '' })

const columnColumns = [
  { title: '列名', dataIndex: 'name', key: 'name', width: 120, ellipsis: true },
  { title: '类型', dataIndex: 'data_type', key: 'data_type', width: 100 },
  { title: '可空', dataIndex: 'is_nullable', key: 'is_nullable', width: 50, align: 'center' },
  { title: '主键', dataIndex: 'is_primary_key', key: 'is_primary_key', width: 50, align: 'center' },
  { title: '默认值', dataIndex: 'default_value', key: 'default_value', width: 80, ellipsis: true },
  { title: '说明', dataIndex: 'comment', key: 'comment', width: 80, ellipsis: true },
  { title: '操作', key: 'column_action', width: 120, fixed: 'right', align: 'center' }
]

const columnRowSelection = {
  selectedRowKeys: selectedColumnKeys,
  onChange: (selectedKeys) => { selectedColumnKeys.value = selectedKeys },
  columnWidth: 30, columnTitle: ' '
}

watch(() => props.open, async (val) => {
  if (val && props.table) { await loadColumns() }
})

const loadColumns = async () => {
  try {
    const data = await projectsApi.getTableColumns(props.table.id)
    currentColumns.value = data
  } catch (error) { message.error('加载列信息失败: ' + error) }
}

const showAddColumnDialog = () => {
  columnDialogMode.value = 'add'
  Object.assign(columnForm, { id: null, name: '', dataType: 'varchar', length: null, isNullable: true, isPrimaryKey: false, isUnique: false, defaultValue: '', comment: '' })
  columnDialogVisible.value = true
}

const editColumn = (column) => {
  columnDialogMode.value = 'edit'
  Object.assign(columnForm, { id: column.id, name: column.name, dataType: column.data_type, length: column.length, isNullable: column.is_nullable, isPrimaryKey: column.is_primary_key, isUnique: column.is_unique, defaultValue: column.default_value || '', comment: column.comment || '' })
  columnDialogVisible.value = true
}

const saveColumn = async () => {
  try {
    if (columnDialogMode.value === 'add') {
      const maxPosition = currentColumns.value.reduce((max, col) => Math.max(max, col.ordinal_position || 0), 0)
      await projectsApi.createColumn({ tableId: props.table.id, name: columnForm.name, dataType: columnForm.dataType, length: columnForm.length, isNullable: columnForm.isNullable, isPrimaryKey: columnForm.isPrimaryKey, isUnique: columnForm.isUnique, defaultValue: columnForm.defaultValue, comment: columnForm.comment, ordinalPosition: maxPosition + 1 })
      message.success('字段添加成功')
    } else {
      await projectsApi.updateColumn(columnForm.id, { name: columnForm.name, dataType: columnForm.dataType, length: columnForm.length, isNullable: columnForm.isNullable, isPrimaryKey: columnForm.isPrimaryKey, isUnique: columnForm.isUnique, defaultValue: columnForm.defaultValue, comment: columnForm.comment })
      message.success('字段更新成功')
    }
    closeColumnDialog()
    await loadColumns()
    emit('columns-updated')
  } catch (error) { message.error('保存字段失败: ' + error) }
}

const closeColumnDialog = () => {
  columnDialogVisible.value = false
  Object.assign(columnForm, { id: null, name: '', dataType: 'varchar', length: null, isNullable: true, isPrimaryKey: false, isUnique: false, defaultValue: '', comment: '' })
}

const deleteColumn = async (column) => {
  try {
    await projectsApi.deleteColumn(column.id)
    message.success(`字段 "${column.name}" 删除成功`)
    await loadColumns()
    emit('columns-updated')
  } catch (error) { message.error('删除字段失败: ' + error) }
}

const batchDeleteColumns = async () => {
  if (selectedColumnKeys.value.length === 0) { message.warning('请先选择要删除的字段'); return }
  Modal.confirm({
    title: '确认删除', content: `确定要删除选中的 ${selectedColumnKeys.value.length} 个字段吗？此操作不可恢复！`, okText: '确定', cancelText: '取消', okType: 'danger',
    onOk: async () => {
      try {
        await Promise.all(selectedColumnKeys.value.map(id => projectsApi.deleteColumn(id)))
        message.success(`成功删除 ${selectedColumnKeys.value.length} 个字段`)
        selectedColumnKeys.value = []
        await loadColumns()
        emit('columns-updated')
      } catch (error) { message.error('批量删除失败: ' + error) }
    }
  })
}

const handleColumnDragStart = (e, index) => { dragColumnIndex = index; e.dataTransfer.effectAllowed = 'move'; e.target.closest('tr').style.opacity = '0.5' }
const handleColumnDragOver = (e) => { e.preventDefault(); e.dataTransfer.dropEffect = 'move' }
const handleColumnDrop = async (e, index) => {
  e.preventDefault()
  if (dragColumnIndex === null || dragColumnIndex === index) return
  const newColumns = [...currentColumns.value]
  const [dragged] = newColumns.splice(dragColumnIndex, 1)
  newColumns.splice(index, 0, dragged)
  newColumns.forEach((col, i) => { col.ordinal_position = i + 1 })
  currentColumns.value = newColumns
  try {
    await projectsApi.reorderColumns(props.table.id, newColumns.map(c => c.id))
    message.success('字段排序已更新')
  } catch (error) { message.error('排序保存失败: ' + error); await loadColumns() }
}
const handleColumnDragEnd = (e) => { dragColumnIndex = null; e.target.closest('tr').style.opacity = '1' }
</script>

<style scoped>
.comment-text { display: inline-block; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--color-text-secondary); cursor: default; }
.comment-empty { color: var(--color-text-muted); }
.drawer-resize-handle {
  position: absolute;
  top: 0;
  left: 0;
  width: 4px;
  height: 100%;
  cursor: col-resize;
  z-index: 10;
}
.drawer-resize-handle:hover,
.drawer-resize-handle:active {
  background: rgba(0, 0, 0, 0.15);
}
</style>
