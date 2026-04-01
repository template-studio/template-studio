<template>
  <div class="template-preview">
    <div v-if="template" class="preview-content">
      <div class="template-header">
        <div class="template-title">{{ template.name }}</div>
        <div class="template-description">{{ template.description }}</div>
      </div>

      <div class="template-details">
        <div class="detail-item">
          <span class="label">分类：</span>
          <span class="value">{{ getCategoryName(template.categoryId) || '未分类' }}</span>
        </div>

        <div class="detail-item" v-if="template.languages && template.languages.length > 0">
          <span class="label">支持语言：</span>
          <div class="languages">
            <n-tag
              v-for="lang in template.languages"
              :key="lang.languageId"
              size="small"
              :type="lang.isPrimary ? 'primary' : 'default'"
            >
              {{ getLanguageName(lang.languageId) }}
            </n-tag>
          </div>
        </div>

        <div class="detail-item" v-if="template.introduction">
          <span class="label">详细介绍：</span>
          <div class="introduction" v-html="template.introduction"></div>
        </div>
      </div>

      <div class="template-actions">
        <n-button type="primary" size="large" @click="useTemplate" block>
          <template #icon>
            <n-icon><DownloadOutline /></n-icon>
          </template>
          使用此模板
        </n-button>
      </div>
    </div>

    <div v-else class="empty-preview">
      <n-empty description="未选择模板">
        <template #icon>
          <n-icon size="48"><DocumentTextOutline /></n-icon>
        </template>
      </n-empty>
    </div>
  </div>
</template>

<script setup>
  import { computed } from 'vue';
  import { useRouter } from 'vue-router';
  import { NButton, NIcon, NTag, NEmpty } from 'naive-ui';
  import { DownloadOutline, DocumentTextOutline } from '@vicons/ionicons5';
  import { useCategoryStore } from '@/store/modules/categoryStore';
  import { useLanguageStore } from '@/store/modules/languageStore';
  import { storeToRefs } from 'pinia';

  const props = defineProps({
    template: {
      type: Object,
      default: null,
    },
  });

  const router = useRouter();
  const categoryStore = useCategoryStore();
  const languageStore = useLanguageStore();
  const { categoriesList } = storeToRefs(categoryStore);
  const { languagesList } = storeToRefs(languageStore);

  // 获取分类名称
  const getCategoryName = (categoryId) => {
    if (!categoryId) return null;
    const category = categoriesList.value.find((cat) => cat.id === Number(categoryId));
    return category ? category.name : null;
  };

  // 获取语言名称
  const getLanguageName = (languageId) => {
    if (!languageId) return '未知';
    const language = languagesList.value.find((lang) => lang.id === Number(languageId));
    return language ? language.name : '未知';
  };

  // 使用模板
  const useTemplate = () => {
    if (props.template) {
      router.push(`/template-generator/${props.template.id}`);
    }
  };
</script>

<style scoped>
  .template-preview {
    padding: 20px;
  }

  .preview-content {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .template-header {
    text-align: center;
    padding-bottom: 16px;
    border-bottom: 1px solid #f0f0f0;
  }

  .template-title {
    font-size: 24px;
    font-weight: 600;
    color: #333;
    margin-bottom: 8px;
  }

  .template-description {
    font-size: 16px;
    color: #666;
    line-height: 1.5;
  }

  .template-details {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .detail-item {
    display: flex;
    gap: 8px;
  }

  .detail-item .label {
    font-weight: 500;
    color: #333;
    min-width: 80px;
    flex-shrink: 0;
  }

  .detail-item .value {
    color: #666;
  }

  .languages {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .introduction {
    color: #666;
    line-height: 1.6;
    max-height: 200px;
    overflow-y: auto;
  }

  .template-actions {
    padding-top: 16px;
    border-top: 1px solid #f0f0f0;
  }

  .empty-preview {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 200px;
  }
</style>
