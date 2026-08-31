<template>
  <a-drawer :open="open" @update:open="$emit('update:open', $event)" placement="right" :width="'100vw'" :closable="true" @close="cancelWizard">
    <template #title>
      <div class="wizard-header">
        <div class="header-left">
          <span class="wz-name">{{ wizardData.template?.name || '' }}</span>
          <a-tag v-if="currentStep < 4" color="purple">模板配置</a-tag>
          <a-tag v-else color="blue">预览确认</a-tag>
        </div>
        <div class="header-right">
          <div v-if="currentStep < 4" class="steps-compact">
            <div v-for="(step, index) in steps" :key="step.title" class="step-item" :class="{ active: currentStep === index + 1, completed: currentStep > index + 1 }">
              <div class="step-dot">{{ index + 1 }}</div>
              <div class="step-text">{{ step.title }}</div>
            </div>
          </div>
          <div v-else class="wizard-header-actions">
            <a-button @click="prevStep" size="small">上一步</a-button>
            <a-button type="primary" @click="confirmCreateProject" size="small">创建项目</a-button>
          </div>
        </div>
      </div>
    </template>
    <div class="wizard-content" :class="{ 'wizard-content-preview': currentStep === 4, 'wizard-content-variables': currentStep === 3 && variableMode === 'normal' }">
      <StepTemplateIntro v-if="currentStep === 1"
        :wizardData="wizardData"
        :versionList="versionList"
        :versionOptions="versionOptions"
        :renderedIntro="renderedIntro"
        :isDownloading="isDownloading"
        :downloadProgress="downloadProgress"
        :getCategoryName="getCategoryName"
        :getLanguageName="getLanguageName"
        @update:version="wizardData.version = $event"
        @version-change="handleVersionChange"
      />
      <StepPathConfig v-if="currentStep === 2"
        ref="stepPathConfigRef"
        v-model:projectName="wizardData.projectName"
        :outputDir="wizardData.outputDir"
        :finalOutputPath="finalOutputPath"
        :outputPathExists="outputPathExists"
        :projectNameRules="projectNameRules"
        @select-output-dir="selectOutputDir"
      />
      <StepVariables v-if="currentStep === 3"
        ref="stepVariablesRef"
        :loadingVariables="loadingVariables"
        :variableDefinitions="variableDefinitions"
        :variableMode="variableMode"
        :variables="wizardData.variables"
        :jsonValid="jsonValid"
        :jsonError="jsonError"
        @update:variableMode="variableMode = $event"
        @update:variable="wizardData.variables[$event.name] = $event.value"
        @format-json="formatVariablesJson"
        @sync-normal-mode="syncFromNormalMode"
      />
      <StepPreview v-if="currentStep === 4"
        ref="stepPreviewRef"
        :loadingPreview="loadingPreview"
        :showPreview="showPreview"
        :fileTreeData="fileTreeData"
        :selectedFileKey="selectedFileKey"
        :expandedKeys="expandedKeys"
        :currentFileName="currentFileName"
        :currentFileContent="currentFileContent"
        @file-select="onFileSelect"
        @expand="onExpand"
        @copy-file-content="copyFileContent"
      />
    </div>
    <template v-if="currentStep < 4" #footer>
      <div class="wizard-footer">
        <a-button v-if="currentStep > 1" @click="prevStep" size="small" class="footer-btn">上一步</a-button>
        <a-button type="primary" @click="nextStep" :disabled="currentStep === 1 && versionList.length === 0" size="small" class="footer-btn">{{ currentStep === 1 && versionList.length === 0 ? '模板尚未开放' : '下一步' }}</a-button>
      </div>
    </template>
  </a-drawer>
</template>

