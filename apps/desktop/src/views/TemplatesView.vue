<template>
  <div class="templates-view">
    <!-- 顶部工具栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <h2 class="page-title">模板库</h2>
        <span class="result-count">共 {{ filteredTemplates.length }} 个模板</span>
      </div>
      <div class="toolbar-right">
        <a-input
          v-model:value="searchKeyword"
          placeholder="搜索模板..."
          style="width: 240px;"
          allow-clear
        >
          <template #prefix>
            <SearchOutlined />
          </template>
        </a-input>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="filter-bar">
      <!-- 分类筛选 -->
      <div class="filter-row">
        <span class="filter-label">分类</span>
        <a-radio-group v-model:value="selectedCategory" button-style="solid" size="small">
          <a-radio-button
            v-for="cat in categories"
            :key="cat.id"
            :value="cat.id"
          >
            {{ cat.name }}
          </a-radio-button>
        </a-radio-group>
      </div>

      <!-- 语言筛选 -->
      <div class="filter-row">
        <span class="filter-label">语言</span>
        <a-radio-group v-model:value="selectedLanguage" button-style="solid" size="small">
          <a-radio-button
            v-for="lang in languages"
            :key="lang.id"
            :value="lang.id"
          >
            {{ lang.name }}
          </a-radio-button>
        </a-radio-group>
      </div>
    </div>

    <!-- 模板列表 -->
    <div class="templates-content">
      <a-spin :spinning="loading">
        <div class="templates-grid">
          <div
            v-for="template in filteredTemplates"
            :key="template.id"
            class="template-card"
            :class="{ selected: selectedTemplate?.id === template.id }"
            @click="selectTemplate(template)"
          >
            <div class="card-visual">
              <div class="visual-bg">
                <div class="code-preview">{{ getCodeSnippet(template) }}</div>
              </div>
              <div v-if="template.isFeatured === 1" class="template-badge">
                <span>推荐</span>
              </div>
            </div>

            <div class="card-content">
              <h3 class="template-name">{{ template.name }}</h3>
              <p class="template-desc">{{ template.description }}</p>

              <div class="template-languages">
                <span
                  v-for="lang in template.languages"
                  :key="lang.languageId"
                  class="template-tag"
                >
                  {{ getLanguageName(lang.languageId) }}
                </span>
              </div>

              <div class="card-footer">
                <div class="card-author">
                  <div class="author-avatar">
                    <UserOutlined />
                  </div>
                  <span class="author-name">Template Studio</span>
                </div>
                <a-button type="primary" size="small" @click.stop="useTemplate(template)">
                  使用
                </a-button>
              </div>
            </div>
          </div>
        </div>
      </a-spin>
    </div>

    <!-- 模板配置向导抽屉 -->
    <a-drawer
      v-model:open="showWizardModal"
      placement="right"
      :width="'100vw'"
      :closable="true"
      @close="cancelWizard"
    >
      <!-- 自定义头部 -->
      <template #title>
        <div class="wizard-header">
          <div class="header-left">
            <span v-if="currentStep < 4" class="template-name">{{ wizardData.template?.name || '' }}</span>
            <span v-else class="template-name">{{ wizardData.template?.name || '' }}</span>
            <a-tag v-if="currentStep < 4" color="purple">模板配置</a-tag>
            <a-tag v-else color="blue">预览确认</a-tag>
          </div>
          <div class="header-right">
            <!-- 步骤1-3: 显示步骤指示器 -->
            <div v-if="currentStep < 4" class="steps-compact">
              <div
                v-for="(step, index) in steps"
                :key="step.title"
                class="step-item"
                :class="{
                  active: currentStep === index + 1,
                  completed: currentStep > index + 1,
                }"
              >
                <div class="step-dot">{{ index + 1 }}</div>
                <div class="step-text">{{ step.title }}</div>
              </div>
            </div>
            <!-- 步骤4: 显示操作按钮 -->
            <div v-else class="wizard-header-actions">
              <a-button @click="prevStep" size="small">
                上一步
              </a-button>
              <a-button
                type="primary"
                @click="confirmCreateProject"
                size="small"
              >
                创建项目
              </a-button>
            </div>
          </div>
        </div>
      </template>

      <!-- 步骤内容 -->
      <div class="wizard-content" :class="{ 'wizard-content-preview': currentStep === 4 }">
        <!-- 步骤1: 模板介绍 -->
        <div v-if="currentStep === 1" class="step-panel">
          <div class="template-intro">
            <div class="intro-header">
              <h2>{{ wizardData.template?.name }}</h2>
              <a-tag color="purple" size="large">{{ getCategoryName(wizardData.template?.categoryId) }}</a-tag>
              <a-tag v-if="wizardData.template?.isFeatured === 1" color="gold" size="large">
                <template #icon>
                  <StarOutlined />
                </template>
                推荐
              </a-tag>
            </div>

            <a-divider />

            <p class="intro-description">{{ wizardData.template?.description }}</p>

            <div v-if="versionList.length > 0" class="intro-section">
              <h3>
                <TagsOutlined />
                选择版本
              </h3>
              <a-select
                v-model:value="wizardData.version"
                :options="versionOptions"
                size="large"
                style="width: 100%;"
                @change="handleVersionChange"
                placeholder="选择版本"
              >
                <template #suffixIcon>
                  <DownloadOutlined />
                </template>
              </a-select>
              <div class="form-item-hint">
                当前选择：{{ wizardData.version || '未选择' }}
              </div>
            </div>

            <div class="intro-section">
              <h3>
                <CodeOutlined />
                支持的语言
              </h3>
              <div class="languages-list">
                <a-tag
                  v-for="lang in wizardData.template?.languages"
                  :key="lang.id"
                  :color="lang.isPrimary === 1 ? 'blue' : 'default'"
                  size="large"
                >
                  {{ getLanguageName(lang.languageId) }}
                  <span v-if="lang.isPrimary === 1">(主语言)</span>
                </a-tag>
              </div>
            </div>

            <div v-if="wizardData.template?.introduction" class="intro-section">
              <h3>
                <FileTextOutlined />
                详细介绍
              </h3>
              <div class="intro-markdown">{{ wizardData.template.introduction }}</div>
            </div>

            <!-- 无版本提示 -->
            <a-alert
              v-if="versionList.length === 0 && !isDownloading"
              message="该模板暂未开放使用"
              description="此模板正在准备中，请稍后再来"
              type="info"
              show-icon
              style="margin-bottom: 24px;"
            >
              <template #icon>
                <InfoCircleOutlined />
              </template>
            </a-alert>

            <!-- 下载进度提示 -->
            <div v-if="isDownloading" class="intro-section">
              <a-alert
                message="请等待模板下载完成"
                type="info"
                show-icon
              >
                <template #icon>
                  <LoadingOutlined style="animation: spin 1s linear infinite;" />
                </template>
                <template #description>
                  <a-progress :percent="downloadProgress" :show-info="false" />
                </template>
              </a-alert>
            </div>
          </div>
        </div>

        <!-- 步骤2: 路径配置 -->
        <div v-if="currentStep === 2" class="step-panel">
          <a-form
            ref="configFormRef"
            :model="wizardData"
            layout="vertical"
          >
            <a-alert
              message="配置项目路径"
              description="请设置项目名称和输出目录，项目将在指定目录下创建"
              type="info"
              show-icon
              style="margin-bottom: 24px;"
            />

            <a-form-item
              label="项目名称"
              name="projectName"
              :rules="projectNameRules"
            >
              <a-input
                v-model:value="wizardData.projectName"
                placeholder="请输入项目名称（如：my-awesome-project）"
                size="large"
                allow-clear
              />
              <div class="form-item-hint">
                只能包含字母、数字、下划线和连字符，长度 2-50 个字符
              </div>
            </a-form-item>

            <a-form-item
              label="输出目录"
              name="outputDir"
              :rules="[{ required: true, message: '请选择输出目录' }]"
            >
              <div style="display: flex; gap: 12px; align-items: center;">
                <a-input
                  v-model:value="wizardData.outputDir"
                  placeholder="请选择项目输出目录"
                  disabled
                  size="large"
                  style="flex: 1"
                />
                <a-button type="primary" size="large" @click="selectOutputDir" style="width: 120px; flex-shrink: 0;">
                  浏览...
                </a-button>
              </div>
            </a-form-item>

            <div class="section-divider">
              <FolderOpenOutlined />
              <span>路径预览</span>
            </div>

            <a-form-item>
              <a-alert
                v-if="finalOutputPath && !outputPathExists"
                :message="finalOutputPath"
                type="success"
                show-icon
              >
                <template #icon>
                  <CheckCircleOutlined />
                </template>
                <template #description>
                  项目将在以上路径创建
                </template>
              </a-alert>
              <a-alert
                v-else-if="finalOutputPath && outputPathExists"
                :message="finalOutputPath"
                type="error"
                show-icon
              >
                <template #icon>
                  <WarningOutlined />
                </template>
                <template #description>
                  警告：该路径已存在，创建项目将覆盖原有内容
                </template>
              </a-alert>
              <a-alert
                v-else
                message="路径未配置"
                description="请输入项目名称并选择输出目录"
                type="warning"
                show-icon
              >
                <template #icon>
                  <WarningOutlined />
                </template>
              </a-alert>
            </a-form-item>
          </a-form>
        </div>

        <!-- 步骤3: 配置变量 -->
        <div v-if="currentStep === 3" class="step-panel">
          <!-- 加载中 -->
          <div v-if="loadingVariables" class="loading-container">
            <a-spin size="large">
              <template #description>正在加载模板变量...</template>
            </a-spin>
          </div>

          <!-- 无变量 -->
          <div v-else-if="!variableDefinitions || variableDefinitions.length === 0" class="no-variables">
            <a-empty description="该模板没有配置变量">
              <template #image>
                <SettingOutlined style="font-size: 64px; color: #d9d9d9;" />
              </template>
            </a-empty>
          </div>

          <!-- 变量表单 -->
          <div v-else class="variables-form">
            <!-- 左右布局容器 -->
            <div class="variables-layout">
              <!-- 左侧模式切换 -->
              <div class="variables-sidebar">
                <div class="sidebar-header">
                  <h3 class="sidebar-title">编辑模式</h3>
                </div>
                <div class="mode-tabs-vertical">
                  <div
                    class="mode-tab"
                    :class="{ active: variableMode === 'normal' }"
                    @click="variableMode = 'normal'"
                  >
                    <EditOutlined class="mode-icon" />
                    <span class="mode-label">普通模式</span>
                  </div>
                  <div
                    class="mode-tab"
                    :class="{ active: variableMode === 'advanced' }"
                    @click="variableMode = 'advanced'"
                  >
                    <CodeOutlined class="mode-icon" />
                    <span class="mode-label">高级模式</span>
                  </div>
                </div>
              </div>

              <!-- 右侧内容区域 -->
              <div class="variables-content">
                <!-- 普通模式 -->
                <div v-show="variableMode === 'normal'" class="normal-mode">
              <a-form
                :model="wizardData.variables"
                layout="horizontal"
                :label-col="{ style: 'width: 120px' }"
              >
                <div
                  v-for="variable in variableDefinitions"
                  :key="variable.name"
                  class="form-field-item"
                >
                  <!-- 字符串类型 -->
                  <a-form-item
                    v-if="variable.type === 'string'"
                    :label="variable.title || variable.name"
                    :required="variable.required"
                  >
                    <a-tooltip v-if="variable.description" placement="top">
                      <template #title>{{ variable.description }}</template>
                      <a-input
                        v-model:value="wizardData.variables[variable.name]"
                        :placeholder="variable.description || `请输入${variable.title || variable.name}`"
                        allow-clear
                      >
                        <template v-if="variable.name === 'author'" #prefix>
                          <UserOutlined style="color: rgba(0, 0, 0, 0.25);" />
                        </template>
                      </a-input>
                    </a-tooltip>
                    <a-input
                      v-else
                      v-model:value="wizardData.variables[variable.name]"
                      :placeholder="variable.description || `请输入${variable.title || variable.name}`"
                      allow-clear
                    >
                      <template v-if="variable.name === 'author'" #prefix>
                        <UserOutlined style="color: rgba(0, 0, 0, 0.25);" />
                      </template>
                    </a-input>
                  </a-form-item>

                  <!-- 数字类型 -->
                  <a-form-item
                    v-else-if="variable.type === 'number'"
                    :label="variable.title || variable.name"
                    :required="variable.required"
                  >
                    <a-tooltip v-if="variable.description" placement="top">
                      <template #title>{{ variable.description }}</template>
                      <a-input-number
                        v-model:value="wizardData.variables[variable.name]"
                        :min="variable.min"
                        :max="variable.max"
                        style="width: 100%;"
                      />
                    </a-tooltip>
                    <a-input-number
                      v-else
                      v-model:value="wizardData.variables[variable.name]"
                      :min="variable.min"
                      :max="variable.max"
                      style="width: 100%;"
                    />
                  </a-form-item>

                  <!-- 布尔类型 -->
                  <a-form-item
                    v-else-if="variable.type === 'boolean'"
                    :label="variable.title || variable.name"
                    class="boolean-form-item"
                  >
                    <a-switch
                      v-model:checked="wizardData.variables[variable.name]"
                      :checked-children="variable.trueText || '是'"
                      :un-checked-children="variable.falseText || '否'"
                    />
                  </a-form-item>

                  <!-- 选择类型 -->
                  <a-form-item
                    v-else-if="variable.type === 'select' || variable.type === 'enum'"
                    :label="variable.title || variable.name"
                    :required="variable.required"
                  >
                    <a-select
                      v-model:value="wizardData.variables[variable.name]"
                      :placeholder="`请选择${variable.title || variable.name}`"
                      :options="variable.options"
                      allow-clear
                    />
                  </a-form-item>

                  <!-- 多选类型 -->
                  <a-form-item
                    v-else-if="variable.type === 'multi-select'"
                    :label="variable.title || variable.name"
                    :required="variable.required"
                  >
                    <a-select
                      v-model:value="wizardData.variables[variable.name]"
                      :placeholder="`请选择${variable.title || variable.name}`"
                      :options="variable.options"
                      mode="multiple"
                      allow-clear
                    />
                  </a-form-item>

                  <!-- 文本类型 -->
                  <a-form-item
                    v-else-if="variable.type === 'text'"
                    :label="variable.title || variable.name"
                    :required="variable.required"
                  >
                    <a-textarea
                      v-model:value="wizardData.variables[variable.name]"
                      :placeholder="variable.description || `请输入${variable.title || variable.name}`"
                      :rows="4"
                      :maxlength="variable.maxLength"
                      show-count
                    />
                  </a-form-item>

                  <!-- JSON 对象类型 -->
                  <a-form-item
                    v-else-if="variable.type === 'object'"
                    :label="variable.title || variable.name"
                    :required="variable.required"
                  >
                    <a-textarea
                      v-model:value="wizardData.variables[variable.name]"
                      :placeholder="variable.description || '请输入 JSON 对象'"
                      :rows="6"
                    />
                  </a-form-item>

                  <!-- JSON 数组类型 -->
                  <a-form-item
                    v-else-if="variable.type === 'array'"
                    :label="variable.title || variable.name"
                    :required="variable.required"
                  >
                    <a-textarea
                      v-model:value="wizardData.variables[variable.name]"
                      :placeholder="variable.description || '请输入 JSON 数组'"
                      :rows="6"
                    />
                  </a-form-item>

                  <!-- 默认字符串输入 -->
                  <a-form-item
                    v-else
                    :label="variable.title || variable.name"
                    :required="variable.required"
                  >
                    <a-input
                      v-model:value="wizardData.variables[variable.name]"
                      :placeholder="variable.description || `请输入${variable.title || variable.name}`"
                      allow-clear
                    />
                  </a-form-item>
                </div>
              </a-form>
            </div>

            <!-- 高级模式 -->
            <div v-show="variableMode === 'advanced'" class="advanced-mode">
              <div class="editor-wrap">
                <div class="editor-header">
                  <span>JSON 编辑器</span>
                  <div class="actions">
                    <a-button size="small" @click="formatVariablesJson">格式化</a-button>
                    <a-button size="small" type="primary" @click="syncFromNormalMode">同步普通模式</a-button>
                  </div>
                </div>
                <div ref="jsonEditorContainer" class="json-editor"></div>
                <div class="editor-footer">
                  <span v-if="jsonValid" class="ok">✅ JSON 格式正确</span>
                  <span v-else class="err">❌ {{ jsonError }}</span>
                </div>
              </div>
            </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 步骤4: 预览确认 -->
        <div v-if="currentStep === 4" class="step-panel step-panel-preview">
          <!-- 加载中 -->
          <div v-if="loadingPreview" class="loading-container">
            <a-spin size="large">
              <template #description>正在渲染文件预览...</template>
            </a-spin>
          </div>

          <!-- 预览内容 - 全屏布局 -->
          <div v-else-if="showPreview" class="preview-fullscreen">
            <!-- 主内容区域 -->
            <div class="preview-main">
              <!-- 左侧文件资源管理器 -->
              <div class="file-explorer">
                <div class="explorer-header">
                  <span class="explorer-title">模板文件</span>
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
                  <a-empty
                    v-else
                    description="暂无文件"
                    :image="null"
                  >
                    <template #description>
                      <span style="color: #999;">暂无文件</span>
                    </template>
                  </a-empty>
                </div>
              </div>

              <!-- 右侧代码预览区 -->
              <div class="code-preview">
                <!-- 文件信息栏 - 统一放在这里 -->
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
                    <template #icon>
                      <CopyOutlined />
                    </template>
                    复制
                  </a-button>
                </div>

                <!-- 无文件选中提示 -->
                <div v-if="!currentFileContent" class="no-file-selected">
                  <div class="no-file-icon">
                    <FileTextOutlined style="font-size: 48px; color: #d9d9d9;" />
                  </div>
                  <div class="no-file-text">请选择左侧文件进行预览</div>
                </div>

                <!-- CodeMirror 编辑器容器 -->
                <div v-else class="code-content">
                  <div class="codemirror-container" ref="codeContainer"></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 底部操作按钮 -->
      <template v-if="currentStep < 4" #footer>
        <div class="wizard-footer">
          <a-button v-if="currentStep > 1" @click="prevStep" size="small" class="footer-btn">
            上一步
          </a-button>
          <a-button
            type="primary"
            @click="nextStep"
            :disabled="currentStep === 1 && versionList.length === 0"
            size="small"
            class="footer-btn"
          >
            {{ currentStep === 1 && versionList.length === 0 ? '模板尚未开放' : '下一步' }}
          </a-button>
        </div>
      </template>
    </a-drawer>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount, watch, h, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { message, Modal } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { debounce } from 'lodash-es'
