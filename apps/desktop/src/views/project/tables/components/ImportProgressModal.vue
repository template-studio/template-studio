<template>
  <a-modal :open="open" title="导入表结构" :closable="!importing" width="1200px" @update:open="$emit('update:open', $event)">
    <template #footer>
      <template v-if="importStep.current === 1"></template>
      <template v-else-if="importStep.current === 3">
        <a-button @click="closeImportDialog">取消</a-button>
        <a-button type="primary" @click="closeImportDialog">确认</a-button>
      </template>
      <template v-else>
        <a-button @click="$emit('update:open', false)">取消</a-button>
      </template>
    </template>

    <a-steps :current="importStep.current" size="small">
      <a-step title="连接数据库" />
      <a-step title="选择表" />
      <a-step title="导入数据" />
      <a-step title="完成" />
    </a-steps>

    <!-- 步骤 0：连接中 -->
    <div v-if="importStep.current === 0" style="margin-top: 24px; text-align: center; padding: 60px 0">
      <LoadingOutlined style="font-size: 48px; color: var(--color-info); margin-bottom: 16px" />
      <div style="font-size: 16px">{{ importProgress.message }}</div>
    </div>

    <!-- 步骤 1：选择表 -->
    <div v-if="importStep.current === 1" style="margin-top: 24px">
      <div v-if="availableTables.length === 0" style="text-align: center; padding: 40px 0">
        <a-empty :description="importProgress.message" />
      </div>
      <div v-else>
        <div style="margin-bottom: 12px">
          <span style="font-weight: 500; font-size: 14px">{{ importProgress.message }}</span>
        </div>
        <div style="margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center; gap: 12px">
          <a-input v-model:value="searchKeyword" placeholder="搜索表名..." style="width: 200px" allowClear size="small">
            <template #prefix><SearchOutlined /></template>
          </a-input>
          <div style="display: flex; align-items: center; gap: 8px;">
            <span v-if="selectedTables.length > 0" style="color: var(--color-primary); font-size: 12px;">已选 {{ selectedTables.length }} 张</span>
            <a-space>
              <a-button size="small" @click="selectAllTables">全选</a-button>
              <a-button size="small" @click="invertSelection">反选</a-button>
              <a-button size="small" @click="unselectAllTables">清空</a-button>
            </a-space>
          </div>
        </div>
        <a-table :columns="importTableColumns" :data-source="filteredImportTables" :row-selection="importRowSelection"
          :pagination="{ pageSize: 20, size: 'small', showSizeChanger: true, showTotal: (total) => `共 ${total} 条` }"
          :scroll="{ y: 400 }" size="small" row-key="name">
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'name'">
              <a-tag v-if="record.table_type === 'view'" color="purple" size="small">视图</a-tag>
              <span style="margin-left: 8px">{{ record.name }}</span>
            </template>
            <template v-else-if="column.key === 'comment'">
              <a-tooltip v-if="record.comment" :title="record.comment">
                <span class="comment-text">{{ record.comment }}</span>
              </a-tooltip>
              <span v-else class="comment-empty">-</span>
            </template>
            <template v-else-if="column.key === 'table_type'">
              <a-tag :color="record.table_type === 'BASE TABLE' ? 'blue' : 'purple'" size="small">
                {{ record.table_type === 'BASE TABLE' ? '表' : '视图' }}
              </a-tag>
            </template>
            <template v-else-if="column.key === 'engine'">
              <span style="color: var(--color-text-secondary)">{{ record.engine || '-' }}</span>
            </template>
          </template>
        </a-table>
        <div style="margin-top: 16px; text-align: right">
          <a-button @click="$emit('update:open', false)">取消</a-button>
          <a-button type="primary" @click="startImport" :disabled="selectedTables.length === 0" style="margin-left: 8px">
            导入选中的 {{ selectedTables.length }} 张表
          </a-button>
        </div>
      </div>
    </div>

    <!-- 步骤 2：导入进度 -->
    <div v-if="importStep.current === 2" style="margin-top: 24px">
      <a-progress :percent="importProgress.percent" :status="importProgress.status" :format="() => importProgress.message" />
      <div v-if="importProgress.details.length > 0" style="margin-top: 16px; max-height: 300px; overflow-y: auto">
        <a-list size="small" :data-source="importProgress.details">
          <template #renderItem="{ item }">
            <a-list-item>
              <CheckCircleOutlined v-if="item.status === 'success'" style="color: var(--color-success)" />
              <LoadingOutlined v-else-if="item.status === 'loading'" style="color: var(--color-info)" />
              <CloseCircleOutlined v-else style="color: var(--color-error)" />
              <span style="margin-left: 8px">{{ item.table }} - {{ item.message }}</span>
            </a-list-item>
          </template>
        </a-list>
      </div>
    </div>

    <!-- 步骤 3：完成 -->
    <div v-if="importStep.current === 3" style="margin-top: 24px; text-align: center">
      <CheckCircleOutlined style="font-size: 48px; color: var(--color-success); margin-bottom: 16px" />
      <div style="font-size: 16px; margin-bottom: 8px">导入完成！</div>
      <div style="color: var(--color-text-secondary)">成功导入 {{ importProgress.successCount }} 张表</div>
      <div v-if="importProgress.failCount > 0" style="color: var(--color-error); margin-top: 8px">失败 {{ importProgress.failCount }} 张表</div>
    </div>
  </a-modal>
