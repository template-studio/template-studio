<template>
  <a-modal
    v-model:open="show"
    title="从项目提取模板"
    width="760px"
    :footer="null"
    :mask-closable="false"
  >
    <a-steps :current="step" size="small" style="margin-bottom: 20px">
      <a-step title="选择目录" />
      <a-step title="选择文件" />
      <a-step title="AI 分析" />
      <a-step title="创建模板" />
    </a-steps>

    <!-- 步骤1:选择目录 -->
    <div v-if="step === 0">
      <a-space direction="vertical" style="width: 100%">
        <a-button :loading="scanning" @click="pickDir">
          <template #icon><FolderOpenOutlined /></template>
          选择项目目录
        </a-button>
        <template v-if="scanResult">
          <a-alert type="success" show-icon>
            <template #message>
              {{ dirPath }} — 共 {{ scanResult.files.length }} 个候选文件,约
              {{ formatSize(scanResult.totalSize) }}
            </template>
          </a-alert>
          <div style="max-height: 260px; overflow-y: auto; border: 1px solid var(--color-border); border-radius: 8px; padding: 8px 12px">
            <div v-for="f in scanResult.files" :key="f.path" style="font-size: 12px; color: var(--color-text-secondary)">
              {{ f.path }}
            </div>
          </div>
        </template>
      </a-space>
      <div class="wizard-actions">
        <a-button @click="show = false">取消</a-button>
        <a-button type="primary" :disabled="!scanResult || scanResult.files.length === 0" @click="step = 1">
          下一步
        </a-button>
      </div>
    </div>

    <!-- 步骤2:选择文件 -->
    <div v-else-if="step === 1">
      <p style="color: var(--color-text-secondary); font-size: 13px">
        勾选要纳入模板的文件(已按扩展名白名单过滤,最多 120 个文件)
      </p>
      <a-tree
        v-model:checked-keys="checkedKeys"
        :tree-data="treeData"
        checkable
        :height="280"
        style="border: 1px solid var(--color-border); border-radius: 8px; padding: 8px"
      />
      <div class="wizard-actions">
        <a-button @click="step = 0">上一步</a-button>
        <a-button type="primary" :disabled="checkedFileKeys.length === 0" @click="runAnalyze">
          下一步:AI 分析
        </a-button>
      </div>
    </div>

    <!-- 步骤3:AI 变量建议 -->
    <div v-else-if="step === 2">
      <a-spin :spinning="analyzing">
        <template v-if="suggestions.length > 0">
          <p style="color: var(--color-text-secondary); font-size: 13px">
            勾选要参数化的字面量(替换为 <code v-pre>{{ 变量名 }}</code> 占位符),变量名可编辑
          </p>
          <a-table
            :columns="suggestColumns"
            :data-source="suggestions"
            :pagination="false"
            size="small"
            :row-selection="{ selectedRowKeys, onChange: onSelectionChange }"
            :scroll="{ y: 260 }"
            row-key="value"
          />
        </template>
        <a-empty v-else-if="!analyzing" description="未发现适合参数化的字面量,可直接创建静态模板" />
      </a-spin>
      <div class="wizard-actions">
        <a-button @click="step = 1">上一步</a-button>
        <a-button type="primary" @click="step = 3">下一步</a-button>
      </div>
    </div>

    <!-- 步骤4:创建 -->
    <div v-else-if="step === 3">
      <a-form layout="vertical">
        <a-form-item label="模板名称" required>
          <a-input v-model:value="form.name" placeholder="例如:GoFrame 脚手架" />
        </a-form-item>
        <a-form-item label="模板类型" required>
          <a-select v-model:value="form.templateType" placeholder="选择类型" style="width: 100%">
            <a-select-option v-for="t in templateTypes" :key="t.value ?? t" :value="t.value ?? t">
              {{ t.label ?? t.value ?? t }}
            </a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="分类" required>
          <a-select v-model:value="form.categoryId" placeholder="选择分类" style="width: 100%">
            <a-select-option v-for="cat in categories" :key="cat.id" :value="cat.id">
              {{ cat.name }}
            </a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="描述" required>
          <a-textarea v-model:value="form.description" :rows="2" placeholder="一句话描述该模板用途" />
        </a-form-item>
      </a-form>
      <a-alert
        type="info"
        show-icon
        :message="`将创建模板并上传 ${checkedFileKeys.length} 个文件(已应用 ${acceptedSuggestions.length} 个变量替换),完成后进入编辑器`"
      />
      <div class="wizard-actions">
        <a-button @click="step = 2">上一步</a-button>
        <a-button type="primary" :loading="creating" @click="handleCreate">创建并编辑</a-button>
      </div>
    </div>
  </a-modal>
</template>

<script setup>
import { ref, computed, watch, h } from 'vue'
import { message } from 'ant-design-vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpenOutlined } from '@ant-design/icons-vue'
import { createUserTemplate } from '@/api/editor/templates/contribution'
import { addTemplateFile, editTemplateFile } from '@/api/editor/templateFiles'

const props = defineProps({
  templateTypes: { type: Array, default: () => [] },
  categories: { type: Array, default: () => [] },
})

const show = defineModel('open', { type: Boolean, default: false })
const router = useRouter()

const step = ref(0)
const scanning = ref(false)
const dirPath = ref('')
const scanResult = ref(null)
const checkedKeys = ref([])
const analyzing = ref(false)
const suggestions = ref([])
const selectedRowKeys = ref([])
const creating = ref(false)
const form = ref({ name: '', templateType: undefined, categoryId: undefined, description: '' })

watch(show, (v) => {
  if (v) {
    step.value = 0
    scanResult.value = null
    dirPath.value = ''
    suggestions.value = []
    selectedRowKeys.value = []
  }
})

