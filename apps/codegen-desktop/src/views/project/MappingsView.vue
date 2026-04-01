<template>
  <div class="project-mappings-view">
    <a-card :bordered="false" class="mappings-card">
      <div class="card-header">
        <h2 class="card-title">
          <SwapOutlined style="margin-right: 8px" />
          映射管理
          <span v-if="backendLanguageName || frontendLanguageName" class="language-info">
            <span v-if="backendLanguageName">{{ backendLanguageName }}</span>
            <span v-if="backendLanguageName && frontendLanguageName" class="lang-separator">|</span>
            <span v-if="frontendLanguageName">{{ frontendLanguageName }}</span>
          </span>
        </h2>
        <div class="header-actions">
          <a-button type="primary" @click="showAddDialog">
            <PlusOutlined style="margin-right: 4px" />
            添加映射
          </a-button>
          <a-button @click="resetToDefault">
            <ReloadOutlined style="margin-right: 4px" />
            重置
          </a-button>
          <a-button @click="showSwitchLanguageDialog">
            <SwapOutlined style="margin-right: 4px" />
            切换语言
          </a-button>
        </div>
      </div>

      <!-- 前后端Tab -->
      <a-tabs v-model:activeKey="activeScope" tab-position="left" type="card" size="large" @change="handleScopeChange">
        <a-tab-pane key="frontend" tab="前端映射">
          <div class="mapping-content">
            <a-alert
              v-if="!frontendLanguageId"
              message="请先设置前端语言"
              type="warning"
              show-icon
              style="margin-bottom: 16px"
            />
            <a-table
              :columns="columns"
              :data-source="frontendMappings"
              :pagination="false"
              :scroll="{ x: 800 }"
              row-key="id"
              class="mapping-table"
              :loading="loading"
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.key === 'dbType'">
                  <span style="font-weight: 500">{{ record.db_type }}</span>
                </template>
                <template v-else-if="column.key === 'sourceType'">
                  <span style="font-weight: 500">{{ record.source_type }}</span>
                </template>
                <template v-else-if="column.key === 'targetType'">
                  <a-input
                    v-if="editingKey === record.id"
                    v-model:value="record.target_type"
                    @blur="saveMapping(record)"
                    @keyup.enter="saveMapping(record)"
                    @keyup.esc="cancelEdit(record)"
                    ref="editingInput"
                    size="small"
                  />
                  <span v-else @dblclick="editMapping(record)" style="cursor: pointer">
                    {{ record.target_type }}
                  </span>
                </template>
                <template v-else-if="column.key === 'action'">
                  <a-space>
                    <a-button type="link" size="small" @click="editMapping(record)">
                      编辑
                    </a-button>
                    <a-popconfirm
                      title="确定要删除这个映射吗？"
                      ok-text="确定"
                      cancel-text="取消"
                      @confirm="deleteMapping(record)"
                    >
                      <a-button type="link" size="small" danger>
                        删除
                      </a-button>
                    </a-popconfirm>
                  </a-space>
                </template>
              </template>
            </a-table>
          </div>
        </a-tab-pane>

        <a-tab-pane key="backend" tab="后端映射">
          <div class="mapping-content">
            <a-alert
              v-if="!backendLanguageId"
              message="请先选择后端语言"
              type="warning"
              show-icon
              style="margin-bottom: 16px"
            />
            <a-table
              :columns="columns"
              :data-source="backendMappings"
              :pagination="false"
              :scroll="{ x: 800 }"
              row-key="id"
              class="mapping-table"
              :loading="loading"
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.key === 'dbType'">
                  <span style="font-weight: 500">{{ record.db_type }}</span>
                </template>
                <template v-else-if="column.key === 'sourceType'">
                  <span style="font-weight: 500">{{ record.source_type }}</span>
                </template>
                <template v-else-if="column.key === 'targetType'">
                  <a-input
                    v-if="editingKey === record.id"
                    v-model:value="record.target_type"
                    @blur="saveMapping(record)"
                    @keyup.enter="saveMapping(record)"
                    @keyup.esc="cancelEdit(record)"
                    ref="editingInput"
                    size="small"
                  />
                  <span v-else @dblclick="editMapping(record)" style="cursor: pointer">
                    {{ record.target_type }}
                  </span>
                </template>
                <template v-else-if="column.key === 'action'">
                  <a-space>
                    <a-button type="link" size="small" @click="editMapping(record)">
                      编辑
                    </a-button>
                    <a-popconfirm
                      title="确定要删除这个映射吗？"
                      ok-text="确定"
                      cancel-text="取消"
                      @confirm="deleteMapping(record)"
                    >
                      <a-button type="link" size="small" danger>
                        删除
                      </a-button>
                    </a-popconfirm>
                  </a-space>
                </template>
              </template>
            </a-table>
          </div>
        </a-tab-pane>
      </a-tabs>
    </a-card>

    <!-- 添加映射对话框 -->
    <a-modal
      v-model:open="addDialogVisible"
      title="添加类型映射"
      ok-text="添加"
      cancel-text="取消"
      @ok="addMapping"
    >
      <a-form layout="vertical">
        <a-form-item label="数据库字段类型">
          <a-input v-model:value="addForm.sourceType" placeholder="请输入数据库字段类型（如：varchar）" />
        </a-form-item>
        <a-form-item label="语言字段类型">
          <a-input v-model:value="addForm.targetType" placeholder="请输入语言字段类型" />
        </a-form-item>
      </a-form>
    </a-modal>

    <!-- 切换语言对话框 -->
    <a-modal
      v-model:open="switchLanguageDialogVisible"
      title="切换语言"
      ok-text="确认切换"
      cancel-text="取消"
      ok-type="danger"
      @ok="confirmSwitchLanguage"
    >
      <a-alert
        message="警告：切换语言将导致当前映射丢失"
        description="切换语言后，项目当前的自定义映射将被删除，并从系统默认映射重新复制。此操作不可恢复。"
        type="warning"
        show-icon
        style="margin-bottom: 16px"
      />
      <a-form layout="vertical">
        <a-form-item label="前端语言">
          <a-select v-model:value="switchForm.frontendLanguageId" placeholder="选择前端语言" allow-clear>
            <a-select-option v-for="lang in languages" :key="lang.id" :value="lang.id">
              {{ lang.icon }} {{ lang.name }}
            </a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="后端语言">
          <a-select v-model:value="switchForm.backendLanguageId" placeholder="选择后端语言" allow-clear>
            <a-select-option v-for="lang in languages" :key="lang.id" :value="lang.id">
              {{ lang.icon }} {{ lang.name }}
            </a-select-option>
          </a-select>
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, nextTick } from 'vue'
import { useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import {
  SwapOutlined,
  ReloadOutlined,
  PlusOutlined
} from '@ant-design/icons-vue'
import { message, Modal } from 'ant-design-vue'
import * as projectsApi from '@/api/projects'

const route = useRoute()
const projectId = computed(() => parseInt(route.params.id))

// 状态
const loading = ref(false)
const project = ref(null)
const languages = ref([])
const frontendLanguageId = ref(null)
const backendLanguageId = ref(null)
const activeScope = ref('frontend')
const frontendMappings = ref([])
const backendMappings = ref([])

// 编辑状态
const editingKey = ref(null)
const originalTargetType = ref('')

// 添加映射表单
const addDialogVisible = ref(false)
const addForm = reactive({
  dbType: 'mysql',
  sourceType: '',
  targetType: ''
})

// 切换语言对话框
const switchLanguageDialogVisible = ref(false)
const switchForm = reactive({
  frontendLanguageId: null,
  backendLanguageId: null
})

// 计算语言名称
const frontendLanguageName = computed(() => {
  if (!frontendLanguageId.value) return ''
  const lang = languages.value.find(l => l.id === frontendLanguageId.value)
  return lang ? `${lang.icon} ${lang.name}` : ''
})

const backendLanguageName = computed(() => {
  if (!backendLanguageId.value) return ''
  const lang = languages.value.find(l => l.id === backendLanguageId.value)
  return lang ? `${lang.icon} ${lang.name}` : ''
})

// 表格列定义
const columns = [
  {
    title: '数据库类型',
    dataIndex: 'db_type',
    key: 'dbType',
    width: 120,
    fixed: 'left'
  },
  {
    title: '数据库字段类型',
    dataIndex: 'source_type',
    key: 'sourceType',
    width: 150
  },
  {
    title: '语言字段类型',
    dataIndex: 'target_type',
    key: 'targetType',
    width: 150
  },
  {
    title: '操作',
    key: 'action',
    width: 120,
    fixed: 'right'
  }
]

// 加载项目信息
const loadProject = async () => {
  try {
    const projectData = await projectsApi.getProject(projectId.value)
    project.value = projectData
    frontendLanguageId.value = projectData.frontend_language_id
    backendLanguageId.value = projectData.backend_language_id

    // 获取项目的数据库类型
    if (projectData.datasource_id) {
      const datasourceData = await invoke('db_get_datasource', { id: projectData.datasource_id })
      const datasource = JSON.parse(datasourceData)
      addForm.dbType = datasource.type_
    }
  } catch (error) {
    console.error('加载项目信息失败:', error)
    message.error('加载项目信息失败')
  }
}

// 加载语言列表
const loadLanguages = async () => {
  try {
    const result = await invoke('db_get_all_languages')
    languages.value = JSON.parse(result)
  } catch (error) {
    console.error('加载语言列表失败:', error)
  }
}

// 显示切换语言对话框
const showSwitchLanguageDialog = () => {
  switchForm.frontendLanguageId = frontendLanguageId.value
  switchForm.backendLanguageId = backendLanguageId.value
  switchLanguageDialogVisible.value = true
}

// 确认切换语言
const confirmSwitchLanguage = async () => {
  try {
    // 更新项目的语言设置
    await projectsApi.updateProject(projectId.value, {
      frontend_language_id: switchForm.frontendLanguageId,
      backend_language_id: switchForm.backendLanguageId
    })

    // 更新本地状态
    frontendLanguageId.value = switchForm.frontendLanguageId
    backendLanguageId.value = switchForm.backendLanguageId

    // 删除现有项目映射并复制系统映射
    if (switchForm.frontendLanguageId) {
      await invoke('db_copy_system_mappings_to_project', {
        projectId: projectId.value,
        languageId: switchForm.frontendLanguageId,
        scope: 'frontend',
        dbType: 'all'
      })
    }
    if (switchForm.backendLanguageId) {
      await invoke('db_copy_system_mappings_to_project', {
        projectId: projectId.value,
        languageId: switchForm.backendLanguageId,
        scope: 'backend',
        dbType: 'all'
      })
    }

    message.success('语言切换成功，已从系统默认映射重新复制')
    switchLanguageDialogVisible.value = false
    await loadMappings()
  } catch (error) {
    console.error('切换语言失败:', error)
    message.error('切换语言失败')
  }
}

// 处理作用域切换
const handleScopeChange = () => {
  editingKey.value = null
}

// 加载映射
const loadMappings = async () => {
  loading.value = true
  try {
    const result = await invoke('db_get_project_type_mappings', { projectId: projectId.value })
    const allMappings = JSON.parse(result)

    // 按作用域分组
    frontendMappings.value = allMappings.filter(m => m.scope === 'frontend')
    backendMappings.value = allMappings.filter(m => m.scope === 'backend')
  } catch (error) {
    console.error('加载映射失败:', error)
    message.error('加载映射失败')
  } finally {
    loading.value = false
  }
}

// 编辑映射
const editMapping = (record) => {
  editingKey.value = record.id
  originalTargetType.value = record.target_type
  nextTick(() => {
    const input = document.querySelector('.ant-table-cell input')
    if (input) {
      input.focus()
    }
  })
}

// 保存映射
const saveMapping = async (record) => {
  if (!record.target_type.trim()) {
    record.target_type = originalTargetType.value
    editingKey.value = null
    return
  }

  try {
    await invoke('db_update_project_type_mapping', {
      id: record.id,
      targetType: record.target_type,
      priority: record.priority || 0
    })
    message.success('映射更新成功')
    editingKey.value = null
  } catch (error) {
    console.error('更新映射失败:', error)
    record.target_type = originalTargetType.value
    message.error('更新映射失败')
    editingKey.value = null
  }
}

// 取消编辑
const cancelEdit = (record) => {
  record.target_type = originalTargetType.value
  editingKey.value = null
}

// 删除映射
const deleteMapping = async (record) => {
  try {
    await invoke('db_delete_project_type_mapping', { id: record.id })
    message.success('映射删除成功')
    await loadMappings()
  } catch (error) {
    console.error('删除映射失败:', error)
    message.error('删除映射失败')
  }
}

// 显示添加映射对话框
const showAddDialog = () => {
  // dbType 保持从项目获取的值，不再重置
  addForm.sourceType = ''
  addForm.targetType = ''
  addDialogVisible.value = true
}

// 添加映射
const addMapping = async () => {
  if (!addForm.sourceType.trim() || !addForm.targetType.trim()) {
    message.warning('请输入完整的类型信息')
    return
  }

  const scope = activeScope.value
  const languageId = scope === 'frontend' ? frontendLanguageId.value : backendLanguageId.value

  if (!languageId) {
    message.warning(`请先选择${scope === 'frontend' ? '前端' : '后端'}语言`)
    return
  }

  try {
    await invoke('db_create_project_type_mapping', {
      projectId: projectId.value,
      scope,
      dbType: addForm.dbType,
      sourceType: addForm.sourceType,
      targetType: addForm.targetType,
      priority: 0
    })
    message.success('映射添加成功')
    addDialogVisible.value = false
    await loadMappings()
  } catch (error) {
    console.error('添加映射失败:', error)
    message.error('添加映射失败')
  }
}

// 重置为系统默认映射
const resetToDefault = async () => {
  const scope = activeScope.value
  const languageId = scope === 'frontend' ? frontendLanguageId.value : backendLanguageId.value

  if (!languageId) {
    message.warning(`请先选择${scope === 'frontend' ? '前端' : '后端'}语言`)
    return
  }

  Modal.confirm({
    title: '确认重置',
    content: `确定要将${scope === 'frontend' ? '前端' : '后端'}映射重置为系统默认吗？当前的自定义映射将被覆盖。`,
    okText: '确定',
    cancelText: '取消',
    okType: 'danger',
    onOk: async () => {
      try {
        await invoke('db_copy_system_mappings_to_project', {
          projectId: projectId.value,
          languageId,
          scope,
          dbType: 'all'
        })
        message.success('映射已重置为系统默认')
        await loadMappings()
      } catch (error) {
        console.error('重置映射失败:', error)
        message.error('重置映射失败')
      }
    }
  })
}

// 组件挂载时加载数据
onMounted(async () => {
  await loadLanguages()
  await loadProject()
  await loadMappings()
})
</script>

<style scoped>
.project-mappings-view {
  padding: var(--spacing-lg);
  min-height: calc(100vh - var(--navbar-height));
  background: var(--color-background);
}

.mappings-card {
  height: 100%;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.card-title {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text);
  display: flex;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: var(--spacing-sm);
}

.language-info {
  margin-left: 12px;
  font-size: 14px;
  font-weight: normal;
  color: var(--color-text-secondary);
}

.lang-separator {
  margin: 0 4px;
  color: var(--color-text-secondary);
}

.mapping-content {
  padding: var(--spacing-sm);
}

/* 左侧垂直Tab样式 */
:deep(.ant-tabs-left) {
  .ant-tabs-nav {
    min-width: 100px;
    border-right: 1px solid var(--color-border);
  }

  .ant-tabs-tab {
    text-align: center;
    padding: 12px 16px;
  }

  .ant-tabs-content {
    padding-left: var(--spacing-md);
  }
}

.mapping-table {
  background: var(--color-surface);
  border-radius: var(--border-radius-md);
  overflow: hidden;
}

.mapping-table :deep(.ant-table-thead > tr > th) {
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  font-weight: 600;
  color: var(--color-text);
}

.mapping-table :deep(.ant-table-tbody > tr:hover > td) {
  background: var(--color-hover);
}
</style>
