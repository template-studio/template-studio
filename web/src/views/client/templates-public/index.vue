<template>
  <div class="templates-page">
    <!-- 筛选区域 -->
    <div class="filters-section">
      <div class="container">
        <!-- 分类 -->
        <div class="filter-block">
          <h2>分类</h2>
          <div class="tiles">
            <div
              v-for="cat in categoryTags"
              :key="cat.id"
              class="tile"
              :class="{ active: selectedCategory === cat.id }"
              @click="selectCategory(cat.id)"
            >
              {{ cat.name }}
            </div>
          </div>
        </div>

        <!-- 语言 -->
        <div class="filter-block">
          <h2>语言</h2>
          <div class="tiles">
            <div
              v-for="tag in tags"
              :key="tag.id"
              class="tile"
              :class="{ active: selectedTag === tag.id }"
              @click="selectTag(tag.id)"
            >
              {{ tag.name }}
            </div>
          </div>
        </div>

        <!-- 类型 -->
        <div class="filter-block">
          <h2>类型</h2>
          <div class="tiles">
            <div
              v-for="type in templateTypes"
              :key="type.value"
              class="tile"
              :class="{ active: selectedTemplateType === type.value }"
              @click="selectTemplateType(type.value)"
            >
              {{ type.label }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 模板列表 -->
    <div class="templates-section">
      <div class="container">
        <div class="templates-header">
          <div class="templates-title-section">
            <h2>模板列表</h2>
            <div class="templates-count">共 {{ totalTemplates }} 个模板</div>
          </div>
        </div>

        <n-spin :show="loading">
          <div class="templates-grid">
            <TemplateCard
              v-for="template in templates"
              :key="template.id"
              :template="template"
              @click="useTemplate"
              @fork="handleFork"
            />
          </div>
        </n-spin>

        <!-- 分页 -->
        <div class="pagination-section" v-if="totalPages > 1">
          <n-pagination
            v-model:page="currentPage"
            :page-count="totalPages"
            :page-sizes="[20, 40, 60]"
            :page-size="pageSize"
            show-size-picker
            @update:page="handlePageChange"
            @update:page-size="handlePageSizeChange"
          />
        </div>
      </div>
    </div>

    <!-- 模板预览抽屉 -->
    <n-drawer v-model:show="showPreview" :width="800" placement="right">
      <n-drawer-content title="模板预览" closable>
        <TemplatePreview v-if="selectedTemplate" :template="selectedTemplate" />
      </n-drawer-content>
    </n-drawer>

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
            <n-select v-model:value="forkFormData.categoryId" :options="forkCategoryOptions" placeholder="选择分类（默认使用源模板分类）" clearable />
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
  import { useRouter, useRoute } from 'vue-router';
  import { NSpin, NPagination, NDrawer, NDrawerContent, useMessage } from 'naive-ui';
  import { useLanguageStore } from '@/store/modules/languageStore';
  import { useCategoryStore } from '@/store/modules/categoryStore';
  import { storeToRefs } from 'pinia';
  import { getPublicTemplateTypes, getPublicTemplates } from '@/api/public';
  import { forkTemplate } from '@/api/templates';
  import TemplateCard from '@/components/TemplateCard.vue';
  import TemplatePreview from '@/components/TemplatePreview.vue';

  const router = useRouter();
  const route = useRoute();
  const message = useMessage();

  const languageStore = useLanguageStore();
  const { languagesList } = storeToRefs(languageStore);

  const categoryStore = useCategoryStore();
  const { categoriesList } = storeToRefs(categoryStore);

  // 分类tag数据
  const categoryTags = computed(() => [
    { id: 'all', name: '全部' },
    ...categoriesList.value.map((cat) => ({ id: cat.id, name: cat.name })),
  ]);
  const selectedCategory = ref('all');

  // 状态管理
  const selectedTag = ref('all');
  const selectedTemplateType = ref('all');
  const currentPage = ref(1);
  const pageSize = ref(20);
  const loading = ref(false);
  const showPreview = ref(false);
  const selectedTemplate = ref(null);
  const selectedTemplateId = ref(null);
  const viewMode = ref('grid'); // 'grid' | 'list'

  // 标签数据
  const tags = computed(() => {
    const arr = [
      { id: 'all', name: '全部' },
      ...languagesList.value.map((lang) => ({ id: lang.id, name: lang.name })),
    ];
    return arr;
  });

  // 模板数据
  const allTemplates = ref([]);

  // 模板类型数据
  const allTemplateTypes = ref([]);
  const templateTypes = computed(() => {
    return [{ value: 'all', label: '全部' }, ...allTemplateTypes.value];
  });

  // 计算属性
  // 由于现在使用后端过滤，allTemplates.value 已经是过滤后的数据
  const totalTemplates = computed(() => allTemplates.value.length);
  const totalPages = computed(() => Math.ceil(totalTemplates.value / pageSize.value));

  const templates = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value;
    const end = start + pageSize.value;
    return allTemplates.value.slice(start, end);
  });

  // 方法
  const selectCategory = async (catId) => {
    selectedCategory.value = catId === 'all' ? 'all' : Number(catId);
    currentPage.value = 1;

    // 重新加载数据
    const searchKeyword = route.query.search || '';
    await loadTemplates(
      searchKeyword,
      selectedCategory.value,
      selectedTag.value,
      selectedTemplateType.value
    );
  };

  const selectTag = async (tagId) => {
    selectedTag.value = tagId;
    currentPage.value = 1;

    // 重新加载数据
    const searchKeyword = route.query.search || '';
    await loadTemplates(
      searchKeyword,
      selectedCategory.value,
      selectedTag.value,
      selectedTemplateType.value
    );
  };

  const selectTemplateType = async (typeValue) => {
    selectedTemplateType.value = typeValue;
    currentPage.value = 1;

    // 重新加载数据
    const searchKeyword = route.query.search || '';
    await loadTemplates(
      searchKeyword,
      selectedCategory.value,
      selectedTag.value,
      selectedTemplateType.value
    );
  };

  const handlePageChange = (page) => {
    currentPage.value = page;
  };

  const handlePageSizeChange = (size) => {
    pageSize.value = size;
    currentPage.value = 1;
  };

  const selectTemplate = (template) => {
    selectedTemplateId.value = template.id;
  };

  const useTemplate = (template) => {
    router.push(`/template-generator/${template.id}`);
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
  const forkCategoryOptions = computed(() =>
    categoriesList.value.map((cat) => ({ label: cat.name, value: cat.id }))
  );

  const handleFork = (template) => {
    forkingTemplate.value = template;
    forkFormData.value = {
      name: `${template.name} - Fork`,
      description: template.description || '',
      introduction: template.introduction || '',
      categoryId: template.categoryId || null,
    };
    showForkModal.value = true;
  };

  const handleForkSubmit = async () => {
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
        router.push(`/editor/${data.data}`);
      } else {
        message.error(data.message || 'Fork 失败');
      }
    } catch (error) {
      message.error('Fork 失败，请稍后重试');
      console.error('Fork error:', error);
    } finally {
      forkSubmitting.value = false;
    }
  };

  const previewTemplate = (template) => {
    selectedTemplate.value = template;
    showPreview.value = true;
  };

  // 获取分类名称
  const getCategoryName = (categoryId) => {
    if (!categoryId) return null;
    const category = categoriesList.value.find((cat) => cat.id === Number(categoryId));
    return category ? category.name : null;
  };

  // 加载模板数据
  const loadTemplates = async (
    searchKeyword = '',
    categoryId = null,
    languageId = null,
    templateType = null
  ) => {
    try {
      loading.value = true;

      const params = {};

      // 搜索关键词
      if (searchKeyword.trim()) {
        params.name = searchKeyword.trim();
        params.description = searchKeyword.trim();
      }

      // 分类筛选
      if (categoryId && categoryId !== 'all') {
        params.categoryId = Number(categoryId);
      }

      // 语言筛选
      if (languageId && languageId !== 'all') {
        params.languageId = Number(languageId);
      }

      // 模板类型筛选
      if (templateType && templateType !== 'all') {
        params.templateType = templateType;
      }

      const res = await getPublicTemplates(params);
      allTemplates.value = res.data.data.templatesList || [];
    } catch (error) {
      console.error('获取模板列表失败:', error);
      message.error('获取模板列表失败');
    } finally {
      loading.value = false;
    }
  };

  // 加载模板类型数据
  const loadTemplateTypes = async () => {
    try {
      const res = await getPublicTemplateTypes();
      allTemplateTypes.value = res.data.data.templateTypes || [];
    } catch (error) {
      console.error('获取模板类型失败:', error);
      // 静默失败，使用默认的空数组
    }
  };

  // 监听路由变化，重新搜索
  watch(
    () => route.query.search,
    (newSearchKeyword) => {
      loadTemplates(
        newSearchKeyword || '',
        selectedCategory.value,
        selectedTag.value,
        selectedTemplateType.value
      );
    }
  );

  // 初始化
  onMounted(async () => {
    await languageStore.getLanguages();
    await categoryStore.getCategories();
    await loadTemplateTypes();

    // 从路由查询参数获取搜索关键词并加载数据
    const searchKeyword = route.query.search || '';
    await loadTemplates(
      searchKeyword,
      selectedCategory.value,
      selectedTag.value,
      selectedTemplateType.value
    );
  });
