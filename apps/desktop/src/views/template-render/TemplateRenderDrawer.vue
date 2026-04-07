<template>
  <a-drawer :open="open" @update:open="$emit('update:open', $event)" placement="right" :width="'100vw'" :closable="true" @close="onClose">
    <template #title>
      <div class="wizard-header">
        <div class="header-left">
          <span class="wz-title">模板渲染</span>
          <a-tag v-if="template" color="purple">{{ template.name }}</a-tag>
        </div>
        <div class="header-right">
          <div class="steps-compact">
            <template v-for="(step, idx) in steps" :key="step.title">
              <div
                class="step-item"
                :class="{ active: currentStep === idx + 1, completed: currentStep > idx + 1 }"
              >
                <div class="step-dot">{{ idx + 1 }}</div>
                <div class="step-text">{{ step.title }}</div>
              </div>
              <div v-if="idx < steps.length - 1" class="step-arrow">›</div>
            </template>
          </div>
        </div>
      </div>
    </template>
    <div class="wizard-content" :class="{ 'wizard-content-preview': currentStep === 3 }">
      <!-- Step 1: 模板详情 -->
      <div v-if="currentStep === 1" class="step-panel">
        <div class="template-detail">
          <div class="detail-header">
            <div class="detail-title-row">
              <h2 class="detail-name">{{ template?.name }}</h2>
              <a-tag v-if="template?.categoryId" color="purple" size="large">
                {{ getCategoryName(template.categoryId) }}
              </a-tag>
              <a-tag v-if="template?.isFeatured === 1" color="gold" size="large">
                <template #icon><StarOutlined /></template>
                推荐
              </a-tag>
            </div>
            <a-divider />
            <p class="detail-desc">{{ template?.description || '暂无描述' }}</p>
          </div>

          <!-- 版本选择 -->
          <div v-if="versionList.length > 0" class="detail-section">
            <h3 class="detail-section-title"><TagsOutlined /> 选择版本</h3>
            <a-select
              v-model:value="selectedVersion"
              :options="versionOptions"
              size="large"
              style="width: 100%"
              placeholder="选择版本"
            >
              <template #suffixIcon><DownloadOutlined /></template>
            </a-select>
            <div class="form-hint">当前选择：{{ selectedVersion || '未选择' }}</div>
          </div>
          <a-alert
            v-else-if="!loadingVersions"
            message="该模板暂未开放使用"
            description="此模板正在准备中，请稍后再来"
            type="info"
            show-icon
            style="margin-bottom: 24px"
          >
            <template #icon><InfoCircleOutlined /></template>
          </a-alert>
          <div v-if="loadingVersions" class="loading-ct">
            <a-spin size="small"><template #description>加载版本列表...</template></a-spin>
          </div>

          <!-- 支持的语言 -->
          <div class="detail-section">
            <h3 class="detail-section-title"><CodeOutlined /> 支持的语言</h3>
            <div class="languages-list">
              <a-tag
                v-for="lang in template?.languages || []"
                :key="lang.id"
                :color="lang.isPrimary === 1 ? 'blue' : 'default'"
                size="large"
              >
                {{ getLanguageName(lang.languageId) }}
                <span v-if="lang.isPrimary === 1">(主语言)</span>
              </a-tag>
              <span v-if="!template?.languages?.length" class="no-lang">暂无语言信息</span>
            </div>
          </div>

          <!-- 详细介绍 -->
          <div v-if="template?.introduction" class="detail-section">
            <h3 class="detail-section-title"><FileTextOutlined /> 详细介绍</h3>
            <div class="intro-markdown" v-html="renderedIntro"></div>
          </div>
        </div>
      </div>

      <!-- Step 2: 配置变量 -->
      <div v-if="currentStep === 2" class="step-panel">
        <div class="variables-layout">
          <!-- 左侧模式切换 -->
          <div class="variables-sidebar">
            <div class="sidebar-header">
              <h3 class="sidebar-title">编辑模式</h3>
            </div>
            <div class="mode-tabs-vertical">
              <div
                class="mode-tab"
                :class="{ active: mode === 'simple' }"
                @click="mode = 'simple'"
              >
                <EditOutlined class="mode-icon" />
                <span class="mode-label">普通模式</span>
              </div>
              <div
                class="mode-tab"
                :class="{ active: mode === 'advanced' }"
                @click="mode = 'advanced'"
              >
                <CodeOutlined class="mode-icon" />
                <span class="mode-label">高级模式</span>
              </div>
            </div>

            <!-- 上下文注入 -->
            <div class="sidebar-section">
              <h3 class="sidebar-title">上下文注入</h3>
              <div class="context-options">
                <a-checkbox v-model:checked="injectProject">
                  项目信息
                </a-checkbox>
                <a-checkbox v-model:checked="injectTables">
                  表信息
                </a-checkbox>
              </div>
              <div v-if="injectProject || injectTables" class="project-selector">
                <a-select
                  v-model:value="selectedProjectId"
                  placeholder="选择项目"
                  :options="projectOptions"
                  allow-clear
                  size="small"
                  style="width: 100%"
                  @change="onProjectChange"
                />
              </div>
              <div v-if="injectTables && projectTables.length > 0" class="table-selector">
                <a-checkbox-group
                  v-model:value="contextSelectedTables"
                  :options="projectTables.map(t => ({ label: t.name, value: t.id }))"
                />
              </div>
            </div>
          </div>

          <!-- 右侧表单/编辑器 -->
          <div class="variables-content">
            <!-- 普通模式 -->
            <div v-show="mode === 'simple'" class="normal-mode">
              <VariableForm
                :schema="schema"
                :model-value="variables"
                @update:model-value="onVariablesUpdate"
              />
            </div>
            <!-- 高级模式 -->
            <div v-show="mode === 'advanced'" class="advanced-mode">
              <div class="editor-wrap">
                <div class="editor-header">
                  <span>JSON 编辑器</span>
                  <div class="editor-actions">
                    <a-button size="small" @click="formatJson">格式化</a-button>
                    <a-button size="small" type="primary" @click="syncFromSimple">同步普通模式</a-button>
                  </div>
                </div>
                <div ref="jsonEditorEl" class="json-editor" />
                <div class="editor-footer">
                  <span v-if="jsonValid" class="ok">JSON 格式正确</span>
                  <span v-else class="err">{{ jsonError }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Step 3: 预览导出 -->
      <div v-if="currentStep === 3" class="step-panel step-panel-preview">
        <div v-if="rendering" class="loading-ct">
          <a-spin size="large"><template #description>正在渲染文件预览...</template></a-spin>
        </div>
        <div v-else class="preview-layout">
          <div class="file-explorer">
            <div class="explorer-header">
              <span class="explorer-title">模板文件</span>
              <span class="explorer-count">{{ renderResult.length }} 个文件</span>
            </div>
            <div class="explorer-content">
              <a-tree
                v-if="fileTreeData.length > 0"
                :tree-data="fileTreeData"
                :selected-keys="[selectedFileKey]"
                :expanded-keys="expandedKeys"
                show-icon
                @select="onFileSelect"
                @expand="onExpand"
              />
              <a-empty v-else description="暂无文件" />
            </div>
          </div>
          <div class="code-preview-pane">
            <div class="file-header">
              <div class="file-info">
                <FileTextOutlined />
                <span class="file-name">{{ currentFileName || '未选择文件' }}</span>
              </div>
              <a-button
                size="small"
                @click="copyFileContent"
                :disabled="!currentFileContent"
                type="text"
              >
                <template #icon><CopyOutlined /></template>
                复制
              </a-button>
            </div>
            <div v-if="!currentFileContent && !renderError" class="no-file-selected">
              <FileTextOutlined style="font-size: 48px; color: var(--color-text-muted)" />
              <div>请选择左侧文件进行预览</div>
            </div>
            <a-alert
              v-if="renderError"
              :message="renderError"
              type="error"
              show-icon
              style="margin: 16px"
            />
            <div v-if="currentFileContent" class="code-content">
              <div ref="codeContainer" class="codemirror-container" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部按钮 -->
    <template #footer>
      <div class="wizard-footer">
        <a-button v-if="currentStep > 1" @click="prevStep" class="footer-btn">
          上一步
        </a-button>
        <div class="footer-spacer" />
        <a-button
          v-if="currentStep === 3"
          type="primary"
          @click="exportDialogVisible = true"
          class="footer-btn"
        >
          <template #icon><ExportOutlined /></template>
          导出文件
        </a-button>
        <a-button
          v-if="currentStep < 3"
          type="primary"
          :disabled="currentStep === 1 && versionList.length > 0 && !selectedVersion"
          @click="nextStep"
          class="footer-btn"
        >
          下一步
        </a-button>
      </div>
    </template>
  </a-drawer>

  <!-- 导出弹窗 -->
  <a-modal
    v-model:open="exportDialogVisible"
    title="导出渲染结果"
    @ok="doExport"
    :confirm-loading="exporting"
    ok-text="导出"
    cancel-text="取消"
  >
    <div class="export-form">
      <div class="export-field">
        <span class="export-label">输出目录</span>
        <a-input v-model:value="exportDir" placeholder="请输入输出目录路径" />
      </div>
      <div v-if="renderResult.length > 0" class="export-stats">
        <a-tag color="success">{{ renderResult.filter(f => !f.error).length }} 个文件</a-tag>
        <a-tag v-if="renderResult.some(f => f.error)" color="error">
          {{ renderResult.filter(f => f.error).length }} 个错误
        </a-tag>
      </div>
    </div>
  </a-modal>
</template>

<script setup>
import { ref, computed, watch, nextTick, onBeforeUnmount, h } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { message } from 'ant-design-vue'
import { marked } from 'marked'
import {
  CodeOutlined, EditOutlined,
  FileTextOutlined, CopyOutlined, ExportOutlined,
  FileOutlined, FolderOutlined,
  StarOutlined, TagsOutlined, InfoCircleOutlined, DownloadOutlined
} from '@ant-design/icons-vue'
import { getCategories, getLanguages } from '@/api/templates'
import { listReleases } from '@/api/releases'
import { getAllProjects, getProjectTables } from '@/api/projects'
import { getTemplateVariables } from '@/api/templateVariables'
import VariableForm from './VariableForm.vue'

const props = defineProps({
  open: Boolean,
  template: Object,
})
const emit = defineEmits(['update:open', 'exported'])

// 步骤
const currentStep = ref(1)
const steps = [{ title: '模板详情' }, { title: '配置变量' }, { title: '预览导出' }]

// 版本
const versionList = ref([])
const selectedVersion = ref('')
const loadingVersions = ref(false)

const versionOptions = computed(() =>
  versionList.value.map(v => ({
    label: v.version + (v.isLatest ? ' (最新)' : '') + (v.isDeprecated ? ' (已弃用)' : ''),
    value: v.version,
  }))
)

const renderedIntro = computed(() => {
  const text = props.template?.introduction
  if (!text) return ''
  return marked(text)
})

// 分类和语言
const categories = ref([])
const languages = ref([])

// 变量
const schema = ref(null)
const variables = ref({})
const rawJson = ref('')
const mode = ref('simple')
const jsonValid = ref(true)
const jsonError = ref('')

// 项目
const selectedProjectId = ref(null)
const projectOptions = ref([])
const projectTables = ref([])

// 上下文
const injectProject = ref(true)
const injectTables = ref(false)
const contextSelectedTables = ref([])

// 渲染
const rendering = ref(false)
const renderResult = ref([])
const renderError = ref('')

// 预览
const fileTreeData = ref([])
const selectedFileKey = ref('')
const expandedKeys = ref([])
const currentFileName = ref('')
const currentFileContent = ref('')
const fileContentMap = new Map() // key -> { fileName, fileContent, isDirectory }

// 导出
const exportDialogVisible = ref(false)
const exportDir = ref('')
const exporting = ref(false)

// 编辑器
const jsonEditorEl = ref(null)
const codeContainer = ref(null)
let jsonEditorView = null
let codeEditorView = null

// 打开抽屉时加载版本
watch(() => props.open, async (val) => {
  if (val && props.template) {
    currentStep.value = 1
    await Promise.all([loadCategories(), loadLanguages(), loadProjects()])
    await loadVersions(props.template.id)
  }
})

const onClose = () => {
  emit('update:open', false)
}

const getCategoryName = (categoryId) => categories.value.find(c => c.id === categoryId)?.name || categoryId
const getLanguageName = (languageId) => languages.value.find(l => l.id === languageId)?.name || languageId

const loadCategories = async () => {
  try {
    const res = await getCategories({ all: 1 })
    categories.value = res?.data?.categoriesList || []
  } catch { categories.value = [] }
}

const loadLanguages = async () => {
  try {
    const res = await getLanguages({ all: 1 })
    languages.value = res?.data?.languagesList || []
  } catch { languages.value = [] }
}

const loadVersions = async (templateId) => {
  loadingVersions.value = true
  versionList.value = []
  selectedVersion.value = ''
  try {
    const res = await listReleases(templateId)
    if (res?.data?.versions) {
      versionList.value = res.data.versions
      const latest = versionList.value.find(v => v.isLatest)
      selectedVersion.value = latest?.version || versionList.value[0]?.version || ''
    }
  } catch (e) {
    console.error('加载版本列表失败:', e)
  } finally {
    loadingVersions.value = false
  }
}

const loadProjects = async () => {
  try {
    const projects = await getAllProjects()
    console.log('projects:', projects)
    projectOptions.value = (Array.isArray(projects) ? projects : []).map(p => ({ label: p.name, value: p.id }))
  } catch (e) {
    console.error('loadProjects error:', e)
  }
}

const onProjectChange = async (projectId) => {
  projectTables.value = []
  contextSelectedTables.value = []
  if (!projectId) return
  try {
    const tables = await getProjectTables(projectId)
    projectTables.value = Array.isArray(tables) ? tables : []
  } catch (e) {
    console.error('onProjectChange error:', e)
  }
}

// 步骤导航
const nextStep = async () => {
  if (currentStep.value === 1) {
    if (versionList.value.length > 0 && !selectedVersion.value) {
      message.warning('请选择一个版本')
      return
    }
    await loadSchema()
    currentStep.value = 2
  } else if (currentStep.value === 2) {
    await doRender()
    if (renderResult.value.length > 0 || renderError.value) {
      currentStep.value = 3
    }
  }
}

const prevStep = () => {
  if (currentStep.value > 1) currentStep.value--
}

// 加载 schema
const loadSchema = async () => {
  variables.value = {}
  schema.value = null
  renderResult.value = []
  renderError.value = ''

  try {
    const res = await getTemplateVariables(props.template.id, selectedVersion.value)
    if (res?.data?.fieldSchemaJson) {
      const raw = JSON.parse(res.data.fieldSchemaJson)
      const defMap = { string: '', number: 0, boolean: false, text: '', select: undefined, 'multi-select': [], object: '{}', array: '[]' }
      const fields = Object.entries(raw).map(([name, cfg]) => ({
        name,
        title: cfg.title || name,
        description: cfg.description || '',
        type: cfg.type || 'string',
        required: !!cfg.required,
        default: cfg.default !== undefined ? cfg.default : (defMap[cfg.type] ?? ''),
        min: cfg.min,
        max: cfg.max,
        maxLength: cfg.maxLength,
        trueText: cfg.trueText,
        falseText: cfg.falseText,
        options: cfg.options?.map(o => ({ label: o.label || o, value: o.value !== undefined ? o.value : o })),
      }))
      schema.value = { fields }
      const defaults = {}
      for (const field of fields) {
        defaults[field.name] = field.default
      }
      variables.value = defaults
    } else {
      schema.value = { fields: [] }
    }
  } catch (e) {
    console.error('加载变量定义失败:', e)
    schema.value = { fields: [] }
  }
  syncToJson()
}

const onVariablesUpdate = (val) => {
  variables.value = val
  syncToJson()
}

const buildVariablesJson = () => {
  const result = { ...variables.value }
  const ctx = {}
  if (injectProject.value && selectedProjectId.value) {
    const proj = projectOptions.value.find(p => p.value === selectedProjectId.value)
    ctx.project = { id: selectedProjectId.value, name: proj?.label || '' }
  }
  if (injectTables.value && contextSelectedTables.value.length > 0) {
    ctx.tables = projectTables.value
      .filter(t => contextSelectedTables.value.includes(t.id))
      .map(t => ({ name: t.name, comment: t.comment || '', engine: t.engine || '' }))
  }
  if (Object.keys(ctx).length > 0) result.__context = ctx
  return result
}

const syncToJson = () => {
  const merged = buildVariablesJson()
  rawJson.value = JSON.stringify(merged, null, 2)
  if (jsonEditorView) {
    const tr = jsonEditorView.state.update({
      changes: { from: 0, to: jsonEditorView.state.doc.length, insert: rawJson.value },
    })
    jsonEditorView.dispatch(tr)
  }
}

// JSON 编辑器
const initJsonEditor = async () => {
  if (jsonEditorView) { jsonEditorView.destroy(); jsonEditorView = null }
  await nextTick()
  if (!jsonEditorEl.value) return

  const { EditorView, basicSetup } = await import('codemirror')
  const { json: jsonLang } = await import('@codemirror/lang-json')
  const { oneDark } = await import('@codemirror/theme-one-dark')

  const isDark = document.documentElement.getAttribute('data-theme') === 'dark' ||
    document.documentElement.classList.contains('dark')

  const extensions = [
    basicSetup,
    jsonLang(),
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        rawJson.value = update.state.doc.toString()
        validateJson()
      }
    }),
  ]
  if (isDark) extensions.push(oneDark)

  jsonEditorView = new EditorView({
    doc: rawJson.value || '{}',
    extensions,
    parent: jsonEditorEl.value,
  })
}

