<template>
  <a-modal
    v-model:open="dialogVisible"
    :title="mode === 'edit' ? '编辑项目' : '新建项目'"
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
        <a-input v-model:value="formData.name" placeholder="请输入项目名称" size="large" />
      </a-form-item>

      <a-form-item label="项目描述" name="description">
        <div class="description-input-wrapper">
          <a-textarea
            v-model:value="formData.description"
            placeholder="简要描述项目用途（可选）"
            size="large"
            :rows="3"
          />
          <div
            class="ai-btn-overlay"
            @click="optimizeDescription"
            @mouseenter="aiHover = true"
            @mouseleave="aiHover = false"
            :style="{ cursor: (!formData.name || aiOptimizing) ? 'not-allowed' : 'pointer' }"
          >
            <a-spin v-if="aiOptimizing" size="small" />
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 12 12" fill="none" :style="{ opacity: (!formData.name || aiOptimizing) ? 0.35 : 1 }">
              <g clip-path="url(#aiClip2)">
                <path d="M5.25371 1.57519C5.51471 0.889687 6.48371 0.889687 6.74621 1.57519L7.63346 3.90469C7.71371 4.11769 7.88096 4.28644 8.09471 4.36744L10.4242 5.25394C11.1105 5.51494 11.1105 6.48394 10.4242 6.74644L8.09471 7.63369C7.98968 7.67375 7.89429 7.73554 7.8148 7.81503C7.73532 7.89452 7.67352 7.9899 7.63346 8.09494L6.74621 10.4244C6.48371 11.1107 5.51471 11.1107 5.25371 10.4244L4.36646 8.09494C4.3264 7.9899 4.26461 7.89452 4.18512 7.81503C4.10563 7.73554 4.01024 7.67375 3.90521 7.63369L1.57571 6.74644C0.889461 6.48394 0.889461 5.51494 1.57571 5.25394L3.90521 4.36669C4.11821 4.28569 4.28546 4.11844 4.36646 3.90544L5.25371 1.57594V1.57519Z" fill="url(#aiGrad2)" fill-opacity="0.9"></path>
              </g>
              <defs>
                <linearGradient id="aiGrad2" x1="10" y1="1" x2="4" y2="10" gradientUnits="userSpaceOnUse">
                  <stop offset="0.13" stop-color="#16a34a"></stop>
                  <stop offset="0.93" stop-color="#90caf9"></stop>
                </linearGradient>
                <clipPath id="aiClip2">
                  <rect width="12" height="12" fill="white"></rect>
                </clipPath>
              </defs>
            </svg>
          </div>
        </div>
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
              <a-tag :color="getDatabaseColor(ds.type_)">{{ getDatabaseLabel(ds.type_) }}</a-tag>
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
            <a-space><span style="font-size: 16px">{{ lang.icon }}</span><span>{{ lang.name }}</span></a-space>
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
            <a-space><span style="font-size: 16px">{{ lang.icon }}</span><span>{{ lang.name }}</span></a-space>
          </a-select-option>
        </a-select>
      </a-form-item>

      <a-form-item label="数据库名称" name="databaseName">
        <a-input v-model:value="formData.databaseName" :placeholder="getDatabasePlaceholder()" size="large">
          <template #prefix v-if="selectedDatasource?.type_ === 'sqlite'"><FileOutlined /></template>
        </a-input>
        <div v-if="selectedDatasource?.type_ === 'sqlite'" class="form-item-tip">SQLite 将使用文件名作为数据库名称</div>
        <div v-else class="form-item-tip">请输入要连接的数据库名称</div>
      </a-form-item>
    </a-form>

    <template #footer>
      <a-button @click="dialogVisible = false">取消</a-button>
      <a-button type="primary" :loading="submitting" @click="handleSubmit">
        {{ mode === 'edit' ? '保存' : '创建' }}
      </a-button>
    </template>
  </a-modal>
</template>

