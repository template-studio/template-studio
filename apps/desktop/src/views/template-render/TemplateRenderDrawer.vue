<template>
  <div class="render-wizard">
    <!-- 顶部：步骤指示器 -->
    <div class="wizard-header">
      <div class="header-left">
        <span class="wz-title">模板渲染</span>
        <a-tag v-if="selectedTemplate" color="purple">{{ selectedTemplate.name }}</a-tag>
      </div>
      <div class="header-right">
        <div class="steps-compact">
          <div
            v-for="(step, idx) in steps"
            :key="step.title"
            class="step-item"
            :class="{ active: currentStep === idx + 1, completed: currentStep > idx + 1 }"
          >
            <div class="step-dot">{{ idx + 1 }}</div>
            <div class="step-text">{{ step.title }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 内容区 -->
    <div class="wizard-content" :class="{ 'wizard-content-preview': currentStep === 3 }">
      <!-- Step 1: 选择模板 -->
      <div v-if="currentStep === 1" class="step-panel">
        <div class="template-grid-wrapper">
          <!-- 工具栏 -->
          <div class="template-toolbar">
            <div class="toolbar-left">
              <span class="result-count">共 {{ filteredTemplates.length }} 个模板</span>
            </div>
            <div class="toolbar-right">
              <a-input
                v-model:value="searchText"
                placeholder="搜索模板..."
                style="width: 200px"
                allow-clear
              >
                <template #prefix><SearchOutlined /></template>
              </a-input>
            </div>
          </div>
          <!-- 筛选栏 -->
          <div class="filter-bar">
            <div class="filter-row">
              <span class="filter-label">分类</span>
              <a-radio-group v-model:value="selectedType" button-style="solid" size="small">
                <a-radio-button v-for="cat in [{ id: 'all', name: '全部' }, ...templateTypes]" :key="cat.id" :value="String(cat.id)">{{ cat.name }}</a-radio-button>
              </a-radio-group>
            </div>
            <div class="filter-row">
              <span class="filter-label">语言</span>
              <a-radio-group v-model:value="selectedLang" button-style="solid" size="small">
                <a-radio-button v-for="lang in [{ id: 'all', name: '全部' }, ...templateLangs]" :key="lang.id" :value="String(lang.id)">{{ lang.name }}</a-radio-button>
              </a-radio-group>
            </div>
          </div>
          <!-- 模板卡片网格 -->
          <div class="template-grid">
            <div
              v-for="tpl in filteredTemplates"
              :key="tpl.id"
              class="template-card"
              :class="{ selected: selectedTemplate?.id === tpl.id }"
              @click="selectTemplate(tpl)"
            >
              <div class="card-visual">
                <div class="visual-bg">
                  <div class="code-preview">{{ getCodeSnippet(tpl) }}</div>
                </div>
                <div v-if="tpl.isFeatured === 1" class="template-badge">推荐</div>
                <div v-if="selectedTemplate?.id === tpl.id" class="selected-badge">
                  <CheckCircleFilled />
                </div>
              </div>
              <div class="card-content">
                <h3 class="template-name">{{ tpl.name }}</h3>
                <p class="template-desc">{{ tpl.description || '通用模板' }}</p>
                <div class="template-tags">
                  <span
                    v-for="lang in (tpl.languages || [])"
                    :key="lang.languageId"
                    class="template-tag lang-tag"
                  >{{ getLanguageName(lang.languageId) }}</span>
                  <span v-if="tpl.categoryId" class="template-tag type-tag">{{ getCategoryName(tpl.categoryId) }}</span>
                </div>
                <div class="card-footer">
                  <div class="card-author">
                    <div class="author-avatar">
                      <UserOutlined class="author-avatar-fallback" />
                    </div>
                    <span class="author-name">{{ tpl.ownerName || 'Template Studio' }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <a-empty
            v-if="filteredTemplates.length === 0"
            description="暂无匹配的模板"
            :image-style="{ height: '80px' }"
            style="margin-top: 80px"
          />
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
                <a-checkbox
                  :checked="contextOptions.project"
                  @change="contextOptions.project = $event.target.checked"
                >
                  项目信息
                </a-checkbox>
                <a-checkbox
                  :checked="contextOptions.tables"
                  @change="contextOptions.tables = $event.target.checked"
                >
                  表信息
                </a-checkbox>
              </div>
              <div v-if="contextOptions.project || contextOptions.tables" class="project-selector">
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
              <div v-if="contextOptions.tables && projectTables.length > 0" class="table-selector">
                <a-checkbox-group
                  v-model:value="contextOptions.selectedTables"
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
          <div class="code-preview">
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
        :disabled="currentStep === 1 && !selectedTemplate"
        @click="nextStep"
        class="footer-btn"
      >
        下一步
      </a-button>
    </div>

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
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, nextTick, onBeforeUnmount, h } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { message } from 'ant-design-vue'
import {
  SearchOutlined, CodeOutlined, EditOutlined, CheckCircleFilled,
  FileTextOutlined, CopyOutlined, ExportOutlined,
  FileOutlined, FolderOutlined, UserOutlined
} from '@ant-design/icons-vue'
import { getTemplates, getCategories, getLanguages } from '@/api/templates'
import VariableForm from './VariableForm.vue'

// 步骤
const currentStep = ref(1)
const steps = [{ title: '选择模板' }, { title: '配置变量' }, { title: '预览导出' }]

// 模板
const templates = ref([])
const searchText = ref('')
const selectedType = ref('all')
const selectedLang = ref('all')
const loading = ref(false)
const selectedTemplate = ref(null)

// 分类和语言（从 API 加载）
const categories = ref([{ id: 'all', name: '全部' }])
const languages = ref([{ id: 'all', name: '全部' }])

const templateTypes = computed(() => categories.value.filter(c => c.id !== 'all'))
const templateLangs = computed(() => languages.value.filter(l => l.id !== 'all'))

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
const contextOptions = ref({
  project: true,
  tables: false,
  selectedTables: [],
})

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

// 导出
const exportDialogVisible = ref(false)
const exportDir = ref('')
const exporting = ref(false)

// 编辑器
const jsonEditorEl = ref(null)
const codeContainer = ref(null)
let jsonEditorView = null
let codeEditorView = null

// 搜索+筛选
const filteredTemplates = computed(() => {
  let result = templates.value
  if (searchText.value) {
    const q = searchText.value.toLowerCase()
    result = result.filter(
      (t) =>
        (t.name && t.name.toLowerCase().includes(q)) ||
        (t.description && t.description.toLowerCase().includes(q))
    )
  }
  if (selectedType.value !== 'all') {
    result = result.filter(t => t.categoryId === Number(selectedType.value))
  }
  if (selectedLang.value !== 'all') {
    result = result.filter(t => t.languages?.some(l => l.languageId === Number(selectedLang.value)))
  }
  return result
})

onMounted(async () => {
  await Promise.all([loadCategories(), loadLanguages(), loadTemplates(), loadProjects()])
})

const getCodeSnippet = (tpl) => {
  const primaryLang = tpl.languages?.find(l => l.isPrimary === 1)
  const lang = languages.value.find(l => l.id === primaryLang?.languageId)?.name || ''
  const name = tpl.name || 'Template'
  if (lang.includes('Rust') || lang.includes('rust')) return `fn main() {\n    println!("Hello, ${name}!");\n}`
  if (lang.includes('Go') || lang.includes('go') || lang.includes('Golang')) return `package main\n\nimport "fmt"\n\nfunc main() {\n    fmt.Printf("Hello, ${name}!\\n")\n}`
  if (lang.includes('Python') || lang.includes('python')) return `def main():\n    print(f"Hello, {name}!")\n\nif __name__ == "__main__":\n    main()`
  if (lang.includes('JavaScript') || lang.includes('javascript')) return `function main() {\n  console.log('Hello, ${name}!');\n}\n\nmain();`
  if (lang.includes('TypeScript') || lang.includes('typescript')) return `function main(): void {\n  console.log('Hello, ${name}!');\n}\n\nmain();`
  if (lang.includes('Java') || lang.includes('java')) return `public class App {\n  public static void main(String[] args) {\n    System.out.println("Hello, ${name}!");\n  }\n}`
  return `// ${name}\nclass App {\n  constructor() {\n    this.name = '${name}';\n  }\n\n  run() {\n    console.log('Running', this.name);\n  }\n}`
}

const getLanguageName = (languageId) => languages.value.find(l => l.id === languageId)?.name || languageId
const getCategoryName = (categoryId) => categories.value.find(c => c.id === categoryId)?.name || categoryId

const loadCategories = async () => {
  try {
    const res = await getCategories({ all: 1 })
    categories.value = res?.data?.categoriesList
      ? [{ id: 'all', name: '全部' }, ...res.data.categoriesList]
      : [{ id: 'all', name: '全部' }]
  } catch {
    categories.value = [{ id: 'all', name: '全部' }]
  }
}

const loadLanguages = async () => {
  try {
    const res = await getLanguages({ all: 1 })
    languages.value = res?.data?.languagesList
      ? [{ id: 'all', name: '全部' }, ...res.data.languagesList]
      : [{ id: 'all', name: '全部' }]
  } catch {
    languages.value = [{ id: 'all', name: '全部' }]
  }
}

const loadTemplates = async () => {
  try {
    const res = await getTemplates()
    templates.value = res?.data?.templatesList || []
  } catch (e) {
    console.error('加载模板失败:', e)
    message.error('加载模板列表失败')
  }
}

const loadProjects = async () => {
  try {
    const result = await invoke('db_get_projects')
    const projects = Array.isArray(result) ? result : []
    projectOptions.value = projects.map(p => ({ label: p.name, value: p.id }))
  } catch {}
}

const onProjectChange = async (projectId) => {
  projectTables.value = []
  contextOptions.value.selectedTables = []
  if (!projectId) return
  try {
    const result = await invoke('db_get_project_tables', { projectId: Number(projectId) })
    projectTables.value = Array.isArray(result) ? result : []
  } catch {}
}

const selectTemplate = (tpl) => {
  selectedTemplate.value = tpl
}

// 步骤导航
const nextStep = async () => {
  if (currentStep.value === 1) {
    if (!selectedTemplate.value) {
      message.warning('请先选择一个模板')
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
    const schemaStr = await invoke('get_template_variables', {
      templateId: String(selectedTemplate.value.id),
    })
    if (schemaStr) {
      const parsed = JSON.parse(schemaStr)
      schema.value = parsed
      const defaults = {}
      for (const field of parsed.fields || []) {
        if (field.default !== undefined) {
          defaults[field.name] = field.default
        } else if (field.type === 'boolean') {
          defaults[field.name] = false
        } else if (field.type === 'multi-select') {
          defaults[field.name] = []
        } else if (field.type === 'number') {
          defaults[field.name] = 0
        } else {
          defaults[field.name] = ''
        }
      }
      variables.value = defaults
    }
  } catch (e) {
    console.error('加载变量定义失败:', e)
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
  if (contextOptions.value.project && selectedProjectId.value) {
    const proj = projectOptions.value.find(p => p.value === selectedProjectId.value)
    ctx.project = { id: selectedProjectId.value, name: proj?.label || '' }
  }
  if (contextOptions.value.tables && contextOptions.value.selectedTables.length > 0) {
    ctx.tables = projectTables.value
      .filter(t => contextOptions.value.selectedTables.includes(t.id))
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

watch(contextOptions, () => {
  if (mode.value === 'advanced') syncToJson()
}, { deep: true })

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
      templateId: String(selectedTemplate.value.id),
      variables: vars,
      version: null,
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

  const map = new Map()
  flat.forEach(n => map.set(n.id, { ...n, children: [] }))
  const roots = []
  flat.forEach(n => {
    const cur = map.get(n.id)
    if (n.parent_id === 0) roots.push(cur)
    else {
      const p = map.get(n.parent_id)
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
        key: n.key || n.id,
        title: n.file_name || n.key,
        fileName: n.file_name,
        isDirectory: n.is_directory === 1,
        filePath: n.file_path,
        fileContent: n.file_content,
        icon: n.is_directory === 1 ? h(FolderOutlined) : h(FileOutlined),
        children: n.children?.length ? toAntTree(n.children) : [],
      }))
  }

  fileTreeData.value = toAntTree(roots)
  expandedKeys.value = flat.filter(n => n.is_directory === 1).map(n => n.key || n.id).filter(Boolean)
}

const findFirstFile = (nodes) => {
  for (const n of nodes) {
    if (!n.isDirectory) return n
    if (n.children?.length) { const f = findFirstFile(n.children); if (f) return f }
  }
  return null
}

const selectFirstFile = () => {
  const ff = findFirstFile(fileTreeData.value)
  if (ff) {
    selectedFileKey.value = ff.key
    currentFileName.value = ff.fileName || ''
    currentFileContent.value = ff.fileContent || ''
    nextTick(() => createCodeEditor())
  }
}

const onFileSelect = (keys) => {
  if (!keys?.length) return
  selectedFileKey.value = keys[0]

  const find = (nodes, key) => {
    for (const n of nodes) {
      if (n.key === key) return n
      if (n.children?.length) { const f = find(n.children, key); if (f) return f }
    }
    return null
  }
  const node = find(fileTreeData.value, keys[0])
  if (!node) return
  if (node.isDirectory) { currentFileName.value = ''; currentFileContent.value = ''; return }
  currentFileName.value = node.fileName || ''
  currentFileContent.value = node.fileContent || ''
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
      templateId: String(selectedTemplate.value.id),
      version: null,
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
.render-wizard {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-container);
}

/* Header */
.wizard-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
  min-height: 56px;
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

/* Step 1: Template Grid */
.template-grid-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 16px 24px;
}

.template-toolbar {
  flex-shrink: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.toolbar-left {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.result-count {
  color: var(--color-text-secondary);
  font-size: 13px;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.filter-bar {
  flex-shrink: 0;
  margin-bottom: 16px;
  padding: 12px 16px;
  background: var(--color-surface);
  border-radius: 8px;
  border: 1px solid var(--color-border);
}

.filter-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 0;
}

.filter-row:not(:last-child) {
  margin-bottom: 8px;
  border-bottom: 1px solid var(--color-border);
  padding-bottom: 8px;
}

.filter-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
  white-space: nowrap;
  min-width: 40px;
}

.template-grid {
  flex: 1;
  overflow: auto;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 16px;
  align-content: start;
}

.template-card {
  background: var(--color-bg-container);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.template-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(15, 23, 42, 0.12);
  border-color: var(--color-primary);
}

.template-card.selected {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.15);
}

.card-visual {
  height: 120px;
  position: relative;
  overflow: hidden;
}

.visual-bg {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.visual-bg::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 200%;
  height: 100%;
  background: linear-gradient(90deg, transparent 0%, rgba(24,144,255,0.03) 45%, rgba(24,144,255,0.08) 50%, rgba(24,144,255,0.03) 55%, transparent 100%);
  animation: shimmer 4s ease-in-out infinite;
}

@keyframes shimmer {
  0% { transform: translateX(0); }
  100% { transform: translateX(50%); }
}

.code-preview {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 10px;
  line-height: 1.5;
  color: rgba(148, 163, 184, 0.4);
  white-space: pre;
  padding: 16px 20px;
  text-align: left;
  position: relative;
  z-index: 1;
  overflow: hidden;
}

.template-badge {
  position: absolute;
  top: 10px;
  right: 10px;
  background: rgba(24, 144, 255, 0.9);
  backdrop-filter: blur(8px);
  padding: 3px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  color: #fff;
  z-index: 2;
}

.selected-badge {
  position: absolute;
  top: 10px;
  left: 10px;
  color: var(--color-primary);
  font-size: 22px;
  z-index: 2;
  background: rgba(255, 255, 255, 0.9);
  border-radius: 50%;
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.card-content {
  padding: 14px 16px 16px;
}

.template-name {
  margin: 0 0 6px 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.template-card:hover .template-name {
  color: var(--color-primary);
}

.template-desc {
  margin: 0 0 10px 0;
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.6;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.template-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.template-tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.lang-tag {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  color: var(--color-text-secondary);
}

.type-tag {
  background: rgba(114, 46, 209, 0.08);
  border: 1px solid rgba(114, 46, 209, 0.2);
  color: #722ed1;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 10px;
  border-top: 1px solid var(--color-border);
  margin-top: 10px;
}

.card-author {
  display: flex;
  align-items: center;
  gap: 8px;
}

.author-avatar {
  width: 22px;
  height: 22px;
  border-radius: 4px;
  background: linear-gradient(135deg, #0f172a 0%, #334155 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: #fff;
  font-size: 11px;
}

.author-avatar-fallback {
  font-size: 11px;
  color: #fff;
}

.author-name {
  font-size: 12px;
  color: var(--color-text-secondary);
  font-weight: 500;
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

.code-preview {
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
  padding: 12px 24px;
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
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
