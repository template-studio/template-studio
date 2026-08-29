<template>
  <div class="my-templates-page">
    <div class="page-inner">
      <!-- 顶部 -->
      <div class="page-header">
        <div>
          <h1>我的模板</h1>
          <p>管理你创建的模板，编辑内容并提交发布</p>
        </div>
        <a-button type="primary" @click="handleCreate">
          <template #icon>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          </template>
          创建模板
        </a-button>
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
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="#cbd5e1" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/>
        </svg>
        <p v-if="currentFilter === 'all'">还没有创建模板</p>
        <p v-else>没有{{ currentFilter === 'private' ? '草稿' : currentFilter === 'pending' ? '待审核' : '已发布' }}的模板</p>
        <span v-if="currentFilter === 'all'">点击右上角"创建模板"开始</span>
      </div>

      <!-- 模板列表 -->
      <div v-else class="template-grid">
        <div
          v-for="tmpl in templates"
          :key="tmpl.id"
          class="template-card"
          @click="goEditor(tmpl.id)"
          @contextmenu.prevent="(e) => showContextMenu(e, tmpl)"
        >
          <!-- 顶部视觉区域 -->
          <div class="card-visual-area">
            <div class="visual-bg">
              <div class="code-snippet">{{ getCodeSnippet(tmpl) }}</div>
            </div>
            <div class="status-badge" :class="getStatusClass(tmpl.visibility)">
              {{ getStatusLabel(tmpl.visibility) }}
            </div>
            <div class="card-hover-actions">
              <button class="hover-action-btn" title="Fork" @click.stop="handleFork(tmpl)">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="6" y1="3" x2="6" y2="15"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/></svg>
              </button>
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

      <!-- 分页 -->
      <div v-if="total > pageSize" class="pagination">
        <a-pagination v-model:current="currentPage" :total="total" :page-size="pageSize" @change="loadTemplates" />
      </div>
    </div>

    <!-- 右键菜单 -->
    <a-dropdown v-model:open="showMenu" :trigger="['contextmenu']">
      <div :style="{ position: 'fixed', left: contextMenuX + 'px', top: contextMenuY + 'px', width: 0, height: 0 }"></div>
      <template #overlay>
        <a-menu @click="({ key }) => handleMenuSelect(key)">
          <template v-for="item in contextMenuOptions" :key="item.key">
            <a-menu-divider v-if="item.type === 'divider'" />
            <a-menu-item v-else :style="item.props?.style">
              {{ item.label }}
            </a-menu-item>
          </template>
        </a-menu>
      </template>
    </a-dropdown>

    <!-- 创建/编辑弹窗 -->
    <a-modal v-model:open="showModal" :title="editingId ? '编辑模板' : '创建模板'" :mask-closable="false" :width="700" :footer="null">
      <a-form ref="formRef" :model="formData" :rules="formRules" :label-col="{ span: 5 }" :wrapper-col="{ span: 19 }">
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

    <!-- Fork模板弹窗 -->
    <a-modal v-model:open="showForkModal" title="Fork 模板" :mask-closable="false" :width="600" :footer="null">
      <a-form ref="forkFormRef" :model="forkFormData" :rules="forkFormRules" :label-col="{ span: 6 }" :wrapper-col="{ span: 18 }">
        <a-form-item label="源模板">
          <div style="padding: 8px 12px; background: #f8fafc; border-radius: 6px; color: #64748b; width: 100%; border: 1px solid #e2e8f0;">
            {{ forkingTemplate?.name }}
          </div>
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
import { ref, computed, onMounted, watch, h } from 'vue';
import { useRouter } from 'vue-router';
import { message, Modal } from 'ant-design-vue';
import {
  CreateOutline, PencilOutline, TrashOutline,
  GitBranchOutline, SendOutline, ArrowUndoOutline,
} from '@/icons/ionicons5';
import { listMyTemplates, createUserTemplate, updateUserTemplate, deleteUserTemplate, submitForReview } from '@/api/templates/contribution';
import { forkTemplate } from '@/api/templates';
import { getPublicCategories, getPublicLanguages } from '@/api/public/index';

const renderIcon = (icon) => () => h(icon, { style: 'font-size: 16px' });

const router = useRouter();

const templates = ref([]);
const loading = ref(false);
const currentPage = ref(1);
const pageSize = 12;
const total = ref(0);
const currentFilter = ref('all');

