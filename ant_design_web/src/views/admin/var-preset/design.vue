<template>
  <div class="var-preset-design">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <a-button type="link" @click="goBack" class="back-button">
          <template #icon>
            <ArrowBackOutline />
          </template>
          返回
        </a-button>
        <div class="title-info">
          <h1>数据结构设计</h1>
          <div
            class="preset-info"
            v-if="
              presetInfo && (presetInfo.displayName || presetInfo.name || presetInfo.description)
            "
          >
            <a-tag color="blue">{{ presetInfo.displayName || presetInfo.name }}</a-tag>
            <span v-if="presetInfo.description" class="preset-desc">{{
              presetInfo.description
            }}</span>
          </div>
        </div>
      </div>
      <div class="header-actions">
        <a-button @click="handleSave" :loading="saving" type="primary">
          <template #icon>
            <SaveOutline />
          </template>
          保存
        </a-button>
      </div>
    </div>

    <!-- 编辑器区域 -->
    <div class="editor-container">
      <SimpleVarPresetEditor v-if="presetInfo" v-model="schemaJson" :key="presetInfo.id" />
      <div v-else class="loading-container">
        <a-spin size="large">
          <template #indicator>
            <div style="text-align: center">
              <a-spin />
              <div style="margin-top: 8px">加载预设信息中...</div>
            </div>
          </template>
        </a-spin>
      </div>
    </div>
  </div>
</template>

<script setup>
  import { ref, onMounted, onUnmounted, watch } from 'vue';
  import { useRoute, useRouter } from 'vue-router';
  import { message } from 'ant-design-vue';
  import { ArrowBackOutline, SaveOutline } from '@/icons/ionicons5';
  import { getVarPresetDetail, editVarPreset } from '@/api/varPreset';
  import SimpleVarPresetEditor from '@/components/SimpleVarPresetEditor.vue';

  const route = useRoute();
  const router = useRouter();
  // 数据状态
  const presetInfo = ref(null);
  const schemaJson = ref('');
  const saving = ref(false);
  const hasChanges = ref(false);

  // 加载预设信息
  const loadPresetInfo = async () => {
    try {
      const presetId = route.params.id;
      if (!presetId) {
        message.error('缺少预设ID');
        goBack();
        return;
      }

      const response = await getVarPresetDetail({ id: presetId });
      console.log('API响应:', response.data); // 调试用
      presetInfo.value = response.data.data.varPreset || response.data.data;
      schemaJson.value =
        presetInfo.value.schemaJson ||
        '{\n  "example_field": {\n    "type": "string",\n    "description": "示例字段",\n    "default": "示例值"\n  }\n}';
    } catch (error) {
      console.error('加载预设信息失败:', error);
      message.error('加载预设信息失败');
      goBack();
    }
  };

  // 保存数据结构
  const handleSave = async () => {
    try {
      // 验证预设信息是否加载完成
      if (!presetInfo.value || !presetInfo.value.id) {
        message.error('预设信息未加载完成，请稍后重试');
        return;
      }

      // 验证 JSON 格式
      JSON.parse(schemaJson.value);

      saving.value = true;

      const saveData = {
        id: presetInfo.value.id,
        name: presetInfo.value.name,
        displayName: presetInfo.value.displayName,
        description: presetInfo.value.description,
        category: presetInfo.value.category,
        schemaJson: schemaJson.value,
        defaultDataJson: presetInfo.value.defaultDataJson,
        icon: presetInfo.value.icon,
        sort: presetInfo.value.sort,
        version: presetInfo.value.version,
        isEnabled: presetInfo.value.isEnabled,
      };

      console.log('保存数据:', saveData); // 调试用

      await editVarPreset(saveData);

      hasChanges.value = false;
      message.success('数据结构保存成功');
    } catch (error) {
      if (error instanceof SyntaxError) {
        message.error('JSON 格式错误，请检查语法');
      } else {
        console.error('保存失败:', error);
        const errorMsg = error.response?.data?.message || error.message || '保存失败';
        message.error(errorMsg);
      }
    } finally {
      saving.value = false;
    }
  };

  // 返回列表页
  const goBack = () => {
    if (hasChanges.value) {
      // 可以在这里添加确认对话框
      if (!confirm('有未保存的更改，确定要离开吗？')) {
        return;
      }
    }
    router.push({ name: 'admin-var-presets' });
  };

  // 监听内容变化
  watch(schemaJson, () => {
    hasChanges.value = true;
  });

  // 监听浏览器关闭/刷新
  const handleBeforeUnload = (e) => {
    if (hasChanges.value) {
      e.preventDefault();
      e.returnValue = '有未保存的更改，确定要离开吗？';
    }
  };

  // 生命周期
  onMounted(() => {
    loadPresetInfo();
    window.addEventListener('beforeunload', handleBeforeUnload);
  });

  // 清理
  onUnmounted(() => {
    window.removeEventListener('beforeunload', handleBeforeUnload);
  });
</script>

<style scoped>
  .var-preset-design {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: #f5f5f5;
  }

  .page-header {
    background: white;
    border-bottom: 1px solid #e0e0e0;
    padding: 16px 24px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: flex-start;
    gap: 16px;
  }

  .back-button {
    font-size: 14px;
    color: #666;
    margin-top: 2px; /* 微调对齐 */
  }

  .back-button:hover {
    color: #1890ff;
  }

  .title-info {
    flex: 1;
  }

  .title-info h1 {
    margin: 0 0 4px 0;
    font-size: 20px;
    font-weight: 600;
    color: #333;
    line-height: 1.2;
  }

  .preset-info {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }

  .preset-desc {
    font-size: 14px;
    color: #666;
  }

  .header-actions {
    display: flex;
    gap: 12px;
  }

  .editor-container {
    flex: 1;
    padding: 24px;
    overflow: hidden;
  }

  .loading-container {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  :deep(.simple-var-preset-editor) {
    height: 100%;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }
</style>
