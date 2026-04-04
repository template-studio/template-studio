<template>
  <div class="datasource-view">
    <!-- 页面头部 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <h2 class="page-title">数据源管理</h2>
        <span class="result-count">共 {{ filteredDatasources.length }} 个数据源</span>
      </div>
      <div class="toolbar-right">
        <SearchBar
          v-model="searchQuery"
          placeholder="搜索数据源名称..."
          :filters="databaseFilters"
          :sort-options="sortOptions"
          @search="handleSearch"
          @filter="handleFilter"
          @sort="handleSort"
        />
        <a-button type="primary" size="large" @click="openCreateDialog">
          <template #icon>
            <PlusOutlined />
          </template>
          新建数据源
        </a-button>
      </div>
    </div>

    <!-- 数据源卡片列表 -->
    <div class="datasources-content">
      <a-spin :spinning="loading">
        <div v-if="paginatedDatasources.length > 0" class="datasources-grid">
          <div
            v-for="datasource in paginatedDatasources"
            :key="datasource.id"
            class="datasource-card"
            @click="openEditDialog(datasource)"
          >
            <!-- 视觉区域 -->
            <div class="card-visual" :class="`datasource-${datasource.type_}`">
              <div class="visual-bg">
                <div class="code-preview" v-html="getPythonCode(datasource)"></div>
              </div>
            </div>

            <!-- 内容区域 -->
            <div class="card-content">
              <h3 class="datasource-name">{{ datasource.name }}</h3>

              <!-- 连接信息 -->
              <div class="datasource-details">
                <div class="detail-row">
                  <CheckCircleOutlined
                    v-if="datasource.is_active"
                    class="status-icon status-active"
                  />
                  <CloseCircleOutlined
                    v-else
                    class="status-icon status-inactive"
                  />
                  <span class="detail-text">
                    {{ datasource.is_active ? '连接正常' : '未连接' }}
                  </span>
                </div>

                <!-- MySQL/PostgreSQL 用户信息 -->
                <div v-if="datasource.type_ !== 'sqlite'" class="detail-row">
                  <UserOutlined class="detail-icon" />
                  <span class="detail-text">{{ datasource.username || '-' }}</span>
                </div>

                <!-- SQLite 文件路径 -->
                <div v-else class="detail-row">
                  <FileOutlined class="detail-icon" />
                  <span class="detail-text file-path">{{ datasource.sqlite_file || '-' }}</span>
                </div>

                <!-- 创建时间 -->
                <div class="detail-row">
                  <ClockCircleOutlined class="detail-icon" />
                  <span class="detail-text">{{ formatDate(datasource.created_at) }}</span>
                </div>
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="card-actions">
              <a-button type="text" size="small" @click.stop="openEditDialog(datasource)" class="action-btn" :title="'编辑'">
                <EditOutlined />
              </a-button>
              <a-button type="text" size="small" @click.stop="testConnection(datasource)" class="action-btn" :title="'测试连接'">
                <ApiOutlined />
              </a-button>
              <a-button type="text" size="small" danger @click.stop="confirmDelete(datasource)" class="action-btn" :title="'删除'">
                <DeleteOutlined />
              </a-button>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <a-empty
          v-else-if="!loading && filteredDatasources.length === 0"
          :description="searchQuery ? '没有找到匹配的数据源' : '暂无数据源'"
          :image="Empty.PRESENTED_IMAGE_SIMPLE"
          class="empty-state"
        >
          <a-button type="primary" size="large" @click="openCreateDialog">
            <template #icon>
              <PlusOutlined />
            </template>
            创建第一个数据源
          </a-button>
        </a-empty>

        <!-- 分页 -->
        <Pagination
          v-if="filteredDatasources.length > 0"
          v-model:current="currentPage"
          v-model:pageSize="pageSize"
          :total="filteredDatasources.length"
          @change="handlePageChange"
          @sizeChange="handleSizeChange"
        />
      </a-spin>
    </div>

    <!-- 创建/编辑对话框 -->
    <a-modal
      v-model:open="dialogVisible"
      :title="isEditMode ? '编辑数据源' : '新建数据源'"
      width="580px"
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
        <a-form-item label="数据源名称" name="name">
          <a-input
            v-model:value="formData.name"
            placeholder="请输入数据源名称"
            size="large"
          />
        </a-form-item>

        <a-form-item label="数据库类型" name="type">
          <a-select
            v-model:value="formData.type"
            placeholder="请选择数据库类型"
            size="large"
            @change="handleTypeChange"
          >
            <a-select-option value="mysql">MySQL</a-select-option>
            <a-select-option value="postgresql">PostgreSQL</a-select-option>
            <a-select-option value="sqlite">SQLite</a-select-option>
          </a-select>
        </a-form-item>

        <!-- MySQL/PostgreSQL 网络配置 -->
        <template v-if="formData.type !== 'sqlite'">
          <a-row :gutter="16">
            <a-col :span="16">
              <a-form-item label="主机地址" name="host">
                <a-input
                  v-model:value="formData.host"
                  placeholder="localhost 或 IP 地址"
                  size="large"
                />
              </a-form-item>
            </a-col>
            <a-col :span="8">
              <a-form-item label="端口" name="port">
                <a-input-number
                  v-model:value="formData.port"
                  :min="1"
                  :max="65535"
                  placeholder="端口"
                  size="large"
                  style="width: 100%"
                />
              </a-form-item>
            </a-col>
          </a-row>

          <a-form-item label="用户名" name="username">
            <a-input
              v-model:value="formData.username"
              placeholder="数据库用户名"
              size="large"
            />
          </a-form-item>

          <a-form-item label="密码" name="password">
            <a-input-password
              v-model:value="formData.password"
              placeholder="数据库密码"
              size="large"
            />
          </a-form-item>

          <!-- PostgreSQL 初始数据库 -->
          <a-form-item v-if="formData.type === 'postgresql'" label="初始数据库" name="database">
            <a-input
              v-model:value="formData.database"
              placeholder="默认: postgres"
              size="large"
            />
          </a-form-item>
        </template>

        <!-- SQLite 文件选择 -->
        <template v-else>
          <a-form-item label="数据库文件" name="sqliteFile">
            <a-input-search
              v-model:value="formData.sqliteFile"
              placeholder="请选择 SQLite 数据库文件"
              size="large"
              enter-button="选择文件"
              readonly
              @search="selectSqliteFile"
            />
          </a-form-item>
        </template>
      </a-form>

      <!-- 对话框底部按钮 -->
      <template #footer>
        <a-button @click="dialogVisible = false">取消</a-button>
        <a-button
          type="default"
          :loading="testing"
          @click="handleTestConnection"
        >
          测试连接
        </a-button>
        <a-button type="primary" :loading="submitting" @click="handleSubmit">
          {{ isEditMode ? '保存' : '创建' }}
        </a-button>
      </template>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, computed } from 'vue'
