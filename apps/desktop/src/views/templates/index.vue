<template>
  <div class="templates-view">
    <!-- 顶部工具栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <h2 class="page-title">模板库</h2>
        <span class="result-count">共 {{ filteredTemplates.length }} 个模板</span>
      </div>
      <div class="toolbar-right">
        <a-button v-if="configStore.hasApiKey" type="primary" @click="openCreateModal">
          <template #icon><PlusOutlined /></template>
          新建模板
        </a-button>
        <a-input v-model:value="searchKeyword" placeholder="搜索模板..." style="width: 200px;" allow-clear>
          <template #prefix><SearchOutlined /></template>
        </a-input>
        <a-select v-model:value="sortValue" style="width: 140px;" size="default">
          <a-select-option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</a-select-option>
        </a-select>
      </div>
    </div>
    <!-- 筛选栏 -->
    <div class="filter-bar">
      <div class="filter-row">
        <span class="filter-label">分类</span>
        <a-radio-group v-model:value="selectedCategory" button-style="solid" size="small">
          <a-radio-button v-for="cat in categories" :key="cat.id" :value="cat.id">{{ cat.name }}</a-radio-button>
        </a-radio-group>
      </div>
      <div class="filter-row">
        <span class="filter-label">语言</span>
        <a-radio-group v-model:value="selectedLanguage" button-style="solid" size="small">
          <a-radio-button v-for="lang in languages" :key="lang.id" :value="lang.id">{{ lang.name }}</a-radio-button>
        </a-radio-group>
      </div>
    </div>
    <!-- 模板列表 -->
    <div class="templates-content">
      <a-spin :spinning="loading">
        <div class="templates-grid">
          <div v-for="template in paginatedTemplates" :key="template.id" class="template-card" :class="{ selected: selectedTemplate?.id === template.id }" @click="selectTemplate(template)">
            <div class="card-visual">
              <div class="visual-bg"><div class="code-preview">{{ getCodeSnippet(template) }}</div></div>
              <div v-if="template.isFeatured === 1" class="template-badge"><span>推荐</span></div>
            </div>
            <div class="card-content">
              <h3 class="template-name">{{ template.name }}</h3>
              <p class="template-desc">{{ template.description }}</p>
              <div class="template-languages">
                <span v-for="lang in template.languages" :key="lang.languageId" class="template-tag">{{ getLanguageName(lang.languageId) }}</span>
              </div>
              <div class="card-footer">
                <div class="card-author">
                  <div class="author-avatar">
                    <UserOutlined class="author-avatar-fallback" />
                    <img v-if="getOwnerAvatarUrl(template)" :src="getOwnerAvatarUrl(template)" alt="" class="author-avatar-img" @error="$event.target.style.display='none'" />
                  </div>
                  <span class="author-name">{{ template.ownerName || 'Template Studio' }}</span>
                </div>
                <div class="card-footer-right">
                  <a-tooltip v-if="configStore.hasApiKey" title="编辑模板">
                    <a-button type="text" size="small" class="edit-entry" @click.stop="goEdit(template)">
                      <template #icon><EditOutlined /></template>
                    </a-button>
                  </a-tooltip>
                  <span class="creation-time">{{ formatCreationTime(template.createdAt) }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </a-spin>
    </div>
    <!-- 新建模板弹窗 -->
    <a-modal v-model:open="showCreateModal" title="新建模板" :confirm-loading="creating" @ok="handleCreate" ok-text="创建并编辑" cancel-text="取消">
      <a-form layout="vertical" style="margin-top: 12px;">
        <a-form-item label="模板名称" required>
          <a-input v-model:value="createForm.name" placeholder="例如：GoFrame 脚手架" />
        </a-form-item>
        <a-form-item label="模板类型" required>
          <a-select v-model:value="createForm.templateType" placeholder="选择类型" style="width: 100%;">
            <a-select-option v-for="t in templateTypes" :key="t.value ?? t" :value="t.value ?? t">{{ t.label ?? t.value ?? t }}</a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="分类" required>
          <a-select v-model:value="createForm.categoryId" placeholder="选择分类" style="width: 100%;">
            <a-select-option v-for="cat in selectableCategories" :key="cat.id" :value="cat.id">{{ cat.name }}</a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="主语言">
          <a-select v-model:value="createForm.primaryLanguage" placeholder="选择主语言（可选）" style="width: 100%;" allow-clear>
            <a-select-option v-for="lang in selectableLanguages" :key="lang.id" :value="lang.id">{{ lang.name }}</a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="模板描述" required>
          <a-textarea v-model:value="createForm.description" placeholder="一句话描述模板用途" :rows="3" />
        </a-form-item>
      </a-form>
    </a-modal>
    <!-- 模板配置向导抽屉 -->
    <TemplateWizardDrawer v-model:open="showWizardModal" :template="selectedTemplate" @created="onProjectCreated" />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { message } from 'ant-design-vue'
import { useLayoutStore } from '@/stores/layout'
import { useConfigStore } from '@/stores/config'
import { SearchOutlined, UserOutlined, PlusOutlined, EditOutlined } from '@ant-design/icons-vue'
import { getCategories, getLanguages, getTemplates } from '@/api/templates'
import { getTemplateTypes } from '@/api/editor/templates'
import { createUserTemplate } from '@/api/editor/templates/contribution'
import TemplateWizardDrawer from './components/TemplateWizardDrawer.vue'

const layoutStore = useLayoutStore()
const configStore = useConfigStore()
const router = useRouter()

const loading = ref(false)
const searchKeyword = ref('')
const selectedCategory = ref('all')
const selectedLanguage = ref('all')
const selectedTemplate = ref(null)
const sortValue = ref('newest')
const sortOptions = [
  { label: '最新创建', value: 'newest' }, { label: '最早创建', value: 'oldest' },
  { label: '名称 A-Z', value: 'name_asc' }, { label: '名称 Z-A', value: 'name_desc' },
  { label: '推荐优先', value: 'featured' }
]
const showWizardModal = ref(false)
const categories = ref([{ id: 'all', name: '全部' }])
const languages = ref([{ id: 'all', name: '全部' }])
const templates = ref([])

const filteredTemplates = computed(() => {
  let result = templates.value
  if (searchKeyword.value.trim()) {
    const keyword = searchKeyword.value.toLowerCase()
    result = result.filter(t => (t.name?.toLowerCase() || '').includes(keyword) || (t.description?.toLowerCase() || '').includes(keyword))
  }
  if (selectedCategory.value !== 'all') result = result.filter(t => t.categoryId === Number(selectedCategory.value))
  if (selectedLanguage.value !== 'all') result = result.filter(t => t.languages?.some(l => l.languageId === Number(selectedLanguage.value)))
  result = [...result].sort((a, b) => {
    switch (sortValue.value) {
      case 'newest': return new Date(b.createdAt || 0).getTime() - new Date(a.createdAt || 0).getTime()
      case 'oldest': return new Date(a.createdAt || 0).getTime() - new Date(b.createdAt || 0).getTime()
      case 'name_asc': return (a.name || '').localeCompare(b.name || '')
      case 'name_desc': return (b.name || '').localeCompare(a.name || '')
      case 'featured': return (b.isFeatured || 0) - (a.isFeatured || 0)
      default: return 0
    }
  })
  return result
})

const paginatedTemplates = computed(() => {
  const { current, pageSize: size } = layoutStore.footerPagination
  return filteredTemplates.value.slice((current - 1) * size, current * size)
})

const selectTemplate = (template) => { useTemplate(template) }

const useTemplate = (template) => {
  selectedTemplate.value = template
  showWizardModal.value = true
}

const onProjectCreated = () => { showWizardModal.value = false }

const getCodeSnippet = (template) => {
  const primaryLanguage = template.languages?.find(l => l.isPrimary === 1)
  const lang = languages.value.find(l => l.id === primaryLanguage?.languageId)?.name || ''
  const name = template.name || 'Template'
  if (lang.includes('Rust') || lang.includes('rust')) return `fn main() {\n    println!("Hello, ${name}!");\n}`
  if (lang.includes('Go') || lang.includes('go') || lang.includes('Golang')) return `package main\n\nimport "fmt"\n\nfunc main() {\n    fmt.Printf("Hello, ${name}!\\n")\n}`
  if (lang.includes('Python') || lang.includes('python')) return `def main():\n    print(f"Hello, {name}!")\n\nif __name__ == "__main__":\n    main()`
  if (lang.includes('JavaScript') || lang.includes('javascript')) return `function main() {\n  console.log('Hello, ${name}!');\n}\n\nmain();`
  if (lang.includes('TypeScript') || lang.includes('typescript')) return `function main(): void {\n  console.log('Hello, ${name}!');\n}\n\nmain();`
  return `// ${name}\nclass App {\n  constructor() {\n    this.name = '${name}';\n  }\n\n  run() {\n    console.log('Running', this.name);\n  }\n}`
}

const getLanguageName = (languageId) => languages.value.find(l => l.id === languageId)?.name || languageId
const getCategoryName = (categoryId) => categories.value.find(c => c.id === categoryId)?.name || categoryId

const getOwnerAvatarUrl = (template) => {
  const avatar = template.ownerAvatar
  if (!avatar) return ''
  if (avatar.startsWith('http')) return avatar
  const base = (import.meta.env.VITE_API_URL || 'http://127.0.0.1:8080').replace(/\/+$/, '')
  return `${base}${avatar.startsWith('/') ? '' : '/'}${avatar}`
}

const formatCreationTime = (createdAt) => {
  if (!createdAt) return ''
  try {
    const date = new Date(createdAt); const now = new Date()
    const diffTime = Math.abs(now - date); const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24))
    if (diffDays === 0) { const h = Math.floor(diffTime / (1000 * 60 * 60)); if (h === 0) { const m = Math.floor(diffTime / (1000 * 60)); return m <= 0 ? '刚刚' : `${m}分钟前` }; return `${h}小时前` }
    if (diffDays === 1) return '昨天'
    if (diffDays < 7) return `${diffDays}天前`
    if (diffDays < 30) return `${Math.floor(diffDays / 7)}周前`
    if (diffDays < 365) return `${Math.floor(diffDays / 30)}个月前`
    return `${Math.floor(diffDays / 365)}年前`
  } catch { return '' }
}

