<template>
  <a-modal
    v-model:open="showModal"
    title="模板变量分析"
    :width="1000"
    :mask-closable="false"
    :footer="null"
    @cancel="handleClose"
  >
    <a-spin :spinning="analyzing" tip="正在分析模板变量...">
      <div v-if="analysisResult">
        <!-- 调试信息 -->
        <a-alert type="info" style="margin-bottom: 16px" :show-icon="false">
          调试信息: 缺失变量{{ analysisResult.missingVariables?.length || 0 }}个, 检测变量{{
            analysisResult.detectedVariables?.length || 0
          }}个, 未使用变量{{ analysisResult.unusedVariables?.length || 0 }}个
        </a-alert>

        <a-tabs type="line" animated>
          <!-- 缺失变量 -->
          <a-tab-pane key="missing">
            <template #tab>
              缺失变量 ({{ analysisResult.missingVariables?.length || 0 }})
            </template>
            <div
              v-if="analysisResult.missingVariables && analysisResult.missingVariables.length > 0"
            >
              <div style="margin-bottom: 16px">
                <span style="color: #999">模板中使用但未定义的变量：</span>
                <a-button
                  size="small"
                  type="primary"
                  style="margin-left: 12px"
                  @click="handleAddAllMissing"
                >
                  添加全部到变量树
                </a-button>
              </div>
              <a-table
                :columns="missingColumns"
                :data-source="analysisResult.missingVariables"
                :pagination="false"
                size="small"
                row-key="name"
              />
            </div>
            <a-empty v-else description="没有缺失的变量" />
          </a-tab-pane>

          <!-- 检测到的变量 -->
          <a-tab-pane key="detected">
            <template #tab>
              检测变量 ({{ analysisResult.detectedVariables?.length || 0 }})
            </template>
            <div
              v-if="
                analysisResult.detectedVariables && analysisResult.detectedVariables.length > 0
              "
            >
              <div style="margin-bottom: 16px">
                <span style="color: #999">模板中检测到的所有变量：</span>
              </div>
              <a-table
                :columns="detectedColumns"
                :data-source="analysisResult.detectedVariables"
                :pagination="false"
                size="small"
                row-key="name"
              />
            </div>
            <a-empty v-else description="没有检测到变量" />
          </a-tab-pane>

          <!-- 冲突变量 -->
          <a-tab-pane key="conflict">
            <template #tab>
              类型冲突 ({{ analysisResult.conflictVariables?.length || 0 }})
            </template>
            <div v-if="analysisResult.conflictVariables?.length > 0">
              <div style="margin-bottom: 16px">
                <span style="color: #999">已定义但类型可能不匹配的变量：</span>
              </div>
              <a-table
                :columns="conflictColumns"
                :data-source="analysisResult.conflictVariables"
                :pagination="false"
                size="small"
                row-key="name"
              />
            </div>
            <a-empty v-else description="没有类型冲突" />
          </a-tab-pane>

          <!-- 未使用变量 -->
          <a-tab-pane key="unused">
            <template #tab>
              未使用 ({{ analysisResult.unusedVariables?.length || 0 }})
            </template>
            <div
              v-if="analysisResult.unusedVariables && analysisResult.unusedVariables.length > 0"
            >
              <div style="margin-bottom: 16px">
                <span style="color: #999">已定义但模板中未使用的变量：</span>
                <a-button
                  size="small"
                  danger
                  style="margin-left: 12px"
                  @click="handleDeleteAllUnused"
                >
                  删除全部未使用变量
                </a-button>
              </div>
              <a-table
                :columns="unusedColumns"
                :data-source="analysisResult.unusedVariables.map((name) => ({ name }))"
                :pagination="false"
                size="small"
                row-key="name"
              />
            </div>
            <a-empty v-else description="所有变量都已使用" />
          </a-tab-pane>

          <!-- 统计信息 -->
          <a-tab-pane key="stats" tab="统计信息">
            <a-descriptions :column="2" bordered size="small">
              <a-descriptions-item label="总变量数">
                <a-tag color="blue">{{ analysisResult.totalVariableCount || 0 }}</a-tag>
              </a-descriptions-item>
              <a-descriptions-item label="分析文件数">
                <a-tag>{{ analysisResult.analyzedFileCount || 0 }}</a-tag>
              </a-descriptions-item>
              <a-descriptions-item label="缺失变量">
                <a-tag
                  :color="
                    (analysisResult.missingVariables?.length || 0) > 0 ? 'orange' : 'green'
                  "
                >
                  {{ analysisResult.missingVariables?.length || 0 }}
                </a-tag>
              </a-descriptions-item>
              <a-descriptions-item label="类型冲突">
                <a-tag
                  :color="
                    (analysisResult.conflictVariables?.length || 0) > 0 ? 'red' : 'green'
                  "
                >
                  {{ analysisResult.conflictVariables?.length || 0 }}
                </a-tag>
              </a-descriptions-item>
              <a-descriptions-item label="未使用变量">
                <a-tag
                  :color="
                    (analysisResult.unusedVariables?.length || 0) > 0 ? 'orange' : 'green'
                  "
                >
                  {{ analysisResult.unusedVariables?.length || 0 }}
                </a-tag>
              </a-descriptions-item>
            </a-descriptions>

            <!-- 快速操作 -->
            <div style="margin-top: 24px">
              <span style="color: #999; margin-bottom: 12px; display: block">快速操作：</span>
              <a-space>
                <a-button
                  size="small"
                  type="primary"
                  :disabled="
                    !analysisResult.missingVariables ||
                    analysisResult.missingVariables.length === 0
                  "
                  @click="handleAddAllMissing"
                >
                  添加所有缺失变量 ({{ analysisResult.missingVariables?.length || 0 }})
                </a-button>
                <a-button
                  size="small"
                  danger
                  :disabled="
                    !analysisResult.unusedVariables || analysisResult.unusedVariables.length === 0
                  "
                  @click="handleDeleteAllUnused"
                >
                  删除未使用变量 ({{ analysisResult.unusedVariables?.length || 0 }})
                </a-button>
              </a-space>
            </div>
          </a-tab-pane>
        </a-tabs>
      </div>
    </a-spin>

    <template #footer>
      <div style="display: flex; justify-content: flex-end; gap: 12px">
        <a-button @click="handleAnalyze" :loading="analyzing">重新分析</a-button>
        <a-button @click="handleClose">关闭</a-button>
      </div>
    </template>
  </a-modal>
