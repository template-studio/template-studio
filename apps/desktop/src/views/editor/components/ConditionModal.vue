<template>
  <!-- 文件条件设置弹窗 -->
  <a-modal
    v-model:open="modalVisible"
    title="设置生成条件"
    style="width: 800px; max-height: 80vh"
    :mask-closable="false"
    :footer="null"
    @cancel="handleClose"
  >
    <template #title>
      <div class="condition-modal-header">
        <span class="modal-title">设置生成条件</span>
        <span v-if="selectedFileForCondition" class="file-path">{{
          selectedFileForCondition.fileName || selectedFileForCondition.title
        }}</span>
      </div>
    </template>

    <div class="condition-content">
      <a-alert v-if="!hasCondition" type="info" style="margin-bottom: 16px">
        <template #message> 📋 如何设置生成条件 </template>
        <template #description>
          <div class="condition-help">
            <p><strong>第一步：</strong>确保模板已定义变量（在"变量配置"中添加）</p>
            <p><strong>第二步：</strong>点击下方"添加条件"按钮</p>
            <p><strong>第三步：</strong>选择条件类型（条件/且/或/非/多分支）</p>
            <p><strong>第四步：</strong>配置变量、操作符和比较值</p>
            <p><strong>第五步：</strong>点击"保存条件"完成设置</p>
          </div>
        </template>
      </a-alert>

      <a-alert
        v-if="variableOptions.length === 0 && hasCondition"
        type="warning"
        style="margin-bottom: 16px"
        message="⚠️ 当前模板没有定义变量"
        description="请先在'变量配置'中添加变量后再设置条件。"
      />

      <div class="condition-actions" style="margin-bottom: 16px">
        <a-button v-if="!hasCondition" type="primary" @click="addCondition"> 添加条件 </a-button>
        <template v-else>
          <a-button @click="clearCondition"> 清除条件 </a-button>
          <a-button type="primary" @click="testCondition"> 测试条件 </a-button>
        </template>
      </div>

      <div v-if="hasCondition" class="condition-editor">
        <condition-builder v-model="conditionData" :variables="variableOptions" />
      </div>

      <!-- 测试结果 -->
      <div v-if="testResult" class="test-result" style="margin-top: 16px">
        <a-alert :type="testResult.success ? 'success' : 'error'" message="测试结果">
          <template #description>{{ testResult.message }}</template>
        </a-alert>
      </div>
    </div>

    <div class="modal-footer">
      <a-button @click="handleClose">取消</a-button>
      <a-button type="primary" @click="handleSave" :loading="saving"> 保存条件 </a-button>
    </div>
  </a-modal>
</template>