import {
  PlusOutlined,
  DatabaseOutlined,
  ApiOutlined,
  FileOutlined,
  UserOutlined,
  ClockCircleOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
  DeleteOutlined,
  EditOutlined
} from '@ant-design/icons-vue'
import { Empty, message, Modal } from 'ant-design-vue'
import { open } from '@tauri-apps/plugin-dialog'
import * as datasourcesApi from '../api/datasources'
import { SearchBar, Pagination } from '../components/common'

// 状态
const loading = ref(false)
const datasources = ref([])

// 搜索、筛选、排序、分页状态
const searchQuery = ref('')
const filterValue = ref(undefined)
const sortValue = ref('created_at:desc')
const currentPage = ref(1)
const pageSize = ref(12)

// 筛选选项
const databaseFilters = [
  { label: 'MySQL', value: 'mysql' },
  { label: 'PostgreSQL', value: 'postgresql' },
  { label: 'SQLite', value: 'sqlite' }
]

// 排序选项
const sortOptions = [
  { label: '最新创建', value: 'created_at:desc' },
  { label: '最早创建', value: 'created_at:asc' },
  { label: '名称 A-Z', value: 'name:asc' },
  { label: '名称 Z-A', value: 'name:desc' }
]

// 对话框状态
const dialogVisible = ref(false)
const isEditMode = ref(false)
const editingId = ref(null)
const submitting = ref(false)
const testing = ref(false)

