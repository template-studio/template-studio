<template>
  <n-modal v-model:show="showModal" :mask-closable="false">
    <n-card
      style="width: 1000px; max-height: 80vh"
      title="模板变量分析"
      :bordered="false"
      size="huge"
    >
      <template #header-extra>
        <n-button quaternary circle @click="handleClose">
          <template #icon>
            <n-icon><CloseOutline /></n-icon>
          </template>
        </n-button>
      </template>

      <n-spin :show="analyzing">
        <div v-if="analysisResult">
          <!-- 调试信息 -->
          <n-alert type="info" style="margin-bottom: 16px" :show-icon="false">
            调试信息: 缺失变量{{ analysisResult.missingVariables?.length || 0 }}个, 检测变量{{
              analysisResult.detectedVariables?.length || 0
            }}个, 未使用变量{{ analysisResult.unusedVariables?.length || 0 }}个
          </n-alert>

          <n-tabs type="line" animated>
            <!-- 缺失变量 -->
            <n-tab-pane
              name="missing"
              :tab="`缺失变量 (${analysisResult.missingVariables?.length || 0})`"
            >
              <div
                v-if="analysisResult.missingVariables && analysisResult.missingVariables.length > 0"
              >
                <div style="margin-bottom: 16px">
                  <n-text depth="3">模板中使用但未定义的变量：</n-text>
                  <n-button
                    size="small"
                    type="primary"
                    style="margin-left: 12px"
                    @click="handleAddAllMissing"
                  >
                    添加全部到变量树
                  </n-button>
                </div>
                <n-data-table
                  :columns="missingColumns"
                  :data="analysisResult.missingVariables"
                  :pagination="false"
                  size="small"
                />
              </div>
              <n-empty v-else description="没有缺失的变量" />
            </n-tab-pane>

            <!-- 检测到的变量 -->
            <n-tab-pane
              name="detected"
              :tab="`检测变量 (${analysisResult.detectedVariables?.length || 0})`"
            >
              <div
                v-if="
                  analysisResult.detectedVariables && analysisResult.detectedVariables.length > 0
                "
              >
                <div style="margin-bottom: 16px">
                  <n-text depth="3">模板中检测到的所有变量：</n-text>
                </div>
                <n-data-table
                  :columns="detectedColumns"
                  :data="analysisResult.detectedVariables"
                  :pagination="false"
                  size="small"
                />
              </div>
              <n-empty v-else description="没有检测到变量" />
            </n-tab-pane>

            <!-- 冲突变量 -->
            <n-tab-pane
              name="conflict"
              :tab="`类型冲突 (${analysisResult.conflictVariables?.length || 0})`"
            >
              <div v-if="analysisResult.conflictVariables?.length > 0">
                <div style="margin-bottom: 16px">
                  <n-text depth="3">已定义但类型可能不匹配的变量：</n-text>
                </div>
                <n-data-table
                  :columns="conflictColumns"
                  :data="analysisResult.conflictVariables"
                  :pagination="false"
                  size="small"
                />
              </div>
              <n-empty v-else description="没有类型冲突" />
            </n-tab-pane>

            <!-- 未使用变量 -->
            <n-tab-pane
              name="unused"
              :tab="`未使用 (${analysisResult.unusedVariables?.length || 0})`"
            >
              <div
                v-if="analysisResult.unusedVariables && analysisResult.unusedVariables.length > 0"
              >
                <div style="margin-bottom: 16px">
                  <n-text depth="3">已定义但模板中未使用的变量：</n-text>
                  <n-button
                    size="small"
                    type="warning"
                    style="margin-left: 12px"
                    @click="handleDeleteAllUnused"
                  >
                    删除全部未使用变量
                  </n-button>
                </div>
                <n-data-table
                  :columns="unusedColumns"
                  :data="analysisResult.unusedVariables.map((name) => ({ name }))"
                  :pagination="false"
                  size="small"
                />
              </div>
              <n-empty v-else description="所有变量都已使用" />
            </n-tab-pane>

            <!-- 统计信息 -->
            <n-tab-pane name="stats" tab="统计信息">
              <n-descriptions label-placement="left" :column="2">
                <n-descriptions-item label="总变量数">
                  <n-tag type="info" size="small">{{
                    analysisResult.totalVariableCount || 0
                  }}</n-tag>
                </n-descriptions-item>
                <n-descriptions-item label="分析文件数">
                  <n-tag type="default" size="small">{{
                    analysisResult.analyzedFileCount || 0
                  }}</n-tag>
                </n-descriptions-item>
                <n-descriptions-item label="缺失变量">
                  <n-tag
                    :type="
                      (analysisResult.missingVariables?.length || 0) > 0 ? 'warning' : 'success'
                    "
                    size="small"
                  >
                    {{ analysisResult.missingVariables?.length || 0 }}
                  </n-tag>
                </n-descriptions-item>
                <n-descriptions-item label="类型冲突">
                  <n-tag
                    :type="
                      (analysisResult.conflictVariables?.length || 0) > 0 ? 'error' : 'success'
                    "
                    size="small"
                  >
                    {{ analysisResult.conflictVariables?.length || 0 }}
                  </n-tag>
                </n-descriptions-item>
                <n-descriptions-item label="未使用变量">
                  <n-tag
                    :type="
                      (analysisResult.unusedVariables?.length || 0) > 0 ? 'warning' : 'success'
                    "
                    size="small"
                  >
                    {{ analysisResult.unusedVariables?.length || 0 }}
                  </n-tag>
                </n-descriptions-item>
              </n-descriptions>

              <!-- 快速操作 -->
              <div style="margin-top: 24px">
                <n-text depth="3" style="margin-bottom: 12px; display: block">快速操作：</n-text>
                <n-space>
                  <n-button
                    size="small"
                    type="primary"
                    :disabled="
                      !analysisResult.missingVariables ||
                      analysisResult.missingVariables.length === 0
                    "
                    @click="handleAddAllMissing"
                  >
                    添加所有缺失变量 ({{ analysisResult.missingVariables?.length || 0 }})
                  </n-button>
                  <n-button
                    size="small"
                    type="warning"
                    :disabled="
                      !analysisResult.unusedVariables || analysisResult.unusedVariables.length === 0
                    "
                    @click="handleDeleteAllUnused"
                  >
                    删除未使用变量 ({{ analysisResult.unusedVariables?.length || 0 }})
                  </n-button>
                </n-space>
              </div>
            </n-tab-pane>
          </n-tabs>
        </div>

        <template #description> 正在分析模板变量... </template>
      </n-spin>

      <template #footer>
        <div style="display: flex; justify-content: flex-end; gap: 12px">
          <n-button @click="handleAnalyze" :loading="analyzing">重新分析</n-button>
          <n-button @click="handleClose">关闭</n-button>
        </div>
      </template>
    </n-card>
  </n-modal>