import {
  SearchOutlined,
  UserOutlined,
  FolderOpenOutlined,
  WarningOutlined,
  SettingOutlined,
  StarOutlined,
  EditOutlined,
  CodeOutlined,
  FileTextOutlined,
  CheckCircleOutlined,
  TagsOutlined,
  DownloadOutlined,
  LoadingOutlined,
  InfoCircleOutlined,
  FileOutlined,
  FolderOutlined,
  CopyOutlined
} from '@ant-design/icons-vue'
import { getCategories, getLanguages, getTemplates, getTemplateDetail } from '@/api/templates'
import { listReleases } from '@/api/releases'

// CodeMirror 核心模块
import { EditorView, lineNumbers, highlightActiveLineGutter, keymap } from '@codemirror/view'
import { EditorState } from '@codemirror/state'
import { defaultHighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { defaultKeymap } from '@codemirror/commands'

// Dracula 主题
import { dracula } from '@uiw/codemirror-theme-dracula'

// 语言支持
import { javascript } from '@codemirror/lang-javascript'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { json } from '@codemirror/lang-json'
import { markdown } from '@codemirror/lang-markdown'
import { python } from '@codemirror/lang-python'
import { java } from '@codemirror/lang-java'
import { cpp } from '@codemirror/lang-cpp'
import { rust } from '@codemirror/lang-rust'
import { go } from '@codemirror/lang-go'
import { sql } from '@codemirror/lang-sql'
import { xml } from '@codemirror/lang-xml'
import { yaml } from '@codemirror/lang-yaml'
import { vue } from '@codemirror/lang-vue'

const router = useRouter()
const route = useRoute()

// 状态
const loading = ref(false)
const searchKeyword = ref('')
const selectedCategory = ref('all')
const selectedLanguage = ref('all')
const selectedTemplate = ref(null)

// 模板配置向导
const showWizardModal = ref(false)
const currentStep = ref(1)
const wizardData = ref({
  template: null,
  version: '',  // 直接存储实际版本号，不是 "latest"
  projectName: '',
  outputDir: '',
  variables: {}
})
const configFormRef = ref(null)
const versionList = ref([])
const isDownloading = ref(false)
const downloadProgress = ref(0)
const outputPathExists = ref(false)  // 输出路径是否存在
const loadingVariables = ref(false)  // 变量定义加载状态
const variableDefinitions = ref([])  // 变量定义列表
const variableMode = ref('normal')  // 变量编辑模式: 'normal' | 'advanced'

// JSON 编辑器相关
const jsonEditorContainer = ref(null)
let jsonEditor = null
const jsonValid = ref(true)
const jsonError = ref('')

// 预览相关状态
const loadingPreview = ref(false)  // 预览加载状态
const fileTreeData = ref([])  // 文件树数据
const selectedFileKey = ref('')  // 当前选中的文件key
const expandedKeys = ref([])  // 展开的节点
const currentFileName = ref('')  // 当前文件名
const currentFileContent = ref('')  // 当前文件内容

// CodeMirror 相关
const codeContainer = ref(null)
let editorView = null

// 语言映射
const languageMap = {
  js: javascript(),
  javascript: javascript(),
  ts: javascript({ typescript: true }),
  typescript: javascript({ typescript: true }),
  jsx: javascript({ jsx: true }),
  tsx: javascript({ typescript: true, jsx: true }),
  vue: vue(),
  html: html(),
  htm: html(),
  css: css(),
  scss: css(),
  sass: css(),
  less: css(),
  json: json(),
  md: markdown(),
  markdown: markdown(),
  py: python(),
  python: python(),
  java: java(),
  cpp: cpp(),
  cc: cpp(),
  cxx: cpp(),
  c: cpp(),
  rs: rust(),
  rust: rust(),
  go: go(),
  sql: sql(),
  xml: xml(),
  yaml: yaml(),
  yml: yaml(),
}

// 根据文件扩展名获取语言支持
function getLanguageExtension(filename) {
  const ext = filename.split('.').pop()?.toLowerCase()
  return languageMap[ext] || null
}

// 确保预览容器总是有一个有效的值
const showPreview = computed(() => {
  return !loadingPreview.value && fileTreeData.value !== null && fileTreeData.value !== undefined
})

// 表单验证规则
const projectNameRules = [
  { required: true, message: '请输入项目名称', trigger: 'blur' },
  { min: 2, max: 50, message: '项目名称长度应在 2 到 50 个字符之间', trigger: 'blur' },
  { pattern: /^[a-zA-Z0-9_-]+$/, message: '项目名称只能包含字母、数字、下划线和连字符', trigger: 'blur' }
]

// 步骤配置
const steps = [
  { title: '模板介绍', description: '了解模板详情' },
  { title: '路径配置', description: '设置项目路径' },
  { title: '配置变量', description: '配置模板变量' },
  { title: '预览确认', description: '确认并创建' }
]

// 数据
const categories = ref([
  { id: 'all', name: '全部' }
])

const languages = ref([
  { id: 'all', name: '全部' }
])

const templates = ref([])

// 版本选项（不包含 Latest，与 CLI 保持一致）
const versionOptions = computed(() => {
  const options = []
  versionList.value.forEach((v) => {
    const label = `${v.version}${v.isLatest ? ' (当前)' : ''}${v.isDeprecated ? ' [已弃用]' : ''}`
    options.push({ label, value: v.version })
  })
  return options
})

// 最终输出路径
const finalOutputPath = computed(() => {
  if (wizardData.value.outputDir && wizardData.value.projectName) {
    // 移除末尾可能存在的分隔符，然后统一使用反斜杠
    const dir = wizardData.value.outputDir.replace(/[\\/]+$/, '')
    return `${dir}\\${wizardData.value.projectName}`
  }
  return ''
})

// 过滤后的模板
const filteredTemplates = computed(() => {
  let result = templates.value

  // 搜索过滤
  if (searchKeyword.value.trim()) {
    const keyword = searchKeyword.value.toLowerCase()
    result = result.filter(t => {
      const name = t.name?.toLowerCase() || ''
      const desc = t.description?.toLowerCase() || ''
      return name.includes(keyword) || desc.includes(keyword)
    })
  }

  // 分类过滤
  if (selectedCategory.value !== 'all') {
    result = result.filter(t => {
      return t.categoryId === Number(selectedCategory.value)
    })
  }

  // 语言过滤
  if (selectedLanguage.value !== 'all') {
    result = result.filter(t => {
      return t.languages?.some(l => l.languageId === Number(selectedLanguage.value))
    })
  }

  return result
})

// 选择模板
const selectTemplate = (template) => {
  selectedTemplate.value = template
}

const useTemplate = async (template) => {
  // 打开模板配置向导
  wizardData.value = {
    template,
    version: '',  // 会在 loadVersions 中自动设置为最新版本
    projectName: '',
    outputDir: '',
    variables: {}
  }
  currentStep.value = 1
  showWizardModal.value = true

  // 清理预览状态
  loadingPreview.value = false
  fileTreeData.value = []
  selectedFileKey.value = ''
  expandedKeys.value = []
  currentFileName.value = ''
  currentFileContent.value = ''

  // 加载版本列表（会自动选择最新版本）
  await loadVersions(template.id)
}

// 向导步骤导航
const nextStep = async () => {
  // 从步骤1跳到步骤2时，先检查和下载模板
  if (currentStep.value === 1) {
    try {
      const downloadVersion = wizardData.value.version

      // 先检查模板是否已下载
      const isDownloaded = await invoke('check_template_downloaded', {
        templateId: wizardData.value.template.id.toString(),
        version: downloadVersion
      })

      if (isDownloaded) {
        // 已下载，加载变量定义后进入下一步
        await loadVariableDefinitions()
        if (currentStep.value < 4) {
          currentStep.value++
        }
        return
      }

      // 模板未下载，开始下载
      isDownloading.value = true
      downloadProgress.value = 0

      // 调用 Tauri 命令下载模板
      await invoke('download_template', {
        templateId: wizardData.value.template.id.toString(),
        version: downloadVersion
      })

      isDownloading.value = false
      downloadProgress.value = 100

      // 下载完成，加载变量定义后进入下一步
      await loadVariableDefinitions()
      if (currentStep.value < 4) {
        currentStep.value++
      }
    } catch (error) {
      message.error(`下载模板失败: ${error}`)
      isDownloading.value = false
      downloadProgress.value = 0
      // 不进入下一步，留在步骤1
      return
    }
  } else if (currentStep.value === 2) {
    // 从步骤2跳到步骤3时，验证表单
    try {
      await configFormRef.value.validate()
      if (currentStep.value < 4) {
        currentStep.value++
      }
    } catch (error) {
      // 表单验证失败，不跳转
    }
  } else if (currentStep.value === 3) {
    // 从步骤3跳到步骤4时，加载预览
    await loadPreviewFiles()
    if (currentStep.value < 4) {
      currentStep.value++
    }
  } else {
    // 其他步骤直接跳转
    if (currentStep.value < 4) {
      currentStep.value++
    }
  }
}

const prevStep = () => {
  if (currentStep.value > 1) {
    currentStep.value--
  }
}

// 选择输出目录
const selectOutputDir = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择项目输出目录'
    })

    if (selected) {
      wizardData.value.outputDir = selected
    }
  } catch (error) {
    console.error('选择目录失败:', error)
    message.error('选择目录失败')
  }
}

