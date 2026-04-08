<template>
  <div class="template-config">
    <FilterSection
      :search-query="searchKeyword"
      :categories="categories"
      :languages="languages"
      :selected-category="selectedCategory"
      :selected-language="selectedLanguage"
      @update:search-query="searchKeyword = $event"
      @update:selected-category="selectedCategory = $event"
      @update:selected-language="selectedLanguage = $event"
    />

    <!-- 模板列表 -->
    <div class="templates-section">
      <div class="templates-header">
        <div class="templates-title-section">
          <h2>模板列表</h2>
          <div class="templates-count">共 {{ filteredTemplates.length }} 个模板</div>
        </div>
      </div>

      <div class="templates-grid">
        <TemplateCard
          v-for="template in filteredTemplates"
          :key="template.id"
          :template="template"
          :is-selected="selectedTemplate?.id === template.id"
          @select="selectTemplate"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import FilterSection from './template/FilterSection.vue';
import TemplateCard from './template/TemplateCard.vue';

// Props
const props = defineProps({
  templates: {
    type: Array,
    default: () => []
  }
});

// Emits
const emit = defineEmits(['update:template', 'update:variables']);

// 模拟数据
const categories = ref([
  { id: 'all', name: '全部' },
  { id: 'web', name: 'Web 应用' },
  { id: 'cli', name: 'CLI 工具' },
  { id: 'desktop', name: '桌面应用' }
]);

const languages = ref([
  { id: 'all', name: '全部' },
  { id: 'rust', name: 'Rust' },
  { id: 'go', name: 'Go' },
  { id: 'python', name: 'Python' },
  { id: 'javascript', name: 'JavaScript' },
  { id: 'typescript', name: 'TypeScript' }
]);

// 状态
const searchKeyword = ref('');
const selectedCategory = ref('all');
const selectedLanguage = ref('all');
const selectedTemplate = ref(null);

// 过滤后的模板
const filteredTemplates = computed(() => {
  let result = props.templates;

  // 搜索过滤
  if (searchKeyword.value.trim()) {
    const keyword = searchKeyword.value.toLowerCase();
    result = result.filter(t => {
      const name = t.name?.toLowerCase() || '';
      const desc = t.description?.toLowerCase() || '';
      return name.includes(keyword) || desc.includes(keyword);
    });
  }

  // 分类过滤
  if (selectedCategory.value !== 'all') {
    result = result.filter(t => t.templateType === selectedCategory.value);
  }

  // 语言过滤
  if (selectedLanguage.value !== 'all') {
    result = result.filter(t => {
      const lang = t.language?.toLowerCase() || '';
      return lang.includes(selectedLanguage.value);
    });
  }

  return result;
});

const selectTemplate = async (template) => {
  selectedTemplate.value = template;

  // 加载模板变量
  try {
    const varsJson = await invoke('get_template_variables', {
      templateId: template.id
    });

    // 解析 schema 并设置默认值
    const defaults = {};
    try {
      const schema = JSON.parse(varsJson);
      const fields = schema.fields || [];
      fields.forEach(v => {
        if (v.default !== undefined) {
          defaults[v.name] = v.default;
        } else if (v.type === 'boolean') {
          defaults[v.name] = false;
        } else {
          defaults[v.name] = '';
        }
      });
    } catch (e) {
      // 无 schema 或解析失败，跳过
    }

    emit('update:template', template);
    emit('update:variables', defaults);
  } catch (error) {
    console.error('加载变量失败:', error);
  }
};

</script>

<style scoped>
.template-config {
  height: 100%;
  overflow-y: auto;
  padding: 0;
  background: #f5f7fa;
}

.templates-section {
  background: transparent;
}

.templates-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
  padding: 0 4px;
}

.templates-title-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.templates-header h2 {
  margin: 0;
  color: #333;
  font-size: 1.25rem;
  font-weight: 600;
}

.templates-count {
  color: #666;
  font-size: 14px;
}

.templates-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}
</style>
