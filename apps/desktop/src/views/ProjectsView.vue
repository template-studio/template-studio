<template>
  <div class="projects-view">
    <!-- 页面头部 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <h2 class="page-title">项目列表</h2>
        <span class="result-count">共 {{ filteredProjects.length }} 个项目</span>
      </div>
      <div class="toolbar-right">
        <SearchBar
          v-model="searchQuery"
          placeholder="搜索项目名称或描述..."
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
          新建项目
        </a-button>
      </div>
    </div>

    <!-- 项目卡片列表 -->
    <div class="projects-content">
      <a-spin :spinning="loading">
        <div v-if="paginatedProjects.length > 0" class="projects-grid">
          <div
            v-for="project in paginatedProjects"
            :key="project.id"
            class="project-card"
            @click="openProject(project.id)"
          >
            <!-- 视觉区域 -->
            <div class="card-visual" :class="`project-${getDatabaseType(project)}`">
              <div class="visual-bg">
                <!-- 抽象几何图形 -->
                <div class="geometric-pattern">
                  <div class="circle circle-1"></div>
                  <div class="circle circle-2"></div>
                  <div class="circle circle-3"></div>
                  <div class="wave wave-1"></div>
                  <div class="wave wave-2"></div>
                </div>

                <!-- 光效 -->
                <div class="glow-effect"></div>
              </div>
            </div>

            <!-- 内容区域 -->
            <div class="card-content">
              <h3 class="project-name">{{ project.name }}</h3>

              <!-- 项目详情 -->
              <div class="project-details">
                <!-- 数据源标签 -->
                <div class="detail-row">
                  <DatabaseOutlined class="detail-icon" />
                  <a-tag :color="getDatabaseColor(getDatabaseType(project))">
                    {{ getDatabaseLabel(getDatabaseType(project)) }}
                  </a-tag>
                  <span class="detail-text datasource-name">{{ project.datasource?.name || '未关联数据源' }}</span>
                </div>

                <!-- 语言标签 -->
                <div v-if="getProjectLanguages(project).length > 0" class="detail-row">
                  <CodeOutlined class="detail-icon" />
                  <a-space :size="4">
                    <a-tag
                      v-for="lang in getProjectLanguages(project)"
                      :key="lang.id"
                      :color="getLanguageColor(lang.color)"
                    >
                      {{ lang.name }}<span v-if="lang.is_primary" style="margin-left: 4px; font-weight: 600;">★</span>
                    </a-tag>
                  </a-space>
                </div>

                <!-- 数据库名称 -->
                <div class="detail-row">
                  <ApiOutlined class="detail-icon" />
                  <span class="detail-text database-name">{{ project.database_name }}</span>
                </div>

                <!-- 表数量统计 -->
                <div class="detail-row">
                  <TableOutlined class="detail-icon" />
                  <span class="detail-text">
                    {{ project.table_count || 0 }} 张表
                  </span>
                </div>

                <!-- 创建时间 -->
                <div class="detail-row">
                  <ClockCircleOutlined class="detail-icon" />
                  <span class="detail-text">{{ formatDate(project.created_at) }}</span>
                </div>
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="card-actions">
              <a-button type="text" size="small" @click.stop="openProject(project.id)" class="action-btn" :title="'打开'">
                <FolderOpenOutlined />
              </a-button>
              <a-button type="text" size="small" @click.stop="openEditDialog(project)" class="action-btn" :title="'编辑'">
                <EditOutlined />
              </a-button>
              <a-button type="text" size="small" danger @click.stop="confirmDelete(project)" class="action-btn" :title="'删除'">
                <DeleteOutlined />
              </a-button>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <a-empty
          v-else-if="!loading && filteredProjects.length === 0"
          :description="searchQuery ? '没有找到匹配的项目' : '暂无项目'"
          :image="Empty.PRESENTED_IMAGE_SIMPLE"
          class="empty-state"
        >
          <a-button type="primary" size="large" @click="openCreateDialog">
            <template #icon>
              <PlusOutlined />
            </template>
            创建第一个项目
          </a-button>
        </a-empty>
      </a-spin>
    </div>

    <!-- 底部 Footer - 分页 -->
    <Pagination
      v-if="filteredProjects.length > 0"
      v-model:current="currentPage"
      v-model:pageSize="pageSize"
      :total="filteredProjects.length"
      fixed
      @change="handlePageChange"
      @sizeChange="handleSizeChange"
    />

    <!-- 创建/编辑对话框 -->
    <a-modal
      v-model:open="dialogVisible"
      :title="isEditMode ? '编辑项目' : '新建项目'"
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
        <a-form-item label="项目名称" name="name">
          <a-input
            v-model:value="formData.name"
            placeholder="请输入项目名称"
            size="large"
          />
        </a-form-item>

        <a-form-item label="项目描述" name="description">
          <a-textarea
            v-model:value="formData.description"
            placeholder="简要描述项目用途（可选）"
            size="large"
            :rows="3"
          />
        </a-form-item>

        <a-form-item label="数据源" name="datasourceId">
          <a-select
            v-model:value="formData.datasourceId"
            placeholder="请选择数据源"
            size="large"
            @change="handleDatasourceChange"
            :loading="datasourcesLoading"
          >
            <a-select-option v-for="ds in datasources" :key="ds.id" :value="ds.id">
              <a-space>
                <a-tag :color="getDatabaseColor(ds.type_)">
                  {{ getDatabaseLabel(ds.type_) }}
                </a-tag>
                <span>{{ ds.name }}</span>
              </a-space>
            </a-select-option>
          </a-select>
        </a-form-item>

        <a-form-item label="主语言" name="primaryLanguageId">
          <a-select
            v-model:value="formData.primaryLanguageId"
            placeholder="请选择主语言"
            size="large"
            show-search
            :filter-option="(input, option) => option.text.toLowerCase().includes(input.toLowerCase())"
          >
            <a-select-option v-for="lang in languages" :key="lang.id" :value="lang.id" :text="lang.name">
              <a-space>
                <span style="font-size: 16px">{{ lang.icon }}</span>
                <span>{{ lang.name }}</span>
              </a-space>
            </a-select-option>
          </a-select>
        </a-form-item>

        <a-form-item label="其它语言" name="otherLanguageIds">
          <a-select
            v-model:value="formData.otherLanguageIds"
            mode="multiple"
            placeholder="请选择其它语言（可多选）"
            size="large"
            show-search
            :filter-option="(input, option) => option.text.toLowerCase().includes(input.toLowerCase())"
          >
            <a-select-option v-for="lang in filteredOtherLanguages" :key="lang.id" :value="lang.id" :text="lang.name">
              <a-space>
                <span style="font-size: 16px">{{ lang.icon }}</span>
                <span>{{ lang.name }}</span>
              </a-space>
            </a-select-option>
          </a-select>
        </a-form-item>

        <a-form-item label="数据库名称" name="databaseName">
          <a-input
            v-model:value="formData.databaseName"
            :placeholder="getDatabasePlaceholder()"
            size="large"
          >
            <template #prefix v-if="selectedDatasource?.type_ === 'sqlite'">
              <FileOutlined />
            </template>
          </a-input>
          <div v-if="selectedDatasource?.type_ === 'sqlite'" class="form-item-tip">
            SQLite 将使用文件名作为数据库名称
          </div>
          <div v-else class="form-item-tip">
            请输入要连接的数据库名称
          </div>
        </a-form-item>
      </a-form>

      <!-- 对话框底部按钮 -->
      <template #footer>
        <a-button @click="dialogVisible = false">取消</a-button>
        <a-button type="primary" :loading="submitting" @click="handleSubmit">
          {{ isEditMode ? '保存' : '创建' }}
        </a-button>
      </template>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import {
  PlusOutlined,
  DatabaseOutlined,
  ApiOutlined,
  FileOutlined,
  TableOutlined,
  ClockCircleOutlined,
  FolderOpenOutlined,
  DeleteOutlined,
  EditOutlined,
  CodeOutlined
} from '@ant-design/icons-vue'
import { Empty, message, Modal } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import * as projectsApi from '../api/projects'
import * as datasourcesApi from '../api/datasources'
import * as languagesApi from '../api/languages'
import { SearchBar, Pagination } from '../components/common'

