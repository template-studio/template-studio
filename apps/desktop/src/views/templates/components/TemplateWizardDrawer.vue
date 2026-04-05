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
    <div class="wizard-content" :class="{ 'wizard-content-preview': currentStep === 4 }">
      <!-- 步骤1: 模板介绍 -->
      <div v-if="currentStep === 1" class="step-panel">
        <div class="template-intro">
          <div class="intro-header">
            <h2>{{ wizardData.template?.name }}</h2>
            <a-tag color="purple" size="large">{{ getCategoryName(wizardData.template?.categoryId) }}</a-tag>
            <a-tag v-if="wizardData.template?.isFeatured === 1" color="gold" size="large"><template #icon><StarOutlined /></template>推荐</a-tag>
          </div>
          <a-divider />
          <p class="intro-desc">{{ wizardData.template?.description }}</p>
          <div v-if="versionList.length > 0" class="intro-section">
            <h3><TagsOutlined /> 选择版本</h3>
            <a-select v-model:value="wizardData.version" :options="versionOptions" size="large" style="width:100%;" @change="handleVersionChange" placeholder="选择版本"><template #suffixIcon><DownloadOutlined /></template></a-select>
            <div class="form-hint">当前选择：{{ wizardData.version || '未选择' }}</div>
          </div>
          <div class="intro-section">
            <h3><CodeOutlined /> 支持的语言</h3>
            <div class="languages-list">
              <a-tag v-for="lang in wizardData.template?.languages" :key="lang.id" :color="lang.isPrimary === 1 ? 'blue' : 'default'" size="large">{{ getLanguageName(lang.languageId) }}<span v-if="lang.isPrimary === 1">(主语言)</span></a-tag>
            </div>
          </div>
          <div v-if="wizardData.template?.introduction" class="intro-section">
            <h3><FileTextOutlined /> 详细介绍</h3>
            <div class="intro-markdown">{{ wizardData.template.introduction }}</div>
          </div>
          <a-alert v-if="versionList.length === 0 && !isDownloading" message="该模板暂未开放使用" description="此模板正在准备中，请稍后再来" type="info" show-icon style="margin-bottom:24px;"><template #icon><InfoCircleOutlined /></template></a-alert>
          <div v-if="isDownloading" class="intro-section">
            <a-alert message="请等待模板下载完成" type="info" show-icon>
              <template #icon><LoadingOutlined style="animation:spin 1s linear infinite;" /></template>
              <template #description><a-progress :percent="downloadProgress" :show-info="false" /></template>
            </a-alert>
          </div>
        </div>
      </div>
      <!-- 步骤2: 路径配置 -->
      <div v-if="currentStep === 2" class="step-panel">
        <a-form ref="configFormRef" :model="wizardData" layout="vertical">
          <a-alert message="配置项目路径" description="请设置项目名称和输出目录，项目将在指定目录下创建" type="info" show-icon style="margin-bottom:24px;" />
          <a-form-item label="项目名称" name="projectName" :rules="projectNameRules">
            <a-input v-model:value="wizardData.projectName" placeholder="请输入项目名称（如：my-awesome-project）" size="large" allow-clear />
            <div class="form-hint">只能包含字母、数字、下划线和连字符，长度 2-50 个字符</div>
          </a-form-item>
          <a-form-item label="输出目录" name="outputDir" :rules="[{ required: true, message: '请选择输出目录' }]">
            <div style="display:flex;gap:12px;align-items:center;">
              <a-input v-model:value="wizardData.outputDir" placeholder="请选择项目输出目录" disabled size="large" style="flex:1" />
              <a-button type="primary" size="large" @click="selectOutputDir" style="width:120px;flex-shrink:0;">浏览...</a-button>
            </div>
          </a-form-item>
          <div class="section-divider"><FolderOpenOutlined /><span>路径预览</span></div>
          <a-form-item>
            <a-alert v-if="finalOutputPath && !outputPathExists" :message="finalOutputPath" type="success" show-icon><template #icon><CheckCircleOutlined /></template><template #description>项目将在以上路径创建</template></a-alert>
            <a-alert v-else-if="finalOutputPath && outputPathExists" :message="finalOutputPath" type="error" show-icon><template #icon><WarningOutlined /></template><template #description>警告：该路径已存在，创建项目将覆盖原有内容</template></a-alert>
            <a-alert v-else message="路径未配置" description="请输入项目名称并选择输出目录" type="warning" show-icon><template #icon><WarningOutlined /></template></a-alert>
          </a-form-item>
        </a-form>
      </div>
      <!-- 步骤3: 配置变量 -->
      <div v-if="currentStep === 3" class="step-panel">
        <div v-if="loadingVariables" class="loading-ct"><a-spin size="large"><template #description>正在加载模板变量...</template></a-spin></div>
        <div v-else-if="!variableDefinitions?.length" class="loading-ct"><a-empty description="该模板没有配置变量"><template #image><SettingOutlined style="font-size:64px;color:var(--color-text-muted);" /></template></a-empty></div>
        <div v-else class="variables-form">
          <div class="variables-layout">
            <div class="variables-sidebar">
              <div class="sidebar-header"><h3 class="sidebar-title">编辑模式</h3></div>
              <div class="mode-tabs-vertical">
                <div class="mode-tab" :class="{ active: variableMode === 'normal' }" @click="variableMode = 'normal'"><EditOutlined class="mode-icon" /><span class="mode-label">普通模式</span></div>
                <div class="mode-tab" :class="{ active: variableMode === 'advanced' }" @click="variableMode = 'advanced'"><CodeOutlined class="mode-icon" /><span class="mode-label">高级模式</span></div>
              </div>
            </div>
            <div class="variables-content">
              <div v-show="variableMode === 'normal'" class="normal-mode">
                <a-form :model="wizardData.variables" layout="horizontal" :label-col="{ style: 'width:120px' }">
                  <div v-for="v in variableDefinitions" :key="v.name" class="form-field-item">
                    <a-form-item :label="v.title || v.name" :required="v.required" :class="{ 'boolean-form-item': v.type === 'boolean' }">
                      <a-switch v-if="v.type === 'boolean'" v-model:checked="wizardData.variables[v.name]" :checked-children="v.trueText || '是'" :un-checked-children="v.falseText || '否'" />
                      <a-input-number v-else-if="v.type === 'number'" v-model:value="wizardData.variables[v.name]" :min="v.min" :max="v.max" style="width:100%;" />
                      <a-select v-else-if="v.type === 'select' || v.type === 'enum'" v-model:value="wizardData.variables[v.name]" :placeholder="`请选择${v.title||v.name}`" :options="v.options" allow-clear />
                      <a-select v-else-if="v.type === 'multi-select'" v-model:value="wizardData.variables[v.name]" :placeholder="`请选择${v.title||v.name}`" :options="v.options" mode="multiple" allow-clear />
                      <a-textarea v-else-if="v.type === 'object' || v.type === 'array'" v-model:value="wizardData.variables[v.name]" :placeholder="v.description || '请输入 JSON'" :rows="6" />
                      <a-textarea v-else-if="v.type === 'text'" v-model:value="wizardData.variables[v.name]" :placeholder="v.description || `请输入${v.title||v.name}`" :rows="4" :maxlength="v.maxLength" show-count />
                      <a-input v-else v-model:value="wizardData.variables[v.name]" :placeholder="v.description || `请输入${v.title||v.name}`" allow-clear><template v-if="v.name==='author'" #prefix><UserOutlined style="color:rgba(0,0,0,0.25);" /></template></a-input>
                    </a-form-item>
                  </div>
                </a-form>
              </div>
              <div v-show="variableMode === 'advanced'" class="advanced-mode">
                <div class="editor-wrap">
                  <div class="editor-header"><span>JSON 编辑器</span><div class="actions"><a-button size="small" @click="formatVariablesJson">格式化</a-button><a-button size="small" type="primary" @click="syncFromNormalMode">同步普通模式</a-button></div></div>
                  <div ref="jsonEditorContainer" class="json-editor"></div>
                  <div class="editor-footer"><span v-if="jsonValid" class="ok">✅ JSON 格式正确</span><span v-else class="err">❌ {{ jsonError }}</span></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <!-- 步骤4: 预览确认 -->
      <div v-if="currentStep === 4" class="step-panel step-panel-preview">
        <div v-if="loadingPreview" class="loading-ct"><a-spin size="large"><template #description>正在渲染文件预览...</template></a-spin></div>
        <div v-else-if="showPreview" class="preview-fullscreen">
          <div class="preview-main">
            <div class="file-explorer">
              <div class="explorer-header"><span class="explorer-title">模板文件</span></div>
              <div class="explorer-content">
                <a-tree v-if="fileTreeData.length > 0" :tree-data="fileTreeData" :selected-keys="[selectedFileKey]" :expanded-keys="expandedKeys" show-icon @select="onFileSelect" @expand="onExpand" />
                <a-empty v-else description="暂无文件" :image="null"><template #description><span style="color:var(--color-text-muted);">暂无文件</span></template></a-empty>
              </div>
            </div>
            <div class="code-preview">
              <div class="file-header">
                <div class="file-info"><FileTextOutlined /><span class="file-name">{{ currentFileName || '未选择文件' }}</span></div>
                <a-button size="small" @click="copyFileContent" :disabled="!currentFileContent" type="text"><template #icon><CopyOutlined /></template>复制</a-button>
              </div>
              <div v-if="!currentFileContent" class="no-file-selected">
                <div class="no-file-icon"><FileTextOutlined style="font-size:48px;color:var(--color-text-muted);" /></div>
                <div class="no-file-text">请选择左侧文件进行预览</div>
              </div>
              <div v-else class="code-content"><div class="codemirror-container" ref="codeContainer"></div></div>
            </div>
          </div>
        </div>
      </div>
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
import { debounce } from 'lodash-es'
import { UserOutlined, FolderOpenOutlined, WarningOutlined, SettingOutlined, StarOutlined, EditOutlined, CodeOutlined, FileTextOutlined, CheckCircleOutlined, TagsOutlined, DownloadOutlined, LoadingOutlined, InfoCircleOutlined, FileOutlined, FolderOutlined, CopyOutlined } from '@ant-design/icons-vue'
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