const showModal = ref(false);
const editingId = ref(null);
const submitting = ref(false);
const formRef = ref();

const formData = ref({
  name: '',
  templateType: 'basic',
  categoryId: null,
  languages: [],
  primaryLanguage: null,
  description: '',
});

const formRules = {
  name: { required: true, message: '请输入模板名称', trigger: ['blur', 'input'] },
  templateType: { required: true, message: '请选择类型', trigger: ['change', 'blur'] },
  categoryId: { required: true, type: 'number', message: '请选择分类', trigger: ['change', 'blur'] },
  description: { required: true, message: '请输入描述', trigger: ['blur', 'input'] },
};

// Fork 弹窗相关
const showForkModal = ref(false);
const forkingTemplate = ref(null);
const forkFormRef = ref();
const forkSubmitting = ref(false);
const forkFormData = ref({
  name: '',
  description: '',
  introduction: '',
  categoryId: null,
});
const forkFormRules = {
  name: { required: true, message: '请输入新模板名称', trigger: ['blur', 'input'] },
  description: { required: true, message: '请输入新模板描述', trigger: ['blur', 'input'] },
};

// 右键菜单
const showMenu = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const menuTemplate = ref(null);

const contextMenuOptions = computed(() => {
  const tmpl = menuTemplate.value;
  if (!tmpl) return [];
  const items = [
    { label: '编辑内容', key: 'edit-content', icon: renderIcon(CreateOutline) },
    { label: '修改信息', key: 'edit-info', icon: renderIcon(PencilOutline) },
    { label: 'Fork', key: 'fork', icon: renderIcon(GitBranchOutline) },
  ];
  if (tmpl.visibility === 'private') {
    items.push({ label: '提交审核', key: 'submit-review', icon: renderIcon(SendOutline) });
  }
  if (tmpl.visibility === 'pending') {
    items.push({ label: '撤回', key: 'withdraw', icon: renderIcon(ArrowUndoOutline) });
  }
  items.push({ type: 'divider', key: 'd1' });
  items.push({ label: '删除', key: 'delete', icon: renderIcon(TrashOutline), props: { style: 'color: #ef4444' } });
  return items;
});

function showContextMenu(e, tmpl) {
  menuTemplate.value = tmpl;
  contextMenuX.value = e.clientX;
  contextMenuY.value = e.clientY;
  showMenu.value = true;
}

function handleMenuSelect(key) {
  showMenu.value = false;
  const tmpl = menuTemplate.value;
  if (!tmpl) return;
  switch (key) {
    case 'edit-content': goEditor(tmpl.id); break;
    case 'edit-info': handleEdit(tmpl); break;
    case 'fork': handleFork(tmpl); break;
    case 'submit-review': handleSubmitReview(tmpl.id); break;
    case 'withdraw': handleWithdraw(tmpl.id); break;
    case 'delete': handleDelete(tmpl.id, tmpl.name); break;
  }
}

const typeOptions = [
  { label: '基础模板', value: 'basic' },
  { label: '脚手架模板', value: 'scaffold' },
  { label: '数据驱动模板', value: 'data_driven' },
];

const filters = [
  { label: '全部', value: 'all' },
  { label: '草稿', value: 'private' },
  { label: '待审核', value: 'pending' },
  { label: '已发布', value: 'public' },
];

const categoryOptions = ref([]);
const languageOptions = ref([]);

const primaryLanguageOptions = computed(() =>
  languageOptions.value.filter((l) => formData.value.languages.includes(l.value))
);

function onLanguagesChange(val) {
  if (val.length && !val.includes(formData.value.primaryLanguage)) {
    formData.value.primaryLanguage = val[0];
  }
}

async function loadOptions() {
  try {
    const res = await getPublicCategories({ all: 1 });
    const list = res?.data?.data?.categoriesList || [];
    categoryOptions.value = list.map((c) => ({ label: c.name, value: c.id }));
  } catch {}
  try {
    const res = await getPublicLanguages({ all: 1 });
    const list = res?.data?.data?.languagesList || res?.data?.data || [];
    languageOptions.value = (Array.isArray(list) ? list : []).map((l) => ({ label: l.name || l.displayName, value: l.id }));
  } catch {}
}

