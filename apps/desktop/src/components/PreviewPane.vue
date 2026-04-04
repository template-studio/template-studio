<template>
  <div class="preview-pane">
    <n-card title="👁️ 实时预览" size="small" :bordered="false">
      <template #header-extra>
        <n-space size="small">
          <n-tag size="small" :type="previewStatus.type">
            {{ previewStatus.text }}
          </n-tag>
          <n-button size="small" type="primary" @click="generateProject" :disabled="!templateId">
            <template #icon>
              <n-icon><Flash /></n-icon>
            </template>
            生成项目
          </n-button>
        </n-space>
      </template>

      <!-- 文件预览 -->
      <div class="preview-content">
        <!-- 未选择模板时的提示 -->
        <n-empty
          v-if="!templateId"
          description="请先选择一个模板"
          size="large"
        >
          <template #icon>
            <n-icon size="64" color="#ccc">
              <Document />
            </n-icon>
          </template>
          <template #extra>
            <n-text depth="3" style="font-size: 13px;">
              从左侧列表选择模板后，这里将显示实时预览
            </n-text>
          </template>
        </n-empty>

        <!-- 文件树 -->
        <div class="file-tree" v-else-if="previewFiles.length > 0">
          <n-tree
            :data="fileTreeData"
            :show-irrelevant-nodes="false"
            :expanded-keys="expandedKeys"
            @update:expanded-keys="handleExpand"
            @update:selected-keys="handleSelectFile"
            key-field="key"
            children-field="children"
          />
        </div>

        <!-- 文件内容 -->
        <div v-if="selectedFile" class="file-content">
          <div class="file-header">
            <n-icon size="16"><Document /></n-icon>
            <span class="file-name">{{ selectedFile.name }}</span>
          </div>
          <n-code
            :language="getLanguage(selectedFile.name)"
            :code="selectedFile.content"
          />
        </div>

        <n-empty
          v-else-if="previewFiles.length === 0"
          description="暂无预览内容"
          size="small"
        />
      </div>
    </n-card>
  </div>
</template>

<script setup>
import { ref, computed, watch, h } from 'vue';
import {
  NCard, NSpace, NTag, NButton, NIcon, NTree, NCode, NEmpty
} from 'naive-ui';
import { Refresh, Document, Flash } from '@vicons/ionicons5';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps({
  templateId: {
    type: String,
    default: ''
  },
  variables: {
    type: Object,
    default: () => ({})
  }
});

const previewFiles = ref([]);
const selectedFile = ref(null);
const expandedKeys = ref([]);

const previewStatus = computed(() => {
  if (previewFiles.value.length === 0) {
    return { type: 'default', text: '未渲染' };
  }
  return { type: 'success', text: `已生成 ${previewFiles.value.length} 个文件` };
});

// 构建文件树数据
const fileTreeData = computed(() => {
  const buildTree = (files, parentPath = '') => {
    const tree = [];
    const pathMap = new Map();

    files.forEach((file, index) => {
      const parts = file.path.split('/');
      let currentPath = '';

      parts.forEach((part, partIndex) => {
        const prevPath = currentPath;
        currentPath = partIndex === 0 ? part : `${prevPath}/${part}`;

        if (!pathMap.has(currentPath)) {
          const isDir = partIndex < parts.length - 1;
          const node = {
            key: currentPath,
            label: part,
            isLeaf: !isDir,
            children: []
          };

          pathMap.set(currentPath, node);

          if (partIndex === 0) {
            tree.push(node);
          } else {
            const parent = pathMap.get(prevPath);
            if (parent) {
              parent.children.push(node);
            }
          }
        }
      });
    });

    return tree;
  };

  return buildTree(previewFiles.value);
});

// 方法
const handleExpand = (keys) => {
  expandedKeys.value = keys;
};

const handleSelectFile = (keys) => {
  if (keys.length > 0) {
    const file = previewFiles.value.find(f => f.path === keys[0]);
    selectedFile.value = file || null;
  }
};