const validateJson = () => {
  try {
    const parsed = JSON.parse(rawJson.value)
    jsonValid.value = true
    jsonError.value = ''
    const { __context, ...userVars } = parsed
    variables.value = userVars
  } catch (e) {
    jsonValid.value = false
    jsonError.value = e.message
  }
}

const formatJson = () => {
  if (!jsonEditorView) return
  try {
    const formatted = JSON.stringify(JSON.parse(rawJson.value), null, 2)
    jsonEditorView.dispatch({
      changes: { from: 0, to: jsonEditorView.state.doc.length, insert: formatted },
    })
  } catch {
    message.error('JSON 格式错误，无法格式化')
  }
}

const syncFromSimple = () => {
  syncToJson()
  message.success('已同步到高级模式')
}

watch(mode, async (newMode) => {
  if (newMode === 'advanced') {
    syncToJson()
    await initJsonEditor()
  } else {
    validateJson()
  }
})

watch([injectProject, injectTables, contextSelectedTables], () => {
  if (mode.value === 'advanced') syncToJson()
})

// 渲染
const doRender = async () => {
  rendering.value = true
  renderError.value = ''
  renderResult.value = []
  fileTreeData.value = []
  selectedFileKey.value = ''
  currentFileName.value = ''
  currentFileContent.value = ''

  try {
    const vars = buildVariablesJson()
    const resultJson = await invoke('render_template_preview', {
      templateId: String(props.template.id),
      variables: vars,
      version: selectedVersion.value || null,
    })
    const parsed = JSON.parse(resultJson)
    renderResult.value = Array.isArray(parsed) ? parsed : []
    buildFileTree()
    selectFirstFile()
  } catch (e) {
    renderError.value = String(e)
  } finally {
    rendering.value = false
  }
}

