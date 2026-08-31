<template>
  <div class="my-templates-view">
    <!-- 顶部 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <h2 class="page-title">我的模板</h2>
        <span class="result-count">管理你创建的模板，编辑内容并提交发布</span>
      </div>
      <div class="toolbar-right">
        <a-button type="primary" @click="handleCreate">
          <template #icon><PlusOutlined /></template>
          创建模板
        </a-button>
      </div>
    </div>

    <!-- 状态筛选 -->
    <div class="filter-bar">
      <div
        v-for="f in filters"
        :key="f.value"
        class="filter-chip"
        :class="{ active: currentFilter === f.value }"
        @click="currentFilter = f.value"
      >{{ f.label }}</div>
    </div>

    <!-- 空状态 -->
    <div v-if="templates.length === 0 && !loading" class="empty-state">
      <FileTextOutlined class="empty-icon" />
      <p v-if="currentFilter === 'all'">还没有创建模板</p>
      <p v-else>没有{{ currentFilter === 'private' ? '草稿' : currentFilter === 'pending' ? '待审核' : '已发布' }}的模板</p>
      <span v-if="currentFilter === 'all'">点击右上角「创建模板」开始</span>
    </div>

    <!-- 模板列表 -->
    <div v-else class="templates-content">
      <a-spin :spinning="loading">
        <div class="template-grid">
          <div
            v-for="tmpl in templates"
            :key="tmpl.id"
            class="template-card"
            @click="goEditor(tmpl.id)"
            @contextmenu.prevent="(e) => showContextMenu(e, tmpl)"
          >
            <!-- 顶部视觉区域 -->
            <div class="card-visual-area">
              <div class="visual-bg"><div class="code-snippet">{{ getCodeSnippet(tmpl) }}</div></div>
              <div class="status-badge" :class="getStatusClass(tmpl.visibility)">
                {{ getStatusLabel(tmpl.visibility) }}
              </div>
              <div class="card-hover-actions">
                <a-tooltip title="编辑内容">
                  <button class="hover-action-btn" @click.stop="goEditor(tmpl.id)">
                    <EditOutlined />
                  </button>
                </a-tooltip>
                <a-tooltip title="修改信息">
                  <button class="hover-action-btn" @click.stop="handleEdit(tmpl)">
                    <ProfileOutlined />
                  </button>
                </a-tooltip>
                <a-tooltip title="Fork">
                  <button class="hover-action-btn" @click.stop="handleFork(tmpl)">
                    <ForkOutlined />
                  </button>
                </a-tooltip>
              </div>
            </div>
            <!-- 内容区域 -->
            <div class="card-content-area">
              <h4 class="card-title">{{ tmpl.name }}</h4>
              <p class="card-desc">{{ tmpl.description }}</p>
              <div class="card-footer">
                <span class="card-type">{{ getTypeLabel(tmpl.templateType) }}</span>
                <span class="card-time">{{ formatDate(tmpl.createdAt) }}</span>
              </div>
            </div>
          </div>
        </div>
      </a-spin>
    </div>

    <!-- 分页 -->
    <div v-if="total > pageSize" class="pagination">
      <a-pagination v-model:current="currentPage" :total="total" :page-size="pageSize" size="small" @change="loadTemplates" />
    </div>

    <!-- 右键菜单（悬浮按钮之外的补充操作） -->
    <a-dropdown v-model:open="showMenu" :trigger="['contextmenu']">
      <div :style="{ position: 'fixed', left: contextMenuX + 'px', top: contextMenuY + 'px', width: 0, height: 0 }"></div>
      <template #overlay>
        <a-menu @click="({ key }) => handleMenuSelect(key)">
          <a-menu-item key="edit-content">编辑内容</a-menu-item>
          <a-menu-item key="edit-info">修改信息</a-menu-item>
          <a-menu-item key="fork">Fork</a-menu-item>
          <a-menu-item v-if="menuTemplate?.visibility === 'private'" key="submit-review">提交审核</a-menu-item>
          <a-menu-item v-if="menuTemplate?.visibility === 'pending'" key="withdraw">撤回</a-menu-item>
          <a-menu-divider />
          <a-menu-item key="delete" style="color: #ef4444">删除</a-menu-item>
        </a-menu>
      </template>
    </a-dropdown>

    <!-- 创建/编辑弹窗 -->
    <a-modal v-model:open="showModal" :title="editingId ? '编辑模板' : '创建模板'" :mask-closable="false" :width="640" :footer="null">
      <a-form ref="formRef" :model="formData" :rules="formRules" layout="vertical" style="margin-top: 12px;">
        <a-form-item label="模板名称" name="name">
          <a-input v-model:value="formData.name" placeholder="请输入模板名称" :maxlength="100" show-count />
        </a-form-item>
        <a-form-item label="模板类型" name="templateType">
          <a-select v-model:value="formData.templateType" :options="typeOptions" placeholder="选择类型" :disabled="!!editingId" />
        </a-form-item>
        <a-form-item label="所属分类" name="categoryId">
          <a-select v-model:value="formData.categoryId" :options="categoryOptions" placeholder="选择分类" />
        </a-form-item>
        <a-form-item label="支持语言" name="languages">
          <a-select v-model:value="formData.languages" :options="languageOptions" mode="multiple" placeholder="选择语言" @change="onLanguagesChange" />
        </a-form-item>
        <a-form-item v-if="formData.languages.length" label="主语言">
          <a-select v-model:value="formData.primaryLanguage" :options="primaryLanguageOptions" placeholder="选择主语言" />
        </a-form-item>
        <a-form-item label="模板描述" name="description">
          <a-textarea v-model:value="formData.description" placeholder="描述模板的用途和特点" :maxlength="500" show-count :rows="3" />
        </a-form-item>
      </a-form>
      <div style="display: flex; gap: 12px; justify-content: flex-end; margin-top: 16px">
        <a-button @click="showModal = false">取消</a-button>
        <a-button type="primary" @click="handleSubmit" :loading="submitting">{{ editingId ? '更新' : '创建' }}</a-button>
      </div>
    </a-modal>

    <!-- Fork 模板弹窗 -->
    <a-modal v-model:open="showForkModal" title="Fork 模板" :mask-closable="false" :width="560" :footer="null">
      <a-form ref="forkFormRef" :model="forkFormData" :rules="forkFormRules" layout="vertical" style="margin-top: 12px;">
        <a-form-item label="源模板">
          <div class="fork-source">{{ forkingTemplate?.name }}</div>
        </a-form-item>
        <a-form-item label="新模板名称" name="name">
          <a-input v-model:value="forkFormData.name" placeholder="请输入新模板名称" />
        </a-form-item>
        <a-form-item label="新模板描述" name="description">
          <a-textarea v-model:value="forkFormData.description" :rows="3" placeholder="请输入新模板描述" />
        </a-form-item>
        <a-form-item label="详细介绍" name="introduction">
          <a-textarea v-model:value="forkFormData.introduction" :rows="4" placeholder="请输入详细介绍（可选）" />
        </a-form-item>
        <a-form-item label="分类" name="categoryId">
          <a-select v-model:value="forkFormData.categoryId" :options="categoryOptions" placeholder="选择分类（默认使用源模板分类）" allow-clear />
        </a-form-item>
      </a-form>
      <div style="display: flex; gap: 12px; justify-content: flex-end; margin-top: 16px">
        <a-button @click="showForkModal = false">取消</a-button>
        <a-button type="primary" @click="handleForkSubmit" :loading="forkSubmitting">确认 Fork</a-button>
      </div>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { message, Modal } from 'ant-design-vue'