</template>

<script setup>
  import { ref, watch, h } from 'vue';
  import {
    NModal,
    NCard,
    NButton,
    NSpace,
    NText,
    NIcon,
    NSpin,
    NTabs,
    NTabPane,
    NDataTable,
    NEmpty,
    NAlert,
    NDescriptions,
    NDescriptionsItem,
    NTag,
    useMessage,
  } from 'naive-ui';
  import { CloseOutline } from '@vicons/ionicons5';
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

  const message = useMessage();

  // Refs
  const showModal = ref(false);
  const analyzing = ref(false);
  const analysisResult = ref(null);

  // ========== 表格列定义 ==========

  const missingColumns = [
    {
      title: '变量名',
      key: 'name',
      width: 150,
    },
    {
      title: '推测类型',
      key: 'type',
      width: 100,
    },
    {
      title: '出现文件',
      key: 'files',
      render: (row) => row.files?.join(', ') || '',
    },
    {
      title: '操作',
      key: 'actions',
      width: 100,
      render: (row) => {
        return h('div', { style: 'display: flex; gap: 8px;' }, [
          h(
            NButton,
            {
              size: 'tiny',
              type: 'primary',
              onClick: () => handleAddVariable(row),
            },
            () => '添加'
          ),
        ]);
      },
    },
  ];

  const detectedColumns = [
    {
      title: '变量名',
      key: 'name',
      width: 150,
    },
    {
      title: '类型',
      key: 'type',
      width: 100,
    },
    {
      title: '出现文件',
      key: 'files',
      render: (row) => row.files?.join(', ') || '',
    },
    {
      title: '上下文',
      key: 'contexts',
      render: (row) => row.contexts?.join('; ') || '',
    },
  ];

  const conflictColumns = [
    { title: '变量名', key: 'name', width: 120 },
    { title: '推测类型', key: 'type', width: 100 },
    { title: '出现文件', key: 'files', render: (row) => row.files?.join(', ') },
    { title: '冲突说明', key: 'suggestions' },
  ];

  const unusedColumns = [
    {
      title: '变量名',
      key: 'name',
      width: 200,
    },
    {
      title: '操作',
      key: 'actions',
      width: 100,
      render: (row) => {
        return h('div', { style: 'display: flex; gap: 8px;' }, [
          h(
            NButton,
            {
              size: 'tiny',
              type: 'error',
              onClick: () => handleDeleteVariable(row.name),
            },
            () => '删除'
          ),
        ]);
      },
    },
  ];

  // ========== 方法 ==========

  /**
   * 执行变量分析
   */
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

  /**
   * 添加单个变量
   */
  const handleAddVariable = (detectedVar) => {
    if (!detectedVar) return;

    emit('add-components', [detectedVar]);

    // 实时更新缺失变量列表
    if (analysisResult.value?.missingVariables) {
      analysisResult.value.missingVariables = analysisResult.value.missingVariables.filter(
        (v) => v.name !== detectedVar.name
      );
    }

    message.success(`已添加变量 "${detectedVar.name}"`);
  };

  /**
   * 添加所有缺失变量
   */
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

  /**
   * 删除单个未使用变量
   */
  const handleDeleteVariable = (variableName) => {
    emit('remove-components', [variableName]);

    // 实时更新未使用变量列表
    if (analysisResult.value?.unusedVariables) {
      analysisResult.value.unusedVariables = analysisResult.value.unusedVariables.filter(
        (v) => v !== variableName
      );
    }

    message.success(`已删除变量 "${variableName}"`);
  };

  /**
   * 删除所有未使用变量
   */
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

  /**
   * 关闭模态框
   */
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