async function loadTemplates() {
  loading.value = true;
  try {
    const params = { page: currentPage.value, pageSize };
    if (currentFilter.value !== 'all') params.visibility = currentFilter.value;
    const res = await listMyTemplates(params);
    templates.value = res?.templatesList || [];
    total.value = res?.total || 0;
  } catch {
    message.error('加载模板失败');
  } finally {
    loading.value = false;
  }
}

watch(currentFilter, () => { currentPage.value = 1; loadTemplates(); });
onMounted(() => { loadTemplates(); loadOptions(); });

function goEditor(id) {
  router.push(`/editor/${id}`);
}

function handleCreate() {
  editingId.value = null;
  formData.value = { name: '', templateType: 'basic', categoryId: null, languages: [], primaryLanguage: null, description: '' };
  showModal.value = true;
}

function handleEdit(tmpl) {
  editingId.value = tmpl.id;
  formData.value = {
    name: tmpl.name,
    templateType: tmpl.templateType,
    categoryId: tmpl.categoryId,
    languages: (tmpl.languages || []).map((l) => l.languageId),
    primaryLanguage: (tmpl.languages || []).find((l) => l.isPrimary)?.languageId || null,
    description: tmpl.description,
  };
  showModal.value = true;
}

async function handleSubmit() {
  try { await formRef.value?.validate(); } catch { return; }
  submitting.value = true;
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
    };
    if (editingId.value) {
      await updateUserTemplate(editingId.value, data);
      message.success('模板更新成功');
    } else {
      await createUserTemplate(data);
      message.success('模板创建成功');
    }
    showModal.value = false;
    loadTemplates();
  } catch { message.error(editingId.value ? '更新失败' : '创建失败'); }
  finally { submitting.value = false; }
}

function handleFork(tmpl) {
  forkingTemplate.value = tmpl;
  forkFormData.value = {
    name: `${tmpl.name} - Fork`,
    description: tmpl.description || '',
    introduction: tmpl.introduction || '',
    categoryId: tmpl.categoryId || null,
  };
  showForkModal.value = true;
}

async function handleForkSubmit() {
  try { await forkFormRef.value?.validate(); } catch { return; }
  forkSubmitting.value = true;
  try {
    const res = await forkTemplate({
      sourceId: forkingTemplate.value.id,
      name: forkFormData.value.name,
      description: forkFormData.value.description,
      introduction: forkFormData.value.introduction,
      categoryId: forkFormData.value.categoryId,
    });
    const data = res.data || res;
    if (data.code === 0 && data.data) {
      message.success('Fork 成功，正在跳转到编辑器...');
      showForkModal.value = false;
      router.push(`/editor/${data.data}`).then(() => {
        message.info('如文件树未加载，请刷新页面');
      });
    } else {
      message.error(data.message || 'Fork 失败');
    }
  } catch (error) {
    message.error('Fork 失败，请稍后重试');
    console.error('Fork error:', error);
  } finally {
    forkSubmitting.value = false;
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
        await deleteUserTemplate(id);
        message.success('删除成功');
        loadTemplates();
      } catch { message.error('删除失败'); }
    },
  });
}

async function handleSubmitReview(id) {
  try {
    await submitForReview(id);
    message.success('已提交审核');
    loadTemplates();
  } catch { message.error('提交失败'); }
}

async function handleWithdraw(id) {
  try {
    await updateUserTemplate(id, { ...formData.value, id, visibility: 'private' });
    message.success('已撤回');
    loadTemplates();
  } catch { message.error('撤回失败'); }
}

function getTypeLabel(t) {
  return { basic: '基础模板', scaffold: '脚手架', data_driven: '数据驱动' }[t] || t;
}

function getCodeSnippet(tmpl) {
  const name = tmpl.name || 'Template';
  return `// ${name}\nclass ${name.replace(/\s+/g, '')} {\n  constructor() {\n    this.name = '${name}'\n  }\n\n  init() {\n    console.log('Initializing...')\n    this.run()\n  }\n\n  run() {\n    console.log('Running', this.name)\n  }\n}`;
}

function getStatusLabel(v) {
  return { private: '草稿', pending: '待审核', public: '已发布', rejected: '已拒绝' }[v] || v;
}
function getStatusClass(v) {
  return { private: 'draft', pending: 'pending', public: 'published', rejected: 'rejected' }[v] || '';
}
function formatDate(d) {
  if (!d) return '-';
  return new Date(d).toLocaleDateString('zh-CN');
}
</script>