const loadCategories = async () => {
  try { const res = await getCategories({ all: 1 }); categories.value = res?.data?.categoriesList ? [{ id: 'all', name: '全部' }, ...res.data.categoriesList] : [{ id: 'all', name: '全部' }] } catch { categories.value = [{ id: 'all', name: '全部' }] }
}
const loadLanguages = async () => {
  try { const res = await getLanguages({ all: 1 }); languages.value = res?.data?.languagesList ? [{ id: 'all', name: '全部' }, ...res.data.languagesList] : [{ id: 'all', name: '全部' }] } catch { languages.value = [{ id: 'all', name: '全部' }] }
}
const loadTemplates = async () => {
  try {
    loading.value = true
    const params = {}
    if (searchKeyword.value.trim()) { params.name = searchKeyword.value.trim(); params.description = searchKeyword.value.trim() }
    if (selectedCategory.value !== 'all') params.categoryId = Number(selectedCategory.value)
    if (selectedLanguage.value !== 'all') params.languageId = Number(selectedLanguage.value)
    const res = await getTemplates(params)
    templates.value = res?.data?.templatesList || []
  } catch (error) { console.error('加载模板失败:', error); message.error('加载模板失败') } finally { loading.value = false }
}

watch(filteredTemplates, (newVal) => { layoutStore.showFooterPagination(newVal.length, layoutStore.footerPagination.current, layoutStore.footerPagination.pageSize) })

