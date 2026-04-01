<template>
  <div class="edit-header">
    <div class="header-left">
      <div class="title-area">
        <span class="edit-title">模板编辑</span>
      </div>
      <!-- 文件树切换按钮 - 只在基础模板时隐藏 -->
      <div
        v-if="mode !== 'basic'"
        class="file-tree-toggle"
        :class="{ active: isFileTreeVisible }"
        @click="$emit('toggle-file-tree')"
      >
        <n-icon class="toggle-icon">
          <svg viewBox="0 0 24 24" width="16" height="16">
            <path fill="currentColor" d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z" />
          </svg>
        </n-icon>
        <span class="toggle-text">文件树</span>
      </div>
      <!-- 变量侧边栏切换按钮 - 只在基础模板时隐藏 -->
      <div
        v-if="mode !== 'basic'"
        class="variable-sidebar-toggle"
        :class="{ active: isVariableSidebarVisible }"
        @click="$emit('toggle-variable-sidebar')"
      >
        <n-icon class="toggle-icon">
          <svg viewBox="0 0 24 24" width="16" height="16">
            <path
              fill="currentColor"
              d="M16 6l2.29 2.29-4.88 4.88-4-4L2 16.59 3.41 18l6-6 4 4 6.3-6.29L22 12V6z"
            />
          </svg>
        </n-icon>
        <span class="toggle-text">变量</span>
      </div>
      <div v-if="currentFileName" class="file-status">
        <span class="file-name">{{ currentFileName }}</span>
        <span v-if="hasUnsavedChanges" class="unsaved-indicator" title="有未保存的更改">●</span>
      </div>
    </div>
    <div class="header-actions">
      <n-button size="small" @click="$emit('show-advanced')" quaternary title="高级设置">
        <template #icon>
          <n-icon>
            <svg viewBox="0 0 24 24" width="16" height="16">
              <path
                fill="currentColor"
                d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.488.488 0 0 0-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.58 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"
              />
            </svg>
          </n-icon>
        </template>
        高级
      </n-button>
      <n-button size="small" quaternary @click="$emit('full-render')">
        <template #icon>
          <n-icon>
            <svg viewBox="0 0 24 24" width="16" height="16">
              <path
                fill="currentColor"
                d="M22 11V3h-7v3H9V3H2v8h7v-2h2v10h4v3h7v-8h-7v3h-2V9h2v2h7z"
              />
            </svg>
          </n-icon>
        </template>
        全量渲染
      </n-button>
      <n-button quaternary circle class="edit-close-btn" @click="$emit('close-edit')">
        <template #icon>
          <n-icon
            ><svg viewBox="0 0 24 24" width="20" height="20">
              <path
                fill="currentColor"
                d="M18.3 5.71a1 1 0 0 0-1.41 0L12 10.59 7.11 5.7A1 1 0 0 0 5.7 7.11L10.59 12l-4.89 4.89a1 1 0 1 0 1.41 1.41L12 13.41l4.89 4.89a1 1 0 0 0 1.41-1.41L13.41 12l4.89-4.89a1 1 0 0 0 0-1.4z"
              /></svg
          ></n-icon>
        </template>
      </n-button>
    </div>
  </div>
</template>

<script setup>
  import { NButton, NIcon } from 'naive-ui';
  import { ChevronDown, Settings } from '@vicons/ionicons5';

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
    height: 56px;
    background: #fff;
    border-bottom: 1px solid #e0e0e0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 32px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.03);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .title-area {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .edit-title {
    font-size: 1.2rem;
    font-weight: bold;
    color: #18a058;
  }

  .file-status {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .file-name {
    font-size: 14px;
    color: #666;
    font-weight: 500;
  }

  .unsaved-indicator {
    color: #ff9500;
    font-size: 20px;
    line-height: 1;
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }

    50% {
      opacity: 0.5;
    }
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  /* 文件树切换按钮样式 */
  .file-tree-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    background: #f8f9fa;
    border: 1px solid #e9ecef;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
    user-select: none;
  }

  .file-tree-toggle:hover {
    background: #e9ecef;
    border-color: #18a058;
  }

  .file-tree-toggle.active {
    background: #e8f5e8;
    border-color: #18a058;
  }

  .file-tree-toggle.active .toggle-icon {
    color: #18a058;
  }

  .file-tree-toggle.active .toggle-text {
    color: #18a058;
  }

  .toggle-icon {
    font-size: 16px;
    color: #666;
  }

  .toggle-text {
    font-size: 14px;
    color: #333;
    font-weight: 500;
  }

  /* 变量侧边栏切换按钮样式 */
  .variable-sidebar-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    background: #f8f9fa;
    border: 1px solid #e9ecef;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
    user-select: none;
  }

  .variable-sidebar-toggle:hover {
    background: #e9ecef;
    border-color: #722ed1;
  }

  .variable-sidebar-toggle.active {
    background: #f9f0ff;
    border-color: #722ed1;
  }

  .variable-sidebar-toggle.active .toggle-icon {
    color: #722ed1;
  }

  .variable-sidebar-toggle.active .toggle-text {
    color: #722ed1;
  }
</style>
