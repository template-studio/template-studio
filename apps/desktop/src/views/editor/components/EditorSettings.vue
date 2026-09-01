<template>
  <!-- 设置面板弹框 -->
  <a-modal v-model:open="modalVisible" title="编辑器设置" style="width: 600px" :mask-closable="false" :footer="null">
    <template #title>
      <div class="settings-header">
        <svg viewBox="0 0 24 24" style="width: 20px; height: 20px; margin-right: 8px">
          <path
            fill="currentColor"
            d="M19.14,12.94c0.04-0.3,0.06-0.61,0.06-0.94c0-0.32-0.02-0.64-0.07-0.94l2.03-1.58c0.18-0.14,0.23-0.41,0.12-0.61 l-1.92-3.32c-0.12-0.22-0.37-0.29-0.59-0.22l-2.39,0.96c-0.5-0.38-1.03-0.7-1.62-0.94L14.4,2.81c-0.04-0.24-0.24-0.41-0.48-0.41 h-3.84c-0.24,0-0.43,0.17-0.47,0.41L9.25,5.35C8.66,5.59,8.12,5.92,7.63,6.29L5.24,5.33c-0.22-0.08-0.47,0-0.59,0.22L2.74,8.87 C2.62,9.08,2.66,9.34,2.86,9.48l2.03,1.58C4.84,11.36,4.82,11.69,4.82,12s0.02,0.64,0.07,0.94l-2.03,1.58 c-0.18,0.14-0.23,0.41-0.12,0.61l1.92,3.32c0.12,0.22,0.37,0.29,0.59,0.22l2.39-0.96c0.5,0.38,1.03,0.7,1.62,0.94l0.36,2.54 c0.05,0.24,0.24,0.41,0.48,0.41h3.84c0.24,0,0.44-0.17,0.47-0.41l0.36-2.54c0.59-0.24,1.13-0.56,1.62-0.94l2.39,0.96 c0.22,0.08,0.47,0,0.59-0.22l1.92-3.32c0.12-0.22,0.07-0.47-0.12-0.61L19.14,12.94z M12,15.6c-1.98,0-3.6-1.62-3.6-3.6 s1.62-3.6,3.6-3.6s3.6,1.62,3.6,3.6S13.98,15.6,12,15.6z"
          />
        </svg>
        <span class="modal-title">编辑器设置</span>
      </div>
    </template>

    <div class="settings-container">
      <!-- 左侧菜单 -->
      <div class="settings-menu">
        <div
          v-for="category in settingsCategories"
          :key="category.key"
          class="menu-item"
          :class="{ active: activeCategory === category.key }"
          @click="activeCategory = category.key"
        >
          <component :is="category.icon" style="font-size: 16px; margin-right: 8px" />
          {{ category.label }}
        </div>
      </div>

      <!-- 右侧设置内容 -->
      <div class="settings-content">
        <!-- 编辑器设置 -->
        <div v-show="activeCategory === 'editor'" class="settings-section">
          <h3 class="section-title">编辑器</h3>

          <div class="setting-item">
            <div class="setting-label">
              <span>自动保存</span>
              <span class="setting-description">文件修改后自动保存</span>
            </div>
            <div class="setting-control">
              <a-switch v-model:checked="localSettings.autoSave.enabled" />
            </div>
          </div>

          <div v-if="localSettings.autoSave.enabled" class="setting-item">
            <div class="setting-label">
              <span>自动保存间隔</span>
              <span class="setting-description">自动保存的时间间隔（秒）</span>
            </div>
            <div class="setting-control">
              <a-input-number
                v-model:value="localSettings.autoSave.interval"
                :min="5"
                :max="300"
                :step="5"
                style="width: 120px"
                placeholder="30"
              />
              <span style="margin-left: 8px; color: var(--editor-muted, #666)">秒</span>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-label">
              <span>字体大小</span>
              <span class="setting-description">编辑器字体大小</span>
            </div>
            <div class="setting-control">
              <a-input-number
                v-model:value="localSettings.editor.fontSize"
                :min="10"
                :max="24"
                style="width: 120px"
                placeholder="14"
              />
              <span style="margin-left: 8px; color: var(--editor-muted, #666)">px</span>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-label">
              <span>显示行号</span>
              <span class="setting-description">在编辑器中显示行号</span>
            </div>
            <div class="setting-control">
              <a-switch v-model:checked="localSettings.editor.lineNumbers" />
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-label">
              <span>自动换行</span>
              <span class="setting-description">长行自动换行显示</span>
            </div>
            <div class="setting-control">
              <a-switch v-model:checked="localSettings.editor.wordWrap" />
            </div>
          </div>
        </div>

        <!-- 界面设置 -->
        <div v-show="activeCategory === 'interface'" class="settings-section">
          <h3 class="section-title">界面</h3>

          <div class="setting-item">
            <div class="setting-label">
              <span>主题</span>
              <span class="setting-description">选择编辑器主题</span>
            </div>
            <div class="setting-control">
              <a-select
                v-model:value="localSettings.interface.theme"
                :options="themeOptions"
                style="width: 150px"
              />
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-label">
              <span>启动时恢复面板布局</span>
              <span class="setting-description">记住并恢复面板的大小和位置</span>
            </div>
            <div class="setting-control">
              <a-switch v-model:checked="localSettings.interface.restoreLayout" />
            </div>
          </div>
        </div>

        <!-- 预览设置 -->
        <div v-show="activeCategory === 'preview'" class="settings-section">
          <h3 class="section-title">预览</h3>

          <div class="setting-item">
            <div class="setting-label">
              <span>实时预览</span>
              <span class="setting-description">编辑时自动更新预览</span>
            </div>
            <div class="setting-control">
              <a-switch v-model:checked="localSettings.preview.realtime" />
            </div>
          </div>

          <div v-if="localSettings.preview.realtime" class="setting-item">
            <div class="setting-label">
              <span>预览延迟</span>
              <span class="setting-description">输入停止后延迟更新预览（毫秒）</span>
            </div>
            <div class="setting-control">
              <a-input-number
                v-model:value="localSettings.preview.debounceDelay"
                :min="100"
                :max="5000"
                :step="100"
                style="width: 120px"
                placeholder="500"
              />
              <span style="margin-left: 8px; color: var(--editor-muted, #666)">ms</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="settings-actions">
      <a-button @click="resetToDefaults" style="margin-right: auto"> 恢复默认 </a-button>
      <a-button @click="handleCancel" style="margin-right: 12px"> 取消 </a-button>
      <a-button type="primary" @click="handleSave"> 保存设置 </a-button>
    </div>
  </a-modal>
