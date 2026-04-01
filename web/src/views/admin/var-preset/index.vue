<template>
  <div class="var-preset-manage">
    <n-flex vertical>
      <n-card :bordered="false">
        <n-form inline :label-width="80" :model="searchForm">
          <n-form-item label="预设名称">
            <n-input
              v-model:value="searchForm.name"
              placeholder="输入预设名称进行搜索"
              clearable
              style="width: 200px"
              @keyup.enter="handleSearch"
            >
              <template #prefix>
                <n-icon>
                  <SearchOutline />
                </n-icon>
              </template>
            </n-input>
          </n-form-item>
          <n-form-item label="分类">
            <n-select
              v-model:value="searchForm.category"
              placeholder="选择分类"
              clearable
              style="width: 120px"
              :options="categoryOptions"
            />
          </n-form-item>
          <n-form-item>
            <n-space>
              <n-button type="primary" @click="handleSearch">
                <template #icon>
                  <n-icon>
                    <SearchOutline />
                  </n-icon>
                </template>
                搜索
              </n-button>
              <n-button @click="handleReset">
                <template #icon>
                  <n-icon>
                    <RefreshOutline />
                  </n-icon>
                </template>
                重置
              </n-button>
            </n-space>
          </n-form-item>
        </n-form>
      </n-card>

      <n-card :bordered="false">
        <BasicTable
          ref="actionRef"
          :columns="columns"
          :request="loadDataTable"
          :row-key="(row) => row.id"
          :actionColumn="actionColumn"
          :scroll-x="1400"
        >
          <template #tableTitle>
            <n-button type="primary" @click="handleAdd">
              <template #icon>
                <n-icon>
                  <AddOutline />
                </n-icon>
              </template>
              新建预设
            </n-button>
          </template>
        </BasicTable>
      </n-card>
    </n-flex>

    <!-- 添加/编辑变量预设弹窗 -->
    <n-modal v-model:show="showAddModal" :mask-closable="false">
      <n-card
        style="width: 600px"
        :title="editingPreset ? '编辑变量预设' : '添加变量预设'"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <template #header-extra>
          <n-button quaternary circle @click="closeModal">
            <template #icon>
              <n-icon>
                <CloseOutline />
              </n-icon>
            </template>
          </n-button>
        </template>

        <n-form ref="formRef" :model="formData" :rules="formRules" label-placement="top">
          <n-grid :cols="2" :x-gap="16">
            <n-grid-item>
              <n-form-item label="预设名称" path="name">
                <n-input
                  v-model:value="formData.name"
                  placeholder="请输入预设名称"
                  :maxlength="50"
                  show-count
                />
              </n-form-item>
            </n-grid-item>
            <n-grid-item>
              <n-form-item label="显示名称" path="displayName">
                <n-input
                  v-model:value="formData.displayName"
                  placeholder="请输入显示名称"
                  :maxlength="100"
                  show-count
                />
              </n-form-item>
            </n-grid-item>
          </n-grid>

          <n-form-item label="描述" path="description">
            <n-input
              v-model:value="formData.description"
              type="textarea"
              placeholder="请输入预设描述"
              :maxlength="500"
              show-count
              :rows="3"
            />
          </n-form-item>

          <n-grid :cols="3" :x-gap="16">
            <n-grid-item>
              <n-form-item label="分类" path="category">
                <n-select
                  v-model:value="formData.category"
                  placeholder="选择分类"
                  :options="categoryOptions"
                />
              </n-form-item>
            </n-grid-item>
            <n-grid-item>
              <n-form-item label="图标" path="icon">
                <n-input v-model:value="formData.icon" placeholder="图标名称或URL" />
              </n-form-item>
            </n-grid-item>
            <n-grid-item>
              <n-form-item label="排序权重" path="sort">
                <n-input-number
                  v-model:value="formData.sort"
                  placeholder="数值越大越靠前"
                  :min="0"
                  :max="999"
                  style="width: 100%"
                />
              </n-form-item>
            </n-grid-item>
          </n-grid>

          <n-grid :cols="2" :x-gap="16">
            <n-grid-item>
              <n-form-item label="版本" path="version">
                <n-input v-model:value="formData.version" placeholder="如: 1.0" />
              </n-form-item>
            </n-grid-item>
            <n-grid-item>
              <n-form-item label="状态" path="isEnabled">
                <n-switch v-model:value="formData.isEnabled" />
                <span style="margin-left: 8px; color: #666">{{
                  formData.isEnabled ? '启用' : '禁用'
                }}</span>
              </n-form-item>
            </n-grid-item>
          </n-grid>

          <n-form-item label="默认数据" path="defaultDataJson">
            <n-input
              v-model:value="formData.defaultDataJson"
              type="textarea"
              placeholder="可选，提供默认数据值..."
              :rows="4"
            />
          </n-form-item>
        </n-form>

        <template #footer>
          <div class="modal-footer">
            <n-button @click="closeModal">取消</n-button>
            <n-button type="primary" @click="handleSubmit" :loading="submitting">
              {{ editingPreset ? '更新' : '添加' }}
            </n-button>
          </div>
        </template>
      </n-card>
    </n-modal>

    <!-- 预览弹窗 -->
    <n-modal v-model:show="showPreviewModal" :mask-closable="true">
      <n-card
        style="width: 80%; max-width: 800px"
        title="预设详情"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <template #header-extra>
          <n-button quaternary circle @click="showPreviewModal = false">
            <template #icon>
              <n-icon>
                <CloseOutline />
              </n-icon>
            </template>
          </n-button>
        </template>

        <div v-if="previewData" class="preview-content">
          <div class="preview-info">
            <n-descriptions :column="2" bordered>
              <n-descriptions-item label="预设名称">{{ previewData.name }}</n-descriptions-item>
              <n-descriptions-item label="显示名称">{{
                previewData.displayName
              }}</n-descriptions-item>
              <n-descriptions-item label="分类">
                <n-tag :type="previewData.category === 'system' ? 'info' : 'success'">
                  {{ previewData.category === 'system' ? '系统' : '自定义' }}
                </n-tag>
              </n-descriptions-item>
              <n-descriptions-item label="版本">{{ previewData.version }}</n-descriptions-item>
              <n-descriptions-item label="状态">
                <n-tag :type="previewData.isEnabled === 1 ? 'success' : 'error'">
                  {{ previewData.isEnabled === 1 ? '启用' : '禁用' }}
                </n-tag>
              </n-descriptions-item>
              <n-descriptions-item label="排序权重">{{ previewData.sort }}</n-descriptions-item>
              <n-descriptions-item label="描述" :span="2">{{
                previewData.description || '无'
              }}</n-descriptions-item>
            </n-descriptions>
          </div>

          <div class="preview-schema" style="margin-top: 20px">
            <h4>数据结构模板：</h4>
            <pre class="json-code" style="max-height: 300px; overflow-y: auto">{{
              formatJson(previewData.schemaJson)
            }}</pre>
          </div>

          <div v-if="previewData.defaultDataJson" class="preview-default" style="margin-top: 20px">
            <h4>默认数据：</h4>
            <pre class="json-code" style="max-height: 200px; overflow-y: auto">{{
              formatJson(previewData.defaultDataJson)
            }}</pre>
          </div>
        </div>
      </n-card>
    </n-modal>

    <!-- 删除确认弹窗 -->
    <n-modal v-model:show="showDeleteModal" :mask-closable="false">
      <n-card
        style="width: 400px"
        title="确认删除"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <div class="delete-content">
          <div class="delete-icon">
            <n-icon size="48" color="#d03050">
              <TrashOutline />
            </n-icon>
          </div>
          <p class="delete-message">
            确定要删除变量预设
            <strong>"{{ deletingPreset?.displayName || deletingPreset?.name }}"</strong> 吗？
          </p>
          <p class="delete-warning"> 删除后将解除与所有模板的关联关系，此操作不可撤销。 </p>
        </div>

        <template #footer>
          <div class="modal-footer">
            <n-button @click="showDeleteModal = false">取消</n-button>
            <n-button type="error" @click="confirmDelete" :loading="deleting"> 确认删除 </n-button>
          </div>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup>
  import { ref, reactive, h } from 'vue';
  import { useRouter } from 'vue-router';
  import { BasicTable, TableAction } from '@/components/Table';
  import { NButton, NIcon, NTag, useMessage, NDescriptions, NDescriptionsItem } from 'naive-ui';
  import {
    AddOutline,
    CloseOutline,
    TrashOutline,
    CreateOutline,
    SearchOutline,
    RefreshOutline,
    EyeOutline,
    ToggleOutline,
    DocumentTextOutline,
  } from '@vicons/ionicons5';
  import {
    listVarPresets,
    addVarPreset,
    editVarPreset,
    deleteVarPreset,
    toggleVarPreset,
    getVarPresetDetail,
  } from '@/api/varPreset';
  import { columns as baseColumns } from './columns';

  const message = useMessage();
  const router = useRouter();
  const actionRef = ref();

  // 选项数据
  const categoryOptions = [
    { label: '系统', value: 'system' },
    { label: '自定义', value: 'custom' },
  ];

  // 数据状态
  const submitting = ref(false);
  const deleting = ref(false);

  // 搜索表单
  const searchForm = reactive({
    name: '',
    category: '',
  });

  // 弹窗状态
  const showAddModal = ref(false);
  const showDeleteModal = ref(false);
  const showPreviewModal = ref(false);
  const editingPreset = ref(null);
  const deletingPreset = ref(null);
  const previewData = ref(null);

  // 表单数据
  const formRef = ref(null);
  const formData = reactive({
    name: '',
    displayName: '',
    description: '',
    category: 'custom',
    schemaJson: '',
    defaultDataJson: '',
    icon: '',
    sort: 0,
    version: '1.0',
    isEnabled: true,
  });

  // 表单验证规则
  const formRules = {
    name: {
      required: true,
      message: '请输入预设名称',
      trigger: ['input', 'blur'],
    },
    displayName: {
      required: true,
      message: '请输入显示名称',
      trigger: ['input', 'blur'],
    },
    category: {
      required: true,
      message: '请选择分类',
      trigger: ['change'],
    },
  };

  // 使用基础列配置
  const columns = baseColumns;

  // 操作列
  const actionColumn = reactive({
    width: 380,
    title: '操作',
    key: 'action',
    fixed: 'right',
    align: 'center',
    render(record) {
      return h(TableAction, {
        style: 'button',
        actions: createActions(record),
      });
    },
  });

  function createActions(record) {
    return [
      {
        label: '预览',
        icon: EyeOutline,
        onClick: handlePreview.bind(null, record),
      },
      {
        label: '编辑',
        icon: CreateOutline,
        onClick: handleEdit.bind(null, record),
      },
      {
        label: '设计结构',
        icon: DocumentTextOutline,
        onClick: handleDesignSchema.bind(null, record),
      },
      {
        label: record.isEnabled === 1 ? '禁用' : '启用',
        icon: ToggleOutline,
        onClick: handleToggle.bind(null, record),
      },
      {
        label: '删除',
        icon: TrashOutline,
        onClick: handleDelete.bind(null, record),
        ifShow: () => record.category !== 'system',
      },
    ];
  }

  // 加载数据
  const loadDataTable = async (res) => {
    try {
      const params = {
        pageNum: res.page,
        pageSize: res.pageSize,
      };

      // 添加搜索条件
      if (searchForm.name) {
        params.name = searchForm.name;
      }
      if (searchForm.category) {
        params.category = searchForm.category;
      }

      const response = await listVarPresets(params);

      // BasicTable期望返回 { list: [], itemCount: number } 格式
      const result = {
        list: response.data.data.varPresetsList || [],
        itemCount: response.data.data.total || 0,
      };
      return result;
    } catch (error) {
      console.error('获取变量预设列表失败:', error);
      message.error('获取变量预设列表失败');
      return {
        list: [],
        itemCount: 0,
      };
    }
  };

  // 刷新表格
  function reloadTable() {
    actionRef.value?.reload();
  }

  // 搜索
  function handleSearch() {
    reloadTable();
  }

  // 重置搜索
  function handleReset() {
    searchForm.name = '';
    searchForm.category = '';
    reloadTable();
  }

  // 添加
  const handleAdd = () => {
    editingPreset.value = null;
    resetForm();
    showAddModal.value = true;
  };

  // 编辑
  const handleEdit = (preset) => {
    editingPreset.value = preset;
    formData.name = preset.name;
    formData.displayName = preset.displayName || '';
    formData.description = preset.description || '';
    formData.category = preset.category || 'custom';
    formData.schemaJson = preset.schemaJson || '';
    formData.defaultDataJson = preset.defaultDataJson || '';
    formData.icon = preset.icon || '';
    formData.sort = preset.sort || 0;
    formData.version = preset.version || '1.0';
    formData.isEnabled = preset.isEnabled === 1;
    showAddModal.value = true;
  };

  // 删除
  const handleDelete = (preset) => {
    deletingPreset.value = preset;
    showDeleteModal.value = true;
  };

  // 预览
  const handlePreview = async (preset) => {
    try {
      const response = await getVarPresetDetail({ id: preset.id });
      // API返回: {code: 0, data: {varPreset: {...}}, message: "OK"}
      const varPreset = response.data.data.varPreset;
      // 转换字段命名从 snake_case 到 camelCase
      previewData.value = {
        ...varPreset,
        defaultDataJson: varPreset.defaultDataJson || varPreset.default_data_json || '',
        schemaJson: varPreset.schemaJson || varPreset.schema_json || '',
        createdAt: varPreset.createdAt || varPreset.created_at,
      };
      showPreviewModal.value = true;
    } catch (error) {
      console.error('获取预设详情失败:', error);
      message.error('获取预设详情失败');
      // 出错时使用当前行数据
      previewData.value = {
        ...preset,
        defaultDataJson: preset.defaultDataJson || preset.default_data_json || '',
        schemaJson: preset.schemaJson || preset.schema_json || '',
      };
      showPreviewModal.value = true;
    }
  };

  // 设计结构
  const handleDesignSchema = (preset) => {
    router.push({
      name: 'var-preset-design',
      params: { id: preset.id },
    });
  };

  // 切换状态
  const handleToggle = async (preset) => {
    try {
      const newStatus = preset.isEnabled === 1 ? 0 : 1;
      await toggleVarPreset({ id: preset.id });
      message.success(`变量预设已${newStatus === 1 ? '启用' : '禁用'}`);
      reloadTable();
    } catch (error) {
      console.error('状态切换失败:', error);
      message.error('状态切换失败');
    }
  };

  // 关闭弹窗
  const closeModal = () => {
    showAddModal.value = false;
    editingPreset.value = null;
    resetForm();
  };

  // 重置表单
  const resetForm = () => {
    formData.name = '';
    formData.displayName = '';
    formData.description = '';
    formData.category = 'custom';
    formData.schemaJson = '';
    formData.defaultDataJson = '';
    formData.icon = '';
    formData.sort = 0;
    formData.version = '1.0';
    formData.isEnabled = true;
    formRef.value?.restoreValidation();
  };

  // 提交表单
  const handleSubmit = async () => {
    try {
      await formRef.value?.validate();
      submitting.value = true;

      const data = {
        name: formData.name,
        displayName: formData.displayName,
        description: formData.description,
        category: formData.category,
        schemaJson: formData.schemaJson,
        defaultDataJson: formData.defaultDataJson,
        icon: formData.icon,
        sort: formData.sort,
        version: formData.version,
        isEnabled: formData.isEnabled ? 1 : 0,
      };

      if (editingPreset.value) {
        // 编辑
        await editVarPreset({ ...data, id: editingPreset.value.id });
        message.success('变量预设更新成功');
      } else {
        // 添加
        await addVarPreset(data);
        message.success('变量预设添加成功');
      }

      closeModal();
      reloadTable();
    } catch (error) {
      console.error('操作失败:', error);
      message.error(editingPreset.value ? '更新变量预设失败' : '添加变量预设失败');
    } finally {
      submitting.value = false;
    }
  };

  // 确认删除
  const confirmDelete = async () => {
    try {
      deleting.value = true;
      await deleteVarPreset({ id: deletingPreset.value.id });
      message.success('变量预设删除成功');
      showDeleteModal.value = false;
      deletingPreset.value = null;
      reloadTable();
    } catch (error) {
      console.error('删除变量预设失败:', error);
      message.error('删除变量预设失败');
    } finally {
      deleting.value = false;
    }
  };

  // 格式化JSON
  const formatJson = (jsonString) => {
    if (!jsonString) return '{}';
    try {
      const obj = typeof jsonString === 'string' ? JSON.parse(jsonString) : jsonString;
      return JSON.stringify(obj, null, 2);
    } catch (error) {
      return jsonString;
    }
  };
</script>

<style scoped>
  .var-preset-manage {
    padding: 16px;
    background: transparent;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }

  .delete-content {
    text-align: center;
    padding: 20px 0;
  }

  .delete-icon {
    margin-bottom: 16px;
  }

  .delete-message {
    font-size: 16px;
    color: #333;
    margin-bottom: 12px;
  }

  .delete-warning {
    font-size: 14px;
    color: #d03050;
    background: #fff2f0;
    padding: 12px;
    border-radius: 6px;
    border: 1px solid #ffccc7;
  }

  .preset-display-name {
    color: #333;
    font-weight: 500;
  }

  .text-placeholder {
    color: #999;
    font-style: italic;
  }

  .preview-content h4 {
    margin-bottom: 12px;
    color: #333;
    font-weight: 600;
  }

  .json-code {
    background: #f5f7fa;
    border: 1px solid #e0e6ed;
    border-radius: 4px;
    padding: 12px;
    font-family: 'Courier New', Consolas, monospace;
    font-size: 13px;
    line-height: 1.5;
    color: #333;
    white-space: pre-wrap;
    word-break: break-all;
  }
</style>