// 确认创建项目
const confirmCreateProject = async () => {
  try {
    const outputPath = finalOutputPath.value

    // 检查目录是否已存在
    const exists = await invoke('check_directory_exists', {
      path: outputPath
    })

    if (exists) {
      // 目录已存在，显示确认对话框
      Modal.confirm({
        title: '目录已存在',
        content: `项目输出路径 "${outputPath}" 已存在，是否覆盖？`,
        okText: '覆盖',
        okType: 'danger',
        cancelText: '取消',
        onOk: async () => {
          try {
            // 删除已存在的目录
            await invoke('remove_directory', {
              path: outputPath
            })

            // 继续创建项目
            await doCreateProject()
          } catch (error) {
            message.error(`删除目录失败: ${error}`)
          }
        }
      })
    } else {
      // 目录不存在，直接创建
      await doCreateProject()
    }
  } catch (error) {
    console.error('检查目录失败:', error)
    message.error('检查目录失败')
  }
}

// 执行项目创建
const doCreateProject = async () => {
  try {
    const outputPath = finalOutputPath.value

    // 调用 Tauri 命令生成项目
    isDownloading.value = true
    downloadProgress.value = 0

    const result = await invoke('generate_project', {
      templateId: wizardData.value.template.id.toString(),
      variables: wizardData.value.variables,
      outputPath,
      version: wizardData.value.version || null
    })

    isDownloading.value = false
    downloadProgress.value = 100

    message.success(result || '项目创建成功！')

    // 关闭向导
    showWizardModal.value = false

    // 重置状态
    isDownloading.value = false
    downloadProgress.value = 0
  } catch (error) {
    console.error('创建项目失败:', error)
    message.error(`创建项目失败: ${error}`)
    isDownloading.value = false
    downloadProgress.value = 0
  }
}