</template>

<script setup>
import { ref, reactive, computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import { CheckCircleOutlined, LoadingOutlined, CloseCircleOutlined, SearchOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  open: { type: Boolean, default: false },
  project: { type: Object, default: null }
})
const emit = defineEmits(['update:open', 'imported'])

const route = useRoute()
const importing = ref(false)
const importStep = reactive({ current: 0 })
const importProgress = reactive({ percent: 0, status: 'active', message: '', details: [], successCount: 0, failCount: 0 })
const availableTables = ref([])
const selectedTables = ref([])
const searchKeyword = ref('')
const existingTableNames = ref(new Set())

const importRowSelection = {
  selectedRowKeys: selectedTables,
  onChange: (selectedKeys) => { selectedTables.value = selectedKeys }
}

const filteredImportTables = computed(() => {
  if (!searchKeyword.value) return availableTables.value
  const keyword = searchKeyword.value.toLowerCase()
  return availableTables.value.filter(t => t.name.toLowerCase().includes(keyword) || (t.comment && t.comment.toLowerCase().includes(keyword)))
})

const importTableColumns = [
  { title: '表名', dataIndex: 'name', key: 'name', width: 150, ellipsis: true },
  { title: '说明', dataIndex: 'comment', key: 'comment', width: 60, ellipsis: true },
  { title: '类型', dataIndex: 'table_type', key: 'table_type', width: 100 },
  { title: '引擎', dataIndex: 'engine', key: 'engine', width: 100 }
]

const selectAllTables = () => { selectedTables.value = availableTables.value.map(t => t.name) }
const invertSelection = () => {
  const allNames = availableTables.value.map(t => t.name)
  selectedTables.value = allNames.filter(name => !selectedTables.value.includes(name))
}
const unselectAllTables = () => { selectedTables.value = [] }

const fetchAvailableTables = async () => {
  const datasource = props.project.datasource
  const dbName = props.project.database_name
  let tablesData = []
  if (datasource.type_ === 'mysql') {
    tablesData = JSON.parse(await invoke('cmd_fetch_mysql_tables', { datasourceId: datasource.id, databaseName: dbName }))
  } else if (datasource.type_ === 'postgresql') {
    tablesData = JSON.parse(await invoke('cmd_fetch_postgresql_tables', { datasourceId: datasource.id, databaseName: dbName }))
  } else if (datasource.type_ === 'sqlite') {
    tablesData = JSON.parse(await invoke('cmd_fetch_sqlite_tables', { datasourceId: datasource.id }))
  } else {
    throw new Error(`不支持的数据源类型: ${datasource.type_}`)
  }
  availableTables.value = tablesData.filter(t => !existingTableNames.value.has(t.name))
}