<script setup>
import { ref, computed, watch, h, nextTick, onBeforeUnmount } from 'vue'
import { message, Modal } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { marked } from 'marked'
import { debounce } from '@/utils/debounce'
import { FolderOutlined, FileOutlined } from '@ant-design/icons-vue'
import { getCategories, getLanguages } from '@/api/templates'
import { listReleases } from '@/api/releases'
import { EditorView, lineNumbers, highlightActiveLineGutter, keymap } from '@codemirror/view'
import { EditorState } from '@codemirror/state'
import { defaultHighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { defaultKeymap } from '@codemirror/commands'
import { dracula } from '@uiw/codemirror-theme-dracula'
import { javascript } from '@codemirror/lang-javascript'; import { html } from '@codemirror/lang-html'; import { css } from '@codemirror/lang-css'
import { json } from '@codemirror/lang-json'; import { markdown } from '@codemirror/lang-markdown'; import { python } from '@codemirror/lang-python'
import { java } from '@codemirror/lang-java'; import { cpp } from '@codemirror/lang-cpp'; import { rust } from '@codemirror/lang-rust'
import { go } from '@codemirror/lang-go'; import { sql } from '@codemirror/lang-sql'; import { xml } from '@codemirror/lang-xml'
import { yaml } from '@codemirror/lang-yaml'; import { vue } from '@codemirror/lang-vue'
import StepTemplateIntro from './wizard/StepTemplateIntro.vue'
import StepPathConfig from './wizard/StepPathConfig.vue'
import StepVariables from './wizard/StepVariables.vue'
import StepPreview from './wizard/StepPreview.vue'

const props = defineProps({ open: Boolean, template: Object })
const emit = defineEmits(['update:open', 'created'])

const currentStep = ref(1); const wizardData = ref({ template: null, version: '', projectName: '', outputDir: '', variables: {} })
const versionList = ref([]); const isDownloading = ref(false); const downloadProgress = ref(0)
const outputPathExists = ref(false); const loadingVariables = ref(false); const variableDefinitions = ref([]); const variableMode = ref('normal')
let jsonEditor = null; const jsonValid = ref(true); const jsonError = ref('')
const loadingPreview = ref(false); const fileTreeData = ref([]); const selectedFileKey = ref(''); const expandedKeys = ref([])
const currentFileName = ref(''); const currentFileContent = ref(''); let editorView = null
const stepPathConfigRef = ref(null); const stepVariablesRef = ref(null); const stepPreviewRef = ref(null)

const languageMap = { js: javascript(), javascript: javascript(), ts: javascript({ typescript: true }), typescript: javascript({ typescript: true }),
  jsx: javascript({ jsx: true }), tsx: javascript({ typescript: true, jsx: true }), vue: vue(), html: html(), htm: html(),
  css: css(), scss: css(), sass: css(), less: css(), json: json(), md: markdown(), markdown: markdown(),
  py: python(), python: python(), java: java(), cpp: cpp(), cc: cpp(), cxx: cpp(), c: cpp(),
  rs: rust(), rust: rust(), go: go(), sql: sql(), xml: xml(), yaml: yaml(), yml: yaml() }

function getLanguageExtension(filename) { return languageMap[filename.split('.').pop()?.toLowerCase()] || null }
const showPreview = computed(() => !loadingPreview.value && fileTreeData.value != null)
const projectNameRules = [{ required: true, message: '请输入项目名称', trigger: 'blur' }, { min: 2, max: 50, message: '项目名称长度应在 2 到 50 个字符之间', trigger: 'blur' }, { pattern: /^[a-zA-Z0-9_-]+$/, message: '项目名称只能包含字母、数字、下划线和连字符', trigger: 'blur' }]
const steps = [{ title: '模板介绍' }, { title: '路径配置' }, { title: '配置变量' }, { title: '预览确认' }]
const categories = ref([{ id: 'all', name: '全部' }]); const languages = ref([{ id: 'all', name: '全部' }])
const versionOptions = computed(() => versionList.value.map(v => ({
  label: `${v.version}${v.isLatest ? ' (当前)' : ''}${v.isDeprecated ? ' [已弃用]' : ''}`, value: v.version
})))
const renderedIntro = computed(() => {
  const text = wizardData.value.template?.introduction
  if (!text) return ''
  return marked(text)
})
const finalOutputPath = computed(() => {
  if (!wizardData.value.outputDir || !wizardData.value.projectName) return ''
  return `${wizardData.value.outputDir.replace(/[\\/]+$/, '')}\\${wizardData.value.projectName}`
})
watch(() => props.open, async (val) => {
  if (val && props.template) {
    wizardData.value = { template: props.template, version: '', projectName: '', outputDir: '', variables: {} }
    currentStep.value = 1
    loadingPreview.value = false; fileTreeData.value = []; selectedFileKey.value = ''
    expandedKeys.value = []; currentFileName.value = ''; currentFileContent.value = ''
    await loadCategories(); await loadLanguages(); await loadVersions(props.template.id)
  }
})

const getLanguageName = (id) => languages.value.find(l => l.id === id)?.name || id
const getCategoryName = (id) => categories.value.find(c => c.id === id)?.name || id
const nextStep = async () => {
  if (currentStep.value === 1) {
    try {
      const dv = wizardData.value.version
      if (await invoke('check_template_downloaded', { templateId: wizardData.value.template.id.toString(), version: dv })) {
        await loadVariableDefinitions(); if (currentStep.value < 4) currentStep.value++; return
      }
      isDownloading.value = true; downloadProgress.value = 0
      await invoke('download_template', { templateId: wizardData.value.template.id.toString(), version: dv })
      isDownloading.value = false; downloadProgress.value = 100
      await loadVariableDefinitions(); if (currentStep.value < 4) currentStep.value++
    } catch (e) { message.error(`下载模板失败: ${e}`); isDownloading.value = false; downloadProgress.value = 0 }
  } else if (currentStep.value === 2) {
    try { await stepPathConfigRef.value?.validate(); if (currentStep.value < 4) currentStep.value++ } catch {}
  } else if (currentStep.value === 3) {
    await loadPreviewFiles(); if (currentStep.value < 4) currentStep.value++
  } else if (currentStep.value < 4) { currentStep.value++ }
}
const prevStep = () => { if (currentStep.value > 1) currentStep.value-- }
const selectOutputDir = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const sel = await open({ directory: true, multiple: false, title: '选择项目输出目录' })
    if (sel) wizardData.value.outputDir = sel
  } catch (e) { console.error('选择目录失败:', e); message.error('选择目录失败') }
}