// 取消创建
const cancelWizard = () => {
  showWizardModal.value = false
  currentStep.value = 1
  wizardData.value = {
    template: null,
    version: '',
    projectName: '',
    outputDir: '',
    variables: {}
  }
  // 清理预览状态
  loadingPreview.value = false
  fileTreeData.value = []
  selectedFileKey.value = ''
  expandedKeys.value = []
  currentFileName.value = ''
  currentFileContent.value = ''
}

// 获取语言名称
const getLanguageName = (languageId) => {
  const lang = languages.value.find(l => l.id === languageId)
  return lang?.name || languageId
}

// 获取分类名称
const getCategoryName = (categoryId) => {
  const cat = categories.value.find(c => c.id === categoryId)
  return cat?.name || categoryId
}

// 加载版本列表
const loadVersions = async (templateId) => {
  try {
    const res = await listReleases(templateId)
    if (res?.data?.versions) {
      versionList.value = res.data.versions

      // 自动选择最新版本（与 CLI 保持一致）
      const latestVersion = versionList.value.find(v => v.isLatest)
      if (latestVersion) {
        wizardData.value.version = latestVersion.version
      } else if (versionList.value.length > 0) {
        // 如果没有找到 is_latest，默认选择第一个版本
        wizardData.value.version = versionList.value[0].version
      }
    }
  } catch (error) {
    console.error('加载版本列表失败:', error)
    versionList.value = []
  }
}

// 处理版本变化
const handleVersionChange = async (version) => {
  wizardData.value.version = version
  // 重新加载变量定义
  await loadVariableDefinitions()
}

// 加载模板变量定义
const loadVariableDefinitions = async () => {
  if (!wizardData.value.template?.id || !wizardData.value.version) {
    return
  }

  try {
    loadingVariables.value = true
    const { getTemplateVariables } = await import('@/api/templateVariables')
    const res = await getTemplateVariables(
      wizardData.value.template.id,
      wizardData.value.version
    )

    if (res?.data?.fieldSchemaJson) {
      const schema = JSON.parse(res.data.fieldSchemaJson)
      variableDefinitions.value = Object.entries(schema).map(([name, config]) => ({
        name,
        title: config.title || name,
        description: config.description || '',
        type: config.type || 'string',
        required: !!config.required,
        default: config.default !== undefined ? config.default : getDefaultValueByType(config.type),
        min: config.min,
        max: config.max,
        maxLength: config.maxLength,
        trueText: config.trueText,
        falseText: config.falseText,
        options: config.options?.map(opt => ({
          label: opt.label || opt,
          value: opt.value !== undefined ? opt.value : opt
        }))
      }))

      // 初始化表单数据
      initVariablesForm()
    } else {
      variableDefinitions.value = []
    }
  } catch (error) {
    console.error('加载变量定义失败:', error)
    variableDefinitions.value = []
    // 不显示错误消息，有些模板可能没有变量定义
  } finally {
    loadingVariables.value = false
  }
}