</template>

<script setup>
  import { ref, watch, h } from 'vue';
  import { message } from 'ant-design-vue';
  import { CloseOutline } from '@/icons/ionicons5';
  import { analyzeTemplateVariables } from '@/api/templates';

  /**
   * VariableAnalysisModal 组件
   * 变量分析模态框
   */

  // Props
  const props = defineProps({
    show: {
      type: Boolean,
      default: false,
    },
    templateId: {
      type: [String, Number],
      required: true,
    },
    components: {
      type: Array,
      default: () => [],
    },
  });

  // Emits
  const emit = defineEmits(['update:show', 'add-components', 'remove-components']);

  // Refs
  const showModal = ref(false);
  const analyzing = ref(false);
  const analysisResult = ref(null);

  // ========== 表格列定义 ==========

  const missingColumns = [
    {
      title: '变量名',
      dataIndex: 'name',
      width: 150,
    },
    {
      title: '推测类型',
      dataIndex: 'type',
      width: 100,
    },
    {
      title: '出现文件',
      dataIndex: 'files',
      customRender: ({ text }) => text?.join(', ') || '',
    },
    {
      title: '操作',
      key: 'actions',
      width: 100,
      customRender: ({ record }) => {
        return h(
          'a-button',
          {
            size: 'small',
            type: 'primary',
            onClick: () => handleAddVariable(record),
          },
          () => '添加'
        );
      },
    },
  ];

  const detectedColumns = [
    {
      title: '变量名',
      dataIndex: 'name',
      width: 150,
    },
    {
      title: '类型',
      dataIndex: 'type',
      width: 100,
    },
    {
      title: '出现文件',
      dataIndex: 'files',
      customRender: ({ text }) => text?.join(', ') || '',
    },
    {
      title: '上下文',
      dataIndex: 'contexts',
      customRender: ({ text }) => text?.join('; ') || '',
    },
  ];

  const conflictColumns = [
    { title: '变量名', dataIndex: 'name', width: 120 },
    { title: '推测类型', dataIndex: 'type', width: 100 },
    { title: '出现文件', dataIndex: 'files', customRender: ({ text }) => text?.join(', ') },
    { title: '冲突说明', dataIndex: 'suggestions' },
  ];

  const unusedColumns = [
    {
      title: '变量名',
      dataIndex: 'name',
      width: 200,
    },
    {
      title: '操作',
      key: 'actions',
      width: 100,
      customRender: ({ record }) => {
        return h(
          'a-button',
          {
            size: 'small',
            danger: true,
            onClick: () => handleDeleteVariable(record.name),
          },
          () => '删除'
        );
      },
    },
  ];

  // ========== 方法 ==========

  const handleAnalyze = async () => {
    if (!props.templateId) {
      message.error('模板ID不能为空');
      return;
    }

    analyzing.value = true;
    try {
      const result = await analyzeTemplateVariables(props.templateId);
      analysisResult.value = result.data.data || result.data;
    } catch (error) {
      console.error('分析变量失败:', error);
      message.error('分析变量失败: ' + (error.message || '未知错误'));
    } finally {
      analyzing.value = false;
    }
  };

  const handleAddVariable = (detectedVar) => {
    if (!detectedVar) return;

    emit('add-components', [detectedVar]);

    if (analysisResult.value?.missingVariables) {
      analysisResult.value.missingVariables = analysisResult.value.missingVariables.filter(
        (v) => v.name !== detectedVar.name
      );
    }

    message.success(`已添加变量 "${detectedVar.name}"`);
  };

  const handleAddAllMissing = () => {
    const missingVars = analysisResult.value?.missingVariables;
    if (!missingVars || missingVars.length === 0) {
      message.warning('没有可添加的变量');
      return;
    }

    emit('add-components', missingVars);
    analysisResult.value.missingVariables = [];
    message.success(`已添加 ${missingVars.length} 个变量到变量树`);
  };

  const handleDeleteVariable = (variableName) => {
    emit('remove-components', [variableName]);

    if (analysisResult.value?.unusedVariables) {
      analysisResult.value.unusedVariables = analysisResult.value.unusedVariables.filter(
        (v) => v !== variableName
      );
    }

    message.success(`已删除变量 "${variableName}"`);
  };

  const handleDeleteAllUnused = () => {
    const unusedVars = analysisResult.value?.unusedVariables;
    if (!unusedVars || unusedVars.length === 0) {
      message.warning('没有可删除的变量');
      return;
    }

    emit('remove-components', unusedVars);
    analysisResult.value.unusedVariables = [];
    message.success(`已删除 ${unusedVars.length} 个未使用的变量`);
  };

  const handleClose = () => {
    emit('update:show', false);
  };

  // ========== 监听 ==========

  watch(
    () => props.show,
    async (newVal) => {
      showModal.value = newVal;
      if (newVal) {
        await handleAnalyze();
      }
    }
  );

  watch(showModal, (newVal) => {
    if (!newVal) {
      emit('update:show', false);
    }
  });
</script>