const router = useRouter()

// 状态
const loading = ref(false)
const projects = ref([])
const datasources = ref([])
const languages = ref([])
const datasourcesLoading = ref(false)

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
  { label: '名称 Z-A', value: 'name:desc' },
  { label: '表数量最多', value: 'table_count:desc' },
  { label: '表数量最少', value: 'table_count:asc' }
]

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
  description: '',
  datasourceId: null,
  databaseName: '',
  primaryLanguageId: null,
  otherLanguageIds: []
})

// 当前选择的数据源
const selectedDatasource = computed(() => {
  if (!formData.datasourceId) return null
  return datasources.value.find(ds => ds.id === formData.datasourceId)
})

// 筛选后的项目列表
const filteredProjects = computed(() => {
  let result = [...projects.value]

  // 搜索筛选
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(project =>
      project.name.toLowerCase().includes(query) ||
      (project.description && project.description.toLowerCase().includes(query)) ||
      (project.database_name && project.database_name.toLowerCase().includes(query))
    )
  }

  // 数据库类型筛选
  if (filterValue.value) {
    result = result.filter(project => {
      const dbType = project.datasource?.type_ || 'default'
      return dbType === filterValue.value
    })
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
        case 'table_count':
          valueA = a.table_count || 0
          valueB = b.table_count || 0
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

// 分页后的项目列表
const paginatedProjects = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredProjects.value.slice(start, end)
})