<style scoped>
.my-templates-page {
  min-height: calc(100vh - 64px);
  background: #f1f5f9;
}
.page-inner {
  max-width: 1200px;
  margin: 0 auto;
  padding: 32px 40px 64px;
}
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}
.page-header h1 { font-size: 22px; font-weight: 700; color: #0f172a; margin: 0 0 4px; }
.page-header p { font-size: 14px; color: #94a3b8; margin: 0; }

.filter-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
}
.filter-chip {
  padding: 6px 16px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
  color: #64748b;
  background: #fff;
  border: 1px solid #e2e8f0;
  cursor: pointer;
  transition: all 0.15s;
}
.filter-chip:hover { border-color: #cbd5e1; color: #334155; }
.filter-chip.active {
  background: var(--client-theme-color, #22c55e);
  color: #fff;
  border-color: var(--client-theme-color, #22c55e);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  color: #94a3b8;
}
.empty-state p { font-size: 16px; font-weight: 500; color: #64748b; margin: 16px 0 4px; }
.empty-state span { font-size: 13px; }

/* ===== 模板卡片（对齐模板市场风格）===== */
.template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.template-card {
  background: #fff;
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid #e2e8f0;
  transition: all 0.25s ease-out;
  cursor: pointer;
  position: relative;
}

.template-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(15, 23, 42, 0.12);
  border-color: var(--client-theme-color, #22c55e);
}

/* 视觉区域 */
.card-visual-area {
  width: 100%;
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
    rgba(34, 197, 94, 0.03) 45%,
    rgba(34, 197, 94, 0.08) 50%,
    rgba(34, 197, 94, 0.03) 55%,
    transparent 100%
  );
  animation: shimmer 4s ease-in-out infinite;
}

@keyframes shimmer {
  0% { transform: translateX(0); }
  100% { transform: translateX(50%); }
}

.code-snippet {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 10px;
  line-height: 1.5;
  color: rgba(148, 163, 184, 0.4);
  white-space: pre;
  overflow: hidden;
  padding: 16px 20px;
  text-align: left;
  position: relative;
  z-index: 1;
}

/* 状态角标 */
.status-badge {
  position: absolute;
  top: 10px;
  left: 10px;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 10px;
  border-radius: 6px;
  z-index: 2;
  letter-spacing: 0.3px;
  backdrop-filter: blur(8px);
}
.status-badge.draft { background: rgba(241, 245, 249, 0.9); color: #64748b; }
.status-badge.pending { background: rgba(255, 251, 235, 0.9); color: #d97706; }
.status-badge.published { background: rgba(240, 253, 244, 0.9); color: #16a34a; }
.status-badge.rejected { background: rgba(254, 242, 242, 0.9); color: #dc2626; }

/* hover 操作 */
.card-hover-actions {
  position: absolute;
  top: 10px;
  right: 10px;
  z-index: 3;
  display: flex;
  gap: 6px;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.template-card:hover .card-hover-actions {
  opacity: 1;
}

.hover-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(8px);
  color: #fff;
  cursor: pointer;
  transition: all 0.15s ease;
}

.hover-action-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

/* 内容区域 */
.card-content-area {
  padding: 16px 20px 20px;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  color: #0f172a;
  margin: 0 0 6px 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  letter-spacing: -0.2px;
}

.template-card:hover .card-title {
  color: var(--client-theme-color, #22c55e);
}

.card-desc {
  font-size: 13px;
  color: #64748b;
  margin: 0 0 14px 0;
  line-height: 1.6;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 12px;
  border-top: 1px solid #f1f5f9;
  font-size: 12px;
  color: #94a3b8;
}

.card-type {
  padding: 2px 8px;
  background: #f1f5f9;
  border-radius: 4px;
  font-weight: 500;
}

.pagination {
  display: flex;
  justify-content: center;
  margin-top: 32px;
}

@media (max-width: 768px) {
  .page-inner { padding: 20px; }
  .template-grid { grid-template-columns: 1fr; }
}

@media (prefers-reduced-motion: reduce) {
  .visual-bg::before { animation: none; }
  .template-card { transition: none; }
}
</style>