const formatSize = (n) => (n > 1024 * 1024 ? `${(n / 1024 / 1024).toFixed(1)} MB` : `${Math.round(n / 1024)} KB`)

const pickDir = async () => {
  const picked = await open({ directory: true, multiple: false })
  if (!picked) return
  dirPath.value = picked
  scanning.value = true
  try {
    const result = await invoke('extract_scan_dir', { path: picked })
    scanResult.value = JSON.parse(result)
    if (scanResult.value.files.length === 0) {
      message.warning('目录中没有符合白名单的候选文件')
    }
  } catch (e) {
    message.error('扫描失败: ' + (e.message || e))
  } finally {
    scanning.value = false
  }
}

// 路径清单 -> 树(目录为非叶子节点)
const treeData = computed(() => {
  const files = scanResult.value?.files || []
  const root = { title: dirPath.value.split(/[\\/]/).pop() || 'root', key: '__root__', children: [] }
  const dirNodes = new Map([['', root]])

  const ensureDir = (dirPath) => {
    if (dirNodes.has(dirPath)) return dirNodes.get(dirPath)
    const idx = dirPath.lastIndexOf('/')
    const parent = ensureDir(idx === -1 ? '' : dirPath.slice(0, idx))
    const node = { title: dirPath.slice(idx + 1), key: `d:${dirPath}`, children: [], selectable: false }
    parent.children.push(node)
    dirNodes.set(dirPath, node)
    return node
  }

  for (const f of files) {
    const idx = f.path.lastIndexOf('/')
    const parent = ensureDir(idx === -1 ? '' : f.path.slice(0, idx))
    parent.children.push({ title: f.path.slice(idx + 1), key: f.path, isLeaf: true })
  }
  return [root]
})

watch(scanResult, (v) => {
  checkedKeys.value = (v?.files || []).map((f) => f.path)
})

const checkedFileKeys = computed(() => checkedKeys.value.filter((k) => !String(k).startsWith('d:')))

const runAnalyze = async () => {
  analyzing.value = true
  step.value = 2
  try {
    const result = await invoke('extract_analyze', {
      path: dirPath.value,
      selectedFiles: checkedFileKeys.value,
    })
    const data = JSON.parse(result)
    suggestions.value = data.suggestions || []
    selectedRowKeys.value = suggestions.value.map((s) => s.value)
  } catch (e) {
    message.error('AI 分析失败: ' + (e.message || e))
  } finally {
    analyzing.value = false
  }
}

const suggestColumns = [
  { title: '原字面量', dataIndex: 'value', width: 180, ellipsis: true },
  {
    title: '变量名',
    dataIndex: 'varName',
    width: 170,
    customRender: ({ record }) =>
      h('a-input', {
        size: 'small',
        value: record.varName,
        'onUpdate:value': (v) => (record.varName = v),
      }),
  },
  { title: '标题', dataIndex: 'title', ellipsis: true },
  { title: '次数', dataIndex: 'count', width: 70 },
]

const onSelectionChange = (keys) => {
  selectedRowKeys.value = keys
}

const acceptedSuggestions = computed(() =>
  suggestions.value.filter((s) => selectedRowKeys.value.includes(s.value) && s.varName?.trim())
)

const handleCreate = async () => {
  const f = form.value
  if (!f.name.trim() || !f.templateType || !f.categoryId || !f.description.trim()) {
    message.warning('请填写名称、类型、分类与描述')
    return
  }

  creating.value = true
  try {
    // 1. 读取所选文件并应用变量替换
    const filesResult = await invoke('extract_read_files', {
      path: dirPath.value,
      selectedFiles: checkedFileKeys.value,
    })
    const files = JSON.parse(filesResult).files || []
    const replaced = files.map((file) => {
      let content = file.content
      for (const s of acceptedSuggestions.value) {
        content = content.split(s.value).join(`{{ ${s.varName.trim()} }}`)
      }
      return { path: file.path, content }
    })

    // 2. 创建模板
    const res = await createUserTemplate({
      name: f.name.trim(),
      templateType: f.templateType,
      categoryId: f.categoryId,
      description: f.description.trim(),
      visibility: 'private',
      languages: [],
    })
    const templateId = res?.data?.data?.id
    if (!templateId) throw new Error('创建模板失败:未返回模板 ID')

    // 3. 先建目录条目(按深度排序),再建文件并写入内容
    const dirSet = new Set()
    for (const file of replaced) {
      const parts = file.path.split('/')
      parts.pop()
      for (let i = 1; i <= parts.length; i++) dirSet.add(parts.slice(0, i).join('/'))
    }
    const dirs = [...dirSet].sort((a, b) => a.split('/').length - b.split('/').length)
    for (const dir of dirs) {
      const parentPath = dir.includes('/') ? dir.slice(0, dir.lastIndexOf('/')) : ''
      await addTemplateFile({
        templateId,
        fileName: dir.split('/').pop(),
        parentPath,
        isDirectory: true,
      })
    }
    for (const file of replaced) {
      const parentPath = file.path.includes('/') ? file.path.slice(0, file.path.lastIndexOf('/')) : ''
      await addTemplateFile({
        templateId,
        fileName: file.path.split('/').pop(),
        parentPath,
        isDirectory: false,
      })
      await editTemplateFile({ templateId, filePath: file.path, content: file.content })
    }

    message.success(`模板创建成功,已上传 ${replaced.length} 个文件`)
    show.value = false
    router.push(`/editor/${templateId}`)
  } catch (e) {
    message.error('创建失败: ' + (e.message || e))
  } finally {
    creating.value = false
  }
}
</script>

<style scoped>
.wizard-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}
</style>