// 过滤后的其它语言列表（排除已选的主语言）
const filteredOtherLanguages = computed(() => {
  if (!formData.primaryLanguageId) {
    return languages.value
  }
  return languages.value.filter(lang => lang.id !== formData.primaryLanguageId)
})

// 表单验证规则
const formRules = {
  name: [{ required: true, message: '请输入项目名称', trigger: 'blur' }],
  datasourceId: [{ required: true, message: '请选择数据源', trigger: 'change' }],
  databaseName: [{ required: true, message: '请输入数据库名称', trigger: 'blur' }],
  primaryLanguageId: [{ required: true, message: '请选择主语言', trigger: 'change' }]
}

// 加载项目列表
const loadProjects = async () => {
  try {
    loading.value = true
    const data = await projectsApi.getAllProjects()

    // 为每个项目关联数据源信息和语言信息
    projects.value = await Promise.all(
      data.map(async (project) => {
        // 并行加载数据源和语言
        await Promise.all([
          (async () => {
            try {
              const datasourceData = await invoke('db_get_datasource', { id: project.datasource_id })
              project.datasource = JSON.parse(datasourceData)
            } catch (error) {
              console.error('加载数据源失败:', error)
              project.datasource = null
            }
          })(),
          (async () => {
            try {
              const languagesData = await languagesApi.getProjectLanguages(project.id)
              project.languages = languagesData
            } catch (error) {
              console.error('加载项目语言失败:', error)
              project.languages = []
            }
          })()
        ])
        return project
      })
    )
  } catch (error) {
    message.error('加载项目失败: ' + error)
  } finally {
    loading.value = false
  }
}

// 加载数据源列表
const loadDatasources = async () => {
  try {
    datasourcesLoading.value = true
    const data = await datasourcesApi.getAllDatasources()
    datasources.value = data
  } catch (error) {
    message.error('加载数据源失败: ' + error)
  } finally {
    datasourcesLoading.value = false
  }
}

// 加载语言列表
const loadLanguages = async () => {
  try {
    const data = await languagesApi.getAllLanguages()
    languages.value = data.filter(lang => lang.is_active)
  } catch (error) {
    message.error('加载语言失败: ' + error)
  }
}

// 获取数据库标签颜色
const getDatabaseColor = (type) => {
  const colors = {
    mysql: 'blue',
    postgresql: 'cyan',
    sqlite: 'green',
    'default': 'default'
  }
  return colors[type] || 'default'
}

// 获取数据库标签文本
const getDatabaseLabel = (type) => {
  const labels = {
    mysql: 'MySQL',
    postgresql: 'PostgreSQL',
    sqlite: 'SQLite',
    'default': 'Database'
  }
  return labels[type] || 'Database'
}

// 获取项目的数据库类型
const getDatabaseType = (project) => {
  return project.datasource?.type_ || 'default'
}

