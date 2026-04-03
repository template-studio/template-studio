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
          size="small"
        >
          {{ getLanguageName(lang.languageId) }}
        </n-tag>
      </div>

      <div class="card-footer">
        <div class="card-author" @click.stop="goProfile">
          <div class="author-avatar">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>
          </div>
          <span class="author-name">{{ template.ownerName }}</span>
        </div>
        <div class="creation-time">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
          <span>{{ formatCreationTime(template.createdAt) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
  import { computed } from 'vue';
  import { NTag } from 'naive-ui';
  import { useRouter } from 'vue-router';
  import { useLanguageStore } from '@/store/modules/languageStore';
  import { storeToRefs } from 'pinia';

  const router = useRouter();

  const props = defineProps({
    template: {
      type: Object,
      required: true,
    },
  });

  const emit = defineEmits(['click']);

  const languageStore = useLanguageStore();
  const { languagesList } = storeToRefs(languageStore);

  const getLanguageName = (languageId) => {
    if (!languageId) return '';
    const language = languagesList.value.find((lang) => lang.id === Number(languageId));
    return language ? language.name : '';
  };

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

  const handleClick = () => {
    emit('click', props.template);
  };

  const goProfile = () => {
    if (props.template.ownerName) {
      router.push(`/u/${props.template.ownerName}`);
    }
  };
</script>

<style scoped>
.template-card {
  background: var(--client-card-bg, #ffffff);
  border-radius: var(--client-card-radius, 12px);
  box-shadow: var(--client-card-shadow, none);
  overflow: hidden;
  transition: all 0.25s ease-out;
  cursor: pointer;
  position: relative;
  border: var(--client-card-border, 1px solid #e2e8f0);
}

.template-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--client-card-hover-shadow, 0 12px 32px rgba(15, 23, 42, 0.12));
  border-color: var(--client-theme-color);
}

.card-visual-area {
  width: 100%;
  height: 160px;
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
  background: linear-gradient(
    90deg,
    transparent 0%,
    rgba(var(--client-theme-rgb), 0.03) 45%,
    rgba(var(--client-theme-rgb), 0.08) 50%,
    rgba(var(--client-theme-rgb), 0.03) 55%,
    transparent 100%
  );
  animation: shimmer 4s ease-in-out infinite;
}

@keyframes shimmer {
  0% { transform: translateX(0); }
  100% { transform: translateX(50%); }
}

.code-snippet-preview {
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

.template-badge {
  position: absolute;
  top: 10px;
  right: 10px;
  background: rgba(var(--client-theme-rgb), 0.9);
  backdrop-filter: blur(8px);
  padding: 3px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  color: #fff;
  z-index: 2;
  letter-spacing: 0.3px;
}

.card-content-area {
  padding: 16px 20px 20px;
}

.template-name {
  font-size: 16px;
  font-weight: 600;
  color: #0f172a;
  margin: 0 0 6px 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  letter-spacing: -0.2px;
}

.template-card:hover .template-name {
  color: var(--client-theme-color);
}

.template-description {
  font-size: 13px;
  color: #64748b;
  margin: 0 0 12px 0;
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
  margin-bottom: 14px;
}

.template-languages :deep(.n-tag) {
  background: #f1f5f9;
  border: 1px solid #e2e8f0;
  color: #475569;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.template-card:hover .template-languages :deep(.n-tag) {
  background: var(--client-theme-bg-light);
  border-color: var(--client-theme-border-light);
  color: var(--client-theme-dark);
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 12px;
  border-top: 1px solid #f1f5f9;
}

.card-author {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.card-author:hover .author-name {
  color: var(--client-theme-color);
}

.author-avatar {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  background: linear-gradient(135deg, #0f172a 0%, #334155 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.author-name {
  font-size: 12px;
  color: #64748b;
  font-weight: 500;
}

.creation-time {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #94a3b8;
}

@media (prefers-reduced-motion: reduce) {
  .visual-bg::before { animation: none; }
  .template-card { transition: none; }
}
</style>