// 根据类型获取默认值
const getDefaultValueByType = (type) => {
  const defaults = {
    string: '',
    number: 0,
    boolean: false,
    text: '',
    select: undefined,
    'multi-select': [],
    object: '{}',
    array: '[]'
  }
  return defaults[type] !== undefined ? defaults[type] : ''
}

// 初始化变量表单
const initVariablesForm = () => {
  variableDefinitions.value.forEach(variable => {
    if (wizardData.value.variables[variable.name] === undefined) {
      // 使用默认值或父组件传入的值
      wizardData.value.variables[variable.name] = variable.default
    }
  })
}

// ========== JSON 编辑器相关函数 ==========

// 初始化 JSON 编辑器
const initJsonEditor = async () => {
  if (!jsonEditorContainer.value) return

  // 清理旧编辑器
  if (jsonEditor) {
    jsonEditor.destroy()
    jsonEditor = null
  }

  try {
    // 延迟导入 CodeMirror 模块
    const [{ EditorView }, { EditorState }, { javascript }, { dracula }, { keymap }, { defaultKeymap }] = await Promise.all([
      import('@codemirror/view'),
      import('@codemirror/state'),
      import('@codemirror/lang-javascript'),
      import('@uiw/codemirror-theme-dracula'),
      import('@codemirror/view'),
      import('@codemirror/commands')
    ])

    const state = EditorState.create({
      doc: JSON.stringify(wizardData.value.variables, null, 2),
      extensions: [
        dracula,
        javascript(),
        keymap.of(defaultKeymap),
        lineNumbers(),
        EditorView.updateListener.of(() => validateJson())
      ]
    })

    jsonEditor = new EditorView({
      state,
      parent: jsonEditorContainer.value
    })

    validateJson()
  } catch (error) {
    console.error('初始化 JSON 编辑器失败:', error)
  }
}

// 验证 JSON
const validateJson = () => {
  if (!jsonEditor) return

  try {
    JSON.parse(jsonEditor.state.doc.toString())
    jsonValid.value = true
    jsonError.value = ''
  } catch (e) {
    jsonValid.value = false
    jsonError.value = e.message
  }
}

// 格式化 JSON
const formatVariablesJson = () => {
  if (!jsonEditor) return

  try {
    const data = JSON.parse(jsonEditor.state.doc.toString())
    const formatted = JSON.stringify(data, null, 2)
    jsonEditor.dispatch({
      changes: {
        from: 0,
        to: jsonEditor.state.doc.length,
        insert: formatted
      }
    })
  } catch (e) {
    message.error('JSON 格式错误，无法格式化')
  }
}

// 从普通模式同步到高级模式
const syncFromNormalMode = () => {
  if (!jsonEditor) return

  try {
    const json = JSON.stringify(wizardData.value.variables, null, 2)
    jsonEditor.dispatch({
      changes: {
        from: 0,
        to: jsonEditor.state.doc.length,
        insert: json
      }
    })
    message.success('已同步到高级模式')
  } catch (e) {
    message.error('同步失败：' + e.message)
  }
}

// 监听模式切换
watch(variableMode, async (newMode, oldMode) => {
  if (newMode === 'advanced') {
    // 切换到高级模式
    if (!jsonEditor) {
      // 第一次切换，需要初始化编辑器
      await nextTick()
      await nextTick() // 双重 nextTick 确保 DOM 完全渲染
      initJsonEditor()
    } else {
      // 编辑器已存在，只更新内容
      await nextTick()
      try {
        const json = JSON.stringify(wizardData.value.variables, null, 2)
        jsonEditor.dispatch({
          changes: {
            from: 0,
            to: jsonEditor.state.doc.length,
            insert: json
          }
        })
      } catch (e) {
        console.warn('同步数据到编辑器失败:', e)
      }
    }
  } else if (oldMode === 'advanced' && jsonEditor) {
    // 从高级模式切换到普通模式，同步数据
    try {
      const data = JSON.parse(jsonEditor.state.doc.toString())
      Object.assign(wizardData.value.variables, data)
    } catch (e) {
      console.warn('解析 JSON 失败:', e)
    }
  }
})

// 渲染变量标签
const renderVariableLabel = (variable) => {
  const label = variable.title
  if (variable.required) {
    return h('span', {}, [
      h('span', { style: { color: '#ff4d4f', marginRight: '4px' } }, '*'),
      label
    ])
  }
  return label
}

// 加载文件预览
const loadPreviewFiles = async () => {
  if (!wizardData.value.template?.id || !wizardData.value.version) {
    return
  }

  try {
    loadingPreview.value = true

    // 重置状态
    selectedFileKey.value = ''
    expandedKeys.value = []
    currentFileName.value = ''
    currentFileContent.value = ''

    // 调用本地 Tauri 命令渲染预览
    const treeJson = await invoke('render_template_preview', {
      templateId: wizardData.value.template.id.toString(),
      variables: wizardData.value.variables,
      version: wizardData.value.version
    })

    // 解析渲染结果（扁平数组）
    const flatTree = JSON.parse(treeJson)

    // 将扁平数组转换为树形结构
    const tree = buildTreeFromFlatArray(flatTree)

    if (Array.isArray(tree)) {
      fileTreeData.value = convertTreeToAntDesign(tree)

      // 默认展开第一层并选择第一个文件
      if (tree.length > 0) {
        expandedKeys.value = tree
          .filter(node => node.isDirectory === 1)
          .map(node => node.key || node.id)
          .filter(key => key) // 过滤掉 undefined

        // 查找第一个文件
        const firstFile = findFirstFile(tree)
        if (firstFile) {
          const key = firstFile.key || firstFile.id
          if (key) {
            // 使用 nextTick 确保 DOM 更新后再选择文件
            await nextTick()
            selectedFileKey.value = key
            currentFileName.value = firstFile.file_name || ''
            currentFileContent.value = firstFile.file_content || ''
          }
        }
      }
    } else {
      fileTreeData.value = []
    }
  } catch (error) {
    console.error('加载文件预览失败:', error)
    message.error('加载文件预览失败')
    fileTreeData.value = []
  } finally {
    loadingPreview.value = false
  }
}

// 将扁平数组转换为树形结构
const buildTreeFromFlatArray = (flatArray) => {
  if (!Array.isArray(flatArray)) return []

  // 创建 ID 到节点的映射
  const nodeMap = new Map()
  const rootNodes = []

  // 第一遍：创建所有节点
  flatArray.forEach(node => {
    nodeMap.set(node.id, { ...node, children: [] })
  })

  // 第二遍：构建父子关系
  flatArray.forEach(node => {
    const currentNode = nodeMap.get(node.id)

    if (node.parent_id === 0) {
      // 根节点
      rootNodes.push(currentNode)
    } else {
      // 子节点，添加到父节点的 children
      const parentNode = nodeMap.get(node.parent_id)
      if (parentNode) {
        parentNode.children.push(currentNode)
      }
    }
  })

  return rootNodes
}

// 转换树数据为 Ant Design Tree 格式
const convertTreeToAntDesign = (tree) => {
  if (!Array.isArray(tree)) return []

  // 排序：目录在前，文件在后
  const customSort = (a, b) => {
    if ((b.is_directory || 0) - (a.is_directory || 0) !== 0) {
      return (b.is_directory || 0) - (a.is_directory || 0)
    }
    const nameA = (a.file_name || a.key || '').toLowerCase()
    const nameB = (b.file_name || b.key || '').toLowerCase()
    return nameA.localeCompare(nameB)
  }

  const sorted = [...tree].sort(customSort)

  return sorted.map((node) => {
    // 同时保留 snake_case 和 camelCase 字段，确保兼容性
    const result = {
      key: node.key || node.id,
      title: node.file_name || node.key,
      // snake_case（后端返回）
      file_name: node.file_name,
      is_directory: node.is_directory,
      file_path: node.file_path,
      file_content: node.file_content,
      // camelCase（前端使用）
      fileName: node.file_name,
      isDirectory: node.is_directory === 1,
      filePath: node.file_path,
      fileContent: node.file_content,
      // 树形结构
      icon: node.is_directory === 1 ? h(FolderOutlined) : h(FileOutlined),
      children: node.children && node.children.length > 0 ? convertTreeToAntDesign(node.children) : []
    }
    return result
  })
}