</template>

<script setup>
  import { ref, computed, watch, onMounted } from 'vue';
  import { message } from 'ant-design-vue';

  // 图标组件（使用内联SVG）
  const EditorIcon = {
    template: `
    <svg viewBox="0 0 24 24">
      <path fill="currentColor" d="M3 3h18v18H3V3zm16 16V5H5v14h14zM7 7h2v2H7V7zm0 4h2v2H7v-2zm0 4h2v2H7v-2zm4-8h6v2h-6V7zm0 4h6v2h-6v-2zm0 4h6v2h-6v-2z"/>
    </svg>
  `,
  };

  const InterfaceIcon = {
    template: `
    <svg viewBox="0 0 24 24">
      <path fill="currentColor" d="M4 2h16a2 2 0 0 1 2 2v16a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zm0 2v4h16V4H4zm16 6H4v10h16V10z"/>
    </svg>
  `,
  };

  const PreviewIcon = {
    template: `
    <svg viewBox="0 0 24 24">
      <path fill="currentColor" d="M12 9a3 3 0 0 1 3 3a3 3 0 0 1-3 3a3 3 0 0 1-3-3a3 3 0 0 1 3-3m0-4.5c5 0 9.27 3.11 11 7.5c-1.73 4.39-6 7.5-11 7.5S2.73 16.39 1 12c1.73-4.39 6-7.5 11-7.5M12 17a5 5 0 0 0 5-5a5 5 0 0 0-5-5a5 5 0 0 0-5 5a5 5 0 0 0 5 5z"/>
    </svg>
  `,
  };

  const props = defineProps({
    show: {
      type: Boolean,
      required: true,
    },
    settings: {
      type: Object,
      default: () => ({}),
    },
  });

  const emit = defineEmits(['update:show', 'save-settings']);

  const modalVisible = computed({
    get: () => props.show,
    set: (value) => emit('update:show', value),
  });

  // 设置分类
  const settingsCategories = [
    { key: 'editor', label: '编辑器', icon: EditorIcon },
    { key: 'interface', label: '界面', icon: InterfaceIcon },
    { key: 'preview', label: '预览', icon: PreviewIcon },
  ];

  // 当前激活的分类
  const activeCategory = ref('editor');

  // 主题选项
  const themeOptions = [
    { label: '浅色主题', value: 'light' },
    { label: '深色主题', value: 'dark' },
    { label: '跟随系统', value: 'auto' },
  ];

  // 默认设置
  const defaultSettings = {
    autoSave: {
      enabled: true,
      interval: 30,
    },
    editor: {
      fontSize: 14,
      lineNumbers: true,
      wordWrap: true,
    },
    interface: {
      theme: 'light',
      restoreLayout: true,
    },
    preview: {
      realtime: true,
      debounceDelay: 500,
    },
  };

  // 本地设置副本
  const localSettings = ref({});

  // 初始化设置
  const initSettings = () => {
    localSettings.value = JSON.parse(
      JSON.stringify({
        ...defaultSettings,
        ...props.settings,
      })
    );
  };

  // 重置为默认值
  const resetToDefaults = () => {
    localSettings.value = JSON.parse(JSON.stringify(defaultSettings));
    message.success('已恢复为默认设置');
  };

  // 保存设置
  const handleSave = () => {
    emit('save-settings', JSON.parse(JSON.stringify(localSettings.value)));
    modalVisible.value = false;
    message.success('设置已保存');
  };

  // 取消
  const handleCancel = () => {
    modalVisible.value = false;
  };

  // 监听设置变化，重新初始化
  watch(() => props.settings, initSettings, { deep: true });

  // 监听弹框显示，初始化设置
  watch(
    () => props.show,
    (show) => {
      if (show) {
        initSettings();
        activeCategory.value = 'editor';
      }
    }
  );

  onMounted(() => {
    initSettings();
  });