import {
  PlusOutlined, EditOutlined, ProfileOutlined, ForkOutlined, FileTextOutlined,
} from '@ant-design/icons-vue'
import { listMyTemplates, createUserTemplate, updateUserTemplate, deleteUserTemplate, submitForReview } from '@/api/editor/templates/contribution'
import { forkTemplate } from '@/api/editor/templates'
import { getCategories, getLanguages } from '@/api/templates'

const router = useRouter()

const templates = ref([])
const loading = ref(false)
const currentPage = ref(1)
const pageSize = 12
const total = ref(0)
const currentFilter = ref('all')

const showModal = ref(false)
const editingId = ref(null)
const submitting = ref(false)
const formRef = ref()

const formData = ref({
  name: '',
  templateType: 'basic',
  categoryId: null,
  languages: [],
  primaryLanguage: null,
  description: '',
})

const formRules = {
  name: { required: true, message: '请输入模板名称', trigger: ['blur', 'input'] },
  templateType: { required: true, message: '请选择类型', trigger: ['change', 'blur'] },
  categoryId: { required: true, type: 'number', message: '请选择分类', trigger: ['change', 'blur'] },
  description: { required: true, message: '请输入描述', trigger: ['blur', 'input'] },
}

// Fork 弹窗相关
const showForkModal = ref(false)
const forkingTemplate = ref(null)
const forkFormRef = ref()
const forkSubmitting = ref(false)
const forkFormData = ref({
  name: '',
  description: '',
  introduction: '',
  categoryId: null,
})
const forkFormRules = {
  name: { required: true, message: '请输入新模板名称', trigger: ['blur', 'input'] },
  description: { required: true, message: '请输入新模板描述', trigger: ['blur', 'input'] },
}