</script>

<style scoped>
  .templates-page {
    min-height: calc(100vh - 64px);
    width: 100%;
    background: #f8fafc;
  }

  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
  }

  /* ===== 筛选区域 ===== */
  .filters-section {
    background: #fff;
    padding: 24px 0;
    border-bottom: 1px solid #e2e8f0;
  }

  .filter-block {
    margin-bottom: 16px;
  }

  .filter-block:last-child {
    margin-bottom: 0;
  }

  .filter-block h2 {
    font-size: 13px;
    font-weight: 600;
    color: #64748b;
    margin: 0 0 10px 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .tiles {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .tile {
    background: #f8fafc;
    border-radius: 8px;
    padding: 6px 14px;
    cursor: pointer;
    transition: all 0.2s ease-out;
    display: inline-flex;
    align-items: center;
    font-size: 13px;
    color: #475569;
    font-weight: 500;
    border: 1px solid #e2e8f0;
  }

  .tile:hover {
    background: var(--client-theme-bg-light);
    border-color: var(--client-theme-border-light);
    color: var(--client-theme-dark);
  }

  .tile.active {
    background: #0f172a;
    color: #fff;
    border-color: #0f172a;
    box-shadow: 0 2px 8px rgba(15, 23, 42, 0.15);
  }

  .tile.active:hover {
    background: #1e293b;
    border-color: #1e293b;
    color: #fff;
  }

  /* ===== 模板列表 ===== */
  .templates-section {
    padding: 40px 0 60px;
  }

  .templates-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    margin-bottom: 28px;
    padding-bottom: 16px;
    border-bottom: 1px solid #e2e8f0;
  }

  .templates-title-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .templates-header h2 {
    margin: 0;
    font-size: 22px;
    font-weight: 700;
    color: #0f172a;
    letter-spacing: -0.3px;
  }

  .templates-count {
    color: #94a3b8;
    font-size: 13px;
    margin-top: 2px;
  }

  /* ===== 模板网格 ===== */
  .templates-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 20px;
    margin-bottom: 40px;
  }

  @media (max-width: 768px) {
    .templates-grid {
      grid-template-columns: 1fr;
      gap: 16px;
    }
  }

  /* ===== 分页 ===== */
  .pagination-section {
    display: flex;
    justify-content: center;
    margin-top: 40px;
  }
</style>