// ---------------------------------------------------------------------------
// 模板编辑入口（方案A：需在设置页配置 API Token 后可用）
// ---------------------------------------------------------------------------
const templateTypes = ref([])
const showCreateModal = ref(false)
const creating = ref(false)
const createForm = ref({ name: '', templateType: undefined, categoryId: undefined, primaryLanguage: undefined, description: '' })

const selectableCategories = computed(() => categories.value.filter(c => c.id !== 'all'))
const selectableLanguages = computed(() => languages.value.filter(l => l.id !== 'all'))

const loadTemplateTypes = async () => {
  try {
    const res = await getTemplateTypes()
    templateTypes.value = res?.data?.templateTypes || res?.data?.template_types || []
  } catch { templateTypes.value = [] }
}

const goEdit = (template) => {
  router.push(`/editor/${template.id}`)
}

const openCreateModal = () => {
  createForm.value = { name: '', templateType: undefined, categoryId: undefined, primaryLanguage: undefined, description: '' }
  showCreateModal.value = true
}

const handleCreate = async () => {
  const form = createForm.value
  if (!form.name.trim() || !form.templateType || !form.categoryId || !form.description.trim()) {
    message.warning('请填写名称、类型、分类与描述')
    return
  }
  creating.value = true
  try {
    const languagesPayload = form.primaryLanguage
      ? [{ languageId: form.primaryLanguage, isPrimary: 1 }]
      : []
    const res = await createUserTemplate({
      name: form.name.trim(),
      templateType: form.templateType,
      categoryId: form.categoryId,
      description: form.description.trim(),
      visibility: 'private',
      languages: languagesPayload,
    })
    const newId = res?.data?.data?.id
    showCreateModal.value = false
    if (newId) {
      message.success('模板创建成功，正在打开编辑器...')
      router.push(`/editor/${newId}`)
    } else {
      message.success('模板创建成功')
      loadTemplates()
    }
  } catch (error) {
    console.error('创建模板失败:', error)
  } finally {
    creating.value = false
  }
}

onMounted(async () => {
  await loadCategories()
  await loadLanguages()
  await loadTemplates()
  if (configStore.hasApiKey) loadTemplateTypes()
  layoutStore.showFooterPagination(filteredTemplates.value.length, 1, 10)
})
</script>