// 表单引用
const formRef = ref()

// 表单数据
const formData = reactive({
  name: '',
  type: 'mysql',
  host: 'localhost',
  port: 3306,
  username: '',
  password: '',
  database: '',
  sqliteFile: ''
})

// 筛选后的数据源列表
const filteredDatasources = computed(() => {
  let result = [...datasources.value]

  // 搜索筛选
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(ds =>
      ds.name.toLowerCase().includes(query) ||
      (ds.host && ds.host.toLowerCase().includes(query)) ||
      (ds.username && ds.username.toLowerCase().includes(query))
    )
  }

  // 数据库类型筛选
  if (filterValue.value) {
    result = result.filter(ds => ds.type_ === filterValue.value)
  }

  // 排序
  if (sortValue.value) {
    const [field, order] = sortValue.value.split(':')
    result.sort((a, b) => {
      let valueA, valueB

      switch (field) {
        case 'name':
          valueA = a.name.toLowerCase()
          valueB = b.name.toLowerCase()
          break
        case 'created_at':
          valueA = new Date(a.created_at).getTime()
          valueB = new Date(b.created_at).getTime()
          break
        default:
          return 0
      }

      if (order === 'asc') {
        return valueA > valueB ? 1 : -1
      } else {
        return valueA < valueB ? 1 : -1
      }
    })
  }

  return result
})

// 分页后的数据源列表
const paginatedDatasources = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredDatasources.value.slice(start, end)
})

// 表单验证规则
const formRules = computed(() => {
  const rules = {
    name: [{ required: true, message: '请输入数据源名称', trigger: 'blur' }],
    type: [{ required: true, message: '请选择数据库类型', trigger: 'change' }]
  }

  // MySQL/PostgreSQL 额外验证
  if (formData.type !== 'sqlite') {
    rules.host = [{ required: true, message: '请输入主机地址', trigger: 'blur' }]
    rules.port = [{ required: true, message: '请输入端口号', trigger: 'blur' }]
    rules.username = [{ required: true, message: '请输入用户名', trigger: 'blur' }]
    rules.password = [{ required: true, message: '请输入密码', trigger: 'blur' }]
    if (formData.type === 'postgresql') {
      rules.database = [{ required: true, message: '请输入初始数据库', trigger: 'blur' }]
    }
  } else {
    rules.sqliteFile = [{ required: true, message: '请选择数据库文件', trigger: 'change' }]
  }

  return rules
})

// 加载数据源列表
const loadDatasources = async () => {
  try {
    loading.value = true
    const data = await datasourcesApi.getAllDatasources()
    datasources.value = data
  } catch (error) {
    message.error('加载数据源失败: ' + error)
  } finally {
    loading.value = false
  }
}

// 获取数据库标签颜色
const getDatabaseColor = (type) => {
  const colors = {
    mysql: 'blue',
    postgresql: 'cyan',
    sqlite: 'green'
  }
  return colors[type] || 'default'
}

// 获取数据库标签文本
const getDatabaseLabel = (type) => {
  const labels = {
    mysql: 'MySQL',
    postgresql: 'PostgreSQL',
    sqlite: 'SQLite'
  }
  return labels[type] || type
}