// 查找第一个文件
const findFirstFile = (nodes) => {
  if (!nodes || nodes.length === 0) return null

  for (const node of nodes) {
    if (node.is_directory !== 1) {
      return node
    }
    if (node.children && node.children.length > 0) {
      const found = findFirstFile(node.children)
      if (found) return found
    }
  }
  return null
}

// 选择文件
const onFileSelect = async (selectedKeys) => {
  if (!selectedKeys || selectedKeys.length === 0) return

  const key = selectedKeys[0]
  if (!key) return

  selectedFileKey.value = key

  // 从树数据中查找节点
  const findNode = (nodes, targetKey) => {
    if (!nodes) return null
    for (const node of nodes) {
      if (node.key === targetKey) {
        return node
      }
      if (node.children && node.children.length > 0) {
        const found = findNode(node.children, targetKey)
        if (found) return found
      }
    }
    return null
  }

  const node = findNode(fileTreeData.value, key)
  if (!node) return

  if (node.isDirectory) {
    currentFileName.value = ''
    currentFileContent.value = ''
    return
  }

  currentFileName.value = node.fileName || node.file_name || ''
  currentFileContent.value = node.fileContent || node.file_content || ''

  // 等待 DOM 更新后创建或更新 CodeMirror 编辑器
  await nextTick()
  createOrUpdateEditor()
}

// 创建或更新 CodeMirror 编辑器
const createOrUpdateEditor = async () => {
  if (!codeContainer.value) return

  // 销毁现有编辑器
  if (editorView) {
    editorView.destroy()
    editorView = null
  }

  // 获取语言扩展
  const languageExt = currentFileName.value ? getLanguageExtension(currentFileName.value) : null

  // 创建右键菜单处理器
  const contextMenuHandler = EditorView.domEventHandlers({
    contextmenu: (event, view) => {
      event.preventDefault()

      // 获取选中的文本
      const selection = view.state.selection.main
      const selectedText = view.state.sliceDoc(selection.from, selection.to)

      // 创建右键菜单
      const menu = document.createElement('div')
      menu.className = 'editor-context-menu'
      menu.style.cssText = `
        position: fixed;
        left: ${event.clientX}px;
        top: ${event.clientY}px;
        background: #282a36;
        border: 1px solid #44475a;
        border-radius: 4px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
        padding: 4px 0;
        min-width: 80px;
        z-index: 1000;
      `

      // 复制菜单项
      const copyItem = document.createElement('div')
      copyItem.style.cssText = `
        padding: 6px 16px;
        cursor: pointer;
        font-size: 13px;
        color: #f8f8f2;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
        transition: all 0.15s ease;
      `
      copyItem.textContent = '复制'

      copyItem.onmouseover = () => {
        copyItem.style.background = '#44475a'
      }
      copyItem.onmouseout = () => {
        copyItem.style.background = 'transparent'
      }

      copyItem.onclick = () => {
        if (selectedText) {
          // 复制选中的文本
          navigator.clipboard.writeText(selectedText).then(() => {
            message.success('内容已复制到剪贴板')
          }).catch(() => {
            message.error('复制失败')
          })
        } else {
          // 复制全部内容
          const allText = view.state.doc.toString()
          navigator.clipboard.writeText(allText).then(() => {
            message.success('内容已复制到剪贴板')
          }).catch(() => {
            message.error('复制失败')
          })
        }
        document.body.removeChild(menu)
      }

      menu.appendChild(copyItem)
      document.body.appendChild(menu)

      // 点击其他地方关闭菜单
      const closeMenu = (e) => {
        if (!menu.contains(e.target)) {
          document.body.removeChild(menu)
          document.removeEventListener('click', closeMenu)
        }
      }
      setTimeout(() => {
        document.addEventListener('click', closeMenu)
      }, 0)
    }
  })

  // 创建编辑器扩展
  const extensions = [
    // Dracula 主题
    dracula,
    // 只读模式
    EditorView.editable.of(false),
    // 行号
    lineNumbers(),
    // 当前行行号高亮
    highlightActiveLineGutter(),
    // 语法高亮
    syntaxHighlighting(defaultHighlightStyle),
    // 默认键盘映射（包含复制快捷键）
    keymap.of(defaultKeymap),
    // 右键菜单
    contextMenuHandler,
    // 确保滚动正常工作
    EditorView.scrollMargins.of(() => ({ top: 10, bottom: 10 })),
    // 强制启用滚动
    EditorView.theme({
      '&': { height: '100%' },
      '.cm-scroller': {
        overflow: 'auto !important',
        height: '100% !important',
      },
    }),
  ]

  // 添加语言支持
  if (languageExt) {
    extensions.push(languageExt)
  }

  // 创建编辑器状态
  const state = EditorState.create({
    doc: currentFileContent.value,
    extensions,
  })

  // 创建编辑器视图
  editorView = new EditorView({
    state,
    parent: codeContainer.value,
  })

  // 确保编辑器可以滚动
  setTimeout(() => {
    if (editorView && editorView.scrollDOM) {
      editorView.requestMeasure()
    }
  }, 100)
}

// 展开/折叠节点
const onExpand = (keys) => {
  expandedKeys.value = keys
}

// 复制文件内容
const copyFileContent = async () => {
  if (!currentFileContent.value) {
    message.warning('没有可复制的内容')
    return
  }

  try {
    await navigator.clipboard.writeText(currentFileContent.value)
    message.success('内容已复制到剪贴板')
  } catch (error) {
    console.error('复制失败:', error)
    message.error('复制失败，请手动复制')
  }
}

// 生成代码片段预览
const getCodeSnippet = (template) => {
  // 获取主要语言
  const primaryLanguage = template.languages?.find(l => l.isPrimary === 1)
  const languageId = primaryLanguage?.languageId
  const lang = languages.value.find(l => l.id === languageId)?.name || ''
  const name = template.name || 'Template'

  if (lang.includes('Rust') || lang.includes('rust')) {
    return `fn main() {\n    println!("Hello, ${name}!");\n}`
  }

  if (lang.includes('Go') || lang.includes('go') || lang.includes('Golang')) {
    return `package main\n\nimport "fmt"\n\nfunc main() {\n    fmt.Printf("Hello, ${name}!\\n")\n}`
  }

  if (lang.includes('Python') || lang.includes('python')) {
    return `def main():\n    print(f"Hello, {name}!")\n\nif __name__ == "__main__":\n    main()`
  }

  if (lang.includes('JavaScript') || lang.includes('javascript')) {
    return `function main() {\n  console.log('Hello, ${name}!');\n}\n\nmain();`
  }

  if (lang.includes('TypeScript') || lang.includes('typescript')) {
    return `function main(): void {\n  console.log('Hello, ${name}!');\n}\n\nmain();`
  }

  return `// ${name}\nclass App {\n  constructor() {\n    this.name = '${name}';\n  }\n\n  run() {\n    console.log('Running', this.name);\n  }\n}`
}

// 加载分类数据
const loadCategories = async () => {
  try {
    const res = await getCategories({ all: 1 })
    if (res?.data?.categoriesList) {
      categories.value = [
        { id: 'all', name: '全部' },
        ...res.data.categoriesList
      ]
    } else {
      categories.value = [{ id: 'all', name: '全部' }]
    }
  } catch (error) {
    console.error('加载分类失败:', error)
    categories.value = [{ id: 'all', name: '全部' }]
  }
}

// 加载语言数据
const loadLanguages = async () => {
  try {
    const res = await getLanguages({ all: 1 })
    if (res?.data?.languagesList) {
      languages.value = [
        { id: 'all', name: '全部' },
        ...res.data.languagesList
      ]
    } else {
      languages.value = [{ id: 'all', name: '全部' }]
    }
  } catch (error) {
    console.error('加载语言失败:', error)
    languages.value = [{ id: 'all', name: '全部' }]
  }
}