// 构建文件树
const buildFileTree = () => {
  const flat = renderResult.value
  if (!Array.isArray(flat) || flat.length === 0) { fileTreeData.value = []; return }

  fileContentMap.clear()
  const nodeMap = new Map()
  flat.forEach(n => {
    const key = n.key || n.id
    nodeMap.set(n.id, { ...n, key, children: [] })
    fileContentMap.set(key, { fileName: n.file_name, fileContent: n.file_content, isDirectory: n.is_directory === 1 })
  })
  const roots = []
  flat.forEach(n => {
    const cur = nodeMap.get(n.id)
    if (n.parent_id === 0) roots.push(cur)
    else {
      const p = nodeMap.get(n.parent_id)
      if (p) p.children.push(cur)
    }
  })

  const toAntTree = (nodes) => {
    return [...nodes]
      .sort((a, b) => {
        const d = (b.is_directory || 0) - (a.is_directory || 0)
        return d !== 0 ? d : (a.file_name || '').localeCompare(b.file_name || '')
      })
      .map(n => ({
        key: n.key,
        title: n.file_name || n.key,
        icon: n.is_directory === 1 ? h(FolderOutlined) : h(FileOutlined),
        children: n.children?.length ? toAntTree(n.children) : [],
      }))
  }

  fileTreeData.value = toAntTree(roots)
  expandedKeys.value = flat.filter(n => n.is_directory === 1).map(n => n.key || n.id).filter(Boolean)
}