// 格式化日期
const formatDate = (dateStr) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now - date
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))

  if (days === 0) {
    const hours = Math.floor(diff / (1000 * 60 * 60))
    if (hours === 0) {
      const minutes = Math.floor(diff / (1000 * 60))
      return minutes <= 0 ? '刚刚' : `${minutes} 分钟前`
    }
    return `${hours} 小时前`
  } else if (days === 1) {
    return '昨天'
  } else if (days < 7) {
    return `${days} 天前`
  } else {
    return date.toLocaleDateString('zh-CN')
  }
}

// 获取文件名
const getFileName = (path) => {
  if (!path) return '-'
  const parts = path.split(/[/\\]/)
  return parts[parts.length - 1] || path
}

// 生成 Python 代码预览
const getPythonCode = (datasource) => {
  if (datasource.type_ === 'mysql') {
    return `<span class="code-keyword">import</span> <span class="code-module">pymysql</span>
<span class="code-variable">conn</span> <span class="code-operator">=</span> <span class="code-module">pymysql</span>.<span class="code-function">connect</span>(
    <span class="code-param">host</span><span class="code-operator">=</span><span class="code-string">'${datasource.host || 'localhost'}'</span>,
    <span class="code-param">port</span><span class="code-operator">=</span><span class="code-number">${datasource.port || 3306}</span>,
    <span class="code-param">user</span><span class="code-operator">=</span><span class="code-string">'${datasource.username || 'root'}'</span>,
    <span class="code-param">password</span><span class="code-operator">=</span><span class="code-string">'xxx'</span>
)`
  } else if (datasource.type_ === 'postgresql') {
    return `<span class="code-keyword">import</span> <span class="code-module">psycopg2</span>
<span class="code-variable">conn</span> <span class="code-operator">=</span> <span class="code-module">psycopg2</span>.<span class="code-function">connect</span>(
    <span class="code-param">host</span><span class="code-operator">=</span><span class="code-string">'${datasource.host || 'localhost'}'</span>,
    <span class="code-param">port</span><span class="code-operator">=</span><span class="code-number">${datasource.port || 5432}</span>,
    <span class="code-param">user</span><span class="code-operator">=</span><span class="code-string">'${datasource.username || 'postgres'}'</span>,
    <span class="code-param">password</span><span class="code-operator">=</span><span class="code-string">'xxx'</span>,
    <span class="code-param">database</span><span class="code-operator">=</span><span class="code-string">'${datasource.database || 'postgres'}'</span>
)`
  } else if (datasource.type_ === 'sqlite') {
    return `<span class="code-keyword">import</span> <span class="code-module">sqlite3</span>
<span class="code-variable">conn</span> <span class="code-operator">=</span> <span class="code-module">sqlite3</span>.<span class="code-function">connect</span>(
    <span class="code-string">'${getFileName(datasource.sqlite_file)}'</span>
)`
  }
  return ''
}

// 打开创建对话框
const openCreateDialog = () => {
  isEditMode.value = false
  editingId.value = null
  Object.assign(formData, {
    name: '',
    type: 'mysql',
    host: 'localhost',
    port: 3306,
    username: '',
    password: '',
    database: '',
    sqliteFile: ''
  })
  dialogVisible.value = true
}

// 打开编辑对话框
const openEditDialog = (datasource) => {
  isEditMode.value = true
  editingId.value = datasource.id
  Object.assign(formData, {
    name: datasource.name,
    type: datasource.type_,
    host: datasource.host || 'localhost',
    port: datasource.port || 3306,
    username: datasource.username || '',
    password: datasource.password || '',
    database: datasource.database || '',
    sqliteFile: datasource.sqlite_file || ''
  })
  dialogVisible.value = true
}

// 处理数据库类型变化
const handleTypeChange = (type) => {
  // 设置默认端口
  if (type === 'mysql') {
    formData.port = 3306
  } else if (type === 'postgresql') {
    formData.port = 5432
  }

  // 清空不相关字段的值
  if (type === 'sqlite') {
    formData.host = ''
    formData.port = null
    formData.username = ''
    formData.password = ''
    formData.database = ''
  } else {
    formData.sqliteFile = ''
    if (type === 'postgresql' && !formData.database) {
      formData.database = 'postgres'
    }
  }
}