// 加载模板数据
const loadTemplates = async () => {
  try {
    loading.value = true

    // 构建查询参数
    const params = {}

    // 搜索关键词
    if (searchKeyword.value.trim()) {
      params.name = searchKeyword.value.trim()
      params.description = searchKeyword.value.trim()
    }

    // 分类筛选
    if (selectedCategory.value !== 'all') {
      params.categoryId = Number(selectedCategory.value)
    }

    // 语言筛选
    if (selectedLanguage.value !== 'all') {
      params.languageId = Number(selectedLanguage.value)
    }

    const res = await getTemplates(params)
    if (res?.data?.templatesList) {
      templates.value = res.data.templatesList
    } else {
      templates.value = []
    }
  } catch (error) {
    console.error('加载模板失败:', error)
    message.error('加载模板失败')
  } finally {
    loading.value = false
  }
}

// 监听筛选条件变化（使用前端过滤，无需重新加载）
watch([selectedCategory, selectedLanguage], () => {
  // 前端过滤会自动更新
})

// 监听搜索框（使用前端过滤，无需重新加载）
watch(searchKeyword, () => {
  // 前端过滤会自动更新
})

// 监听输出路径变化，检查是否存在
const checkOutputPathExists = debounce(async () => {
  if (!finalOutputPath.value) {
    outputPathExists.value = false
    return
  }

  try {
    const exists = await invoke('check_directory_exists', {
      path: finalOutputPath.value
    })
    outputPathExists.value = exists
  } catch (error) {
    console.error('检查目录失败:', error)
    outputPathExists.value = false
  }
}, 300)

watch(finalOutputPath, () => {
  checkOutputPathExists()
})

// 组件卸载时清理编辑器
onBeforeUnmount(() => {
  if (editorView) {
    editorView.destroy()
  }
  if (jsonEditor) {
    jsonEditor.destroy()
  }
})

// 初始化
onMounted(async () => {
  await loadCategories()
  await loadLanguages()
  await loadTemplates()
})
</script>

<style scoped>
.templates-view {
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

/* 筛选栏 */
.filter-bar {
  margin-bottom: var(--spacing-lg);
  padding: var(--spacing-md);
  background: var(--color-surface);
  border-radius: var(--border-radius-lg);
  border: 1px solid var(--color-border);
}

.filter-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-xs) 0;
}

.filter-row:not(:last-child) {
  margin-bottom: var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
  padding-bottom: var(--spacing-md);
}

.filter-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
  white-space: nowrap;
  min-width: 50px;
}

/* 模板列表 */
.templates-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--spacing-lg);
}

/* 模板卡片 */
.template-card {
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
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
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.1);
}

.card-visual {
  height: 140px;
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
  background: linear-gradient(
    90deg,
    transparent 0%,
    rgba(24, 144, 255, 0.03) 45%,
    rgba(24, 144, 255, 0.08) 50%,
    rgba(24, 144, 255, 0.03) 55%,
    transparent 100%
  );
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
  letter-spacing: 0.3px;
}

.card-content {
  padding: 16px 20px 20px;
}

.template-name {
  margin: 0 0 6px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  letter-spacing: -0.2px;
}

.template-card:hover .template-name {
  color: var(--color-primary);
}

.template-desc {
  margin: 0 0 12px 0;
  font-size: 13px;
  color: #64748b;
  line-height: 1.6;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.template-languages {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 14px;
}

.template-tag {
  background: #f1f5f9;
  border: 1px solid #e2e8f0;
  color: #475569;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.template-card:hover .template-tag {
  background: rgba(24, 144, 255, 0.08);
  border-color: rgba(24, 144, 255, 0.2);
  color: #1890ff;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 12px;
  border-top: 1px solid #f1f5f9;
}

.card-author {
  display: flex;
  align-items: center;
  gap: 8px;
}

.author-avatar {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  background: linear-gradient(135deg, #0f172a 0%, #334155 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: #fff;
  font-size: 12px;
}

.author-name {
  font-size: 12px;
  color: #64748b;
  font-weight: 500;
}

/* 模板配置向导 */
/* 抽屉头部自定义 */
.wizard-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 12px 24px;
  flex-shrink: 0;
  min-height: 56px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.template-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.header-right {
  display: flex;
  align-items: center;
}

/* 步骤4头部操作按钮 */
.wizard-header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.wizard-header-actions .ant-btn {
  font-size: 14px;
  height: 32px;
  padding: 4px 16px;
  border-radius: 6px;
  font-weight: 500;
  transition: all 0.3s ease;
}

.wizard-header-actions .ant-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.wizard-header-actions .ant-btn-primary {
  background: var(--color-primary);
  border-color: var(--color-primary);
}

.wizard-header-actions .ant-btn-primary:hover {
  background: #1565c0;
  border-color: #1565c0;
}

/* 紧凑型步骤指示器 */
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
  position: relative;
  flex-shrink: 0;
}

.step-item.active .step-dot {
  background: #1890ff;
  color: #fff;
  transform: scale(1.15);
  box-shadow: 0 2px 8px rgba(24, 144, 255, 0.4);
}

.step-item.completed .step-dot {
  background: #52c41a;
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
  color: #1890ff;
  font-weight: 600;
}

.step-item.completed .step-text {
  color: #52c41a;
}

.wizard-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 16px 24px;
  display: flex;
  flex-direction: column;
}

/* 步骤4时移除 padding，占满空间 */
.wizard-content-preview {
  padding: 0 !important;
  overflow: hidden !important;
  max-width: none !important;     /* 移除宽度限制，占满全宽 */
  margin: 0 !important;            /* 移除居中对齐 */
  height: 100% !important;         /* 确保占满父容器高度 */
}

.step-panel {
  animation: fadeIn 0.3s ease-in-out;
  padding: 8px;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* 步骤4预览特别处理 - 占满可用空间 */
.step-panel-preview {
  padding: 0;
  overflow: hidden;
  flex: 1;
  min-height: 0;
  height: 100%;  /* 确保占满父容器高度 */
  display: flex;
  flex-direction: column;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 模板介绍 */
.template-intro {
  padding: 8px;
}

.intro-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 24px;
  flex-wrap: wrap;
}

.intro-header h2 {
  margin: 0;
  font-size: 28px;
  font-weight: 600;
  color: var(--color-text);
}

.intro-description {
  font-size: 16px;
  color: var(--color-text-secondary);
  line-height: 1.8;
  margin-bottom: 32px;
}

.intro-section {
  margin-bottom: 32px;
}

.intro-section h3 {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 16px;
}

.languages-list {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.intro-markdown {
  background: var(--color-surface);
  padding: 20px;
  border-radius: var(--border-radius-md);
  border: 1px solid var(--color-border);
  line-height: 1.8;
  color: var(--color-text-secondary);
  font-size: 14px;
}

/* 底部操作栏 */
.wizard-footer {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 8px;
}

.wizard-footer .footer-btn {
  height: 32px !important;
  font-size: 13px !important;
  margin: 0 !important;
}

/* 调整抽屉内的表单项间距 */
.wizard-content .ant-form-item {
  margin-bottom: 24px;
}

/* 让 Drawer 内容包装器使用 flex 布局 */
:deep(.ant-drawer-content-wrapper) {
  display: flex !important;
  flex-direction: column !important;
}

/* 移除 Drawer 默认的 body padding，并让它占满剩余空间 */
:deep(.ant-drawer-body) {
  padding: 0 !important;
  display: flex !important;
  flex-direction: column !important;
  flex: 1 !important;  /* 关键：使用 flex: 1 占满剩余空间 */
  overflow: hidden !important;
}

/* 抽屉 header 优化 */
:deep(.ant-drawer-header) {
  padding: 12px 24px !important;
  margin: 0 !important;
}

/* 移除 Drawer 默认的 body padding */
:deep(.ant-drawer-container) {
  padding: 0 !important;
}

:deep(.ant-drawer-content) {
  padding: 0 !important;
}

.wizard-content .ant-form-item-label > label {
  font-size: 14px;
  font-weight: 500;
}

.wizard-content .ant-input,
.wizard-content .ant-select-selector {
  font-size: 14px;
}

/* 表单项提示文本 */
.form-item-hint {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 4px;
}

/* 优化标题图标 */
.intro-section h3 {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 优化描述列表样式 */
.wizard-content .ant-descriptions-large .ant-descriptions-item-label {
  font-weight: 500;
}

/* 抽屉内容区域限制最大宽度，避免过宽 */
.wizard-content {
  max-width: 1600px;
  margin: 0 auto;
}

/* 步骤3（变量配置）使用全宽 */
.wizard-content:has(.normal-mode) {
  max-width: none;
}

/* 自定义分隔线，替代 a-divider 以更好的图标对齐 */
.section-divider {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 24px 0;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--color-border);
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.section-divider .anticon {
  font-size: 18px;
  color: #1890ff;
}

/* 旋转动画 */
@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* 变量配置样式 */
.loading-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;
}

.no-variables {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;
}

.variables-form {
  padding: 0;
  max-width: none;
  margin: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* 左右布局容器 */
.variables-layout {
  display: flex;
  gap: 0;
  height: 100%;
  min-height: 500px;
}

/* 左侧边栏 */
.variables-sidebar {
  width: 200px;
  flex-shrink: 0;
  background: var(--color-bg-elevated);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 20px 16px 12px;
  border-bottom: 1px solid var(--color-border);
}

.sidebar-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* 垂直模式切换 */
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

.mode-tab.active .mode-icon {
  color: var(--color-primary);
}

.mode-icon {
  font-size: 18px;
  color: var(--color-text-secondary);
  transition: color 0.2s ease;
}

.mode-label {
  flex: 1;
}

/* 右侧内容区域 */
.variables-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px;
  background: var(--color-bg-container);
}

/* 表单头部 */
.form-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
  gap: 16px;
}

.form-title-section {
  flex: 1;
}

.form-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 4px;
  color: var(--color-text);
}