const findFirstFileKey = (nodes) => {
  for (const n of nodes) {
    const info = fileContentMap.get(n.key)
    if (info && !info.isDirectory) return n.key
    if (n.children?.length) { const k = findFirstFileKey(n.children); if (k) return k }
  }
  return null
}

const selectFirstFile = () => {
  const key = findFirstFileKey(fileTreeData.value)
  if (key) {
    selectedFileKey.value = key
    const info = fileContentMap.get(key)
    currentFileName.value = info?.fileName || ''
    currentFileContent.value = info?.fileContent || ''
    nextTick(() => createCodeEditor())
  }
}

const onFileSelect = (keys) => {
  if (!keys?.length) return
  const key = keys[0]
  selectedFileKey.value = key
  const info = fileContentMap.get(key)
  if (!info) return
  if (info.isDirectory) { currentFileName.value = ''; currentFileContent.value = ''; return }
  currentFileName.value = info.fileName || ''
  currentFileContent.value = info.fileContent || ''
  nextTick(() => createCodeEditor())
}

const onExpand = (keys) => { expandedKeys.value = keys }

const langExtMap = {
  js: 'javascript', ts: 'javascript', jsx: 'javascript', tsx: 'javascript',
  json: 'json', py: 'python', rs: 'rust', go: 'go', java: 'java',
  html: 'html', css: 'css', md: 'markdown', xml: 'xml', yaml: 'yaml',
  yml: 'yaml', sql: 'sql', vue: 'javascript', sh: 'javascript', toml: 'javascript',
}

