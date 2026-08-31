<template>
  <div class="templates-view">
    <!-- 顶部工具栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <h2 class="page-title">模板渲染</h2>
        <span class="result-count">共 {{ filteredTemplates.length }} 个模板</span>
      </div>
      <div class="toolbar-right">
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
          <div v-for="template in paginatedTemplates" :key="template.id" class="template-card" @click="openDrawer(template)">
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
                  <span class="creation-time">{{ formatCreationTime(template.createdAt) }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </a-spin>
    </div>
    <!-- 模板渲染抽屉 -->
    <TemplateRenderDrawer v-model:open="drawerOpen" :template="selectedTemplate" @exported="onExported" />
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { SearchOutlined, UserOutlined } from '@ant-design/icons-vue'
import { getTemplates, getCategories, getLanguages } from '@/api/templates'
import { useLayoutStore } from '@/stores/layout'
import TemplateRenderDrawer from './TemplateRenderDrawer.vue'

const layoutStore = useLayoutStore()

const templates = ref([])
const loading = ref(false)
const searchKeyword = ref('')
const sortValue = ref('default')
const selectedCategory = ref('all')
const selectedLanguage = ref('all')
const drawerOpen = ref(false)
const selectedTemplate = ref(null)

const categories = ref([{ id: 'all', name: '全部' }])
const languages = ref([{ id: 'all', name: '全部' }])

const sortOptions = [
  { label: '默认排序', value: 'default' },
  { label: '最新创建', value: 'newest' },
  { label: '推荐优先', value: 'featured' },
]

const filteredTemplates = computed(() => {
  let result = templates.value
  if (searchKeyword.value) {
    const q = searchKeyword.value.toLowerCase()
    result = result.filter(t =>
      (t.name && t.name.toLowerCase().includes(q)) ||
      (t.description && t.description.toLowerCase().includes(q))
    )
  }
  if (selectedCategory.value !== 'all') {
    result = result.filter(t => t.categoryId === Number(selectedCategory.value))
  }
  if (selectedLanguage.value !== 'all') {
    result = result.filter(t => t.languages?.some(l => l.languageId === Number(selectedLanguage.value)))
  }
  if (sortValue.value === 'newest') {
    result = [...result].sort((a, b) => new Date(b.createdAt) - new Date(a.createdAt))
  } else if (sortValue.value === 'featured') {
    result = [...result].sort((a, b) => (b.isFeatured || 0) - (a.isFeatured || 0))
  }
  return result
})

const paginatedTemplates = computed(() => {
  const { current, pageSize: size } = layoutStore.footerPagination
  return filteredTemplates.value.slice((current - 1) * size, current * size)
})

watch(filteredTemplates, (newVal) => {
  layoutStore.showFooterPagination(newVal.length, layoutStore.footerPagination.current, layoutStore.footerPagination.pageSize)
})

const getLanguageName = (languageId) => languages.value.find(l => l.id === languageId)?.name || languageId

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
    const date = new Date(createdAt)
    const now = new Date()
    const diffTime = Math.abs(now - date)
    const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24))
    if (diffDays === 0) {
      const h = Math.floor(diffTime / (1000 * 60 * 60))
      if (h === 0) {
        const m = Math.floor(diffTime / (1000 * 60))
        return m <= 0 ? '刚刚' : `${m}分钟前`
      }
      return `${h}小时前`
    }
    if (diffDays === 1) return '昨天'
    if (diffDays < 7) return `${diffDays}天前`
    if (diffDays < 30) return `${Math.floor(diffDays / 7)}周前`
    return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
  } catch { return '' }
}

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

const openDrawer = (template) => {
  selectedTemplate.value = template
  drawerOpen.value = true
}

const onExported = () => {
  message.success('导出完成')
}

const loadCategories = async () => {
  try {
    const res = await getCategories({ all: 1 })
    categories.value = res?.data?.categoriesList
      ? [{ id: 'all', name: '全部' }, ...res.data.categoriesList]
      : [{ id: 'all', name: '全部' }]
  } catch { categories.value = [{ id: 'all', name: '全部' }] }
}

const loadLanguages = async () => {
  try {
    const res = await getLanguages({ all: 1 })
    languages.value = res?.data?.languagesList
      ? [{ id: 'all', name: '全部' }, ...res.data.languagesList]
      : [{ id: 'all', name: '全部' }]
  } catch { languages.value = [{ id: 'all', name: '全部' }] }
}

const loadTemplates = async () => {
  loading.value = true
  try {
    const res = await getTemplates()
    templates.value = res?.data?.templatesList || []
  } catch (e) {
    console.error('加载模板失败:', e)
    message.error('加载模板列表失败')
  } finally { loading.value = false }
}

onMounted(async () => {
  await Promise.all([loadCategories(), loadLanguages(), loadTemplates()])
  layoutStore.showFooterPagination(filteredTemplates.value.length, 1, 10)
})
</script>

<style scoped>
.templates-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--color-bg-container);
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.page-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text);
}

.result-count {
  font-size: 13px;
  color: var(--color-text-secondary);
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.filter-bar {
  flex-shrink: 0;
  padding: 12px 24px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
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

.templates-content {
  flex: 1;
  overflow: auto;
  padding: 20px 24px;
}

.templates-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 16px;
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
  border-color: var(--color-border-strong);
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
  background: linear-gradient(90deg, transparent 0%, rgba(28,29,31,0.04) 45%, rgba(28,29,31,0.06) 50%, rgba(28,29,31,0.04) 55%, transparent 100%);
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
  background: rgba(28, 29, 31, 0.92);
  backdrop-filter: blur(8px);
  padding: 3px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  color: #fff;
  z-index: 2;
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

.template-languages {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 14px;
}

.template-tag {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  color: var(--color-text-secondary);
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.template-card:hover .template-tag {
  background: rgba(24, 144, 255, 0.08);
  border-color: rgba(28, 29, 31, 0.14);
  color: var(--color-primary);
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
  border-radius: 6px;
  background: linear-gradient(135deg, #0f172a 0%, #334155 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: #fff;
  font-size: 11px;
  position: relative;
  overflow: hidden;
}

.author-avatar-img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 6px;
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

.card-footer-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.creation-time {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--color-text-muted);
}
</style>
