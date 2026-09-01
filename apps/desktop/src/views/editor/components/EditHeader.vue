<template>
  <div class="edit-header">
    <div class="header-left">
      <div class="title-area">
        <span class="edit-title">模板编辑</span>
      </div>
      <!-- 文件树切换按钮 - 只在基础模板时隐藏 -->
      <button
        v-if="mode !== 'basic'"
        class="toggle-btn"
        :class="{ active: isFileTreeVisible }"
        @click="$emit('toggle-file-tree')"
        title="文件树"
      >
        <FolderOpenOutline style="font-size: 16px" />
        <span class="toggle-text">文件树</span>
      </button>
      <!-- 变量侧边栏切换按钮 - 只在基础模板时隐藏 -->
      <button
        v-if="mode !== 'basic'"
        class="toggle-btn variable-toggle"
        :class="{ active: isVariableSidebarVisible }"
        @click="$emit('toggle-variable-sidebar')"
        title="变量面板"
      >
        <VariablesOutline style="font-size: 16px" />
        <span class="toggle-text">变量</span>
      </button>
      <div v-if="currentFileName" class="file-status">
        <span class="file-name">{{ currentFileName }}</span>
        <span v-if="hasUnsavedChanges" class="unsaved-indicator" title="有未保存的更改"></span>
      </div>
    </div>
    <div class="header-actions">
      <a-tooltip>
        <template #title>高级设置</template>
        <button class="action-icon" @click="$emit('show-advanced')">
          <SettingsOutline style="font-size: 18px" />
        </button>
      </a-tooltip>
      <a-tooltip>
        <template #title>全量渲染</template>
        <button class="action-icon" @click="$emit('full-render')">
          <PlayOutline style="font-size: 18px" />
        </button>
      </a-tooltip>
      <a-tooltip>
        <template #title>关闭编辑器</template>
        <button class="action-icon action-close" @click="$emit('close-edit')">
          <CloseOutline style="font-size: 18px" />
        </button>
      </a-tooltip>
    </div>
  </div>
</template>

<script setup>
  import { h } from 'vue';
  import {
    FolderOpenOutline,
    SettingsOutline,
    PlayOutline,
    CloseOutline,
  } from '@/icons/ionicons5';

  const VariablesOutline = () =>
    h(
      'svg',
      {
        viewBox: '0 0 24 24',
        width: 16,
        height: 16,
        fill: 'none',
        stroke: 'currentColor',
        'stroke-width': 2,
        'stroke-linecap': 'round',
        'stroke-linejoin': 'round',
      },
      [h('path', { d: 'M16 6l2.29 2.29-4.88 4.88-4-4L2 16.59 3.41 18l6-6 4 4 6.3-6.29L22 12V6z' })]
    );

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
  });

  const emit = defineEmits([
    'toggle-variable-sidebar',
    'close-edit',
    'toggle-file-tree',
    'show-advanced',
    'full-render',
  ]);
</script>

<style scoped>
  .edit-header {
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

  .toggle-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    background: transparent;
    border: 1px solid var(--editor-border, #e2e8f0);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
    user-select: none;
    color: var(--editor-muted, #64748b);
    font-size: 13px;
    line-height: 1;
  }

  .toggle-btn:hover {
    background: var(--editor-hover-bg, #f1f5f9);
    border-color: var(--editor-muted, #94a3b8);
  }

  .toggle-btn.active {
    background: var(--editor-active-bg, #ecfdf5);
    border-color: var(--editor-accent, #22c55e);
    color: var(--editor-accent, #22c55e);
  }

  .toggle-btn.variable-toggle.active {
    background: #f5f3ff;
    border-color: #8b5cf6;
    color: #8b5cf6;
  }

  .toggle-text {
    font-size: 13px;
    font-weight: 500;
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

  .action-icon.action-close:hover {
    background: #fef2f2;
    color: #ef4444;
  }

  @media (prefers-reduced-motion: reduce) {
    .unsaved-indicator { animation: none; }
    .toggle-btn, .action-icon { transition: none; }
  }
</style>