.form-desc {
  color: var(--color-text-secondary);
  font-size: 14px;
  margin: 0;
}

.mode-tabs {
  flex-shrink: 0;
}

/* 普通模式 */
.normal-mode {
  width: 100%;
  padding: 0;
}

.normal-mode :deep(.ant-form) {
  background: transparent;
  max-width: none;
}

.normal-mode :deep(.ant-form-item) {
  margin-bottom: 24px;
}

/* 标签区域 */
.normal-mode :deep(.ant-form-item-label) {
  width: 120px;
  max-width: 120px;
  padding-right: 12px;
}

/* 输入控件区域 */
.normal-mode :deep(.ant-form-item-control) {
  flex: 1;
  max-width: none;
}

/* 布尔类型特殊样式 - 不占据整行 */
.normal-mode :deep(.boolean-form-item .ant-form-item-control) {
  flex: 0 0 auto;
  max-width: fit-content;
}

.normal-mode :deep(.boolean-form-item .ant-form-item-control-input) {
  min-height: auto;
}

.normal-mode :deep(.boolean-form-item .ant-switch) {
  flex-shrink: 0;
}

.normal-mode :deep(.ant-form-item-label > label) {
  font-weight: 600;
  color: var(--color-text);
  font-size: 14px;
}

.normal-mode :deep(.ant-form-item-required::before) {
  display: inline-block;
  margin-right: 4px;
  color: #ff4d4f;
  font-size: 14px;
  font-family: SimSun, sans-serif;
  line-height: 1;
  content: '*';
}

.form-field-item {
  width: 100%;
}

/* 确保所有输入控件使用 100% 宽度 */
.normal-mode :deep(.ant-input),
.normal-mode :deep(.ant-input-number),
.normal-mode :deep(.ant-select),
.normal-mode :deep(.ant-switch) {
  width: 100%;
}

/* 输入框容器 */
.normal-mode :deep(.ant-input),
.normal-mode :deep(.ant-select-selector),
.normal-mode :deep(.ant-input-number),
.normal-mode :deep(.ant-input-number-input) {
  width: 100%;
}

/* 文本区域 */
.normal-mode :deep(.ant-input-textarea-show-count::after) {
  bottom: 2px;
  right: 11px;
}

/* 高级模式 */
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
  min-height: 0;
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

.editor-header .actions {
  display: flex;
  gap: 8px;
}

.json-editor {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.editor-footer {
  padding: 8px 16px;
  background: var(--color-bg-elevated);
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 6px;
  border-top: 1px solid var(--color-border);
}

.editor-footer .ok {
  color: #52c41a;
}

.editor-footer .err {
  color: #ff4d4f;
}

/* 保持旧的样式用于兼容 */
.variable-item {
  margin-bottom: 24px;
}

.variable-item.full-width {
  grid-column: 1 / -1;
}

.variable-description {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 4px;
  line-height: 1.5;
}

/* 步骤4预览特别处理 - 全屏布局 */
.preview-step-panel {
  position: absolute;
  inset: 0;
  padding: 0;
  overflow: hidden;
}

.preview-fullscreen {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #f5f5f5;
  min-height: 0;
  height: 100%;  /* 确保始终占满父容器高度 */
  overflow: hidden;
}

/* 预览顶部 Header - 更紧凑的设计 */
.preview-header {
  height: 48px;
  background: #fff;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.02);
  flex-shrink: 0;
  z-index: 10;
}

.preview-header .header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.preview-title {
  font-size: 15px;
  font-weight: 600;
  color: #1890ff;
}

.preview-header .template-name {
  font-size: 13px;
  color: #999;
}

.preview-header .header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.preview-header .header-actions .ant-btn {
  font-size: 13px;
  height: 28px;
  padding: 0 12px;
}

/* 预览主内容区域 */
.preview-main {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
  min-height: 100%;  /* 确保至少占满父容器高度 */
  align-items: stretch;  /* 确保左右两侧高度一致 */
}

/* 左侧文件资源管理器 */
.file-explorer {
  width: 280px;
  background: #fff;
  border-right: 1px solid #e0e0e0;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  height: 100%;  /* 确保占满父容器高度 */
}

.explorer-header {
  height: 48px;
  background: #f8f9fa;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  align-items: center;
  padding: 0 16px;
  flex-shrink: 0;
  margin: 0;
  box-sizing: border-box;
}

.explorer-title {
  font-size: 14px;
  font-weight: 600;
  color: #333;
  line-height: normal;
}

.explorer-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.explorer-content .ant-tree {
  background: transparent;
}

.explorer-content .file-name {
  font-size: 13px;
}

.explorer-content .ant-tree-node-content-wrapper {
  padding: 4px 8px;
  border-radius: 4px;
}

.explorer-content .ant-tree-node-content-wrapper:hover {
  background: rgba(0, 0, 0, 0.04);
}

.explorer-content .ant-tree-node-selected .ant-tree-node-content-wrapper {
  background: #e6f7ff;
}

/* 右侧代码预览 */
.preview-main .code-preview {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #fff;
  overflow: hidden;
  height: 100%;  /* 确保占满父容器高度 */
}

/* 右侧文件信息栏 */
.preview-main .file-header {
  height: 48px;
  background: #f8f9fa;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  flex-shrink: 0;
  margin: 0;
  box-sizing: border-box;
}

.preview-main .file-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.preview-main .file-name {
  font-size: 14px;
  font-weight: 600;
  color: #333;
  font-family: 'Consolas', 'Monaco', monospace;
  line-height: normal;
}

.preview-main .file-header .ant-btn {
  height: 28px;
  padding: 0 12px;
  font-size: 13px;
  line-height: 1;
  border: none;
  background: transparent;
  color: #666;
  transition: all 0.2s ease;
}

.preview-main .file-header .ant-btn:hover:not(:disabled) {
  color: #1890ff;
  background: rgba(24, 144, 255, 0.06);
}

.preview-main .file-header .ant-btn:disabled {
  color: #d9d9d9;
}

.preview-main .no-file-selected {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #999;
}

.preview-main .no-file-icon {
  margin-bottom: 16px;
}

.preview-main .no-file-text {
  font-size: 16px;
}

.preview-main .file-preview {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.preview-main .code-content {
  flex: 1;
  overflow: hidden;
  background: #1e1e1e;
}

.preview-main .codemirror-container {
  height: 100%;
  min-height: 400px;
}

/* CodeMirror 样式覆盖 */
:deep(.cm-editor) {
  height: 100% !important;
  font-size: 14px;
  outline: none !important;
}

:deep(.cm-editor .cm-scroller) {
  font-family: 'Fira Code', 'Consolas', 'Monaco', monospace;
  overflow: auto !important;
  height: 100% !important;
  max-height: none !important;
}

:deep(.cm-editor .cm-line) {
  padding: 0;
}

/* 只读模式下的光标隐藏 */
:deep(.cm-editor .cm-cursor) {
  display: none !important;
}

:deep(.cm-editor .cm-cursor-primary) {
  display: none !important;
}

</style>