const startImport = async () => {
  if (selectedTables.value.length === 0) {
    importProgress.message = '请至少选择一张表'
    importProgress.status = 'exception'
    return
  }
  importing.value = true
  importStep.current = 2
  importProgress.percent = 0
  importProgress.status = 'active'
  importProgress.message = '正在导入表结构...'
  importProgress.details = []
  importProgress.successCount = 0
  importProgress.failCount = 0
  try {
    const projectId = parseInt(route.params.id)
    const datasource = props.project.datasource
    const dbName = props.project.database_name
    const tablesToImport = availableTables.value.filter(t => selectedTables.value.includes(t.name))
    for (let i = 0; i < tablesToImport.length; i++) {
      const table = tablesToImport[i]
      importProgress.percent = Math.round(((i + 1) / tablesToImport.length) * 100)
      importProgress.message = `正在导入 (${i + 1}/${tablesToImport.length}): ${table.name}`
      importProgress.details.push({ table: table.name, status: 'loading', message: '正在导入...' })
      try {
        await invoke('cmd_import_single_table', { projectId, datasourceId: datasource.id, databaseName: dbName, tableName: table.name, tableComment: table.comment || null, tableType: table.table_type, engine: table.engine || null, rowCount: table.row_count || 0 })
        importProgress.details[importProgress.details.length - 1].status = 'success'
        importProgress.details[importProgress.details.length - 1].message = '导入成功'
        importProgress.successCount++
      } catch (error) {
        importProgress.details[importProgress.details.length - 1].status = 'error'
        importProgress.details[importProgress.details.length - 1].message = '导入失败: ' + error
        importProgress.failCount++
      }
    }
    importStep.current = 3
    importProgress.percent = 100
    importProgress.status = importProgress.failCount === 0 ? 'success' : 'exception'
    importProgress.message = importProgress.failCount === 0 ? '导入完成' : `部分表导入失败（成功 ${importProgress.successCount}，失败 ${importProgress.failCount}）`
    emit('imported')
  } catch (error) {
    importProgress.status = 'exception'
    importProgress.message = '导入失败: ' + error
  } finally {
    importing.value = false
  }
}

const closeImportDialog = () => {
  emit('update:open', false)
  importStep.current = 0
  availableTables.value = []
  selectedTables.value = []
}

// 当 open 变为 true 时开始导入流程
watch(() => props.open, async (val) => {
  if (val && props.project?.datasource) {
    importStep.current = 0
    importProgress.percent = 0
    importProgress.status = 'active'
    importProgress.message = '正在连接数据库...'
    importProgress.details = []
    importProgress.successCount = 0
    importProgress.failCount = 0
    availableTables.value = []
    selectedTables.value = []
    try {
      await new Promise(resolve => setTimeout(resolve, 500))
      importStep.current = 1
      importProgress.percent = 50
      importProgress.message = '正在读取表列表...'
      await fetchAvailableTables()
      if (availableTables.value.length === 0) {
        importProgress.message = '未发现新的表（所有表已导入）'
        importProgress.status = 'exception'
      } else {
        importProgress.message = `发现 ${availableTables.value.length} 张表，请选择要导入的表`
      }
    } catch (error) {
      importProgress.status = 'exception'
      importProgress.message = '连接失败: ' + error
    }
  }
})

// 接收已存在的表名列表
const setExistingTables = (names) => { existingTableNames.value = new Set(names) }
defineExpose({ setExistingTables })
</script>

<style scoped>
.comment-text { display: inline-block; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--color-text-secondary); cursor: default; }
.comment-empty { color: var(--color-text-muted); }
</style>