// 右键菜单
const showMenu = ref(false)
const contextMenuX = ref(0)
const contextMenuY = ref(0)
const menuTemplate = ref(null)

function showContextMenu(e, tmpl) {
  menuTemplate.value = tmpl
  contextMenuX.value = e.clientX
  contextMenuY.value = e.clientY
  showMenu.value = true
}

function handleMenuSelect(key) {
  showMenu.value = false
  const tmpl = menuTemplate.value
  if (!tmpl) return
  switch (key) {
    case 'edit-content': goEditor(tmpl.id); break
    case 'edit-info': handleEdit(tmpl); break
    case 'fork': handleFork(tmpl); break
    case 'submit-review': handleSubmitReview(tmpl.id); break
    case 'withdraw': handleWithdraw(tmpl); break
    case 'delete': handleDelete(tmpl.id, tmpl.name); break
  }
}

const typeOptions = [
  { label: '基础模板', value: 'basic' },
  { label: '脚手架模板', value: 'scaffold' },
  { label: '数据驱动模板', value: 'data_driven' },
]

const filters = [
  { label: '全部', value: 'all' },
  { label: '草稿', value: 'private' },
  { label: '待审核', value: 'pending' },
  { label: '已发布', value: 'public' },
]

const categoryOptions = ref([])
const languageOptions = ref([])

const primaryLanguageOptions = computed(() =>
  languageOptions.value.filter((l) => formData.value.languages.includes(l.value))
)

function onLanguagesChange(val) {
  if (val.length && !val.includes(formData.value.primaryLanguage)) {
    formData.value.primaryLanguage = val[0]
  }
}

async function loadOptions() {
  try {
    const res = await getCategories({ all: 1 })
    const list = res?.data?.categoriesList || []
    categoryOptions.value = list.map((c) => ({ label: c.name, value: c.id }))
  } catch { /* 分类加载失败不阻塞页面 */ }
  try {
    const res = await getLanguages({ all: 1 })
    const list = res?.data?.languagesList || []
    languageOptions.value = list.map((l) => ({ label: l.name || l.displayName, value: l.id }))
  } catch { /* 语言加载失败不阻塞页面 */ }
}

async function loadTemplates() {
  loading.value = true
  try {
    const params = { page: currentPage.value, pageSize }
    if (currentFilter.value !== 'all') params.visibility = currentFilter.value
    const res = await listMyTemplates(params)
    // apiRequest 返回 axios response：res.data = 信封 {code,message,data}，
    // 业务数据在 res.data.data（templatesList/total）
    templates.value = res?.data?.data?.templatesList || []
    total.value = res?.data?.data?.total || 0
  } catch {
    message.error('加载模板失败')
  } finally {
    loading.value = false
  }
}

watch(currentFilter, () => { currentPage.value = 1; loadTemplates() })
onMounted(() => { loadTemplates(); loadOptions() })

function goEditor(id) {
  router.push(`/editor/${id}`)
}

function handleCreate() {
  editingId.value = null
  formData.value = { name: '', templateType: 'basic', categoryId: null, languages: [], primaryLanguage: null, description: '' }
  showModal.value = true
}

function handleEdit(tmpl) {
  editingId.value = tmpl.id
  formData.value = {
    name: tmpl.name,
    templateType: tmpl.templateType,
    categoryId: tmpl.categoryId,
    languages: (tmpl.languages || []).map((l) => l.languageId),
    primaryLanguage: (tmpl.languages || []).find((l) => l.isPrimary)?.languageId || null,
    description: tmpl.description || '',
  }
  showModal.value = true
}