<script setup>
  import { ref, computed, watch } from 'vue';
  import { useRoute } from 'vue-router';
  import { message } from 'ant-design-vue';
  import ConditionBuilder from './ConditionBuilder.vue';
  import {
    getFileCondition,
    setFileCondition,
    deleteFileCondition,
    evaluateFileCondition,
  } from '@/api/editor/conditions';

  const route = useRoute();

  const props = defineProps({
    show: {
      type: Boolean,
      required: true,
    },
    selectedFileForCondition: {
      type: Object,
      default: null,
    },
    userVariables: {
      type: Array,
      default: () => [],
    },
  });

  const emit = defineEmits(['update:show', 'close', 'saved']);

  const saving = ref(false);
  const conditionData = ref(null);
  const testResult = ref(null);

  const modalVisible = computed({
    get: () => props.show,
    set: (value) => {
      if (!value) {
        handleClose();
      }
    },
  });

  const hasCondition = computed(() => {
    return conditionData.value !== null && conditionData.value !== undefined;
  });

  // 变量选项：从用户定义的变量转换
  const variableOptions = computed(() => {
    if (!props.userVariables || !Array.isArray(props.userVariables)) {
      return [];
    }

    return props.userVariables.map((v) => ({
      label: v.title || v.name,
      value: v.name,
      type: v.variableType,
      description: v.description,
    }));
  });

  // 添加条件
  const addCondition = () => {
    conditionData.value = {
      type: 'if',
      variable: '',
      operator: 'eq',
      value: '',
    };
  };

  // 清除条件
  const clearCondition = () => {
    conditionData.value = null;
    testResult.value = null;
  };

  // 测试条件
  const testCondition = async () => {
    if (!conditionData.value || !props.selectedFileForCondition) {
      return;
    }

    try {
      testResult.value = { loading: true };

      const templateId = route.params.id;
      const filePath =
        props.selectedFileForCondition.filePath || props.selectedFileForCondition.path;

      const response = await evaluateFileCondition(templateId, filePath, {});

      testResult.value = {
        success: true,
        message: response.data?.message || '条件测试成功',
      };
    } catch (error) {
      testResult.value = {
        success: false,
        message: error.response?.data?.message || error.message || '条件测试失败',
      };
    }
  };

  // 加载现有条件
  const loadCondition = async () => {
    if (!props.selectedFileForCondition) {
      conditionData.value = null;
      return;
    }

    try {
      const templateId = route.params.id;
      const filePath =
        props.selectedFileForCondition.filePath || props.selectedFileForCondition.path;

      const response = await getFileCondition(templateId, filePath);

      if (response.data?.data?.condition) {
        conditionData.value = response.data.data.condition;
      } else {
        conditionData.value = null;
      }
    } catch (error) {
      console.error('加载条件失败:', error);
      // 如果文件不存在条件，设置为 null
      conditionData.value = null;
    }

    testResult.value = null;
  };

  // 保存条件
  const handleSave = async () => {
    if (!props.selectedFileForCondition) {
      message.error('未选择文件');
      return;
    }

    try {
      saving.value = true;

      const templateId = route.params.id;
      const filePath =
        props.selectedFileForCondition.filePath || props.selectedFileForCondition.path;

      if (conditionData.value) {
        await setFileCondition(templateId, filePath, conditionData.value);
        message.success('条件设置已保存');
      } else {
        // 清除条件
        await deleteFileCondition(templateId, filePath);
        message.success('条件已清除');
      }

      emit('saved');
      handleClose();
    } catch (error) {
      console.error('保存条件失败:', error);
      message.error(
        '保存条件失败: ' + (error.response?.data?.message || error.message || '未知错误')
      );
    } finally {
      saving.value = false;
    }
  };

  // 关闭弹窗
  const handleClose = () => {
    conditionData.value = null;
    testResult.value = null;
    emit('update:show', false);
    emit('close');
  };

  // 监听弹窗显示状态
  watch(
    () => props.show,
    (newVal) => {
      if (newVal) {
        loadCondition();
      }
    }
  );

  // 监听文件变化
  watch(
    () => props.selectedFileForCondition,
    () => {
      if (props.show) {
        loadCondition();
      }
    }
  );
</script>

<style scoped>
  .condition-modal-header {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .condition-modal-header .modal-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--editor-primary, #333);
  }

  .condition-modal-header .file-path {
    font-size: 12px;
    color: var(--editor-muted, #999);
    font-weight: normal;
  }

  .condition-content {
    max-height: 60vh;
    overflow-y: auto;
    overflow-x: visible; /* 确保下拉菜单不被裁剪 */
  }

  .condition-help p {
    margin: 4px 0;
    font-size: 13px;
    line-height: 1.6;
  }

  .condition-actions {
    display: flex;
    gap: 8px;
    position: relative;
    z-index: 1;
  }

  .condition-editor {
    border: 1px solid var(--editor-border, #e0e0e0);
    border-radius: 4px;
    padding: 16px;
    background-color: var(--editor-inset-bg, #fafafa);
    overflow: visible; /* 确保下拉菜单不被裁剪 */
  }

  .test-result {
    margin-top: 16px;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }
</style>