<script setup>
import { ref, reactive, computed, watch } from 'vue'
import { FileOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import * as projectsApi from '@/api/projects'
import * as datasourcesApi from '@/api/datasources'
import * as languagesApi from '@/api/languages'

const props = defineProps({
  open: { type: Boolean, default: false },
  mode: { type: String, default: 'create' },
  project: { type: Object, default: null }
})

const emit = defineEmits(['update:open', 'saved'])

const dialogVisible = computed({
  get: () => props.open,
  set: (val) => emit('update:open', val)
})

const submitting = ref(false)
const aiOptimizing = ref(false)
const aiHover = ref(false)
const formRef = ref()
const datasources = ref([])
const languages = ref([])
const datasourcesLoading = ref(false)

const formData = reactive({
  name: '', description: '', datasourceId: null,
  databaseName: '', primaryLanguageId: null, otherLanguageIds: []
})

const selectedDatasource = computed(() => {
  if (!formData.datasourceId) return null
  return datasources.value.find(ds => ds.id === formData.datasourceId)
})

const filteredOtherLanguages = computed(() => {
  if (!formData.primaryLanguageId) return languages.value
  return languages.value.filter(lang => lang.id !== formData.primaryLanguageId)
})

const formRules = {
  name: [{ required: true, message: '请输入项目名称', trigger: 'blur' }],
  datasourceId: [{ required: true, message: '请选择数据源', trigger: 'change' }],
  databaseName: [{ required: true, message: '请输入数据库名称', trigger: 'blur' }],
  primaryLanguageId: [{ required: true, message: '请选择主语言', trigger: 'change' }]
}

const getDatabaseColor = (type) => ({ mysql: 'blue', postgresql: 'cyan', sqlite: 'green' }[type] || 'default')
const getDatabaseLabel = (type) => ({ mysql: 'MySQL', postgresql: 'PostgreSQL', sqlite: 'SQLite' }[type] || 'Database')

const getDatabasePlaceholder = () => {
  if (!selectedDatasource.value) return '请先选择数据源'
  if (selectedDatasource.value.type_ === 'sqlite') return '文件名（自动填充）'
  return '例如: my_database'
}

const getFileName = (path) => {
  if (!path) return '-'
  const parts = path.split(/[/\\]/)
  return parts[parts.length - 1] || path
}

const handleDatasourceChange = (datasourceId) => {
  const ds = datasources.value.find(d => d.id === datasourceId)
  if (ds?.type_ === 'sqlite' && ds.sqlite_file) {
    formData.databaseName = getFileName(ds.sqlite_file)
  } else {
    formData.databaseName = ''
  }
}

watch(() => props.open, async (val) => {
  if (val) {
    // 加载数据源和语言
    datasourcesLoading.value = true
    try {
      const [dsData, langData] = await Promise.all([
        datasourcesApi.getAllDatasources(),
        languagesApi.getAllLanguages()
      ])
      datasources.value = dsData
      languages.value = langData.filter(l => l.is_active)
    } catch (e) {
      console.error('加载数据失败:', e)
    } finally {
      datasourcesLoading.value = false
    }

    if (props.mode === 'edit' && props.project) {
      let otherLanguageIds = []
      try {
        const projectLanguages = await languagesApi.getProjectLanguages(props.project.id)
        otherLanguageIds = projectLanguages.map(pl => pl.id)
      } catch (e) { console.error('加载项目语言失败:', e) }

      Object.assign(formData, {
        name: props.project.name,
        description: props.project.description || '',
        datasourceId: props.project.datasource_id,
        databaseName: props.project.database_name,
        primaryLanguageId: props.project.primary_language_id,
        otherLanguageIds
      })
    } else {
      Object.assign(formData, {
        name: '', description: '', datasourceId: null,
        databaseName: '', primaryLanguageId: null, otherLanguageIds: []
      })
    }
  }
})

const optimizeDescription = async () => {
  if (!formData.name) { message.warning('请先输入项目名称'); return }
  aiOptimizing.value = true
  try {
    const prompt = `请为以下项目生成一段简洁专业的项目描述（2-3句话，中文）：\n项目名称：${formData.name}\n${formData.description ? `当前描述：${formData.description}` : ''}\n要求：描述项目的用途、技术栈和主要功能，语言简洁专业。`
    const { invoke } = await import('@tauri-apps/api/core')
    const aiConfigStore = await import('@/stores/ai-config').then(m => m.useAIConfigStore())
    await aiConfigStore.initialize()
    const provider = aiConfigStore.getDefaultProvider
    if (!provider) { message.warning('请先配置 AI 服务'); return }

    const modelsGrouped = await aiConfigStore.getProviderModelsGrouped(provider.providerName)
    let modelId = ''
    if (Array.isArray(modelsGrouped) && modelsGrouped.length > 0) {
      for (const group of modelsGrouped) {
        if (group.models && group.models.length > 0) { modelId = group.models[0].modelId; break }
      }
    }
    if (!modelId) { message.warning('请先为 AI 服务添加模型'); return }

    const result = await invoke('ai_generate_sql', {
      provider: provider.providerName, model: modelId,
      messages: [{ role: 'user', content: prompt }]
    })
    const parsed = JSON.parse(result)
    if (parsed.content) { formData.description = parsed.content.trim(); message.success('描述已优化') }
  } catch (error) {
    message.error('AI 优化失败: ' + error)
  } finally {
    aiOptimizing.value = false
  }
}

const handleSubmit = async () => {
  try { await formRef.value.validate() } catch { return }
  submitting.value = true
  try {
    const data = {
      name: formData.name, description: formData.description,
      datasourceId: formData.datasourceId, databaseName: formData.databaseName,
      primaryLanguageId: formData.primaryLanguageId
    }

    let projectId
    if (props.mode === 'edit') {
      projectId = props.project.id
      await projectsApi.updateProject(projectId, data)
      message.success('项目更新成功')
    } else {
      projectId = await projectsApi.createProject(data)
      message.success('项目创建成功')
    }

    if (formData.otherLanguageIds?.length > 0) {
      if (props.mode === 'edit') {
        try {
          const currentLanguages = await languagesApi.getProjectLanguages(projectId)
          for (const lang of currentLanguages) await languagesApi.removeProjectLanguage(projectId, lang.id)
        } catch (e) { console.error('清除旧语言关联失败:', e) }
      }
      for (const langId of formData.otherLanguageIds) {
        try { await languagesApi.addProjectLanguage(projectId, langId) } catch (e) { console.error('添加语言关联失败:', e) }
      }
    }

    dialogVisible.value = false
    emit('saved')
  } catch (error) {
    message.error('操作失败: ' + error)
  } finally {
    submitting.value = false
  }
}
</script>

<style scoped>
.form-item-tip { font-size: 12px; color: var(--color-text-secondary); margin-top: 4px; }
.description-input-wrapper { position: relative; }
.description-input-wrapper :deep(.ant-input) { padding-right: 44px; padding-bottom: 36px; }
.ai-btn-overlay { position: absolute; right: 8px; bottom: 8px; width: 28px; height: 28px; display: flex; align-items: center; justify-content: center; background: rgba(25, 118, 210, 0.1); border-radius: 4px; z-index: 100; transition: background 0.2s ease; }
.ai-btn-overlay:hover { background: rgba(25, 118, 210, 0.2); }
.ai-btn-overlay:active { background: rgba(25, 118, 210, 0.3); }
.ai-btn-overlay svg { width: 14px; height: 14px; }
[data-theme="dark"] .ai-btn-overlay { background: rgba(144, 202, 249, 0.12); }
[data-theme="dark"] .ai-btn-overlay:hover { background: rgba(144, 202, 249, 0.22); }
:deep(.ant-modal-content) { border-radius: var(--border-radius-lg); }
:deep(.ant-modal-header) { border-bottom: 1px solid var(--color-border); }
:deep(.ant-form-item-label > label) { font-size: 14px; font-weight: 500; }
:deep(.ant-input), :deep(.ant-select), :deep(.ant-input-password) { font-size: 14px; }
</style>