const createCodeEditor = async () => {
  if (codeEditorView) { codeEditorView.destroy(); codeEditorView = null }
  await nextTick()
  if (!codeContainer.value || !currentFileContent.value) return

  const { EditorView, basicSetup } = await import('codemirror')
  const { oneDark } = await import('@codemirror/theme-one-dark')
  const ext = (currentFileName.value || '').split('.').pop()?.toLowerCase()
  const langMod = langExtMap[ext]

  const isDark = document.documentElement.getAttribute('data-theme') === 'dark' ||
    document.documentElement.classList.contains('dark')

  const extensions = [
    basicSetup,
    EditorView.editable.of(false),
    EditorView.lineWrapping,
  ]
  if (isDark) extensions.push(oneDark)

  if (langMod) {
    try {
      const mod = await import(`@codemirror/lang-${langMod}`)
      const langFn = mod[langMod] || mod.default
      if (langFn) extensions.push(langFn())
    } catch {}
  }

  codeEditorView = new EditorView({
    doc: currentFileContent.value,
    extensions,
    parent: codeContainer.value,
  })
}

const copyFileContent = async () => {
  if (!currentFileContent.value) return
  try {
    await navigator.clipboard.writeText(currentFileContent.value)
    message.success('已复制到剪贴板')
  } catch {
    message.error('复制失败')
  }
}