// 选择 SQLite 文件
const selectSqliteFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'SQLite Database',
        extensions: ['db', 'sqlite', 'sqlite3', 'db3']
      }]
    })

    if (selected) {
      formData.sqliteFile = selected
    }
  } catch (error) {
    message.error('选择文件失败: ' + error)
  }
}

// 测试连接
const handleTestConnection = async () => {
  try {
    await formRef.value.validate()
  } catch (error) {
    message.warning('请先填写完整的连接信息')
    return
  }

  testing.value = true
  try {
    const connectionParams = {
      type: formData.type,
      host: formData.host,
      port: formData.port,
      username: formData.username,
      password: formData.password
    }

    // 只有 PostgreSQL 才需要指定初始数据库
    if (formData.type === 'postgresql') {
      connectionParams.database = formData.database || 'postgres'
    }

    // SQLite 需要文件路径
    if (formData.type === 'sqlite') {
      connectionParams.sqliteFile = formData.sqliteFile
    }

    const result = await datasourcesApi.testConnection(connectionParams)
    message.success(result)
  } catch (error) {
    message.error('连接测试失败: ' + error)
  } finally {
    testing.value = false
  }
}

// 测试现有数据源的连接
const testConnection = async (datasource) => {
  try {
    const connectionParams = {
      type: datasource.type_,
      host: datasource.host,
      port: datasource.port,
      username: datasource.username,
      password: datasource.password
    }

    // PostgreSQL 使用存储的初始数据库
    if (datasource.type_ === 'postgresql') {
      connectionParams.database = datasource.database || 'postgres'
    }

    // SQLite 使用文件路径
    if (datasource.type_ === 'sqlite') {
      connectionParams.sqliteFile = datasource.sqlite_file
    }

    const result = await datasourcesApi.testConnection(connectionParams)
    message.success(result)
  } catch (error) {
    message.error('连接测试失败: ' + error)
  }
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
      type: formData.type,
      host: formData.host,
      port: formData.port,
      username: formData.username,
      password: formData.password,
      database: formData.database,
      sqliteFile: formData.sqliteFile
    }

    if (isEditMode.value) {
      await datasourcesApi.updateDatasource(editingId.value, data)
      message.success('数据源更新成功')
    } else {
      await datasourcesApi.createDatasource(data)
      message.success('数据源创建成功')
    }
    dialogVisible.value = false
    await loadDatasources()
  } catch (error) {
    message.error('操作失败: ' + error)
  } finally {
    submitting.value = false
  }
}