const confirmCreateProject = async () => {
  try {
    const outputPath = finalOutputPath.value
    if (await invoke('check_directory_exists', { path: outputPath })) {
      Modal.confirm({
        title: '目录已存在', content: `项目输出路径 "${outputPath}" 已存在，是否覆盖？`,
        okText: '覆盖', okType: 'danger', cancelText: '取消',
        onOk: async () => { try { await invoke('remove_directory', { path: outputPath }); await doCreateProject() } catch (e) { message.error(`删除目录失败: ${e}`) } }
      })
    } else { await doCreateProject() }
  } catch (e) { console.error('检查目录失败:', e); message.error('检查目录失败') }
}

const doCreateProject = async () => {
  try {
    isDownloading.value = true; downloadProgress.value = 0
    const result = await invoke('generate_project', {
      templateId: wizardData.value.template.id.toString(), variables: wizardData.value.variables,
      outputPath: finalOutputPath.value, version: wizardData.value.version || null
    })
    isDownloading.value = false; downloadProgress.value = 100
    message.success(result || '项目创建成功！')
    emit('created'); emit('update:open', false)
    isDownloading.value = false; downloadProgress.value = 0
  } catch (e) { console.error('创建项目失败:', e); message.error(`创建项目失败: ${e}`); isDownloading.value = false; downloadProgress.value = 0 }
}

const cancelWizard = () => {
  emit('update:open', false); currentStep.value = 1
  wizardData.value = { template: null, version: '', projectName: '', outputDir: '', variables: {} }
  loadingPreview.value = false; fileTreeData.value = []; selectedFileKey.value = ''
  expandedKeys.value = []; currentFileName.value = ''; currentFileContent.value = ''
}

const loadVersions = async (templateId) => {
  try {
    const res = await listReleases(templateId)
    if (res?.data?.versions) {
      versionList.value = res.data.versions
      const latest = versionList.value.find(v => v.isLatest)
      wizardData.value.version = latest?.version || versionList.value[0]?.version || ''
    } else { versionList.value = [] }
  } catch (e) { console.error('加载版本列表失败:', e); versionList.value = [] }
}
const handleVersionChange = async (v) => { wizardData.value.version = v; await loadVariableDefinitions() }
const loadCategories = async () => {
  try { const r = await getCategories({ all: 1 }); categories.value = r?.data?.categoriesList ? [{ id: 'all', name: '全部' }, ...r.data.categoriesList] : [{ id: 'all', name: '全部' }] } catch { categories.value = [{ id: 'all', name: '全部' }] }
}
const loadLanguages = async () => {
  try { const r = await getLanguages({ all: 1 }); languages.value = r?.data?.languagesList ? [{ id: 'all', name: '全部' }, ...r.data.languagesList] : [{ id: 'all', name: '全部' }] } catch { languages.value = [{ id: 'all', name: '全部' }] }
}