// 导出
const doExport = async () => {
  if (!exportDir.value.trim()) {
    message.warning('请输入输出目录')
    return
  }
  exporting.value = true
  try {
    const vars = buildVariablesJson()
    const resultJson = await invoke('cmd_render_and_export', {
      templateId: String(props.template.id),
      version: selectedVersion.value || null,
      variablesJson: vars,
      outputDir: exportDir.value.trim(),
    })
    const result = JSON.parse(resultJson)
    if (result.errors?.length > 0) {
      message.warning(`导出完成，${result.exported} 个文件成功，${result.errors.length} 个失败`)
    } else {
      message.success(`导出成功，共 ${result.exported} 个文件`)
    }
    exportDialogVisible.value = false
    emit('exported')
  } catch (e) {
    message.error('导出失败: ' + e)
  } finally {
    exporting.value = false
  }
}

onBeforeUnmount(() => {
  if (jsonEditorView) jsonEditorView.destroy()
  if (codeEditorView) codeEditorView.destroy()
})
</script>

<style scoped>
/* Header */
.wizard-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.wz-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.header-right {
  display: flex;
  align-items: center;
}

.steps-compact {
  display: flex;
  align-items: center;
  gap: 24px;
}

.step-item {
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.3s ease;
}

.step-dot {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--color-border);
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.3s ease;
  flex-shrink: 0;
}