<style scoped>
.templates-view { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--spacing-md); padding: var(--spacing-sm) var(--spacing-lg); flex-shrink: 0; }
.templates-content { flex: 1; overflow-y: auto; min-height: 0; padding: 0 var(--spacing-lg); }
.toolbar-left { display: flex; align-items: baseline; gap: var(--spacing-md); }
.page-title { margin: 0; font-size: 24px; font-weight: 600; color: var(--color-text); }
.result-count { color: var(--color-text-secondary); font-size: 14px; }
.toolbar-right { display: flex; align-items: center; gap: var(--spacing-md); }
.filter-bar { margin-bottom: var(--spacing-lg); padding: var(--spacing-md); background: var(--color-surface); border-radius: var(--border-radius-lg); border: 1px solid var(--color-border); }
.filter-row { display: flex; align-items: center; gap: var(--spacing-md); padding: var(--spacing-xs) 0; }
.filter-row:not(:last-child) { margin-bottom: var(--spacing-md); border-bottom: 1px solid var(--color-border); padding-bottom: var(--spacing-md); }
.filter-label { font-size: 13px; font-weight: 500; color: var(--color-text-secondary); white-space: nowrap; min-width: 50px; }
.templates-grid { display: grid; grid-template-columns: repeat(5, 1fr); gap: var(--spacing-md); }
.edit-entry { color: var(--color-text-secondary); }
.edit-entry:hover { color: var(--color-primary) !important; }
.template-card { background: var(--color-background); border: 1px solid var(--color-border); border-radius: var(--border-radius-lg); overflow: hidden; cursor: pointer; transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); }
.template-card:hover { transform: translateY(-4px); box-shadow: 0 12px 32px rgba(15, 23, 42, 0.12); border-color: var(--color-primary); }
.template-card.selected { border-color: var(--color-primary); box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.1); }
.card-visual { height: 140px; position: relative; overflow: hidden; }
.visual-bg { width: 100%; height: 100%; background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%); display: flex; align-items: center; justify-content: center; position: relative; overflow: hidden; }
.visual-bg::before { content: ''; position: absolute; top: 0; left: -100%; width: 200%; height: 100%; background: linear-gradient(90deg, transparent 0%, rgba(24,144,255,0.03) 45%, rgba(24,144,255,0.08) 50%, rgba(24,144,255,0.03) 55%, transparent 100%); animation: shimmer 4s ease-in-out infinite; }
@keyframes shimmer { 0% { transform: translateX(0); } 100% { transform: translateX(50%); } }
.code-preview { font-family: 'JetBrains Mono','Fira Code','Consolas',monospace; font-size: 10px; line-height: 1.5; color: rgba(148, 163, 184, 0.4); white-space: pre; padding: 16px 20px; text-align: left; position: relative; z-index: 1; overflow: hidden; }
.template-badge { position: absolute; top: 10px; right: 10px; background: rgba(24,144,255,0.9); backdrop-filter: blur(8px); padding: 3px 10px; border-radius: 6px; font-size: 11px; font-weight: 600; color: #fff; z-index: 2; letter-spacing: 0.3px; }
.card-content { padding: 16px 20px 20px; }
.template-name { margin: 0 0 6px 0; font-size: 16px; font-weight: 600; color: var(--color-text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; letter-spacing: -0.2px; }
.template-card:hover .template-name { color: var(--color-primary); }
.template-desc { margin: 0 0 12px 0; font-size: 13px; color: var(--color-text-secondary); line-height: 1.6; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.template-languages { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 14px; }
.template-tag { background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text-secondary); font-size: 11px; padding: 2px 8px; border-radius: 4px; transition: all 0.2s ease; }
.template-card:hover .template-tag { background: rgba(24,144,255,0.08); border-color: rgba(24,144,255,0.2); color: var(--color-primary); }
.card-footer { display: flex; justify-content: space-between; align-items: center; padding-top: 12px; border-top: 1px solid var(--color-border-light); }
.card-author { display: flex; align-items: center; gap: 8px; }
.author-avatar { width: 24px; height: 24px; border-radius: 6px; background: linear-gradient(135deg, #0f172a 0%, #334155 100%); display: flex; align-items: center; justify-content: center; flex-shrink: 0; color: #fff; font-size: 12px; position: relative; overflow: hidden; }
.author-avatar-fallback { font-size: 12px; color: #fff; }
.author-avatar-img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; border-radius: 6px; }
.author-name { font-size: 12px; color: var(--color-text-secondary); font-weight: 500; }
.card-footer-right { display: flex; align-items: center; gap: 8px; }
.creation-time { display: flex; align-items: center; gap: 4px; font-size: 12px; color: var(--color-text-muted); }
</style>
