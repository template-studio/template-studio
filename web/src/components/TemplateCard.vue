<template>
  <div class="template-card" @click="handleClick">
    <div class="card-visual-area">
      <div class="visual-bg">
        <div class="code-snippet-preview">{{ codeSnippet }}</div>
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
          v-for="lang in template.languages"
          :key="lang.id"
          :color="{ color: '#f0f0f0', textColor: '#666' }"
          size="small"
        >
          {{ getLanguageName(lang.languageId) }}
        </n-tag>
      </div>

      <div class="card-footer">
        <div class="card-author">
          <div class="author-avatar"></div>
          <span class="author-name">Template Studio Lite</span>
        </div>
        <div class="creation-time">
          <span class="time-icon">📅</span>
          {{ formatCreationTime(template.createdAt) }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
  import { computed } from 'vue';
  import { NTag } from 'naive-ui';
  import { useLanguageStore } from '@/store/modules/languageStore';
  import { storeToRefs } from 'pinia';

  const props = defineProps({
    template: {
      type: Object,
      required: true,
    },
  });

  const emit = defineEmits(['click']);

  const languageStore = useLanguageStore();
  const { languagesList } = storeToRefs(languageStore);

  // 获取语言名称
  const getLanguageName = (languageId) => {
    if (!languageId) return '';
    const language = languagesList.value.find((lang) => lang.id === Number(languageId));
    return language ? language.name : '';
  };

  // 生成代码片段预览
  const codeSnippet = computed(() => {
    const template = props.template;
    const name = template.name?.toLowerCase() || '';
    const mainLang = template.languages?.[0];
    const langName = mainLang ? getLanguageName(mainLang.languageId)?.toLowerCase() : '';

    if (
      langName.includes('javascript') ||
      langName.includes('typescript') ||
      name.includes('vue') ||
      name.includes('react')
    ) {
      return `import { ${template.name.replace(/\s+/g, '')} } from './app'

function App() {
  const [state, setState] = useState()
  
  return (
    <div className="app">
      <h1>Hello World</h1>
    </div>
  )
}

export default App`;
    }

    if (langName.includes('python')) {
      return `class ${template.name.replace(/\s+/g, '')}:
    def __init__(self):
        self.name = "${template.name}"
    
    def run(self):
        print(f"Running {self.name}")
        return True

if __name__ == "__main__":
    app = ${template.name.replace(/\s+/g, '')}()
    app.run()`;
    }

    if (langName.includes('java')) {
      return `public class ${template.name.replace(/\s+/g, '')} {
    private String name;
    
    public ${template.name.replace(/\s+/g, '')}() {
        this.name = "${template.name}";
    }
    
    public void run() {
        System.out.println("Running " + name);
    }
    
    public static void main(String[] args) {
        ${template.name.replace(/\s+/g, '')} app = new ${template.name.replace(/\s+/g, '')}();
        app.run();
    }
}`;
    }

    if (langName.includes('go') || langName.includes('golang')) {
      return `package main

import "fmt"

type ${template.name.replace(/\s+/g, '')} struct {
    Name string
}

func (a *${template.name.replace(/\s+/g, '')}) Run() {
    fmt.Printf("Running %s\\n", a.Name)
}

func main() {
    app := &${template.name.replace(/\s+/g, '')}{
        Name: "${template.name}",
    }
    app.Run()
}`;
    }

    // 默认通用代码片段
    return `// ${template.name}
class Application {
  constructor() {
    this.name = '${template.name}'
  }
  
  init() {
    console.log('Initializing...')
    this.run()
  }
  
  run() {
    console.log('Running', this.name)
  }
}

const app = new Application()
app.init()`;
  });

  // 格式化创建时间
  const formatCreationTime = (createdAt) => {
    if (!createdAt) return '未知时间';

    try {
      const date = new Date(createdAt);
      const now = new Date();
      const diffTime = Math.abs(now - date);
      const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));

      if (diffDays === 0) {
        const diffHours = Math.floor(diffTime / (1000 * 60 * 60));
        if (diffHours === 0) {
          const diffMinutes = Math.floor(diffTime / (1000 * 60));
          return diffMinutes <= 0 ? '刚刚' : `${diffMinutes}分钟前`;
        }
        return `${diffHours}小时前`;
      } else if (diffDays === 1) {
        return '昨天';
      } else if (diffDays < 7) {
        return `${diffDays}天前`;
      } else if (diffDays < 30) {
        const weeks = Math.floor(diffDays / 7);
        return `${weeks}周前`;
      } else if (diffDays < 365) {
        const months = Math.floor(diffDays / 30);
        return `${months}个月前`;
      } else {
        const years = Math.floor(diffDays / 365);
        return `${years}年前`;
      }
    } catch (error) {
      return '时间格式错误';
    }
  };

  // 处理点击事件
  const handleClick = () => {
    emit('click', props.template);
  };
</script>

<style scoped>
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
    border: 1px solid rgba(255, 255, 255, 0.6);
    will-change: transform;
    backface-visibility: hidden;
  }

  .template-card:hover {
    transform: translateY(-8px) scale(1.02);
    box-shadow: 0 12px 40px rgba(66, 133, 244, 0.15);
    border-color: rgba(66, 133, 244, 0.3);
    background: rgba(255, 255, 255, 0.95);
  }

  /* 上方视觉区域 */
  .card-visual-area {
    width: 100%;
    height: 180px;
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
    font-size: 10px;
    line-height: 1.4;
    color: rgba(255, 255, 255, 0.4);
    white-space: pre;
    overflow: hidden;
    padding: 20px;
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
    padding: 20px;
    background: #fff;
  }

  .template-name {
    font-size: 18px;
    font-weight: 600;
    color: #202124;
    margin: 0 0 8px 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .template-description {
    font-size: 14px;
    color: #5f6368;
    margin: 0 0 16px 0;
    line-height: 1.6;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .template-languages {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 16px;
  }

  /* 卡片底部 */
  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 16px;
    border-top: 1px solid rgba(0, 0, 0, 0.06);
  }

  .card-author {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .author-avatar {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: linear-gradient(135deg, #4285f4 0%, #34a853 100%);
    flex-shrink: 0;
  }

  .author-name {
    font-size: 13px;
    color: #5f6368;
    font-weight: 500;
  }

  .creation-time {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
    color: #80868b;
  }

  .time-icon {
    font-size: 14px;
  }
</style>
