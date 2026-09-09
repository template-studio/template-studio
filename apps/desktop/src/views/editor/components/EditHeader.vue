<template>
  <div class="edit-header">
    <div class="header-left">
      <div class="title-area">
        <span class="edit-title">模板编辑</span>
      </div>
      <!-- 文件树切换按钮 - 只在基础模板时隐藏 -->
      <a-tooltip>
        <template #title>文件树</template>
        <button
          v-if="mode !== 'basic'"
          class="action-icon"
          :class="{ active: isFileTreeVisible }"
          @click="$emit('toggle-file-tree')"
        >
          <FolderOpenOutline style="font-size: 18px" />
        </button>
      </a-tooltip>
      <!-- 变量侧边栏切换按钮 - 只在基础模板时隐藏 -->
      <a-tooltip>
        <template #title>变量面板</template>
        <button
          v-if="mode !== 'basic'"
          class="action-icon"
          :class="{ active: isVariableSidebarVisible }"
          @click="$emit('toggle-variable-sidebar')"
        >
          <VariableIcon :size="18" />
        </button>
      </a-tooltip>
      <div v-if="currentFileName" class="file-status">
        <span class="file-name">{{ currentFileName }}</span>
        <span v-if="hasUnsavedChanges" class="unsaved-indicator" title="有未保存的更改"></span>
      </div>
    </div>
    <div class="header-actions">
      <a-tooltip>
        <template #title>AI 助手</template>
        <button class="action-icon" :class="{ 'ai-active': aiOpen }" @click="$emit('toggle-ai')">
          <AiIcon :size="18" />
        </button>
      </a-tooltip>
      <a-tooltip>
        <template #title>高级设置</template>
        <button class="action-icon" @click="$emit('show-advanced')">
          <SettingsOutline style="font-size: 18px" />
        </button>
      </a-tooltip>
      <a-tooltip>
        <template #title>全量渲染</template>
        <button class="action-icon" @click="$emit('full-render')">
          <RenderIcon :size="18" />
        </button>
      </a-tooltip>
      <a-tooltip>
        <template #title>返回模板列表</template>
        <button class="action-icon" @click="$emit('close-edit')">
          <ArrowLeftOutlined style="font-size: 16px" />
        </button>
      </a-tooltip>

      <!-- 窗口控制(独立全屏页自带,样式与主布局一致) -->
      <div class="window-controls">
        <button class="action-icon" title="最小化" @click="minimizeWindow">
          <MinusOutlined style="font-size: 16px" />
        </button>
        <button class="action-icon" title="最大化/还原" @click="maximizeWindow">
          <BorderOutlined style="font-size: 13px" />
        </button>
        <button class="action-icon win-close" title="关闭窗口" @click="closeWindow">
          <CloseOutline style="font-size: 18px" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
  import {
    FolderOpenOutline,
    SettingsOutline,
    PlayOutline,
    CloseOutline,
  } from '@/icons/ionicons5';
  import { MinusOutlined, BorderOutlined, ArrowLeftOutlined } from '@ant-design/icons-vue';
  import AiIcon from '@/components/icons/AiIcon.vue';
  import RenderIcon from '@/components/icons/RenderIcon.vue';
  import VariableIcon from '@/components/icons/VariableIcon.vue';
  import { tauriApi } from '@/utils/tauriApi';

  const props = defineProps({
    isFileTreeVisible: {
      type: Boolean,
      required: true,
    },
    isVariableSidebarVisible: {
      type: Boolean,
      default: false,
    },
    hasUnsavedChanges: {
      type: Boolean,
      default: false,
    },
    currentFileName: {
      type: String,
      default: '',
    },
    mode: {
      type: String,
      default: 'basic',
    },
    aiOpen: {
      type: Boolean,
      default: false,
    },
  });

  const emit = defineEmits([
    'toggle-variable-sidebar',
    'close-edit',
    'toggle-file-tree',
    'show-advanced',
    'full-render',
    'toggle-ai',
  ]);

  const minimizeWindow = async () => {
    try {
      await tauriApi.window.minimize();
      document.activeElement.blur();
    } catch (error) {
      console.error('Failed to minimize window:', error);
    }
  };

  const maximizeWindow = async () => {
    try {
      await tauriApi.window.maximize();
      document.activeElement.blur();
    } catch (error) {
      console.error('Failed to maximize window:', error);
    }
  };

  const closeWindow = async () => {
    try {
      await tauriApi.window.close();
    } catch (error) {
      console.error('Failed to close window:', error);
    }
  };
</script>

<style scoped>
  .edit-header {
    /* 无边框窗口的标题栏拖拽区(按钮等交互元素单独 no-drag) */
    -webkit-app-region: drag;
    user-select: none;
    height: 48px;
    background: var(--editor-panel-bg, #ffffff);
    border-bottom: 1px solid var(--editor-border, #e2e8f0);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .title-area {
    display: flex;
    align-items: center;
    margin-right: 4px;
  }

  .edit-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--editor-primary, #1e293b);
    letter-spacing: -0.2px;
  }

  .action-icon.active {
    background: transparent;
    color: var(--editor-accent, #16a34a);
  }

  .file-status {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-left: 4px;
  }

  .file-name {
    font-size: 13px;
    color: var(--editor-muted, #64748b);
    font-weight: 500;
  }

  .unsaved-indicator {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #f59e0b;
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    -webkit-app-region: no-drag;
  }

  .header-left {
    -webkit-app-region: no-drag;
  }

  .action-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    border-radius: 4px;
    cursor: pointer;
    color: var(--editor-muted, #64748b);
    transition: all 0.15s ease;
  }

  .action-icon:hover {
    background: var(--editor-hover-bg, #f1f5f9);
    color: var(--editor-primary, #1e293b);
  }

  .action-icon.ai-active {
    background: transparent;
    color: var(--editor-accent, #16a34a);
  }

  .window-controls {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-left: 10px;
    padding-left: 10px;
    border-left: 1px solid var(--editor-border, #e2e8f0);
  }

  .action-icon.win-close:hover {
    background: #ff4757;
    color: #fff;
  }

  @media (prefers-reduced-motion: reduce) {
    .unsaved-indicator { animation: none; }
    .action-icon { transition: none; }
  }
</style>