const loadVariableDefinitions = async () => {
  if (!wizardData.value.template?.id || !wizardData.value.version) return
  try {
    loadingVariables.value = true
    const { getTemplateVariables } = await import('@/api/templateVariables')
    const res = await getTemplateVariables(wizardData.value.template.id, wizardData.value.version)
    if (res?.data?.fieldSchemaJson) {
      const schema = JSON.parse(res.data.fieldSchemaJson)
      const defMap = { string: '', number: 0, boolean: false, text: '', select: undefined, 'multi-select': [], object: '{}', array: '[]' }
      variableDefinitions.value = Object.entries(schema).map(([name, cfg]) => ({
        name, title: cfg.title || name, description: cfg.description || '', type: cfg.type || 'string',
        required: !!cfg.required, default: cfg.default !== undefined ? cfg.default : (defMap[cfg.type] ?? ''),
        min: cfg.min, max: cfg.max, maxLength: cfg.maxLength, trueText: cfg.trueText, falseText: cfg.falseText,
        options: cfg.options?.map(o => ({ label: o.label || o, value: o.value !== undefined ? o.value : o }))
      }))
      variableDefinitions.value.forEach(v => { if (wizardData.value.variables[v.name] === undefined) wizardData.value.variables[v.name] = v.default })
    } else { variableDefinitions.value = [] }
  } catch (e) { console.error('加载变量定义失败:', e); variableDefinitions.value = [] } finally { loadingVariables.value = false }
}

const initJsonEditor = async () => {
  const container = stepVariablesRef.value?.jsonEditorContainer
  if (!container) return
  if (jsonEditor) { jsonEditor.destroy(); jsonEditor = null }
  try {
    jsonEditor = new EditorView({
      state: EditorState.create({ doc: JSON.stringify(wizardData.value.variables, null, 2), extensions: [dracula, javascript(), keymap.of(defaultKeymap), lineNumbers(), EditorView.updateListener.of(() => validateJson())] }),
      parent: container
    })
    validateJson()
  } catch (e) { console.error('初始化 JSON 编辑器失败:', e) }
}
const validateJson = () => { if (!jsonEditor) return; try { JSON.parse(jsonEditor.state.doc.toString()); jsonValid.value = true; jsonError.value = '' } catch (e) { jsonValid.value = false; jsonError.value = e.message } }
const formatVariablesJson = () => { if (!jsonEditor) return; try { jsonEditor.dispatch({ changes: { from: 0, to: jsonEditor.state.doc.length, insert: JSON.stringify(JSON.parse(jsonEditor.state.doc.toString()), null, 2) } }) } catch { message.error('JSON 格式错误，无法格式化') } }
const syncFromNormalMode = () => { if (!jsonEditor) return; try { jsonEditor.dispatch({ changes: { from: 0, to: jsonEditor.state.doc.length, insert: JSON.stringify(wizardData.value.variables, null, 2) } }); message.success('已同步到高级模式') } catch (e) { message.error('同步失败：' + e.message) } }

watch(variableMode, async (newMode, oldMode) => {
  if (newMode === 'advanced') {
    if (!jsonEditor) { await nextTick(); await nextTick(); initJsonEditor() }
    else { await nextTick(); try { jsonEditor.dispatch({ changes: { from: 0, to: jsonEditor.state.doc.length, insert: JSON.stringify(wizardData.value.variables, null, 2) } }) } catch {} }
  } else if (oldMode === 'advanced' && jsonEditor) {
    try { Object.assign(wizardData.value.variables, JSON.parse(jsonEditor.state.doc.toString())) } catch {}
  }
})

