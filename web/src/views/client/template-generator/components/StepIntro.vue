<template>
  <div class="step-intro">
    <div class="intro-content">
      <div class="template-info">
        <div class="template-header">
          <h1 class="template-name">{{ templateInfo?.name }}</h1>
          <a-tag v-if="templateInfo?.categoryId" color="success">
            {{ getCategoryName(templateInfo?.categoryId) }}
          </a-tag>
        </div>

        <div class="template-description">
          {{ templateInfo?.description }}
        </div>

        <div v-if="templateInfo?.isFeatured" class="featured-badge">
          <svg width="16" height="16" viewBox="0 0 24 24" style="fill: var(--client-theme-color)" stroke="none"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
          <span>推荐模板</span>
        </div>

        <!-- 版本选择 -->
        <div class="config-section">
          <h3 class="section-title">选择版本</h3>
          <a-select
            v-if="versionList.length > 0"
            v-model:value="internalVersion"
            :options="versionOptions"
            size="large"
            placeholder="选择版本"
            @change="handleVersionChange"
          />
          <div v-else class="empty-hint">
            该模板暂无发布版本，请联系管理员发布。
          </div>
        </div>

        <!-- 支持的语言 -->
        <div v-if="getTemplateLanguages.length > 0" class="config-section">
          <h3 class="section-title">支持的语言</h3>
          <div class="languages-list">
            <a-tag
              v-for="lang in getTemplateLanguages"
              :key="lang.id"
              :color="lang.isPrimary === 1 ? 'success' : undefined"
            >
              {{ lang.displayName }}
              <template v-if="lang.isPrimary === 1"> (主语言) </template>
            </a-tag>
          </div>
        </div>

        <!-- 详细介绍 -->
        <div v-if="templateInfo?.introduction" class="config-section">
          <h3 class="section-title">详细介绍</h3>
          <div class="markdown-content">
            <MdPreview :modelValue="templateInfo.introduction" />
          </div>
        </div>
      </div>
    </div>

    <div class="step-actions">
      <a-button size="large" @click="handleFork" :disabled="!templateInfo">
        <template #icon>
          <GitBranchOutline />
        </template>
        Fork 模板
      </a-button>
      <a-button type="primary" size="large" @click="$emit('next')" :disabled="!templateInfo">
        开始配置
        <template #icon>
          <ArrowForward />
        </template>
      </a-button>
    </div>

    <!-- Fork模板弹窗 -->
    <a-modal v-model:open="showForkModal" title="Fork 模板" :mask-closable="false" :width="600" :footer="null">
      <a-form ref="forkFormRef" :model="forkFormData" :rules="forkFormRules" :label-col="{ span: 6 }" :wrapper-col="{ span: 18 }">
        <a-form-item label="源模板">
          <div style="padding: 8px 12px; background: #f8fafc; border-radius: 6px; color: #64748b; width: 100%; border: 1px solid #e2e8f0;">
            {{ templateInfo?.name }}
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
          <a-select v-model:value="forkFormData.categoryId" :options="forkCategoryOptions" placeholder="选择分类（默认使用源模板分类）" allow-clear />
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
  import { computed, onMounted, watch, ref } from 'vue';
  import { useRouter } from 'vue-router';
  import { message } from 'ant-design-vue';
  import { Star, ArrowForward, GitBranchOutline } from '@/icons/ionicons5';
  import { MdPreview } from 'md-editor-v3';
  import { useLanguageStore } from '@/store/modules/languageStore';
  import { useCategoryStore } from '@/store/modules/categoryStore';
  import { forkTemplate } from '@/api/templates';

  const router = useRouter();

  const props = defineProps({
    templateInfo: { type: Object, default: null },
    versionList: { type: Array, default: () => [] },
    selectedVersion: { type: String, default: '' },
  });

  const emit = defineEmits(['next', 'update-version']);

  const showForkModal = ref(false);
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
    categoryStore.categoriesList.map((cat) => ({ label: cat.name, value: cat.id }))
  );

  const handleFork = () => {
    if (!props.templateInfo) return;
    forkFormData.value = {
      name: `${props.templateInfo.name} - Fork`,
      description: props.templateInfo.description || '',
      introduction: props.templateInfo.introduction || '',
      categoryId: props.templateInfo.categoryId || null,
    };
    showForkModal.value = true;
  };

  const handleForkSubmit = async () => {
    try { await forkFormRef.value?.validate(); } catch { return; }
    forkSubmitting.value = true;
    try {
      const res = await forkTemplate({
        sourceId: props.templateInfo.id,
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
  };

  const internalVersion = ref(props.selectedVersion);

  watch(() => props.selectedVersion, (newVal) => {
    internalVersion.value = newVal;
  });

  const versionOptions = computed(() => {
    const options = [{ label: 'Latest (最新版本)', value: '' }];
    props.versionList.forEach((v) => {
      const label = `${v.version}${v.isLatest ? ' (当前)' : ''}${v.isDeprecated ? ' [已弃用]' : ''}`;
      options.push({ label, value: v.version });
    });
    return options;
  });

  const handleVersionChange = (value) => {
    internalVersion.value = value;
    emit('update-version', value);
  };

  const languageStore = useLanguageStore();
  const categoryStore = useCategoryStore();

  const getCategoryName = (categoryId) => {
    const category = categoryStore.categoriesList.find((cat) => cat.id === categoryId);
    return category ? category.name : '未知分类';
  };

  const getTemplateLanguages = computed(() => {
    if (!props.templateInfo?.languages?.length) return [];
    return props.templateInfo.languages.map((templateLang) => {
      const language = languageStore.languagesList.find((lang) => lang.id === templateLang.languageId);
      return {
        ...templateLang,
        name: language?.name || '未知语言',
        displayName: language?.displayName || language?.name || '未知语言',
        color: language?.color || '#22c55e',
      };
    });
  });

  onMounted(async () => {
    try {
      await Promise.all([languageStore.getLanguages(), categoryStore.getCategories()]);
    } catch (error) {
      console.error('加载语言或分类数据失败:', error);
    }
  });
</script>

<style scoped>
  .step-intro {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .intro-content {
    flex: 1;
    padding: 40px;
    overflow-y: auto;
  }

  .template-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
  }

  .template-name {
    font-size: 26px;
    font-weight: 700;
    color: #0f172a;
    margin: 0;
    letter-spacing: -0.5px;
  }

  .template-description {
    font-size: 15px;
    color: #64748b;
    line-height: 1.7;
    margin-bottom: 20px;
  }

  .featured-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 6px;
    background: var(--client-theme-bg-light);
    border: 1px solid var(--client-theme-border-light);
    color: var(--client-theme-color);
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 24px;
  }

  .config-section {
    margin-bottom: 28px;
  }

  .section-title {
    font-size: 15px;
    font-weight: 600;
    color: #334155;
    margin: 0 0 12px 0;
    padding-bottom: 8px;
    border-bottom: 1px solid #f1f5f9;
  }

  .languages-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .empty-hint {
    padding: 12px 16px;
    background: #f8fafc;
    border-radius: 8px;
    color: #94a3b8;
    font-size: 14px;
    border: 1px dashed #e2e8f0;
  }

  .markdown-content {
    background: #f8fafc;
    padding: 20px;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
  }

  .step-actions {
    padding: 16px 32px;
    border-top: 1px solid #e2e8f0;
    background: #fff;
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }
</style>
