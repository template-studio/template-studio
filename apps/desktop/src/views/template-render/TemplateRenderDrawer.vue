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
      <TemplateDetailPanel
        v-if="currentStep === 1"
        :template="template"
        :version-list="versionList"
        :version-options="versionOptions"
        :selected-version="selectedVersion"
        :loading-versions="loadingVersions"
        :categories="categories"
        :languages="languages"
        :rendered-intro="renderedIntro"
        @update:selected-version="selectedVersion = $event"
      />

      <VariableConfigPanel
        v-if="currentStep === 2"
        :mode="mode"
        :schema="schema"
        :variables="variables"
        :inject-project="injectProject"
        :inject-tables="injectTables"
        :selected-project-id="selectedProjectId"
        :project-options="projectOptions"
        :project-tables="projectTables"
        :context-selected-tables="contextSelectedTables"
        :json-valid="jsonValid"
        :json-error="jsonError"
        @update:mode="mode = $event"
        @update:variables="onVariablesUpdate"
        @update:inject-project="injectProject = $event"
        @update:inject-tables="injectTables = $event"
        @update:context-selected-tables="contextSelectedTables = $event"
        @project-change="onProjectChange"
        @format-json="formatJson"
        @sync-from-simple="syncFromSimple"
        @editor-mounted="onJsonEditorMounted"
      />

      <RenderPreviewPanel
        v-if="currentStep === 3"
        :rendering="rendering"
        :file-tree-data="fileTreeData"
        :selected-file-key="selectedFileKey"
        :expanded-keys="expandedKeys"
        :current-file-name="currentFileName"
        :current-file-content="currentFileContent"
        :render-error="renderError"
        :file-count="renderResult.length"
        @file-select="onFileSelect"
        @expand="onExpand"
        @copy-file="copyFileContent"
        @editor-mounted="onCodeEditorMounted"
      />
    </div>

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

  <ExportDialog
    v-model:open="exportDialogVisible"
    :exporting="exporting"
    :export-dir="exportDir"
    :file-count="renderResult.length"
    :success-count="renderResult.filter(f => !f.error).length"
    :error-count="renderResult.filter(f => f.error).length"
    @update:export-dir="exportDir = $event"
    @export="doExport"
  />
</template>

<script setup>
import { ref, computed, watch, nextTick, onBeforeUnmount, h } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { message } from 'ant-design-vue'
import { notify } from '@/utils/notify'
import { marked } from 'marked'
import { ExportOutlined, FolderOutlined, FileOutlined } from '@ant-design/icons-vue'
import { getCategories, getLanguages } from '@/api/templates'
import { listReleases } from '@/api/releases'
import { getAllProjects, getProjectTables } from '@/api/projects'
import { getTemplateVariables } from '@/api/templateVariables'
import TemplateDetailPanel from './components/TemplateDetailPanel.vue'
import VariableConfigPanel from './components/VariableConfigPanel.vue'
import RenderPreviewPanel from './components/RenderPreviewPanel.vue'
import ExportDialog from './components/ExportDialog.vue'

const props = defineProps({
  open: Boolean,
  template: Object,
})
const emit = defineEmits(['update:open', 'exported'])

const currentStep = ref(1)
const steps = [{ title: '模板详情' }, { title: '配置变量' }, { title: '预览导出' }]

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

const categories = ref([])
const languages = ref([])

const schema = ref(null)
const variables = ref({})
const rawJson = ref('')
const mode = ref('simple')
const jsonValid = ref(true)
const jsonError = ref('')

const selectedProjectId = ref(null)
const projectOptions = ref([])
const projectTables = ref([])

const injectProject = ref(true)
const injectTables = ref(false)
const contextSelectedTables = ref([])

const rendering = ref(false)
const renderResult = ref([])
const renderError = ref('')

const fileTreeData = ref([])
const selectedFileKey = ref('')
const expandedKeys = ref([])
const currentFileName = ref('')
const currentFileContent = ref('')
const fileContentMap = new Map()

const exportDialogVisible = ref(false)
const exportDir = ref('')
const exporting = ref(false)

let jsonEditorEl = null
let codeContainer = null
let jsonEditorView = null
let codeEditorView = null

const onJsonEditorMounted = (el) => { jsonEditorEl = el }
const onCodeEditorMounted = (el) => { codeContainer = el }

watch(() => props.open, async (val) => {
  if (val && props.template) {
    currentStep.value = 1
    await Promise.all([loadCategories(), loadLanguages(), loadProjects()])
    await loadVersions(props.template.id)
  }
})

watch(currentStep, (newStep, oldStep) => {
  if (oldStep === 2 && jsonEditorView) {
    jsonEditorView.destroy()
    jsonEditorView = null
  }
  if (oldStep === 3 && codeEditorView) {
    codeEditorView.destroy()
    codeEditorView = null
  }
})

const onClose = () => {
  emit('update:open', false)
}

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

const initJsonEditor = async () => {
  if (jsonEditorView) { jsonEditorView.destroy(); jsonEditorView = null }
  await nextTick()
  if (!jsonEditorEl) return

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
    parent: jsonEditorEl,
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
  if (!codeContainer || !currentFileContent.value) return

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
    parent: codeContainer,
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
      notify({ type: 'warning', title: '导出完成（部分失败）', content: `${result.exported} 个文件成功，${result.errors.length} 个失败` })
    } else {
      notify({ type: 'success', title: '导出成功', content: `共 ${result.exported} 个文件已导出到 ${exportDir.value}` })
    }
    exportDialogVisible.value = false
    emit('exported')
  } catch (e) {
    notify({ type: 'error', title: '导出失败', content: String(e) })
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
</style>