const loadPreviewFiles = async () => {
  if (!wizardData.value.template?.id || !wizardData.value.version) return
  try {
    loadingPreview.value = true; selectedFileKey.value = ''; expandedKeys.value = []; currentFileName.value = ''; currentFileContent.value = ''
    const treeJson = await invoke('render_template_preview', { templateId: wizardData.value.template.id.toString(), variables: wizardData.value.variables, version: wizardData.value.version })
    const flatTree = JSON.parse(treeJson); const tree = buildTreeFromFlatArray(flatTree)
    if (Array.isArray(tree)) {
      fileTreeData.value = convertTreeToAntDesign(tree)
      if (tree.length > 0) {
        expandedKeys.value = tree.filter(n => n.isDirectory === 1).map(n => n.key || n.id).filter(Boolean)
        const ff = findFirstFile(tree)
        if (ff) { const k = ff.key || ff.id; if (k) { await nextTick(); selectedFileKey.value = k; currentFileName.value = ff.file_name || ''; currentFileContent.value = ff.file_content || '' } }
      }
    } else { fileTreeData.value = [] }
  } catch (e) { console.error('加载文件预览失败:', e); message.error('加载文件预览失败'); fileTreeData.value = [] } finally { loadingPreview.value = false }
}

const buildTreeFromFlatArray = (arr) => {
  if (!Array.isArray(arr)) return []
  const map = new Map(); const roots = []
  arr.forEach(n => map.set(n.id, { ...n, children: [] }))
  arr.forEach(n => { const cur = map.get(n.id); if (n.parent_id === 0) roots.push(cur); else { const p = map.get(n.parent_id); if (p) p.children.push(cur) } })
  return roots
}

const convertTreeToAntDesign = (tree) => {
  if (!Array.isArray(tree)) return []
  return [...tree].sort((a, b) => { const d = (b.is_directory || 0) - (a.is_directory || 0); return d !== 0 ? d : (a.file_name || '').toLowerCase().localeCompare((b.file_name || '').toLowerCase()) })
    .map(n => ({ key: n.key || n.id, title: n.file_name || n.key, file_name: n.file_name, is_directory: n.is_directory, file_path: n.file_path, file_content: n.file_content, fileName: n.file_name, isDirectory: n.is_directory === 1, filePath: n.file_path, fileContent: n.file_content, icon: n.is_directory === 1 ? h(FolderOutlined) : h(FileOutlined), children: n.children?.length ? convertTreeToAntDesign(n.children) : [] }))
}

const findFirstFile = (nodes) => { if (!nodes?.length) return null; for (const n of nodes) { if (n.is_directory !== 1) return n; if (n.children?.length) { const f = findFirstFile(n.children); if (f) return f } } return null }

const onFileSelect = async (keys) => {
  if (!keys?.length) return
  selectedFileKey.value = keys[0]
  const find = (ns, k) => { if (!ns) return null; for (const n of ns) { if (n.key === k) return n; if (n.children?.length) { const f = find(n.children, k); if (f) return f } } return null }
  const node = find(fileTreeData.value, keys[0])
  if (!node) return
  if (node.isDirectory) { currentFileName.value = ''; currentFileContent.value = ''; return }
  currentFileName.value = node.fileName || node.file_name || ''
  currentFileContent.value = node.fileContent || node.file_content || ''
  await nextTick(); createOrUpdateEditor()
}

const createOrUpdateEditor = async () => {
  const container = stepPreviewRef.value?.codeContainer
  if (!container) return
  if (editorView) { editorView.destroy(); editorView = null }
  const langExt = currentFileName.value ? getLanguageExtension(currentFileName.value) : null
  const ctxMenu = EditorView.domEventHandlers({
    contextmenu: (event, view) => {
      event.preventDefault()
      const sel = view.state.selection.main; const txt = view.state.sliceDoc(sel.from, sel.to)
      const menu = document.createElement('div'); menu.style.cssText = `position:fixed;left:${event.clientX}px;top:${event.clientY}px;background:#282a36;border:1px solid #44475a;border-radius:4px;box-shadow:0 4px 12px rgba(0,0,0,0.4);padding:4px 0;min-width:80px;z-index:1000;`
      const item = document.createElement('div'); item.style.cssText = 'padding:6px 16px;cursor:pointer;font-size:13px;color:#f8f8f2;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;transition:all 0.15s ease;'; item.textContent = '复制'
      item.onmouseover = () => { item.style.background = '#44475a' }; item.onmouseout = () => { item.style.background = 'transparent' }
      item.onclick = () => { navigator.clipboard.writeText(txt || view.state.doc.toString()).then(() => message.success('内容已复制到剪贴板')).catch(() => message.error('复制失败')); document.body.removeChild(menu) }
      menu.appendChild(item); document.body.appendChild(menu)
      const close = (e) => { if (!menu.contains(e.target)) { document.body.removeChild(menu); document.removeEventListener('click', close) } }
      setTimeout(() => document.addEventListener('click', close), 0)
    }
  })
  const exts = [dracula, EditorView.editable.of(false), lineNumbers(), highlightActiveLineGutter(), syntaxHighlighting(defaultHighlightStyle), keymap.of(defaultKeymap), ctxMenu, EditorView.scrollMargins.of(() => ({ top: 10, bottom: 10 })), EditorView.theme({ '&': { height: '100%' }, '.cm-scroller': { overflow: 'auto !important', height: '100% !important' } })]
  if (langExt) exts.push(langExt)
  editorView = new EditorView({ state: EditorState.create({ doc: currentFileContent.value, extensions: exts }), parent: container })
  setTimeout(() => { if (editorView?.scrollDOM) editorView.requestMeasure() }, 100)
}