// 获取数据库名称输入框占位符
const getDatabasePlaceholder = () => {
  if (!selectedDatasource.value) return '请先选择数据源'

  switch (selectedDatasource.value.type_) {
    case 'mysql':
      return '例如: my_database'
    case 'postgresql':
      return '例如: my_database'
    case 'sqlite':
      return '文件名（自动填充）'
    default:
      return '请输入数据库名称'
  }
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
const getPythonCode = (project) => {
  if (!project.datasource) return ''

  const ds = project.datasource
  if (ds.type_ === 'mysql') {
    return `<span class="code-keyword">import</span> <span class="code-module">pymysql</span>
<span class="code-variable">conn</span> <span class="code-operator">=</span> <span class="code-module">pymysql</span>.<span class="code-function">connect</span>(
    <span class="code-param">database</span><span class="code-operator">=</span><span class="code-string">'${project.database_name}'</span>
)`
  } else if (ds.type_ === 'postgresql') {
    return `<span class="code-keyword">import</span> <span class="code-module">psycopg2</span>
<span class="code-variable">conn</span> <span class="code-operator">=</span> <span class="code-module">psycopg2</span>.<span class="code-function">connect</span>(
    <span class="code-param">database</span><span class="code-operator">=</span><span class="code-string">'${project.database_name}'</span>
)`
  } else if (ds.type_ === 'sqlite') {
    return `<span class="code-keyword">import</span> <span class="code-module">sqlite3</span>
<span class="code-variable">conn</span> <span class="code-operator">=</span> <span class="code-module">sqlite3</span>.<span class="code-function">connect</span>(
    <span class="code-string">'${project.database_name}'</span>
)`
  }
  return ''
}

// 获取项目的所有语言（包括主语言和其它语言）
const getProjectLanguages = (project) => {
  const result = []

  // 添加主语言（如果存在）
  if (project.primary_language_id) {
    const primaryLang = project.primary_language || languages.value.find(l => l.id === project.primary_language_id)
    if (primaryLang) {
      result.push({ ...primaryLang, is_primary: true })
    }
  }

  // 添加其它语言（排除主语言）
  if (project.languages && project.languages.length > 0) {
    for (const lang of project.languages) {
      // 确保不重复添加主语言
      if (lang.id !== project.primary_language_id) {
        result.push({ ...lang, is_primary: false })
      }
    }
  }

  return result
}

// 获取语言颜色映射
const getLanguageColor = (color) => {
  if (!color) return 'default'
  const colorMap = {
    red: 'red',
    orange: 'orange',
    gold: 'gold',
    green: 'green',
    cyan: 'cyan',
    blue: 'blue',
    purple: 'purple',
    pink: 'pink'
  }
  return colorMap[color] || 'default'
}

// 打开创建对话框
const openCreateDialog = () => {
  isEditMode.value = false
  editingId.value = null
  Object.assign(formData, {
    name: '',
    description: '',
    datasourceId: null,
    databaseName: '',
    primaryLanguageId: null,
    otherLanguageIds: []
  })
  dialogVisible.value = true
}

// 打开编辑对话框
const openEditDialog = async (project) => {
  isEditMode.value = true
  editingId.value = project.id

  // 加载项目的其它语言（不包括主语言）
  let otherLanguageIds = []
  try {
    const projectLanguages = await languagesApi.getProjectLanguages(project.id)
    // 所有查询到的都是其它语言，因为主语言不在 project_languages 表中
    otherLanguageIds = projectLanguages.map(pl => pl.id)
  } catch (error) {
    console.error('加载项目语言失败:', error)
  }

  Object.assign(formData, {
    name: project.name,
    description: project.description || '',
    datasourceId: project.datasource_id,
    databaseName: project.database_name,
    primaryLanguageId: project.primary_language_id,
    otherLanguageIds
  })
  dialogVisible.value = true
}

// 处理数据源变化
const handleDatasourceChange = (datasourceId) => {
  const ds = datasources.value.find(d => d.id === datasourceId)
  if (ds?.type_ === 'sqlite' && ds.sqlite_file) {
    // SQLite 自动填充文件名
    formData.databaseName = getFileName(ds.sqlite_file)
  } else {
    formData.databaseName = ''
  }
}

// 打开项目详情
const openProject = (id) => {
  router.push(`/project/${id}`)
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
      description: formData.description,
      datasourceId: formData.datasourceId,
      databaseName: formData.databaseName,
      primaryLanguageId: formData.primaryLanguageId
    }

    let projectId
    if (isEditMode.value) {
      projectId = editingId.value
      await projectsApi.updateProject(projectId, data)
      message.success('项目更新成功')
    } else {
      projectId = await projectsApi.createProject(data)
      message.success('项目创建成功')
    }

    // 处理其它语言
    if (formData.otherLanguageIds && formData.otherLanguageIds.length > 0) {
      // 先移除所有旧的其它语言关联
      if (isEditMode.value) {
        try {
          const currentLanguages = await languagesApi.getProjectLanguages(projectId)
          for (const lang of currentLanguages) {
            await languagesApi.removeProjectLanguage(projectId, lang.id)
          }
        } catch (error) {
          console.error('清除旧语言关联失败:', error)
        }
      }

      // 添加新的其它语言关联
      for (const langId of formData.otherLanguageIds) {
        try {
          await languagesApi.addProjectLanguage(projectId, langId)
        } catch (error) {
          console.error('添加语言关联失败:', error)
        }
      }
    }

    dialogVisible.value = false
    await loadProjects()
  } catch (error) {
    message.error('操作失败: ' + error)
  } finally {
    submitting.value = false
  }
}

