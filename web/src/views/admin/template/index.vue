<template>
  <div class="templates-manage">
    <n-flex vertical>
      <!-- 搜索和工具栏 -->
      <n-card :bordered="false">
        <n-form inline :label-width="80" :model="searchForm">
          <n-form-item label="关键词">
            <n-input
              v-model:value="searchForm.keyword"
              placeholder="输入模板名称或描述进行搜索"
              clearable
              style="width: 240px"
              @keyup.enter="handleSearch"
            >
              <template #prefix>
                <n-icon>
                  <SearchOutline />
                </n-icon>
              </template>
            </n-input>
          </n-form-item>
          <n-form-item label="分类">
            <n-select
              v-model:value="searchForm.categoryId"
              placeholder="选择分类"
              clearable
              style="width: 160px"
              :options="categoryOptions"
            />
          </n-form-item>
          <n-form-item label="语言">
            <n-select
              v-model:value="searchForm.languageId"
              placeholder="选择语言"
              clearable
              style="width: 160px"
              :options="languageOptions"
            />
          </n-form-item>
          <n-form-item>
            <n-space>
              <n-button type="primary" @click="handleSearch">
                <template #icon>
                  <n-icon>
                    <SearchOutline />
                  </n-icon>
                </template>
                搜索
              </n-button>
              <n-button @click="handleReset">
                <template #icon>
                  <n-icon>
                    <RefreshOutline />
                  </n-icon>
                </template>
                重置
              </n-button>
            </n-space>
          </n-form-item>
        </n-form>
      </n-card>

      <!-- 表格 -->
      <n-card :bordered="false">
        <BasicTable
          ref="actionRef"
          :columns="columns"
          :request="loadDataTable"
          :row-key="(row) => row.id"
          :actionColumn="actionColumn"
          :scroll-x="1600"
        >
          <template #tableTitle>
            <n-button type="primary" @click="handleAdd">
              <template #icon>
                <n-icon>
                  <AddOutline />
                </n-icon>
              </template>
              新建模板
            </n-button>
          </template>
        </BasicTable>
      </n-card>
    </n-flex>

    <!-- 添加/编辑模板弹窗 -->
    <n-modal v-model:show="showAddModal" :mask-closable="false">
      <n-card
        style="width: 1000px; height: 700px"
        :title="editingTemplate ? '编辑模板' : '添加模板'"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <template #header-extra>
          <n-button quaternary circle @click="closeModal">
            <template #icon>
              <n-icon>
                <CloseOutline />
              </n-icon>
            </template>
          </n-button>
        </template>

        <n-tabs
          v-model:value="activeTab"
          type="line"
          animated
          style="height: 520px; display: flex; flex-direction: column"
        >
          <!-- 基本信息 Tab -->
          <n-tab-pane name="basic" tab="基本信息" style="flex: 1; overflow-y: auto">
            <n-form
              ref="formRef"
              :model="formData"
              :rules="formRules"
              label-placement="left"
              :label-width="120"
              require-mark-placement="right-hanging"
            >
              <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 24px">
                <!-- 第一行：模板名称 + 模板类型 -->
                <div>
                  <n-form-item label="模板名称" path="name">
                    <n-input
                      v-model:value="formData.name"
                      placeholder="请输入模板名称"
                      :maxlength="100"
                      show-count
                    />
                  </n-form-item>
                </div>
                <div>
                  <n-form-item label="模板类型" path="templateType">
                    <n-select
                      v-model:value="formData.templateType"
                      placeholder="请选择模板类型"
                      :options="templateTypeSelectOptions"
                      style="width: 100%"
                      :disabled="!!editingTemplate"
                    />
                  </n-form-item>
                </div>

                <!-- 第二行：所属分类 + 支持语言 -->
                <div>
                  <n-form-item label="所属分类" path="categoryId">
                    <n-select
                      v-model:value="formData.categoryId"
                      placeholder="请选择分类"
                      :options="categorySelectOptions"
                      style="width: 100%"
                    />
                  </n-form-item>
                </div>
                <div>
                  <n-form-item label="支持语言" path="languages">
                    <n-select
                      v-model:value="formData.languages"
                      placeholder="请选择支持的语言"
                      :options="languageSelectOptions"
                      multiple
                      style="width: 100%"
                      @update:value="onLanguagesChange"
                    />
                  </n-form-item>
                </div>

                <!-- 第三行：主语言占据双栏 -->
                <div style="grid-column: 1 / -1">
                  <n-form-item label="主语言" path="primaryLanguage">
                    <n-select
                      v-model:value="formData.primaryLanguage"
                      placeholder="请选择主语言"
                      :options="primaryLanguageOptions"
                      style="width: 100%"
                    />
                  </n-form-item>
                </div>

                <!-- 第四行：模板描述占据双栏 -->
                <div style="grid-column: 1 / -1">
                  <n-form-item label="模板描述" path="description">
                    <n-input
                      v-model:value="formData.description"
                      type="textarea"
                      placeholder="请输入模板描述"
                      :maxlength="500"
                      show-count
                      :rows="4"
                      style="width: 100%"
                    />
                  </n-form-item>
                </div>
              </div>
            </n-form>
          </n-tab-pane>

          <!-- 详细描述 Tab -->
          <n-tab-pane name="introduction" tab="详细描述" style="flex: 1; overflow-y: auto">
            <div style="height: 500px; display: flex; flex-direction: column; gap: 12px">
              <!-- 工具栏说明 -->
              <div style="display: flex; justify-content: space-between; align-items: center">
                <n-text depth="3" style="font-size: 14px"
                  >支持 Markdown
                  格式，可添加代码块、表格、链接等丰富内容。点击编辑器工具栏小眼睛预览按钮查看实时效果。</n-text
                >
              </div>

              <!-- Markdown 编辑器 -->
              <div
                style="
                  flex: 1;
                  border: 1px solid var(--n-border-color);
                  border-radius: 6px;
                  overflow: hidden;
                "
              >
                <MdEditor
                  v-model="formData.introduction"
                  :style="{ height: '500px' }"
                  :toolbars="editorToolbars"
                  placeholder="请输入模板的详细介绍，支持Markdown格式..."
                />
              </div>
            </div>
          </n-tab-pane>
        </n-tabs>

        <template #footer>
          <div class="modal-footer">
            <n-button @click="closeModal">取消</n-button>
            <n-button type="primary" @click="handleSubmit" :loading="submitting">
              {{ editingTemplate ? '更新' : '添加' }}
            </n-button>
          </div>
        </template>
      </n-card>
    </n-modal>

    <!-- 删除确认弹窗 -->
    <n-modal v-model:show="showDeleteModal" :mask-closable="false">
      <n-card
        style="width: 400px"
        title="确认删除"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <div class="delete-content">
          <div class="delete-icon">
            <n-icon size="48" color="#d03050">
              <TrashOutline />
            </n-icon>
          </div>
          <p class="delete-message">
            确定要删除模板 <strong>"{{ deletingTemplate?.name }}"</strong> 吗？
          </p>
          <p class="delete-warning"> 此操作不可撤销，删除后相关模板文件和配置将无法恢复。 </p>
        </div>

        <template #footer>
          <div class="modal-footer">
            <n-button @click="showDeleteModal = false">取消</n-button>
            <n-button type="error" @click="confirmDelete" :loading="deleting"> 确认删除 </n-button>
          </div>
        </template>
      </n-card>
    </n-modal>

    <!-- 预览弹窗 -->
    <n-modal v-model:show="showPreviewModal" :mask-closable="true">
      <n-card
        style="width: 80%; max-width: 900px"
        title="模板详情"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <template #header-extra>
          <n-button quaternary circle @click="showPreviewModal = false">
            <template #icon>
              <n-icon>
                <CloseOutline />
              </n-icon>
            </template>
          </n-button>
        </template>

        <div v-if="previewData" class="preview-content">
          <n-descriptions :column="2" bordered>
            <n-descriptions-item label="模板名称">
              <div style="display: flex; align-items: center; gap: 8px">
                <span
                  v-if="previewData.isFeatured || previewData.is_featured"
                  style="color: #f0a020"
                  >⭐</span
                >
                {{ previewData.name }}
              </div>
            </n-descriptions-item>
            <n-descriptions-item label="模板类型">
              <n-tag :type="getTemplateTypeColor(previewData.templateType)" size="small">
                {{ getTemplateTypeLabel(previewData.templateType) }}
              </n-tag>
            </n-descriptions-item>
            <n-descriptions-item label="分类">
              {{ getCategoryName(previewData.categoryId || previewData.category_id) || '-' }}
            </n-descriptions-item>
            <n-descriptions-item label="支持语言">
              <n-space :size="4">
                <n-tag
                  v-for="lang in previewData.languages"
                  :key="lang.languageId"
                  :type="lang.isPrimary === 1 || lang.is_primary === 1 ? 'info' : 'default'"
                  size="small"
                >
                  {{ getLanguageName(lang.languageId) }}
                  <span
                    v-if="lang.isPrimary === 1 || lang.is_primary === 1"
                    style="margin-left: 4px"
                    >(主)</span
                  >
                </n-tag>
              </n-space>
            </n-descriptions-item>
            <n-descriptions-item label="模板描述" :span="2">
              {{ previewData.description || '暂无描述' }}
            </n-descriptions-item>
            <n-descriptions-item label="创建时间" :span="2">
              {{ formatDate(previewData.createdAt || previewData.created_at) }}
            </n-descriptions-item>
          </n-descriptions>

          <div v-if="previewData.introduction" style="margin-top: 20px">
            <h4 style="margin-bottom: 12px; color: #333; font-weight: 600">详细介绍：</h4>
            <div class="markdown-preview" v-html="previewData.introduction"></div>
          </div>
        </div>
      </n-card>
    </n-modal>

    <!-- Fork模板对话框 -->
    <n-modal v-model:show="showForkModal" :mask-closable="false">
      <n-card
        style="width: 600px"
        title="Fork模板"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <template #header-extra>
          <n-button quaternary circle @click="closeForkModal">
            <template #icon>
              <n-icon>
                <CloseOutline />
              </n-icon>
            </template>
          </n-button>
        </template>

        <n-form
          ref="forkFormRef"
          :model="forkFormData"
          :rules="forkFormRules"
          label-placement="left"
          :label-width="100"
          require-mark-placement="right-hanging"
        >
          <n-form-item label="源模板" path="">
            <div
              style="
                padding: 8px 12px;
                background: #f5f5f5;
                border-radius: 4px;
                color: #666;
                width: 100%;
              "
            >
              {{ forkingTemplate?.name }}
            </div>
          </n-form-item>

          <n-form-item label="新模板名称" path="name">
            <n-input v-model:value="forkFormData.name" placeholder="请输入新模板名称" />
          </n-form-item>

          <n-form-item label="新模板描述" path="description">
            <n-input
              v-model:value="forkFormData.description"
              type="textarea"
              :rows="3"
              placeholder="请输入新模板描述"
            />
          </n-form-item>

          <n-form-item label="详细介绍" path="introduction">
            <n-input
              v-model:value="forkFormData.introduction"
              type="textarea"
              :rows="5"
              placeholder="请输入新模板的详细介绍（可选）"
            />
          </n-form-item>

          <n-form-item label="分类" path="categoryId">
            <n-select
              v-model:value="forkFormData.categoryId"
              :options="categorySelectOptions"
              placeholder="选择分类（默认使用源模板分类）"
              clearable
            />
          </n-form-item>
        </n-form>

        <template #footer>
          <div class="modal-footer">
            <n-button @click="closeForkModal">取消</n-button>
            <n-button type="primary" @click="handleForkSubmit" :loading="submitting">
              确认Fork
            </n-button>
          </div>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup>
  import { ref, reactive, computed, onMounted, h, watch } from 'vue';
  import { useRouter } from 'vue-router';
  import { MdEditor } from 'md-editor-v3';
  import 'md-editor-v3/lib/style.css';
  import { BasicTable, TableAction } from '@/components/Table';
  import {
    NButton,
    NIcon,
    NSwitch,
    NTag,
    useMessage,
    NText,
    NSpace,
    NDescriptions,
    NDescriptionsItem,
  } from 'naive-ui';
  import {
    AddOutline,
    SearchOutline,
    RefreshOutline,
    CloseOutline,
    TrashOutline,
    CreateOutline,
    EyeOutline,
    CodeOutline,
    GitBranch,
    Star,
  } from '@vicons/ionicons5';
  import {
    listTemplates,
    addTemplate,
    editTemplate,
    deleteTemplate,
    getTemplateTypes,
    forkTemplate,
    toggleTemplateFeatured,
  } from '@/api/templates';
  import { useCategoryStore } from '@/store/modules/categoryStore';
  import { useLanguageStore } from '@/store/modules/languageStore';
  import { storeToRefs } from 'pinia';
  import { columns as baseColumns, setColumnHelpers } from './columns';

  const message = useMessage();
  const router = useRouter();
  const actionRef = ref();

  const categoryStore = useCategoryStore();
  const { categoriesList } = storeToRefs(categoryStore);

  const languageStore = useLanguageStore();
  const { languagesList } = storeToRefs(languageStore);

  // 数据状态
  const submitting = ref(false);
  const deleting = ref(false);

  // 搜索表单
  const searchForm = reactive({
    keyword: '',
    categoryId: null,
    languageId: null,
  });

  // 弹窗状态
  const showAddModal = ref(false);
  const showDeleteModal = ref(false);
  const showForkModal = ref(false);
  const showPreviewModal = ref(false);
  const editingTemplate = ref(null);
  const deletingTemplate = ref(null);
  const forkingTemplate = ref(null);
  const previewData = ref(null);

  // Tab状态
  const activeTab = ref('basic');

  // Markdown编辑器工具栏配置
  const editorToolbars = [
    'bold',
    'italic',
    'underline',
    'strikeThrough',
    '-',
    'title',
    'sub',
    'sup',
    'quote',
    'unorderedList',
    'orderedList',
    'task',
    '-',
    'codeRow',
    'code',
    'link',
    'image',
    'table',
    'mermaid',
    'katex',
    '-',
    'undo',
    'redo',
    'fullscreen',
    'preview',
    'htmlPreview',
  ];

  // 表单数据
  const formRef = ref(null);
  const formData = reactive({
    name: '',
    description: '',
    introduction: '',
    categoryId: null,
    isFeatured: 0,
    templateType: 'basic',
    languages: [],
    primaryLanguage: null,
  });

  // Fork表单数据
  const forkFormRef = ref(null);
  const forkFormData = reactive({
    name: '',
    description: '',
    introduction: '',
    categoryId: null,
  });

  // 表单验证规则
  const formRules = {
    name: {
      required: true,
      message: '请输入模板名称',
      trigger: ['input', 'blur'],
      min: 1,
      max: 100,
      validator: (rule, value) => {
        if (!value || !value.trim()) {
          return new Error('模板名称不能为空');
        }
        if (value.trim().length < 1) {
          return new Error('模板名称至少1个字符');
        }
        if (value.trim().length > 100) {
          return new Error('模板名称不能超过100个字符');
        }
        return true;
      },
    },
    description: {
      required: true,
      message: '请输入模板描述',
      trigger: ['input', 'blur'],
      min: 1,
      max: 500,
      validator: (rule, value) => {
        if (!value || !value.trim()) {
          return new Error('模板描述不能为空');
        }
        if (value.trim().length < 1) {
          return new Error('模板描述至少1个字符');
        }
        if (value.trim().length > 500) {
          return new Error('模板描述不能超过500个字符');
        }
        return true;
      },
    },
    introduction: {
      max: 2000,
      trigger: ['input', 'blur'],
      validator: (rule, value) => {
        if (value && value.trim().length > 2000) {
          return new Error('详细介绍不能超过2000个字符');
        }
        return true;
      },
    },
    categoryId: {
      required: true,
      type: 'number',
      message: '请选择分类',
      trigger: 'change',
      validator: (rule, value) => {
        if (!value || value <= 0) {
          return new Error('请选择有效的分类');
        }
        return true;
      },
    },
    templateType: {
      required: true,
      message: '请选择模板类型',
      trigger: 'change',
      validator: (rule, value) => {
        if (!value || !['basic', 'scaffold', 'data_driven'].includes(value)) {
          return new Error('请选择有效的模板类型');
        }
        return true;
      },
    },
    languages: {
      required: true,
      type: 'array',
      min: 1,
      message: '请选择支持的语言',
      trigger: 'change',
      validator: (rule, value) => {
        if (!value || value.length === 0) {
          return new Error('至少选择一种支持的语言');
        }
        return true;
      },
    },
    primaryLanguage: {
      required: true,
      type: 'number',
      message: '请选择主语言',
      trigger: 'change',
      validator: (rule, value) => {
        if (!value || value <= 0) {
          return new Error('请选择有效的主语言');
        }
        if (formData.languages.length > 0 && !formData.languages.includes(value)) {
          return new Error('主语言必须在支持的语言列表中');
        }
        return true;
      },
    },
  };

  // Fork表单验证规则
  const forkFormRules = {
    name: {
      required: true,
      message: '请输入新模板名称',
      trigger: ['input', 'blur'],
    },
    description: {
      required: true,
      message: '请输入新模板描述',
      trigger: ['input', 'blur'],
    },
  };

  // 选项数据
  const categoryOptions = computed(() =>
    categoriesList.value.map((c) => ({ label: c.name, value: Number(c.id) }))
  );

  const languageOptions = computed(() =>
    languagesList.value.map((lang) => ({ label: lang.name, value: Number(lang.id) }))
  );

  const categorySelectOptions = computed(() =>
    categoriesList.value.map((c) => ({ label: c.name, value: Number(c.id) }))
  );

  const languageSelectOptions = computed(() =>
    languagesList.value.map((lang) => ({ label: lang.name, value: Number(lang.id) }))
  );

  const primaryLanguageOptions = computed(() =>
    languagesList.value
      .filter((lang) => formData.languages.includes(Number(lang.id)))
      .map((lang) => ({
        label: lang.name,
        value: Number(lang.id),
      }))
  );

  // 模板类型数据
  const templateTypes = ref([]);
  const templateTypeSelectOptions = computed(() =>
    templateTypes.value.map((type) => ({ label: type.label, value: type.value }))
  );

  // 设置列辅助函数
  const getCategoryName = (categoryId) => {
    if (!categoryId) return null;
    const category = categoriesList.value.find((cat) => cat.id === Number(categoryId));
    return category ? category.name : null;
  };

  const getLanguageName = (languageId) => {
    if (!languageId) return '';
    const language = languagesList.value.find((lang) => lang.id === Number(languageId));
    return language ? language.name : `未知语言(${languageId})`;
  };

  // 设置列辅助函数
  setColumnHelpers(getCategoryName, getLanguageName);

  // 使用基础列配置
  const columns = baseColumns;

  // 操作列
  const actionColumn = reactive({
    width: 480,
    title: '操作',
    key: 'action',
    fixed: 'right',
    align: 'center',
    render(record) {
      return h(TableAction, {
        style: 'button',
        actions: createActions(record),
      });
    },
  });

  function createActions(record) {
    return [
      {
        label: '查看',
        icon: EyeOutline,
        onClick: handleView.bind(null, record),
      },
      {
        label: '内容编辑',
        icon: CodeOutline,
        onClick: handleContentEdit.bind(null, record),
      },
      {
        label: '编辑',
        icon: CreateOutline,
        onClick: handleEdit.bind(null, record),
      },
      {
        label: 'Fork',
        icon: GitBranch,
        onClick: handleFork.bind(null, record),
      },
      {
        label: '删除',
        icon: TrashOutline,
        onClick: handleDelete.bind(null, record),
      },
    ];
  }

  // 加载数据
  const loadDataTable = async (res) => {
    try {
      const params = {
        pageNum: res.page,
        pageSize: res.pageSize,
      };

      // 添加搜索条件
      if (searchForm.keyword) {
        params.name = searchForm.keyword;
      }
      if (searchForm.categoryId) {
        params.categoryId = searchForm.categoryId;
      }
      if (searchForm.languageId) {
        params.languageId = searchForm.languageId;
      }

      const response = await listTemplates(params);

      // 检查响应状态
      if (response.data.code !== 0) {
        throw new Error(response.data.message || '获取模板列表失败');
      }

      // BasicTable期望返回 { list: [], itemCount: number } 格式
      const result = {
        list: response.data.data.templatesList || [],
        itemCount: response.data.data.total || 0,
      };

      return result;
    } catch (error) {
      console.error('获取模板列表失败:', error);
      message.error('获取模板列表失败');
      return {
        list: [],
        itemCount: 0,
      };
    }
  };

  // 刷新表格
  function reloadTable() {
    actionRef.value?.reload();
  }

  // 搜索
  function handleSearch() {
    reloadTable();
  }

  // 重置搜索
  function handleReset() {
    searchForm.keyword = '';
    searchForm.categoryId = null;
    searchForm.languageId = null;
    reloadTable();
  }

  // 查看（预览）
  const handleView = (template) => {
    previewData.value = template;
    showPreviewModal.value = true;
  };

  // 内容编辑
  const handleContentEdit = (template) => {
    router.push(`/admin/editor/${template.id}`);
  };

  // 添加
  const handleAdd = () => {
    editingTemplate.value = null;
    resetForm();
    showAddModal.value = true;
  };

  // 编辑
  const handleEdit = (template) => {
    editingTemplate.value = template;
    formData.name = template.name;
    formData.description = template.description;
    formData.introduction = template.introduction || '';
    formData.categoryId = template.categoryId || template.category_id;
    formData.isFeatured = template.isFeatured || template.is_featured || 0;
    formData.templateType = template.templateType || 'basic';

    // 语言回显
    if (template.languages && template.languages.length > 0) {
      formData.languages = template.languages.map((l) => Number(l.languageId || l.id));
      const primary = template.languages.find((l) => l.isPrimary === 1 || l.is_primary === 1);
      formData.primaryLanguage = primary ? Number(primary.languageId || primary.id) : null;
    } else {
      formData.languages = [];
      formData.primaryLanguage = null;
    }

    showAddModal.value = true;
  };

  // 删除
  const handleDelete = (template) => {
    deletingTemplate.value = template;
    showDeleteModal.value = true;
  };

  // Fork相关方法
  const handleFork = (template) => {
    forkingTemplate.value = template;
    forkFormData.name = `${template.name} - Fork`;
    forkFormData.description = template.description;
    forkFormData.introduction = template.introduction;
    forkFormData.categoryId = template.categoryId || template.category_id;
    showForkModal.value = true;
  };

  const closeForkModal = () => {
    showForkModal.value = false;
    forkingTemplate.value = null;
    resetForkForm();
  };

  const resetForkForm = () => {
    forkFormData.name = '';
    forkFormData.description = '';
    forkFormData.introduction = '';
    forkFormData.categoryId = null;
    forkFormRef.value?.restoreValidation();
  };

  const handleForkSubmit = async () => {
    try {
      await forkFormRef.value?.validate();
      submitting.value = true;

      const data = {
        sourceId: forkingTemplate.value.id,
        name: forkFormData.name,
        description: forkFormData.description,
        introduction: forkFormData.introduction,
        categoryId: forkFormData.categoryId,
      };

      const response = await forkTemplate(data);

      if (response.data.code === 0) {
        message.success(`模板Fork成功！新模板ID: ${response.data.data.templateId}`);
        closeForkModal();
        reloadTable();
      } else {
        message.error(response.data.message || 'Fork失败');
      }
    } catch (error) {
      console.error('Fork模板失败:', error);
      message.error('Fork模板失败: ' + (error.message || '未知错误'));
    } finally {
      submitting.value = false;
    }
  };

  // 关闭弹窗
  const closeModal = () => {
    showAddModal.value = false;
    editingTemplate.value = null;
    activeTab.value = 'basic';
    resetForm();
  };

  // 重置表单
  const resetForm = () => {
    formData.name = '';
    formData.description = '';
    formData.introduction = '';
    formData.categoryId = null;
    formData.templateType = 'basic';
    formData.languages = [];
    formData.primaryLanguage = null;
    formRef.value?.restoreValidation();
  };

  // 语言选择变更
  const onLanguagesChange = (val) => {
    if (!val.includes(formData.primaryLanguage)) {
      formData.primaryLanguage = null;
    }
  };

  // 提交表单
  const handleSubmit = async () => {
    try {
      await formRef.value?.validate();
      submitting.value = true;

      const languagesArr = formData.languages.map((langId) => ({
        languageId: langId,
        isPrimary: langId === formData.primaryLanguage ? 1 : 0,
      }));

      const data = {
        name: formData.name.trim(),
        description: formData.description.trim(),
        introduction: formData.introduction?.trim() || '',
        categoryId: formData.categoryId,
        templateType: formData.templateType,
        languages: languagesArr,
      };

      if (editingTemplate.value) {
        data.id = editingTemplate.value.id;
        const response = await editTemplate(data);

        if (response.data.code === 0) {
          message.success('模板更新成功');
        } else {
          throw new Error(response.data.message || '更新模板失败');
        }
      } else {
        const response = await addTemplate(data);

        if (response.data.code === 0) {
          message.success('模板添加成功');
        } else {
          throw new Error(response.data.message || '添加模板失败');
        }
      }

      closeModal();
      reloadTable();
    } catch (error) {
      console.error('操作失败:', error);
      const errorMessage = error.response?.data?.message || error.message || '操作失败';
      message.error(`${editingTemplate.value ? '更新' : '添加'}模板失败: ${errorMessage}`);
    } finally {
      submitting.value = false;
    }
  };

  // 确认删除
  const confirmDelete = async () => {
    try {
      deleting.value = true;
      const response = await deleteTemplate({ id: deletingTemplate.value.id });

      if (response.data.code === 0) {
        message.success('模板删除成功');
        showDeleteModal.value = false;
        deletingTemplate.value = null;
        reloadTable();
      } else {
        throw new Error(response.data.message || '删除模板失败');
      }
    } catch (error) {
      console.error('删除模板失败:', error);
      const errorMessage = error.response?.data?.message || error.message || '删除模板失败';
      message.error(`删除模板失败: ${errorMessage}`);
    } finally {
      deleting.value = false;
    }
  };

  // 主语言选择监听
  watch(
    () => formData.languages,
    (langs) => {
      if (!langs.includes(formData.primaryLanguage)) {
        formData.primaryLanguage = null;
      }
    }
  );

  // 加载模板类型
  const loadTemplateTypes = async () => {
    try {
      const res = await getTemplateTypes();
      templateTypes.value = res.data.data.templateTypes || [];
    } catch (error) {
      console.error('获取模板类型失败:', error);
      templateTypes.value = [
        { value: 'basic', label: '基础模板' },
        { value: 'scaffold', label: '脚手架模板' },
        { value: 'data_driven', label: '数据驱动模板' },
      ];
    }
  };

  // 获取模板类型标签
  const getTemplateTypeLabel = (templateType) => {
    if (!templateType) return '基础模板';
    const type = templateTypes.value.find((t) => t.value === templateType);
    return type ? type.label : '基础模板';
  };

  // 获取模板类型颜色
  const getTemplateTypeColor = (templateType) => {
    if (!templateType || templateType === 'basic') return 'default';
    if (templateType === 'scaffold') return 'warning';
    if (templateType === 'data_driven') return 'success';
    return 'default';
  };

  // 格式化日期
  const formatDate = (dateString) => {
    if (!dateString) return '-';
    const date = new Date(dateString);
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  // 生命周期
  onMounted(async () => {
    try {
      await Promise.all([
        categoryStore.getCategories(),
        languageStore.getLanguages(),
        loadTemplateTypes(),
      ]);
    } catch (error) {
      console.error('初始化数据加载失败:', error);
      message.error('初始化数据加载失败');
    }
  });
</script>

<style scoped>
  .templates-manage {
    padding: 16px;
    background: transparent;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }

  .delete-content {
    text-align: center;
    padding: 20px 0;
  }

  .delete-icon {
    margin-bottom: 16px;
  }

  .delete-message {
    font-size: 16px;
    color: #333;
    margin-bottom: 12px;
  }

  .delete-warning {
    font-size: 14px;
    color: #d03050;
    background: #fff2f0;
    padding: 12px;
    border-radius: 6px;
    border: 1px solid #ffccc7;
  }

  .template-name {
    font-weight: 500;
    color: #333;
  }

  .text-placeholder {
    color: #999;
    font-style: italic;
  }

  .preview-content h4 {
    margin-bottom: 12px;
    color: #333;
    font-weight: 600;
  }

  .markdown-preview {
    background: #f5f7fa;
    border: 1px solid #e0e6ed;
    border-radius: 4px;
    padding: 16px;
    line-height: 1.6;
    color: #333;
  }

  .markdown-preview :deep(h1),
  .markdown-preview :deep(h2),
  .markdown-preview :deep(h3) {
    margin-top: 16px;
    margin-bottom: 12px;
    font-weight: 600;
  }

  .markdown-preview :deep(p) {
    margin-bottom: 12px;
  }

  .markdown-preview :deep(code) {
    background: #e8e8e8;
    padding: 2px 6px;
    border-radius: 3px;
    font-family: 'Courier New', Consolas, monospace;
  }

  .markdown-preview :deep(pre) {
    background: #282c34;
    color: #abb2bf;
    padding: 12px;
    border-radius: 4px;
    overflow-x: auto;
  }
</style>
