<template>
  <div class="drawer-header">
    <div class="header-left">
      <span style="font-size: 18px; color: var(--editor-muted, #666)">
        <CodeOutline />
      </span>
      <span class="header-title">Variable Studio</span>
    </div>
    <div class="header-actions">
      <!-- 编辑模式切换 -->
      <a-space :size="4" style="margin-right: 16px">
        <a-button
          size="small"
          :type="editMode === 'design' ? 'primary' : 'default'"
          @click="handleEditModeChange('design')"
        >
          设计
        </a-button>
        <a-button
          size="small"
          :type="editMode === 'tree' ? 'primary' : 'default'"
          @click="handleEditModeChange('tree')"
        >
          变量树
        </a-button>
      </a-space>

      <!-- 显示开关 -->
      <a-space :size="4" style="margin-right: 16px">
        <a-button
          size="small"
          :type="showDesign ? 'primary' : 'default'"
          @click="handleToggleDesign"
        >
          左栏
        </a-button>
        <a-button
          size="small"
          :type="showSchema ? 'primary' : 'default'"
          @click="handleToggleSchema"
        >
          Schema
        </a-button>
        <a-button
          size="small"
          :type="showForm ? 'primary' : 'default'"
          @click="handleToggleForm"
        >
          表单
        </a-button>
      </a-space>

      <!-- 工具按钮 -->
      <a-space :size="4">
        <a-button size="small" @click="handleRefresh" title="清除缓存并刷新变量">
          <template #icon>
            <RefreshOutline />
          </template>
          刷新
        </a-button>
        <a-button size="small" @click="handleShowTestData"> 测试数据 </a-button>
        <a-button size="small" @click="handleShowVariableAnalysis"> 分析变量 </a-button>
        <a-button size="small" type="primary" :loading="saving" @click="handleSave"> 保存 </a-button>
      </a-space>

      <!-- 关闭按钮 -->
      <a-button type="link" size="small" @click="handleClose" class="close-button">
        <template #icon>
          <CloseOutline />
        </template>
      </a-button>
    </div>
  </div>
</template>

<script setup>
  import { CloseOutline, CodeOutline, RefreshOutline } from '@/icons/ionicons5';

  /**
   * StudioHeader 组件
   * 负责显示 Variable Studio 的标题和操作按钮
   */

  // Props
  const props = defineProps({
    editMode: {
      type: String,
      default: 'design',
      validator: (value) => ['design', 'tree'].includes(value),
    },
    showDesign: {
      type: Boolean,
      default: true,
    },
    showSchema: {
      type: Boolean,
      default: true,
    },
    showForm: {
      type: Boolean,
      default: true,
    },
    saving: {
      type: Boolean,
      default: false,
    },
  });

  // Emits
  const emit = defineEmits([
    'update:editMode',
    'update:showDesign',
    'update:showSchema',
    'update:showForm',
    'show-test-data',
    'show-variable-analysis',
    'save',
    'refresh',
    'close',
  ]);

  // 事件处理
  const handleEditModeChange = (mode) => {
    emit('update:editMode', mode);
  };

  const handleToggleDesign = () => {
    emit('update:showDesign', !props.showDesign);
  };

  const handleToggleSchema = () => {
    emit('update:showSchema', !props.showSchema);
  };

  const handleToggleForm = () => {
    emit('update:showForm', !props.showForm);
  };

  const handleShowTestData = () => {
    emit('show-test-data');
  };

  const handleShowVariableAnalysis = () => {
    emit('show-variable-analysis');
  };

  const handleRefresh = () => {
    emit('refresh');
  };

  const handleSave = () => {
    emit('save');
  };

  const handleClose = () => {
    emit('close');
  };
</script>

<style scoped>
  .drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid #e0e0e0;
    background: var(--editor-inset-bg, #fafafa);
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .header-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--editor-muted, #666);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .close-button {
    margin-left: 8px;
    flex-shrink: 0;
    padding: 4px 8px;
  }

  .close-button:hover {
    background-color: var(--editor-inset-bg, #f0f0f0);
  }
</style>
