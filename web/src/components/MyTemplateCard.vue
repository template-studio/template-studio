<template>
  <div class="my-template-card" @click="goToEdit" @contextmenu.prevent="showContextMenu">
    <div class="card-visual-area">
      <div class="visual-bg">
        <div class="code-snippet-preview">{{ codeSnippet }}</div>
      </div>
      <div v-if="template.isFeatured" class="featured-badge">
        <span>⭐ 推荐</span>
      </div>
      <div class="status-badge" :class="statusClass">
        <span>{{ statusLabel }}</span>
      </div>
      <div class="visibility-badge" :class="visibilityClass">
        <span>{{ visibilityLabel }}</span>
      </div>
    </div>

    <div class="card-body">
      <h3 class="template-title">{{ template.name }}</h3>
      <p class="template-description">{{ template.description }}</p>

      <div class="template-meta">
        <div class="languages-section">
          <span class="meta-label">支持语言:</span>
          <div class="language-tags">
            <n-tag
              v-for="lang in template.languages"
              :key="lang.id || lang.languageId"
              size="small"
              :type="lang.isPrimary || lang.is_primary ? 'primary' : 'default'"
            >
              {{ getLanguageName(lang.languageId || lang.id) }}
            </n-tag>
          </div>
        </div>

        <div class="template-stats">
          <div class="stat-item">
            <n-icon size="16">
              <EyeOutline />
            </n-icon>
            <span>{{ template.usageCount || 0 }}</span>
          </div>
          <div class="stat-item">
            <n-icon size="16">
              <TimeOutline />
            </n-icon>
            <span>{{ formatDate(template.createdAt) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 右键菜单 -->
    <n-dropdown
      :show="showDropdown"
      :options="dropdownOptions"
      :x="dropdownX"
      :y="dropdownY"
      placement="bottom-start"
      @clickoutside="hideContextMenu"
      @select="handleMenuSelect"
    />
  </div>
</template>

<script setup>
  import { ref, computed, h, nextTick } from 'vue';
  import { useRouter } from 'vue-router';
  import { NTag, NDropdown, NIcon, useMessage } from 'naive-ui';
  import {
    EyeOutline,
    PencilOutline,
    ShareOutline,
    TrashOutline,
    TimeOutline,
    CheckmarkCircleOutline,
    PauseOutline,
    ArchiveOutline,
    SendOutline,
  } from '@vicons/ionicons5';
  import { useLanguageStore } from '@/store/modules/languageStore';
  import { storeToRefs } from 'pinia';

  const props = defineProps({
    template: {
      type: Object,
      required: true,
    },
  });

  const emit = defineEmits([
    'edit',
    'share',
    'delete',
    'publish',
    'archive',
    'republish',
    'withdraw',
  ]);

  const router = useRouter();
  const message = useMessage();
  const languageStore = useLanguageStore();
  const { languagesList } = storeToRefs(languageStore);

  // 右键菜单状态
  const showDropdown = ref(false);
  const dropdownX = ref(0);
  const dropdownY = ref(0);

  // 获取语言名称
  const getLanguageName = (languageId) => {
    if (!languageId) return '';
    if (!languagesList.value || languagesList.value.length === 0) return `语言${languageId}`;
    const language = languagesList.value.find((lang) => lang.id === Number(languageId));
    return language ? language.name : `未知语言(${languageId})`;
  };

  // 生成代码片段预览
  const codeSnippet = computed(() => {
    const template = props.template;
    const name = template.name?.toLowerCase() || '';
    const mainLang = template.languages?.[0];
    const langName = mainLang
      ? getLanguageName(mainLang.languageId || mainLang.id)?.toLowerCase()
      : '';

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

  // 旧的可见性配置已删除，使用下面的新定义

  // 格式化日期
  const formatDate = (dateString) => {
    if (!dateString) return '';
    const date = new Date(dateString);
    const now = new Date();
    const diff = now - date;
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (days === 0) return '今天';
    if (days === 1) return '昨天';
    if (days < 7) return `${days}天前`;
    if (days < 30) return `${Math.floor(days / 7)}周前`;
    return date.toLocaleDateString('zh-CN');
  };

  // 动态右键菜单选项
  const dropdownOptions = computed(() => {
    const status = props.template.status || 'draft';
    const options = [
      {
        label: '编辑模板',
        key: 'edit',
        icon: () => h(NIcon, { size: 16 }, { default: () => h(PencilOutline) }),
      },
      {
        label: '分享模板',
        key: 'share',
        icon: () => h(NIcon, { size: 16 }, { default: () => h(ShareOutline) }),
      },
    ];

    // 根据状态添加不同的操作
    if (status === 'draft') {
      options.push({
        label: '发布模板',
        key: 'publish',
        icon: () => h(NIcon, { size: 16 }, { default: () => h(SendOutline) }),
      });
    } else if (status === 'published') {
      options.push({
        label: '暂停发布',
        key: 'archive',
        icon: () => h(NIcon, { size: 16 }, { default: () => h(PauseOutline) }),
      });
    } else if (status === 'archived') {
      options.push({
        label: '重新发布',
        key: 'republish',
        icon: () => h(NIcon, { size: 16 }, { default: () => h(CheckmarkCircleOutline) }),
      });
    } else if (status === 'pending_review') {
      options.push({
        label: '撤回审核',
        key: 'withdraw',
        icon: () => h(NIcon, { size: 16 }, { default: () => h(ArchiveOutline) }),
      });
    }

    options.push({ type: 'divider' });
    options.push({
      label: '删除模板',
      key: 'delete',
      icon: () => h(NIcon, { size: 16 }, { default: () => h(TrashOutline) }),
    });

    return options;
  });

  // 显示右键菜单
  const showContextMenu = (e) => {
    showDropdown.value = false;
    nextTick(() => {
      dropdownX.value = e.clientX;
      dropdownY.value = e.clientY;
      showDropdown.value = true;
    });
  };

  // 隐藏右键菜单
  const hideContextMenu = () => {
    showDropdown.value = false;
  };

  // 处理菜单选择
  const handleMenuSelect = (key) => {
    hideContextMenu();

    switch (key) {
      case 'edit':
        goToEdit();
        break;
      case 'share':
        shareTemplate();
        break;
      case 'publish':
        publishTemplate();
        break;
      case 'archive':
        archiveTemplate();
        break;
      case 'republish':
        republishTemplate();
        break;
      case 'withdraw':
        withdrawTemplate();
        break;
      case 'delete':
        deleteTemplate();
        break;
    }
  };

  // 操作方法
  const goToEdit = () => {
    router.push(`/edit-template/${props.template.id}`);
  };

  const shareTemplate = () => {
    const shareUrl = `${window.location.origin}/templates/${props.template.id}`;
    navigator.clipboard.writeText(shareUrl);
    message.success('分享链接已复制到剪贴板');
  };

  const deleteTemplate = () => {
    emit('delete', props.template);
  };

  // 发布模板
  const publishTemplate = () => {
    emit('publish', props.template);
  };

  // 暂停发布
  const archiveTemplate = () => {
    emit('archive', props.template);
  };

  // 重新发布
  const republishTemplate = () => {
    emit('republish', props.template);
  };

  // 撤回审核
  const withdrawTemplate = () => {
    emit('withdraw', props.template);
  };

  // 计算属性
  const visibilityLabel = computed(() => {
    const visibility = props.template.visibility || 'private';
    const visibilityMap = {
      public: '公开',
      private: '私有',
      organization: '组织',
      shared: '共享',
    };
    return visibilityMap[visibility] || '私有';
  });

  const visibilityClass = computed(() => {
    const visibility = props.template.visibility || 'private';
    return {
      'visibility-public': visibility === 'public',
      'visibility-private': visibility === 'private',
      'visibility-organization': visibility === 'organization',
      'visibility-shared': visibility === 'shared',
    };
  });

  const statusClass = computed(() => {
    const status = props.template.status || 'draft';
    return {
      'status-draft': status === 'draft',
      'status-published': status === 'published',
      'status-archived': status === 'archived',
      'status-pending_review': status === 'pending_review',
    };
  });

  const statusLabel = computed(() => {
    const status = props.template.status || 'draft';
    const statusMap = {
      draft: '草稿',
      published: '已发布',
      archived: '已归档',
      pending_review: '待审核',
    };
    return statusMap[status] || '草稿';
  });

  // formatDate 函数已在上文定义，此处删除重复定义
</script>

<style scoped>
  .my-template-card {
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

  .my-template-card:hover {
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

  .featured-badge,
  .visibility-badge {
    position: absolute;
    top: 12px;
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
    backdrop-filter: blur(8px);
    z-index: 2;
  }

  .featured-badge {
    right: 12px;
    background: rgba(255, 255, 255, 0.95);
    color: #f59e0b;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .visibility-badge {
    left: 12px;
    background: rgba(255, 255, 255, 0.95);
    color: #4285f4;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .visibility-badge.private {
    color: #ef4444;
  }

  .visibility-badge.organization {
    color: #8b5cf6;
  }

  .visibility-badge.shared {
    color: #6b7280;
  }

  .status-badge {
    position: absolute;
    top: 50px;
    right: 12px;
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
    backdrop-filter: blur(8px);
    z-index: 2;
  }

  .status-draft {
    background: rgba(148, 163, 184, 0.95);
    color: #334155;
  }

  .status-published {
    background: rgba(34, 197, 94, 0.95);
    color: white;
  }

  .status-archived {
    background: rgba(239, 68, 68, 0.95);
    color: white;
  }

  .status-pending_review {
    background: rgba(245, 158, 11, 0.95);
    color: white;
  }

  .card-body {
    padding: 20px;
    background: #fff;
  }

  .template-title {
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

  .template-meta {
    margin-bottom: 16px;
  }

  .languages-section {
    margin-bottom: 12px;
  }

  .meta-label {
    font-size: 12px;
    color: #80868b;
    font-weight: 500;
    display: block;
    margin-bottom: 6px;
  }

  .language-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .template-stats {
    display: flex;
    gap: 16px;
    font-size: 13px;
    color: #80868b;
  }

  .stat-item {
    display: flex;
    align-items: center;
    gap: 4px;
  }
</style>
