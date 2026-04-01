<template>
  <div class="drawer-header">
    <div class="header-left">
      <n-icon size="18" color="#666">
        <CodeOutline />
      </n-icon>
      <span class="header-title">Variable Studio</span>
    </div>
    <div class="header-actions">
      <!-- 编辑模式切换 -->
      <n-space :size="4" style="margin-right: 16px">
        <n-button
          size="small"
          :type="editMode === 'design' ? 'default' : 'default'"
          :ghost="editMode === 'design'"
          @click="handleEditModeChange('design')"
        >
          设计
        </n-button>
        <n-button
          size="small"
          :type="editMode === 'tree' ? 'default' : 'default'"
          :ghost="editMode === 'tree'"
          @click="handleEditModeChange('tree')"
        >
          变量树
        </n-button>
      </n-space>

      <!-- 显示开关 -->
      <n-space :size="4" style="margin-right: 16px">
        <n-button
          size="small"
          :type="showDesign ? 'default' : 'default'"
          :ghost="showDesign"
          @click="handleToggleDesign"
        >
          左栏
        </n-button>
        <n-button
          size="small"
          :type="showSchema ? 'default' : 'default'"
          :ghost="showSchema"
          @click="handleToggleSchema"
        >
          Schema
        </n-button>
        <n-button
          size="small"
          :type="showForm ? 'default' : 'default'"
          :ghost="showForm"
          @click="handleToggleForm"
        >
          表单
        </n-button>
      </n-space>

      <!-- 工具按钮 -->
      <n-space :size="4">
        <n-button size="small" quaternary @click="handleRefresh" title="清除缓存并刷新变量">
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          刷新
        </n-button>
        <n-button size="small" quaternary @click="handleShowTestData"> 测试数据 </n-button>
        <n-button size="small" quaternary @click="handleShowVariableAnalysis"> 分析变量 </n-button>
        <n-button size="small" quaternary @click="handleSave" :loading="saving"> 保存 </n-button>
      </n-space>

      <!-- 关闭按钮 -->
      <n-button text size="small" @click="handleClose" class="close-button">
        <template #icon>
          <n-icon><CloseOutline /></n-icon>
        </template>
      </n-button>
    </div>
  </div>
</template>

<script setup>
  import { NIcon, NButton, NSpace } from 'naive-ui';
  import { CloseOutline, CodeOutline, RefreshOutline } from '@vicons/ionicons5';

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
    background: #fafafa;
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
    color: #666;
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
    background-color: #f0f0f0;
  }
</style>