const getLanguage = (filename) => {
  const ext = filename.split('.').pop().toLowerCase();
  const langMap = {
    'js': 'javascript',
    'ts': 'typescript',
    'vue': 'vue',
    'jsx': 'javascript',
    'tsx': 'typescript',
    'json': 'json',
    'md': 'markdown',
    'rs': 'rust',
    'go': 'go',
    'py': 'python',
    'java': 'java',
    'cpp': 'cpp',
    'c': 'c',
    'h': 'c',
    'css': 'css',
    'scss': 'scss',
    'less': 'less',
    'html': 'html',
    'xml': 'xml',
    'yaml': 'yaml',
    'toml': 'toml',
    'sh': 'bash',
    'zsh': 'bash'
  };
  return langMap[ext] || 'text';
};

const refreshPreview = async () => {
  if (!props.templateId) {
    previewFiles.value = [];
    selectedFile.value = null;
    return;
  }

  try {
    const files = await invoke('render_template', {
      templateId: props.templateId,
      variables: props.variables
    });

    previewFiles.value = files || [];

    // 自动展开第一层
    expandedKeys.value = previewFiles.value
      .map(f => {
        const parts = f.path.split('/');
        return parts.slice(0, -1).join('/');
      })
      .filter(p => p)
      .slice(0, 1);
  } catch (error) {
    console.error('预览失败:', error);
    window.$message?.error('预览失败: ' + error);
  }
};

const generateProject = async () => {
  if (!props.templateId) return;

  try {
    await invoke('generate_project', {
      templateId: props.templateId,
      variables: props.variables,
      outputPath: '~/projects'
    });

    window.$message?.success('项目生成成功！');
  } catch (error) {
    console.error('生成项目失败:', error);
    window.$message?.error(`生成失败: ${error}`);
  }
};

// 监听 props 变化
watch(() => [props.templateId, props.variables], () => {
  refreshPreview();
}, { deep: true, immediate: true });
</script>

<style scoped>
.preview-pane {
  height: 100%;
  padding: 16px;
  background: #f5f7fa;
}

:deep(.n-card) {
  height: 100%;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  border: none;
}

:deep(.n-card__header) {
  padding: 16px;
  border-bottom: 1px solid #e5e7eb;
  background: linear-gradient(180deg, #ffffff 0%, #f8f9fa 100%);
}

:deep(.n-card__content) {
  display: flex;
  flex-direction: column;
  height: calc(100% - 64px);
  padding: 0;
}

.preview-content {
  flex: 1;
  overflow: auto;
  display: flex;
  flex-direction: column;
  background: #ffffff;
}

.file-tree {
  flex: 1;
  overflow: auto;
  border-bottom: 1px solid #e5e7eb;
  padding: 12px;
  background: #fafbfc;
}

.file-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.file-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: linear-gradient(90deg, #f8f9fa 0%, #ffffff 100%);
  font-weight: 600;
  border-bottom: 1px solid #e5e7eb;
  color: #333;
}

.file-name {
  flex: 1;
  font-family: 'Fira Code', 'Courier New', monospace;
}

:deep(.n-code) {
  flex: 1;
  overflow: auto;
  border-radius: 0;
}

:deep(.n-empty) {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #fafbfc 0%, #f5f7fa 100%);
  border-radius: 8px;
  margin: 16px;
  padding: 40px 20px;
}

:deep(.n-empty__icon) {
  margin-bottom: 16px;
}

:deep(.n-empty__description) {
  font-size: 14px;
  color: #999;
  margin-bottom: 12px;
}

:deep(.n-empty__extra) {
  margin-top: 16px;
}

/* 文件树优化 */
:deep(.n-tree-node-content) {
  border-radius: 6px;
  padding: 4px 8px;
  transition: all 0.2s ease;
  margin: 2px 0;
}

:deep(.n-tree-node-content:hover) {
  background: rgba(24, 160, 88, 0.1);
  transform: translateX(4px);
}

:deep(.n-tree-node--selected > .n-tree-node-content) {
  background: linear-gradient(90deg, rgba(24, 160, 88, 0.15) 0%, rgba(24, 160, 88, 0.05) 100%);
  color: #18a058;
  font-weight: 600;
}

/* 按钮优化 */
:deep(.n-button) {
  border-radius: 8px;
  transition: all 0.2s ease;
  font-weight: 500;
}

:deep(.n-button:hover) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(24, 160, 88, 0.2);
}

:deep(.n-button:active) {
  transform: translateY(0);
}

/* 标签优化 */
:deep(.n-tag) {
  border-radius: 6px;
  font-weight: 500;
  transition: all 0.2s ease;
}

:deep(.n-tag:hover) {
  transform: scale(1.05);
}
</style>