</script>

<style scoped>
  .settings-header {
    display: flex;
    align-items: center;
  }

  .modal-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--editor-primary, #333);
  }

  .settings-container {
    display: flex;
    height: 400px;
    max-height: 60vh;
    margin-bottom: 16px;
  }

  .settings-menu {
    width: 160px;
    border-right: 1px solid var(--editor-border, #f0f0f0);
    padding-right: 16px;
    overflow-y: auto;
    flex-shrink: 0;
  }

  .menu-item {
    display: flex;
    align-items: center;
    padding: 12px 16px;
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.2s;
    font-size: 14px;
    color: var(--editor-muted, #666);
    margin-bottom: 4px;
  }

  .menu-item:hover {
    background: var(--editor-inset-bg, #f5f5f5);
    color: var(--editor-primary, #333);
  }

  .menu-item.active {
    background: #e8f5e8;
    color: #18a058;
  }

  .settings-content {
    flex: 1;
    padding: 0 0 0 24px;
    overflow-y: auto;
    min-height: 0;
  }

  .settings-section {
    /* 移除固定高度，让内容自然流动 */
  }

  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--editor-primary, #333);
    margin: 0 0 24px 0;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--editor-border, #f0f0f0);
  }

  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 16px 0;
    border-bottom: 1px solid #f8f8f8;
  }

  .setting-item:last-child {
    border-bottom: none;
  }

  .setting-label {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .setting-label > span:first-child {
    font-size: 14px;
    font-weight: 500;
    color: var(--editor-primary, #333);
    margin-bottom: 4px;
  }

  .setting-description {
    font-size: 12px;
    color: var(--editor-muted, #666);
    line-height: 1.4;
  }

  .setting-control {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    margin-left: 16px;
  }

  .settings-actions {
    display: flex;
    align-items: center;
    width: 100%;
  }
</style>