.step-item.active .step-dot {
  background: var(--color-primary);
  color: #fff;
  transform: scale(1.15);
  box-shadow: 0 2px 8px rgba(24, 144, 255, 0.4);
}

.step-item.completed .step-dot {
  background: var(--color-success);
  color: #fff;
}

.step-text {
  font-size: 14px;
  color: var(--color-text-secondary);
  font-weight: 500;
  white-space: nowrap;
  transition: all 0.3s ease;
}

.step-item.active .step-text {
  color: var(--color-primary);
  font-weight: 600;
}

.step-item.completed .step-text {
  color: var(--color-success);
}

.step-arrow {
  color: var(--color-text-muted);
  font-size: 18px;
  user-select: none;
  margin: 0 -8px;
}

/* Content */
.wizard-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.wizard-content-preview {
  padding: 0 !important;
  overflow: hidden !important;
}

.step-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.step-panel-preview {
  padding: 0;
  overflow: hidden;
}

/* Step 1: Template Detail */
.template-detail {
  flex: 1;
  overflow-y: auto;
  padding: 32px 48px;
  max-width: 800px;
  margin: 0 auto;
  width: 100%;
}

.detail-header {
  margin-bottom: 24px;
}

.detail-title-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.detail-name {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  color: var(--color-text);
}

.detail-desc {
  font-size: 15px;
  color: var(--color-text-secondary);
  line-height: 1.8;
  margin: 0;
}

.detail-section {
  margin-bottom: 28px;
}

.detail-section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 16px 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.form-hint {
  font-size: 12px;
  color: var(--color-text-muted);
  margin-top: 6px;
}

.languages-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.no-lang {
  font-size: 13px;
  color: var(--color-text-muted);
}

.intro-markdown {
  font-size: 14px;
  line-height: 1.8;
  color: var(--color-text-secondary);
  max-height: 400px;
  overflow-y: auto;
}

.intro-markdown :deep(h1),
.intro-markdown :deep(h2),
.intro-markdown :deep(h3),
.intro-markdown :deep(h4) {
  color: var(--color-text);
  margin: 16px 0 8px;
  font-weight: 600;
}

.intro-markdown :deep(h1) { font-size: 20px; }
.intro-markdown :deep(h2) { font-size: 18px; }
.intro-markdown :deep(h3) { font-size: 16px; }

.intro-markdown :deep(p) {
  margin: 8px 0;
}

.intro-markdown :deep(code) {
  background: var(--color-bg-elevated);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 13px;
  font-family: 'Consolas', 'Monaco', monospace;
}

.intro-markdown :deep(pre) {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 12px 16px;
  overflow-x: auto;
  margin: 12px 0;
}

.intro-markdown :deep(pre code) {
  background: none;
  padding: 0;
}

.intro-markdown :deep(ul),
.intro-markdown :deep(ol) {
  padding-left: 20px;
  margin: 8px 0;
}