async function handleSubmit() {
  try { await formRef.value?.validate() } catch { return }
  submitting.value = true
  try {
    const data = {
      name: formData.value.name,
      templateType: formData.value.templateType,
      categoryId: formData.value.categoryId,
      description: formData.value.description,
      visibility: 'private',
      languages: formData.value.languages.map((lid) => ({
        languageId: lid,
        isPrimary: lid === formData.value.primaryLanguage ? 1 : 0,
      })),
    }
    if (editingId.value) {
      await updateUserTemplate(editingId.value, data)
      message.success('模板更新成功')
    } else {
      await createUserTemplate(data)
      message.success('模板创建成功')
    }
    showModal.value = false
    loadTemplates()
  } catch {
    message.error(editingId.value ? '更新失败' : '创建失败')
  } finally {
    submitting.value = false
  }
}

function handleFork(tmpl) {
  forkingTemplate.value = tmpl
  forkFormData.value = {
    name: `${tmpl.name} - Fork`,
    description: tmpl.description || '',
    introduction: tmpl.introduction || '',
    categoryId: tmpl.categoryId || null,
  }
  showForkModal.value = true
}

async function handleForkSubmit() {
  try { await forkFormRef.value?.validate() } catch { return }
  forkSubmitting.value = true
  try {
    const res = await forkTemplate({
      sourceId: forkingTemplate.value.id,
      name: forkFormData.value.name,
      description: forkFormData.value.description,
      introduction: forkFormData.value.introduction,
      categoryId: forkFormData.value.categoryId,
    })
    // fork 响应 data 为新模板 id（裸值）
    const newId = res?.data?.data
    showForkModal.value = false
    if (newId) {
      message.success('Fork 成功，正在跳转到编辑器...')
      router.push(`/editor/${newId}`)
    } else {
      message.success('Fork 成功')
      loadTemplates()
    }
  } catch (error) {
    message.error('Fork 失败，请稍后重试')
    console.error('Fork error:', error)
  } finally {
    forkSubmitting.value = false
  }
}

function handleDelete(id, name) {
  Modal.confirm({
    title: '确认删除',
    content: `确定删除模板"${name}"吗？此操作不可撤销。`,
    okText: '删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      try {
        await deleteUserTemplate(id)
        message.success('删除成功')
        loadTemplates()
      } catch { message.error('删除失败') }
    },
  })
}

async function handleSubmitReview(id) {
  try {
    await submitForReview(id)
    message.success('已提交审核')
    loadTemplates()
  } catch { message.error('提交失败') }
}

async function handleWithdraw(tmpl) {
  // 撤回：基于菜单命中的模板构造载荷（而非弹窗里可能过期的表单数据）
  try {
    await updateUserTemplate(tmpl.id, {
      name: tmpl.name,
      templateType: tmpl.templateType,
      categoryId: tmpl.categoryId,
      description: tmpl.description || '',
      visibility: 'private',
      languages: (tmpl.languages || []).map((l) => ({
        languageId: l.languageId,
        isPrimary: l.isPrimary ? 1 : 0,
      })),
    })
    message.success('已撤回')
    loadTemplates()
  } catch { message.error('撤回失败') }
}

function getTypeLabel(t) {
  return { basic: '基础模板', scaffold: '脚手架', data_driven: '数据驱动' }[t] || t
}

function getCodeSnippet(tmpl) {
  const name = tmpl.name || 'Template'
  return `// ${name}\nclass ${name.replace(/\s+/g, '')} {\n  constructor() {\n    this.name = '${name}'\n  }\n\n  init() {\n    console.log('Initializing...')\n    this.run()\n  }\n\n  run() {\n    console.log('Running', this.name)\n  }\n}`
}

function getStatusLabel(v) {
  return { private: '草稿', pending: '待审核', public: '已发布', rejected: '已拒绝' }[v] || v
}
function getStatusClass(v) {
  return { private: 'draft', pending: 'pending', public: 'published', rejected: 'rejected' }[v] || ''
}
function formatDate(d) {
  if (!d) return '-'
  return new Date(d).toLocaleDateString('zh-CN')
}
</script>