const onExpand = (keys) => { expandedKeys.value = keys }
const copyFileContent = async () => {
  if (!currentFileContent.value) { message.warning('没有可复制的内容'); return }
  try { await navigator.clipboard.writeText(currentFileContent.value); message.success('内容已复制到剪贴板') } catch { message.error('复制失败，请手动复制') }
}
const checkOutputPathExists = debounce(async () => {
  if (!finalOutputPath.value) { outputPathExists.value = false; return }
  try { outputPathExists.value = await invoke('check_directory_exists', { path: finalOutputPath.value }) } catch { outputPathExists.value = false }
}, 300)

watch(finalOutputPath, () => { checkOutputPathExists() })
onBeforeUnmount(() => { if (editorView) editorView.destroy(); if (jsonEditor) jsonEditor.destroy() })
</script>

<style scoped>
.wizard-header { display: flex; align-items: center; justify-content: space-between; width: 100%; padding: 12px 24px; flex-shrink: 0; min-height: 56px; }
.header-left, .header-right { display: flex; align-items: center; gap: 12px; }
.wz-name { font-size: 16px; font-weight: 600; color: var(--color-text); }
.wizard-header-actions { display: flex; align-items: center; gap: 12px; }
.wizard-header-actions .ant-btn { font-size: 14px; height: 32px; padding: 4px 16px; border-radius: 6px; font-weight: 500; }
.wizard-header-actions .ant-btn-primary { background: var(--color-primary); border-color: var(--color-border-strong); }
.steps-compact { display: flex; align-items: center; gap: 24px; }
.step-item { display: flex; align-items: center; gap: 8px; transition: all 0.3s ease; }
.step-dot { width: 28px; height: 28px; border-radius: 50%; background: var(--color-border); color: var(--color-text-secondary); display: flex; align-items: center; justify-content: center; font-size: 14px; font-weight: 600; transition: all 0.3s ease; flex-shrink: 0; }
.step-item.active .step-dot { background: var(--color-info); color: #fff; transform: scale(1.15); box-shadow: 0 2px 8px rgba(22,163,74,0.4); }
.step-text { font-size: 14px; color: var(--color-text-secondary); font-weight: 500; white-space: nowrap; transition: all 0.3s ease; }
.step-item.active .step-text { color: var(--color-info); font-weight: 600; }
.wizard-content { flex: 1; min-height: 0; overflow-y: auto; overflow-x: hidden; padding: 16px 24px; display: flex; flex-direction: column; max-width: 1600px; margin: 0 auto; }
.wizard-content-variables, .wizard-content-preview { max-width: none; }
.wizard-content-preview { padding: 0 !important; overflow: hidden !important; margin: 0 !important; height: 100% !important; }
.wizard-footer { display: flex; justify-content: flex-end; align-items: center; gap: 8px; }
.wizard-footer .footer-btn { height: 32px !important; font-size: 13px !important; margin: 0 !important; }
:deep(.ant-drawer-content-wrapper) { display: flex !important; flex-direction: column !important; }
:deep(.ant-drawer-body) { padding: 0 !important; display: flex !important; flex-direction: column !important; flex: 1 !important; overflow: hidden !important; }
:deep(.ant-drawer-header) { padding: 12px 24px !important; margin: 0 !important; }
:deep(.ant-drawer-container), :deep(.ant-drawer-content) { padding: 0 !important; }
</style>