const props = defineProps({ open: Boolean, template: Object })
const emit = defineEmits(['update:open', 'created'])

const currentStep = ref(1); const wizardData = ref({ template: null, version: '', projectName: '', outputDir: '', variables: {} })
const configFormRef = ref(null); const versionList = ref([]); const isDownloading = ref(false); const downloadProgress = ref(0)
const outputPathExists = ref(false); const loadingVariables = ref(false); const variableDefinitions = ref([]); const variableMode = ref('normal')
const jsonEditorContainer = ref(null); let jsonEditor = null; const jsonValid = ref(true); const jsonError = ref('')
const loadingPreview = ref(false); const fileTreeData = ref([]); const selectedFileKey = ref(''); const expandedKeys = ref([])
const currentFileName = ref(''); const currentFileContent = ref(''); const codeContainer = ref(null); let editorView = null

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
    try { await configFormRef.value.validate(); if (currentStep.value < 4) currentStep.value++ } catch {}
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
  if (!jsonEditorContainer.value) return
  if (jsonEditor) { jsonEditor.destroy(); jsonEditor = null }
  try {
    jsonEditor = new EditorView({
      state: EditorState.create({ doc: JSON.stringify(wizardData.value.variables, null, 2), extensions: [dracula, javascript(), keymap.of(defaultKeymap), lineNumbers(), EditorView.updateListener.of(() => validateJson())] }),
      parent: jsonEditorContainer.value
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
  if (!codeContainer.value) return
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
  editorView = new EditorView({ state: EditorState.create({ doc: currentFileContent.value, extensions: exts }), parent: codeContainer.value })
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
.wizard-header-actions .ant-btn-primary { background: var(--color-primary); border-color: var(--color-primary); }
.steps-compact { display: flex; align-items: center; gap: 24px; }
.step-item { display: flex; align-items: center; gap: 8px; transition: all 0.3s ease; }
.step-dot { width: 28px; height: 28px; border-radius: 50%; background: var(--color-border); color: var(--color-text-secondary); display: flex; align-items: center; justify-content: center; font-size: 14px; font-weight: 600; transition: all 0.3s ease; flex-shrink: 0; }
.step-item.active .step-dot { background: var(--color-info); color: #fff; transform: scale(1.15); box-shadow: 0 2px 8px rgba(24,144,255,0.4); }
.step-text { font-size: 14px; color: var(--color-text-secondary); font-weight: 500; white-space: nowrap; transition: all 0.3s ease; }
.step-item.active .step-text { color: var(--color-info); font-weight: 600; }
.wizard-content { flex: 1; min-height: 0; overflow-y: auto; overflow-x: hidden; padding: 16px 24px; display: flex; flex-direction: column; max-width: 1600px; margin: 0 auto; }
.wizard-content:has(.normal-mode), .wizard-content-preview { max-width: none; }
.wizard-content-preview { padding: 0 !important; overflow: hidden !important; margin: 0 !important; height: 100% !important; }
.step-panel { padding: 8px; display: flex; flex-direction: column; min-height: 0; }
.step-panel-preview { padding: 0; overflow: hidden; flex: 1; min-height: 0; height: 100%; display: flex; flex-direction: column; }
.template-intro { padding: 8px; }
.intro-header { display: flex; align-items: center; gap: 16px; margin-bottom: 24px; flex-wrap: wrap; }
.intro-header h2 { margin: 0; font-size: 28px; font-weight: 600; color: var(--color-text); }
.intro-desc { font-size: 16px; color: var(--color-text-secondary); line-height: 1.8; margin-bottom: 32px; }
.intro-section { margin-bottom: 32px; }
.intro-section h3 { font-size: 18px; font-weight: 600; color: var(--color-text); margin-bottom: 16px; display: flex; align-items: center; gap: 8px; }
.languages-list { display: flex; flex-wrap: wrap; gap: 12px; }
.intro-markdown { background: var(--color-surface); padding: 20px; border-radius: var(--border-radius-md); border: 1px solid var(--color-border); line-height: 1.8; color: var(--color-text-secondary); font-size: 14px; }
.wizard-footer { display: flex; justify-content: flex-end; align-items: center; gap: 8px; }
.wizard-footer .footer-btn { height: 32px !important; font-size: 13px !important; margin: 0 !important; }
.wizard-content .ant-form-item { margin-bottom: 24px; }
.form-hint { font-size: 12px; color: var(--color-text-secondary); margin-top: 4px; }
.section-divider { display: flex; align-items: center; gap: 8px; margin: 24px 0; padding-bottom: 12px; border-bottom: 1px solid var(--color-border); font-size: 16px; font-weight: 600; color: var(--color-text); }
.loading-ct { display: flex; justify-content: center; align-items: center; min-height: 300px; }
.variables-form { padding: 0; height: 100%; display: flex; flex-direction: column; }
.variables-layout { display: flex; height: 100%; min-height: 500px; }
.variables-sidebar { width: 200px; flex-shrink: 0; background: var(--color-bg-elevated); border-right: 1px solid var(--color-border); display: flex; flex-direction: column; }
.sidebar-header { padding: 20px 16px 12px; border-bottom: 1px solid var(--color-border); font-size: 14px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: 0.5px; }
.mode-tabs-vertical { padding: 12px 8px; display: flex; flex-direction: column; gap: 4px; }
.mode-tab { display: flex; align-items: center; gap: 12px; padding: 12px 16px; border-radius: 6px; cursor: pointer; transition: all 0.2s ease; color: var(--color-text-secondary); font-size: 14px; font-weight: 500; }
.mode-tab:hover { background: var(--color-bg-spotlight); color: var(--color-text); }
.mode-tab.active { background: var(--color-primary-bg); color: var(--color-primary); }
.mode-icon { font-size: 18px; color: var(--color-text-secondary); }
.variables-content { flex: 1; overflow-y: auto; padding: 24px 32px; background: var(--color-bg-container); }
.normal-mode { width: 100%; }
.normal-mode :deep(.ant-form-item) { margin-bottom: 24px; }
.normal-mode :deep(.ant-form-item-label) { width: 120px; padding-right: 12px; }
.normal-mode :deep(.ant-form-item-control) { flex: 1; max-width: none; }
.normal-mode :deep(.boolean-form-item .ant-form-item-control) { flex: 0 0 auto; }
.normal-mode :deep(.ant-form-item-label > label) { font-weight: 600; font-size: 14px; }
.form-field-item { width: 100%; }
.normal-mode :deep(.ant-input), .normal-mode :deep(.ant-input-number), .normal-mode :deep(.ant-select), .normal-mode :deep(.ant-switch), .normal-mode :deep(.ant-select-selector), .normal-mode :deep(.ant-input-number-input) { width: 100%; }
.advanced-mode { width: 100%; height: 100%; display: flex; flex-direction: column; }
.editor-wrap { border: 1px solid var(--color-border); border-radius: 6px; overflow: hidden; background: var(--color-bg-container); flex: 1; display: flex; flex-direction: column; min-height: 0; }
.editor-header { display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; background: var(--color-bg-elevated); border-bottom: 1px solid var(--color-border); font-size: 14px; font-weight: 500; flex-shrink: 0; }
.json-editor { flex: 1; min-height: 0; overflow: auto; }
.editor-footer { padding: 8px 16px; background: var(--color-bg-elevated); font-size: 12px; display: flex; align-items: center; gap: 6px; border-top: 1px solid var(--color-border); }
.editor-footer .ok { color: var(--color-success); } .editor-footer .err { color: var(--color-error); }
.preview-fullscreen { flex: 1; display: flex; flex-direction: column; background: var(--color-surface); min-height: 0; height: 100%; overflow: hidden; }
.preview-main { flex: 1; display: flex; overflow: hidden; }
.file-explorer { width: 280px; background: var(--color-background); border-right: 1px solid var(--color-border); display: flex; flex-direction: column; flex-shrink: 0; height: 100%; }
.explorer-header { height: 48px; background: var(--color-surface-3); border-bottom: 1px solid var(--color-border); display: flex; align-items: center; padding: 0 16px; flex-shrink: 0; }
.explorer-title { font-size: 14px; font-weight: 600; color: var(--color-text); }
.explorer-content { flex: 1; overflow-y: auto; padding: 8px; }
.explorer-content .ant-tree-node-content-wrapper { padding: 4px 8px; border-radius: 4px; }
.preview-main .code-preview { flex: 1; display: flex; flex-direction: column; background: var(--color-background); overflow: hidden; height: 100%; }
.preview-main .file-header { height: 48px; background: var(--color-surface-3); border-bottom: 1px solid var(--color-border); display: flex; align-items: center; justify-content: space-between; padding: 0 16px; flex-shrink: 0; }
.preview-main .file-info { display: flex; align-items: center; gap: 8px; }
.preview-main .file-name { font-size: 14px; font-weight: 600; color: var(--color-text); font-family: 'Consolas','Monaco',monospace; }
.preview-main .file-header .ant-btn { height: 28px; padding: 0 12px; font-size: 13px; line-height: 1; border: none; background: transparent; color: var(--color-text-secondary); }
.preview-main .no-file-selected { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; color: var(--color-text-muted); font-size: 16px; }
.preview-main .code-content { flex: 1; overflow: hidden; background: #1e1e1e; }
.preview-main .codemirror-container { height: 100%; min-height: 400px; }
:deep(.cm-editor) { height: 100% !important; font-size: 14px; outline: none !important; }
:deep(.cm-editor .cm-scroller) { font-family: 'Fira Code','Consolas','Monaco',monospace; overflow: auto !important; height: 100% !important; }
:deep(.cm-editor .cm-line) { padding: 0; }
:deep(.cm-editor .cm-cursor), :deep(.cm-editor .cm-cursor-primary) { display: none !important; }
:deep(.ant-drawer-content-wrapper) { display: flex !important; flex-direction: column !important; }
:deep(.ant-drawer-body) { padding: 0 !important; display: flex !important; flex-direction: column !important; flex: 1 !important; overflow: hidden !important; }
:deep(.ant-drawer-header) { padding: 12px 24px !important; margin: 0 !important; }
:deep(.ant-drawer-container), :deep(.ant-drawer-content) { padding: 0 !important; }
</style>
