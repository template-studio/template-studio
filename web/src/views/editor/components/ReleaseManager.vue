<template>
  <n-modal
    v-model:show="modalVisible"
    preset="card"
    style="width: 800px; max-height: 80vh"
    :mask-closable="false"
  >
    <template #header>
      <div class="release-header">
        <n-icon size="20" style="margin-right: 8px">
          <svg viewBox="0 0 24 24">
            <path
              fill="currentColor"
              d="M12,2C6.48,2 2,6.48 2,12s4.48,10 10,10s10-4.48 10-10S17.52,2 12,2z M13,17h-2v-2h2V17z M13,13h-2V7h2V13z"
            />
          </svg>
        </n-icon>
        <span class="modal-title">版本管理</span>
      </div>
    </template>

    <div class="release-container">
      <!-- 操作区 -->
      <div class="release-actions">
        <n-space>
          <n-button type="primary" @click="showCreateReleaseModal">
            <template #icon>
              <n-icon>
                <svg viewBox="0 0 24 24">
                  <path fill="currentColor" d="M19,13h-6v6h-2v-6H5v-2h6V5h2v6h6V13z" />
                </svg>
              </n-icon>
            </template>
            创建发布
          </n-button>
          <n-button @click="loadReleases">
            <template #icon>
              <n-icon>
                <svg viewBox="0 0 24 24">
                  <path
                    fill="currentColor"
                    d="M17.65,6.35C16.2,4.9 14.21,4 12,4c-4.42,0-7.99,3.58-7.99,8s3.57,8 7.99,8c3.73,0 6.84-2.55 7.73-6h-2.08 c-0.82,2.33-3.04,4-5.65,4c-3.31,0-6-2.69-6-6s2.69-6 6-6c1.66,0 3.14,0.69 4.22,1.78L13,11h7V4L17.65,6.35z"
                  />
                </svg>
              </n-icon>
            </template>
            刷新
          </n-button>
          <n-popconfirm @positive-click="handleResetToLatest">
            <template #trigger>
              <n-button type="warning" :disabled="!hasLatestVersion">
                <template #icon>
                  <n-icon>
                    <svg viewBox="0 0 24 24">
                      <path
                        fill="currentColor"
                        d="M12 3a9 9 0 0 0-9 9H0l4 4 4-4H5a7 7 0 0 1 7-7 7 7 0 0 1 7 7 7 7 0 0 1-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42A8.96 8.96 0 0 0 12 21a9 9 0 0 0 9-9 9 9 0 0 0-9-9m-1 5v5l4.25 2.52.77-1.28-3.52-2.09V8z"
                      />
                    </svg>
                  </n-icon>
                </template>
                重置
              </n-button>
            </template>
            <div style="max-width: 280px">
              <p style="font-weight: 500; margin-bottom: 8px">确认重置？</p>
              <p style="color: #666; font-size: 13px">
                将舍弃所有未提交的更改，恢复到最新发布版本
              </p>
            </div>
          </n-popconfirm>
        </n-space>
      </div>

      <!-- 版本列表 -->
      <n-spin :show="loading">
        <n-data-table
          :columns="columns"
          :data="versions"
          :row-key="(row) => row.id"
          :pagination="pagination"
          size="small"
        />
      </n-spin>
    </div>
  </n-modal>

  <!-- 创建发布对话框 -->
  <n-modal
    v-model:show="createModalVisible"
    preset="card"
    style="width: 500px"
    title="创建发布"
    :mask-closable="false"
  >
    <n-form
      ref="createFormRef"
      :model="createForm"
      :rules="createRules"
      label-placement="left"
      label-width="100px"
    >
      <n-form-item label="版本号" path="version">
        <n-input
          v-model:value="createForm.version"
          placeholder="留空则自动生成（如 v1.1.0）"
          clearable
        />
      </n-form-item>

      <n-form-item label="发布日志" path="changelog">
        <n-input
          v-model:value="createForm.changelog"
          type="textarea"
          placeholder="描述本次发布的变更内容"
          :rows="4"
        />
      </n-form-item>

      <n-form-item label="Git提交信息" path="message">
        <n-input
          v-model:value="createForm.message"
          type="textarea"
          placeholder="Git提交信息（留空使用默认）"
          :rows="2"
        />
      </n-form-item>
    </n-form>

    <template #footer>
      <n-space justify="end">
        <n-button @click="createModalVisible = false">取消</n-button>
        <n-button type="primary" @click="handleCreateRelease" :loading="creating">
          创建发布
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup>
  import { ref, computed, watch, h } from 'vue';
  import { NButton, NTag, NSpace, NPopconfirm, useMessage, useDialog } from 'naive-ui';
  import { listReleases, createRelease, rollbackVersion, deprecateVersion, resetToLatest } from '@/api/releases';

  const props = defineProps({
    show: {
      type: Boolean,
      default: false,
    },
    templateId: {
      type: Number,
      required: true,
    },
  });

  const emit = defineEmits(['update:show', 'reset']);

  const message = useMessage();
  const dialog = useDialog();

  // 对话框显示状态
  const modalVisible = computed({
    get: () => props.show,
    set: (val) => emit('update:show', val),
  });

  // 数据加载状态
  const loading = ref(false);
  const creating = ref(false);
  const versions = ref([]);

  // 分页
  const pagination = ref({
    pageSize: 10,
  });

  // 创建发布表单
  const createModalVisible = ref(false);
  const createFormRef = ref(null);
  const createForm = ref({
    version: '',
    changelog: '',
    message: '',
  });

  const createRules = {
    // 版本号可选，但如果填写需要符合格式
    version: {
      pattern: /^v?\d+\.\d+\.\d+$/,
      message: '版本号格式错误（如 v1.0.0 或 1.0.0）',
      trigger: ['blur', 'input'],
    },
  };

  // 表格列定义
  const columns = [
    {
      title: '版本号',
      key: 'version',
      width: 120,
      render: (row) => {
        const tags = [];
        // 兼容后端的 snake_case 和前端的 camelCase
        const isLatest = row.isLatest ?? row.is_latest ?? false;
        const isDeprecated = row.isDeprecated ?? row.is_deprecated ?? false;

        if (isLatest) {
          tags.push(
            h(
              NTag,
              { type: 'success', size: 'small', style: { marginRight: '4px' } },
              { default: () => '最新' }
            )
          );
        }
        if (isDeprecated) {
          tags.push(h(NTag, { type: 'warning', size: 'small' }, { default: () => '已弃用' }));
        }
        return h('div', { style: { display: 'flex', alignItems: 'center' } }, [
          h('span', { style: { fontWeight: 'bold', marginRight: '8px' } }, row.version),
          ...tags,
        ]);
      },
    },
    {
      title: '发布日志',
      key: 'changelog',
      ellipsis: {
        tooltip: true,
      },
    },
    {
      title: '文件统计',
      key: 'fileCount',
      width: 100,
      render: (row) => {
        // 兼容后端的 snake_case 和前端的 camelCase
        const fileCount = row.fileCount ?? row.file_count ?? 0;
        const totalSize = row.totalSize ?? row.total_size ?? 0;
        const sizeMB = (totalSize / 1024 / 1024).toFixed(2);
        return h('div', [
          h('div', `${fileCount} 个文件`),
          h('div', { style: { fontSize: '12px', color: '#999' } }, `${sizeMB} MB`),
        ]);
      },
    },
    {
      title: '创建者',
      key: 'creatorName',
      width: 100,
      render: (row) => {
        return row.creatorName ?? row.creator_name ?? 'System';
      },
    },
    {
      title: '创建时间',
      key: 'createdAt',
      width: 180,
      render: (row) => {
        // 兼容后端的 snake_case 和前端的 camelCase
        const createdAt = row.createdAt ?? row.created_at;
        const date = new Date(createdAt);
        if (isNaN(date.getTime())) {
          return h('div', { style: { color: '#999' } }, '无效日期');
        }
        return h('div', [
          h('div', date.toLocaleDateString('zh-CN')),
          h(
            'div',
            { style: { fontSize: '12px', color: '#999' } },
            date.toLocaleTimeString('zh-CN')
          ),
        ]);
      },
    },
    {
      title: '操作',
      key: 'actions',
      width: 150,
      fixed: 'right',
      render: (row) => {
        const actions = [];

        // 兼容后端的 snake_case 和前端的 camelCase
        const isLatest = row.isLatest ?? row.is_latest ?? false;
        const isDeprecated = row.isDeprecated ?? row.is_deprecated ?? false;

        // 调试日志
        console.log('Row data:', row, { isLatest, isDeprecated });

        // 如果不是最新版本，显示回滚按钮
        if (!isLatest) {
          actions.push(
            h(
              NPopconfirm,
              {
                positiveText: '确认',
                negativeText: '取消',
                onPositiveClick: () => handleRollback(row.version),
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    { size: 'small', style: { marginRight: '4px' } },
                    { default: () => '回滚' }
                  ),
                default: () => `确定要回滚到版本 ${row.version} 吗？`,
              }
            )
          );
        }

        // 如果未弃用且不是最新版本，显示弃用按钮
        if (!isDeprecated && !isLatest) {
          actions.push(
            h(
              NPopconfirm,
              {
                positiveText: '确认',
                negativeText: '取消',
                onPositiveClick: () => handleDeprecate(row.version),
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    { size: 'small', type: 'warning', ghost: true },
                    { default: () => '弃用' }
                  ),
                default: () => `确定要弃用版本 ${row.version} 吗？`,
              }
            )
          );
        }

        // 如果没有操作按钮，显示提示
        if (actions.length === 0) {
          return h('span', { style: { color: '#999', fontSize: '12px' } }, '当前版本，无需操作');
        }

        return h(NSpace, { size: 'small' }, { default: () => actions });
      },
    },
  ];

  // 加载版本列表
  const loadReleases = async () => {
    loading.value = true;
    try {
      const res = await listReleases(props.templateId);
      // 响应拦截器返回的是 response 对象，所以数据在 res.data
      const data = res.data;
      if (data.code === 0) {
        versions.value = data.data.versions || [];
      } else {
        message.error(data.message || '加载版本列表失败');
      }
    } catch (error) {
      console.error('加载版本列表失败:', error);
      message.error('加载版本列表失败');
    } finally {
      loading.value = false;
    }
  };

  // 显示创建发布对话框
  const showCreateReleaseModal = () => {
    createForm.value = {
      version: '',
      changelog: '',
      message: '',
    };
    createModalVisible.value = true;
  };

  // 创建发布
  const handleCreateRelease = async () => {
    try {
      await createFormRef.value?.validate();

      creating.value = true;
      const data = {
        version: createForm.value.version || undefined,
        changelog: createForm.value.changelog || undefined,
        message: createForm.value.message || undefined,
      };

      const res = await createRelease(props.templateId, data);
      const responseData = res.data;
      if (responseData.code === 0) {
        message.success('发布成功');
        createModalVisible.value = false;
        await loadReleases();
      } else {
        message.error(responseData.message || '发布失败');
      }
    } catch (error) {
      console.error('创建发布失败:', error);
      message.error('创建发布失败');
    } finally {
      creating.value = false;
    }
  };

  // 回滚版本
  const handleRollback = async (version) => {
    try {
      const res = await rollbackVersion(props.templateId, version);
      const responseData = res.data;
      if (responseData.code === 0) {
        message.success(`已回滚到版本 ${version}`);
        await loadReleases();
      } else {
        message.error(responseData.message || '回滚失败');
      }
    } catch (error) {
      console.error('回滚失败:', error);
      message.error('回滚失败');
    }
  };

  // 弃用版本
  const handleDeprecate = async (version) => {
    try {
      const res = await deprecateVersion(props.templateId, version);
      const responseData = res.data;
      if (responseData.code === 0) {
        message.success(`版本 ${version} 已标记为弃用`);
        await loadReleases();
      } else {
        message.error(responseData.message || '操作失败');
      }
    } catch (error) {
      console.error('弃用版本失败:', error);
      message.error('操作失败');
    }
  };

  // 是否有最新版本（用于重置按钮状态）
  const hasLatestVersion = computed(() => {
    return versions.value.some((v) => v.isLatest ?? v.is_latest ?? false);
  });

  // 重置到最新版本
  const resetting = ref(false);
  const handleResetToLatest = async () => {
    if (resetting.value) return;

    resetting.value = true;
    try {
      const res = await resetToLatest(props.templateId);
      const responseData = res.data;
      if (responseData.code === 0) {
        const data = responseData.data;
        message.success(`已重置到版本 ${data.version}`);
        // 触发文件树刷新（通过 emit 事件）
        emit('reset');
      } else {
        message.error(responseData.message || '重置失败');
      }
    } catch (error) {
      console.error('重置到最新版本失败:', error);
      message.error('重置失败');
    } finally {
      resetting.value = false;
    }
  };

  // 监听对话框显示状态
  watch(
    () => props.show,
    (newVal) => {
      if (newVal) {
        loadReleases();
      }
    }
  );
</script>

<style scoped>
  .release-header {
    display: flex;
    align-items: center;
  }

  .modal-title {
    font-size: 16px;
    font-weight: 500;
  }

  .release-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .release-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 12px;
    border-bottom: 1px solid #f0f0f0;
  }
</style>