.intro-markdown :deep(li) {
  margin: 4px 0;
}

.intro-markdown :deep(blockquote) {
  border-left: 3px solid var(--color-primary);
  padding-left: 12px;
  margin: 12px 0;
  color: var(--color-text-muted);
}

.intro-markdown :deep(a) {
  color: var(--color-primary);
  text-decoration: none;
}

.intro-markdown :deep(a:hover) {
  text-decoration: underline;
}

.intro-markdown :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 12px 0;
}

.intro-markdown :deep(th),
.intro-markdown :deep(td) {
  border: 1px solid var(--color-border);
  padding: 8px 12px;
  text-align: left;
}

.intro-markdown :deep(th) {
  background: var(--color-bg-elevated);
  font-weight: 600;
}

.intro-markdown :deep(hr) {
  border: none;
  border-top: 1px solid var(--color-border);
  margin: 16px 0;
}

/* Step 2: Variables */
.variables-layout {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.variables-sidebar {
  width: 200px;
  flex-shrink: 0;
  background: var(--color-bg-elevated);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.sidebar-header {
  padding: 20px 16px 12px;
  border-bottom: 1px solid var(--color-border);
}

.sidebar-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 0;
}

.mode-tabs-vertical {
  padding: 12px 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.mode-tab {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--color-text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.mode-tab:hover {
  background: var(--color-bg-spotlight);
  color: var(--color-text);
}

.mode-tab.active {
  background: var(--color-primary-bg);
  color: var(--color-primary);
}

.mode-icon {
  font-size: 18px;
}

.sidebar-section {
  padding: 12px 16px;
  border-top: 1px solid var(--color-border);
}

.sidebar-section .sidebar-title {
  margin-bottom: 12px;
}

.context-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.context-options :deep(.ant-checkbox-wrapper) {
  display: flex;
  align-items: center;
  min-height: 28px;
  cursor: pointer;
  user-select: none;
}

.project-selector {
  margin-top: 10px;
}

.table-selector {
  margin-top: 8px;
  padding: 8px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  max-height: 150px;
  overflow: auto;
}

.table-selector :deep(.ant-checkbox-group) {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.variables-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px;
  background: var(--color-bg-container);
}

.normal-mode {
  width: 100%;
}

.advanced-mode {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.editor-wrap {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
  background: var(--color-bg-container);
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 400px;
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border);
  font-size: 14px;
  font-weight: 500;
  flex-shrink: 0;
}

.editor-actions {
  display: flex;
  gap: 8px;
}

.json-editor {
  flex: 1;
  min-height: 300px;
  overflow: auto;
}

.json-editor :deep(.cm-editor) {
  height: 100%;
}

.editor-footer {
  padding: 8px 16px;
  background: var(--color-bg-elevated);
  font-size: 12px;
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
}

.editor-footer .ok {
  color: var(--color-success);
}

.editor-footer .err {
  color: var(--color-error);
}

/* Step 3: Preview */
.preview-layout {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.file-explorer {
  width: 280px;
  background: var(--color-bg-elevated);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.explorer-header {
  height: 48px;
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  flex-shrink: 0;
}

.explorer-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
}

.explorer-count {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.explorer-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.code-preview-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.file-header {
  height: 48px;
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  flex-shrink: 0;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  font-family: 'Consolas', 'Monaco', monospace;
}

.no-file-selected {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--color-text-muted);
  font-size: 14px;
}

.code-content {
  flex: 1;
  overflow: hidden;
  background: var(--color-bg-container);
}

.codemirror-container {
  height: 100%;
}

.codemirror-container :deep(.cm-editor) {
  height: 100% !important;
}

.loading-ct {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
}

/* Footer */
.wizard-footer {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 8px;
}

.footer-spacer {
  flex: 1;
}

.footer-btn {
  height: 32px;
  font-size: 13px;
  padding: 0 20px;
}

/* Export Modal */
.export-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.export-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.export-label {
  font-size: 13px;
  font-weight: 500;
}

.export-stats {
  display: flex;
  gap: 8px;
}
</style>