// 确认删除
const confirmDelete = (datasource) => {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除数据源 "${datasource.name}" 吗？此操作不可恢复。`,
    okText: '删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      try {
        await datasourcesApi.deleteDatasource(datasource.id)
        message.success('数据源删除成功')
        await loadDatasources()
      } catch (error) {
        message.error('删除失败: ' + error)
      }
    }
  })
}

// 搜索处理
const handleSearch = () => {
  currentPage.value = 1
}

// 筛选处理
const handleFilter = (value) => {
  filterValue.value = value
  currentPage.value = 1
}

// 排序处理
const handleSort = (value) => {
  sortValue.value = value
  currentPage.value = 1
}

// 分页处理
const handlePageChange = (page) => {
  currentPage.value = page
}

// 每页条数变化
const handleSizeChange = (page, size) => {
  currentPage.value = page
  pageSize.value = size
}

// 组件挂载时加载数据
onMounted(() => {
  loadDatasources()
})
</script>

<style scoped>
.datasource-view {
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
.datasources-content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.datasources-content > :deep(.ant-spin-container) {
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

/* 数据源网格 */
.datasources-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--spacing-md);
}

/* 数据源卡片 */
.datasource-card {
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  overflow: hidden;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.datasource-card:hover {
  transform: translateY(-6px);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.15);
}

/* 视觉区域 */
.card-visual {
  height: 120px;
  position: relative;
  overflow: hidden;
}

.visual-bg {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  position: relative;
}

.visual-bg::before {
  content: '';
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: linear-gradient(
    45deg,
    transparent 30%,
    rgba(255, 255, 255, 0.1) 50%,
    transparent 70%
  );
  animation: shimmer 3s infinite;
}

@keyframes shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.8;
  }
}

/* MySQL 视觉效果 */
.datasource-mysql .visual-bg {
  background: linear-gradient(135deg, #00758f 0%, #f29111 100%);
  position: relative;
}

.datasource-mysql .visual-bg::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, transparent 0%, rgba(0, 0, 0, 0.2) 100%);
}

/* PostgreSQL 视觉效果 */
.datasource-postgresql .visual-bg {
  background: linear-gradient(135deg, #336791 0%, #0064a5 100%);
  position: relative;
}

.datasource-postgresql .visual-bg::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, transparent 0%, rgba(0, 0, 0, 0.2) 100%);
}

/* SQLite 视觉效果 */
.datasource-sqlite .visual-bg {
  background: linear-gradient(135deg, #0f8044 0%, #003b2e 100%);
  position: relative;
}

.datasource-sqlite .visual-bg::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, transparent 0%, rgba(0, 0, 0, 0.2) 100%);
}

/* 代码预览效果 */
.code-preview {
  font-family: 'Courier New', 'Consolas', monospace;
  font-size: 9px;
  line-height: 1.5;
  color: rgba(255, 255, 255, 0.7);
  white-space: pre;
  text-align: left;
  padding: var(--spacing-md);
  position: relative;
  z-index: 1;
  width: 100%;
  box-sizing: border-box;
}

.code-keyword {
  color: rgba(255, 138, 101, 0.95);
  font-weight: 600;
}

.code-module {
  color: rgba(102, 153, 204, 0.95);
}

.code-variable {
  color: rgba(152, 195, 121, 0.95);
}

.code-operator {
  color: rgba(255, 255, 255, 0.6);
}

.code-function {
  color: rgba(86, 156, 214, 0.95);
}

.code-param {
  color: rgba(207, 138, 221, 0.9);
}

.code-string {
  color: rgba(173, 186, 199, 0.95);
}

.code-number {
  color: rgba(189, 147, 249, 0.95);
}

/* 内容区域 */
.card-content {
  padding: var(--spacing-sm) var(--spacing-md);
}

.datasource-name {
  margin: 0 0 8px 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  transition: color 0.2s ease;
}

.datasource-card:hover .datasource-name {
  color: var(--color-primary);
}

/* 数据源详情 */
.datasource-details {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.detail-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.detail-icon {
  font-size: 14px;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.detail-text {
  font-size: 13px;
  color: var(--color-text-secondary);
  flex: 1;
}

.detail-text.file-path {
  font-family: 'Courier New', 'Consolas', monospace;
  font-size: 11px;
  word-break: break-all;
}

.status-icon {
  font-size: 16px;
}

.status-active {
  color: var(--color-success);
  animation: pulse 2s ease-in-out infinite;
}

.status-inactive {
  color: var(--color-text-muted);
}

/* 操作按钮 */
.card-actions {
  display: flex;
  justify-content: flex-end;
  gap: 4px;
  padding: 8px var(--spacing-md);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

.card-actions .ant-btn {
  font-size: 16px;
  padding: 4px 8px;
  height: auto;
  min-width: auto;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.card-actions .ant-btn:hover {
  transform: scale(1.1);
  background: var(--color-hover);
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
:deep(.ant-input-number),
:deep(.ant-select),
:deep(.ant-input-password) {
  font-size: 14px;
}

:deep(.ant-input-number-input) {
  width: 100%;
}
</style>
