<template>
  <div class="my-templates-page">
    <div class="page-inner">
      <!-- 顶部 -->
      <div class="page-header">
        <div>
          <h1>我的模板</h1>
          <p>管理你创建的模板，编辑内容并提交发布</p>
        </div>
        <n-button type="primary" @click="handleCreate">
          <template #icon>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          </template>
          创建模板
        </n-button>
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
        <div v-for="tmpl in templates" :key="tmpl.id" class="template-card">
          <div class="card-header">
            <div class="card-title">{{ tmpl.name }}</div>
            <span class="status-badge" :class="getStatusClass(tmpl.visibility)">
              {{ getStatusLabel(tmpl.visibility) }}
            </span>
          </div>
          <p class="card-desc">{{ tmpl.description }}</p>
          <div class="card-meta">
            <span>{{ tmpl.templateType === 'basic' ? '基础模板' : tmpl.templateType === 'scaffold' ? '脚手架' : '数据驱动' }}</span>
            <span>{{ formatDate(tmpl.createdAt) }}</span>
          </div>
          <div class="card-actions">
            <n-button size="small" type="primary" quaternary @click="goEditor(tmpl.id)">编辑内容</n-button>
            <n-button size="small" quaternary @click="handleFork(tmpl)">Fork</n-button>
            <n-button size="small" quaternary @click="handleEdit(tmpl)">修改信息</n-button>
            <n-button v-if="tmpl.visibility === 'private'" size="small" type="warning" quaternary @click="handleSubmitReview(tmpl.id)">提交审核</n-button>
            <n-button v-if="tmpl.visibility === 'pending'" size="small" quaternary @click="handleWithdraw(tmpl.id)">撤回</n-button>
            <n-button size="small" type="error" quaternary @click="handleDelete(tmpl.id, tmpl.name)">删除</n-button>
          </div>
        </div>
      </div>

      <!-- 分页 -->
      <div v-if="total > pageSize" class="pagination">
        <n-pagination v-model:page="currentPage" :page-count="Math.ceil(total / pageSize)" @update:page="loadTemplates" />
      </div>
    </div>

    <!-- 创建/编辑弹窗 -->
    <n-modal v-model:show="showModal" :mask-closable="false">
      <n-card style="width: 700px" :title="editingId ? '编辑模板' : '创建模板'" :bordered="false" size="huge" role="dialog">
        <template #header-extra>
          <n-button quaternary circle @click="showModal = false">
            <template #icon><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></template>
          </n-button>
        </template>
        <n-form ref="formRef" :model="formData" :rules="formRules" label-placement="left" :label-width="100">
          <n-form-item label="模板名称" path="name">
            <n-input v-model:value="formData.name" placeholder="请输入模板名称" :maxlength="100" show-count />
          </n-form-item>
          <n-form-item label="模板类型" path="templateType">
            <n-select v-model:value="formData.templateType" :options="typeOptions" placeholder="选择类型" :disabled="!!editingId" />
          </n-form-item>
          <n-form-item label="所属分类" path="categoryId">
            <n-select v-model:value="formData.categoryId" :options="categoryOptions" placeholder="选择分类" />
          </n-form-item>
          <n-form-item label="支持语言" path="languages">
            <n-select v-model:value="formData.languages" :options="languageOptions" multiple placeholder="选择语言" @update:value="onLanguagesChange" />
          </n-form-item>
          <n-form-item v-if="formData.languages.length" label="主语言">
            <n-select v-model:value="formData.primaryLanguage" :options="primaryLanguageOptions" placeholder="选择主语言" />
          </n-form-item>
          <n-form-item label="模板描述" path="description">
            <n-input v-model:value="formData.description" type="textarea" placeholder="描述模板的用途和特点" :maxlength="500" show-count :rows="3" />
          </n-form-item>
        </n-form>
        <template #footer>
          <div style="display: flex; gap: 12px; justify-content: flex-end">
            <n-button @click="showModal = false">取消</n-button>
            <n-button type="primary" @click="handleSubmit" :loading="submitting">{{ editingId ? '更新' : '创建' }}</n-button>
          </div>
        </template>
      </n-card>
    </n-modal>

    <!-- Fork模板弹窗 -->
    <n-modal v-model:show="showForkModal" :mask-closable="false">
      <n-card style="width: 600px" title="Fork 模板" :bordered="false" size="huge" role="dialog">
        <template #header-extra>
          <n-button quaternary circle @click="showForkModal = false">
            <template #icon><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></template>
          </n-button>
        </template>
        <n-form ref="forkFormRef" :model="forkFormData" :rules="forkFormRules" label-placement="left" :label-width="100">
          <n-form-item label="源模板">
            <div style="padding: 8px 12px; background: #f8fafc; border-radius: 6px; color: #64748b; width: 100%; border: 1px solid #e2e8f0;">
              {{ forkingTemplate?.name }}
            </div>
          </n-form-item>
          <n-form-item label="新模板名称" path="name">
            <n-input v-model:value="forkFormData.name" placeholder="请输入新模板名称" />
          </n-form-item>
          <n-form-item label="新模板描述" path="description">
            <n-input v-model:value="forkFormData.description" type="textarea" :rows="3" placeholder="请输入新模板描述" />
          </n-form-item>
          <n-form-item label="详细介绍" path="introduction">
            <n-input v-model:value="forkFormData.introduction" type="textarea" :rows="4" placeholder="请输入详细介绍（可选）" />
          </n-form-item>
          <n-form-item label="分类" path="categoryId">
            <n-select v-model:value="forkFormData.categoryId" :options="categoryOptions" placeholder="选择分类（默认使用源模板分类）" clearable />
          </n-form-item>
        </n-form>
        <template #footer>
          <div style="display: flex; gap: 12px; justify-content: flex-end">
            <n-button @click="showForkModal = false">取消</n-button>
            <n-button type="primary" @click="handleForkSubmit" :loading="forkSubmitting">确认 Fork</n-button>
          </div>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useMessage, useDialog } from 'naive-ui';
import { listMyTemplates, createUserTemplate, updateUserTemplate, deleteUserTemplate, submitForReview } from '@/api/templates/contribution';
import { forkTemplate } from '@/api/templates';
import { getPublicCategories, getPublicLanguages } from '@/api/public/index';

const router = useRouter();
const message = useMessage();
const dialog = useDialog();

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
  dialog.warning({
    title: '确认删除',
    content: `确定删除模板"${name}"吗？此操作不可撤销。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
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
  // 撤回 = 改回 private
  try {
    await updateUserTemplate(id, { ...formData.value, id, visibility: 'private' });
    message.success('已撤回');
    loadTemplates();
  } catch { message.error('撤回失败'); }
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
  max-width: 1100px;
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

.template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}
.template-card {
  background: #fff;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  padding: 20px;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.template-card:hover {
  border-color: #cbd5e1;
  box-shadow: 0 4px 12px rgba(0,0,0,0.06);
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
.card-title { font-size: 15px; font-weight: 600; color: #0f172a; }
.status-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 10px;
  border-radius: 10px;
}
.status-badge.draft { background: #f1f5f9; color: #64748b; }
.status-badge.pending { background: #fffbeb; color: #d97706; }
.status-badge.published { background: #f0fdf4; color: #16a34a; }
.status-badge.rejected { background: #fef2f2; color: #dc2626; }

.card-desc {
  font-size: 13px;
  color: #64748b;
  margin: 0 0 12px;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.card-meta {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: #94a3b8;
  margin-bottom: 12px;
}
.card-actions {
  display: flex;
  gap: 4px;
  padding-top: 12px;
  border-top: 1px solid #f1f5f9;
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
</style>
