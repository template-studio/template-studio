<template>
  <div class="template-config">
    <!-- 筛选区域 -->
    <div class="filters-section">
      <!-- 搜索框 -->
      <div class="filter-block search-block">
        <n-input
          v-model:value="searchKeyword"
          placeholder="搜索模板名称或描述..."
          size="large"
          clearable
          @update:value="handleSearch"
        >
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
      </div>

      <!-- 分类 -->
      <div class="filter-block">
        <h2>分类</h2>
        <div class="tiles">
          <div
            v-for="cat in categories"
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
            v-for="lang in languages"
            :key="lang.id"
            class="tile"
            :class="{ active: selectedLanguage === lang.id }"
            @click="selectLanguage(lang.id)"
          >
            {{ lang.name }}
          </div>
        </div>
      </div>
    </div>

    <!-- 模板列表 -->
    <div class="templates-section">
      <div class="templates-header">
        <div class="templates-title-section">
          <h2>模板列表</h2>
          <div class="templates-count">共 {{ filteredTemplates.length }} 个模板</div>
        </div>
      </div>

      <div class="templates-grid">
        <div
          v-for="template in filteredTemplates"
          :key="template.id"
          class="template-card"
          :class="{ selected: selectedTemplate?.id === template.id }"
          @click="selectTemplate(template)"
        >
          <div class="card-visual-area">
            <div class="visual-bg">
              <div class="code-snippet-preview">{{ getCodeSnippet(template) }}</div>
            </div>
            <div v-if="template.isFeatured" class="template-badge">
              <span>推荐</span>
            </div>
          </div>

          <div class="card-content-area">
            <h4 class="template-name">{{ template.name }}</h4>
            <p class="template-description">{{ template.description }}</p>

            <div class="template-languages">
              <n-tag
                v-if="template.language"
                :color="{ color: '#f0f0f0', textColor: '#666' }"
                size="small"
              >
                {{ template.language }}
              </n-tag>
            </div>

            <div class="card-footer">
              <div class="card-author">
                <div class="author-avatar"></div>
                <span class="author-name">Template Studio</span>
              </div>
              <div class="template-type">
                <span class="type-badge">{{ template.templateType }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { NTag, NInput, NIcon } from 'naive-ui';
import { SearchOutline } from '@vicons/ionicons5';
import { invoke } from '@tauri-apps/api/core';

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

// 方法
const handleSearch = () => {
  // 搜索逻辑在 computed 中自动处理
};

const selectCategory = (catId) => {
  selectedCategory.value = catId;
};

const selectLanguage = (langId) => {
  selectedLanguage.value = langId;
};

const selectTemplate = async (template) => {
  selectedTemplate.value = template;

  // 加载模板变量
  try {
    const vars = await invoke('get_template_variables', {
      templateId: template.id
    });

    // 设置默认值
    const defaults = {};
    vars.forEach(v => {
      if (v.default_value !== undefined) {
        defaults[v.name] = v.default_value;
      } else if (v.type_ === 'boolean') {
        defaults[v.name] = false;
      } else {
        defaults[v.name] = '';
      }
    });

    emit('update:template', template);
    emit('update:variables', defaults);
  } catch (error) {
    console.error('加载变量失败:', error);
  }
};

// 生成代码片段预览
const getCodeSnippet = (template) => {
  const lang = template.language?.toLowerCase() || '';
  const name = template.name || 'Template';

  if (lang.includes('rust')) {
    `fn main() {
    println!("Hello, ${name}!");
}`;
  }

  if (lang.includes('go') || lang.includes('golang')) {
    return `package main

import "fmt"

func main() {
    fmt.Printf("Hello, ${name}!\\n")
}`;
  }

  if (lang.includes('python')) {
    return `def main():
    print("Hello, ${name}!")

if __name__ == "__main__":
    main()`;
  }

  if (lang.includes('javascript') || lang.includes('typescript')) {
    return `function main() {
    console.log('Hello, ${name}!');
}

main();`;
  }

  // 默认
  return `// ${name}
class Application {
  constructor() {
    this.name = '${name}';
  }

  run() {
    console.log('Running', this.name);
  }
}

const app = new Application();
app.run();`;
};
</script>

<style scoped>
.template-config {
  height: 100%;
  overflow-y: auto;
  padding: 0;
  background: #f5f7fa;
}

/* 筛选区域样式 */
.filters-section {
  background: #fff;
  padding: 20px;
  border-radius: 0;
  margin-bottom: 0;
  box-shadow: none;
  border-bottom: 1px solid #f0f0f0;
}

/* 筛选区域样式 */
.filters-section {
  background: #fff;
  padding: 20px;
  border-radius: 12px;
  margin-bottom: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.filter-block {
  margin-bottom: 20px;
}

.filter-block:last-child {
  margin-bottom: 0;
}

.search-block {
  margin-bottom: 24px;
}

.search-block :deep(.n-input) {
  border-radius: 12px;
}

.search-block :deep(.n-input__input) {
  font-size: 15px;
}

.filter-block h2 {
  font-size: 1rem;
  font-weight: 600;
  color: #333;
  margin: 0 0 12px 0;
}

/* 磁贴容器 */
.tiles {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

/* 磁贴 */
.tile {
  background: #fff;
  border-radius: 12px;
  padding: 8px 16px;
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  transition: all 0.25s ease;
  display: inline-flex;
  align-items: center;
  font-size: 0.875rem;
  color: #333;
  font-weight: 500;
  border: 1px solid transparent;
}

.tile:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.1);
  color: #4285f4;
  border-color: rgba(66, 133, 244, 0.1);
}

.tile.active {
  background: linear-gradient(135deg, #4285f4 0%, #34a853 100%);
  color: #fff;
  box-shadow: 0 4px 14px rgba(66, 133, 244, 0.3);
}

.tile.active:hover {
  background: linear-gradient(135deg, #3b78e7 0%, #2d9249 100%);
  color: #fff;
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(66, 133, 244, 0.4);
}

/* 模板列表样式 */
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

/* 模板网格 */
.templates-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

/* 模板卡片 */
.template-card {
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(12px);
  border-radius: 16px;
  box-shadow: 0 1px 2px 0 rgba(60, 64, 67, 0.1), 0 1px 3px 1px rgba(60, 64, 67, 0.05);
  overflow: hidden;
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  cursor: pointer;
  position: relative;
  border: 2px solid transparent;
}

.template-card:hover {
  transform: translateY(-6px) scale(1.02);
  box-shadow: 0 12px 40px rgba(66, 133, 244, 0.15);
  background: rgba(255, 255, 255, 0.95);
}

.template-card.selected {
  border-color: #4285f4;
  box-shadow: 0 0 0 4px rgba(66, 133, 244, 0.1);
}

/* 上方视觉区域 */
.card-visual-area {
  width: 100%;
  height: 160px;
  position: relative;
  overflow: hidden;
}

.visual-bg {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #4285f4 0%, #34a853 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

/* Shimmer光泽效果 */
.visual-bg::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(
    45deg,
    transparent 30%,
    rgba(255, 255, 255, 0.2) 50%,
    transparent 70%
  );
  animation: shimmer 3s infinite;
}

@keyframes shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}

.code-snippet-preview {
  font-family: 'Courier New', 'Consolas', 'Monaco', monospace;
  font-size: 9px;
  line-height: 1.4;
  color: rgba(255, 255, 255, 0.4);
  white-space: pre;
  overflow: hidden;
  padding: 16px;
  text-align: left;
  position: relative;
  z-index: 1;
}

.template-badge {
  position: absolute;
  top: 12px;
  right: 12px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(8px);
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  color: #4285f4;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  z-index: 2;
}

/* 卡片内容区域 */
.card-content-area {
  padding: 16px;
  background: #fff;
}

.template-name {
  font-size: 16px;
  font-weight: 600;
  color: #202124;
  margin: 0 0 8px 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.template-description {
  font-size: 13px;
  color: #5f6368;
  margin: 0 0 12px 0;
  line-height: 1.6;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  min-height: 42px;
}

.template-languages {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 12px;
}

/* 卡片底部 */
.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 12px;
  border-top: 1px solid rgba(0, 0, 0, 0.06);
}

.card-author {
  display: flex;
  align-items: center;
  gap: 8px;
}

.author-avatar {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: linear-gradient(135deg, #4285f4 0%, #34a853 100%);
}

.author-name {
  font-size: 12px;
  color: #666;
  font-weight: 500;
}

.template-type {
  display: flex;
  align-items: center;
}

.type-badge {
  font-size: 11px;
  color: #999;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
</style>