<style scoped>
.my-templates-view { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--spacing-md); padding: var(--spacing-sm) var(--spacing-lg); flex-shrink: 0; }
.toolbar-left { display: flex; align-items: baseline; gap: var(--spacing-md); }
.page-title { margin: 0; font-size: 24px; font-weight: 600; color: var(--color-text); }
.result-count { color: var(--color-text-secondary); font-size: 14px; }
.toolbar-right { display: flex; align-items: center; gap: var(--spacing-md); }

.filter-bar { display: flex; gap: 8px; margin-bottom: var(--spacing-md); padding: 0 var(--spacing-lg); flex-shrink: 0; }
.filter-chip {
  padding: 6px 16px; border-radius: 20px; font-size: 13px; font-weight: 500;
  color: var(--color-text-secondary); background: var(--color-surface);
  border: 1px solid var(--color-border); cursor: pointer; transition: all 0.15s;
}
.filter-chip:hover { border-color: var(--color-primary); color: var(--color-primary); }
.filter-chip.active { background: var(--color-primary); color: #fff; border-color: var(--color-primary); }

.templates-content { flex: 1; overflow-y: auto; min-height: 0; padding: 0 var(--spacing-lg); }
.template-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: var(--spacing-md); }

.template-card {
  background: var(--color-background); border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg); overflow: hidden; cursor: pointer;
  transition: all 0.25s ease-out; position: relative;
}
.template-card:hover { transform: translateY(-4px); box-shadow: 0 12px 32px rgba(15, 23, 42, 0.12); border-color: var(--color-primary); }

.card-visual-area { position: relative; height: 110px; overflow: hidden; }
.visual-bg {
  height: 100%; background: linear-gradient(135deg, var(--color-surface), var(--color-hover));
  display: flex; align-items: center; justify-content: center;
}
.code-snippet {
  font-family: 'JetBrains Mono', 'Fira Code', Consolas, monospace; font-size: 9px; line-height: 1.5;
  color: var(--color-text-secondary); opacity: 0.5; white-space: pre; padding: 12px 16px;
  text-align: left; overflow: hidden;
}
.status-badge {
  position: absolute; top: 10px; right: 10px; padding: 2px 10px; border-radius: 12px;
  font-size: 11px; font-weight: 500; color: #fff;
}
.status-badge.draft { background: #94a3b8; }
.status-badge.pending { background: #f59e0b; }
.status-badge.published { background: #22c55e; }
.status-badge.rejected { background: #ef4444; }

.card-hover-actions {
  position: absolute; top: 10px; left: 10px; display: flex; gap: 6px;
  opacity: 0; transition: opacity 0.2s;
}
.template-card:hover .card-hover-actions { opacity: 1; }
.hover-action-btn {
  width: 26px; height: 26px; border-radius: 6px; border: none; cursor: pointer;
  background: rgba(15, 23, 42, 0.55); color: #fff; display: flex; align-items: center; justify-content: center;
}
.hover-action-btn:hover { background: var(--color-primary); }

.card-content-area { padding: 14px 16px 16px; }
.card-title { margin: 0 0 6px; font-size: 15px; font-weight: 600; color: var(--color-text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.card-desc {
  margin: 0 0 12px; font-size: 12px; color: var(--color-text-secondary);
  display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; min-height: 34px;
}
.card-footer { display: flex; justify-content: space-between; align-items: center; }
.card-type { font-size: 11px; color: var(--color-primary); background: var(--color-hover); padding: 1px 8px; border-radius: 10px; }
.card-time { font-size: 11px; color: var(--color-text-secondary); }

.pagination { display: flex; justify-content: center; padding: var(--spacing-md) 0; flex-shrink: 0; }

.empty-state {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  padding: 80px 20px; color: var(--color-text-secondary); flex: 1;
}
.empty-icon { font-size: 48px; opacity: 0.4; }
.empty-state p { font-size: 16px; font-weight: 500; color: var(--color-text); margin: 16px 0 4px; }
.empty-state span { font-size: 13px; }

.fork-source {
  padding: 8px 12px; background: var(--color-hover); border-radius: 6px;
  color: var(--color-text-secondary); width: 100%; border: 1px solid var(--color-border);
}
</style>