// 确认删除
const confirmDelete = (project) => {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除项目 "${project.name}" 吗？此操作将同时删除该项目关联的所有表和字段数据，且不可恢复。`,
    okText: '删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      try {
        await projectsApi.deleteProject(project.id)
        message.success('项目删除成功')
        await loadProjects()
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
onMounted(async () => {
  await Promise.all([
    loadProjects(),
    loadDatasources(),
    loadLanguages()
  ])
})
</script>

<style scoped>
.projects-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 顶部工具栏 */
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-lg);
  flex-shrink: 0;
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
.projects-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  min-height: 0;
  padding: 0 var(--spacing-lg);
}

.projects-content > :deep(.ant-spin-container) {
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

/* 项目网格 */
.projects-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--spacing-md);
}

/* 项目卡片 */
.project-card {
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  overflow: hidden;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.project-card:hover {
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

/* MySQL 视觉效果 */
.project-mysql .visual-bg {
  background: linear-gradient(135deg, #00758f 0%, #00a0e4 50%, #f29111 100%);
  position: relative;
  overflow: hidden;
}

.project-postgresql .visual-bg {
  background: linear-gradient(135deg, #336791 0%, #0064a5 50%, #008bfc 100%);
  position: relative;
  overflow: hidden;
}

.project-sqlite .visual-bg {
  background: linear-gradient(135deg, #0f8044 0%, #00a86b 50%, #98fb98 100%);
  position: relative;
  overflow: hidden;
}

.project-default .visual-bg {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #f093fb 100%);
  position: relative;
  overflow: hidden;
}

/* 抽象几何图案 */
.geometric-pattern {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  opacity: 0.15;
}

/* 圆形图案 */
.circle {
  position: absolute;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  animation: rotate 20s linear infinite;
}

.circle-1 {
  width: 200px;
  height: 200px;
  top: -50px;
  right: -50px;
  animation-duration: 25s;
}

.circle-2 {
  width: 150px;
  height: 150px;
  bottom: -30px;
  left: -30px;
  animation-duration: 20s;
  animation-direction: reverse;
}

.circle-3 {
  width: 100px;
  height: 100px;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  animation-duration: 15s;
}

@keyframes rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* 波浪效果 */
.wave {
  position: absolute;
  width: 200%;
  height: 200%;
  background: radial-gradient(
    circle,
    rgba(255, 255, 255, 0.1) 0%,
    transparent 70%
  );
  animation: wave 8s ease-in-out infinite;
}

.wave-1 {
  top: -50%;
  left: -50%;
  animation-delay: 0s;
}

.wave-2 {
  bottom: -50%;
  right: -50%;
  animation-delay: -4s;
}

@keyframes wave {
  0%, 100% {
    transform: translate(0, 0) scale(1);
  }
  50% {
    transform: translate(20px, 20px) scale(1.1);
  }
}

/* 光效 */
.glow-effect {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 300px;
  height: 300px;
  background: radial-gradient(
    circle,
    rgba(255, 255, 255, 0.2) 0%,
    transparent 70%
  );
  animation: glow 4s ease-in-out infinite;
  pointer-events: none;
}

@keyframes glow {
  0%, 100% {
    opacity: 0.3;
    transform: translate(-50%, -50%) scale(1);
  }
  50% {
    opacity: 0.6;
    transform: translate(-50%, -50%) scale(1.2);
  }
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

.project-name {
  margin: 0 0 8px 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  transition: color 0.2s ease;
}

.project-card:hover .project-name {
  color: var(--color-primary);
}

/* 项目详情 */
.project-details {
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

.detail-text.datasource-name {
  font-weight: 500;
}

.detail-text.database-name {
  font-family: 'Courier New', 'Consolas', monospace;
  font-size: 11px;
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

/* 表单项提示 */
.form-item-tip {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 4px;
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
