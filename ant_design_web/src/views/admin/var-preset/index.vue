<template>
  <div class="var-preset-manage">
    <div style="display: flex; flex-direction: column; gap: 16px">
      <a-card :bordered="false">
        <a-form layout="inline" :model="searchForm">
          <a-form-item label="预设名称">
            <a-input
              v-model:value="searchForm.name"
              placeholder="输入预设名称进行搜索"
              allow-clear
              style="width: 200px"
              @keyup.enter="handleSearch"
            >
              <template #prefix>
                <SearchOutline />
              </template>
            </a-input>
          </a-form-item>
          <a-form-item label="分类">
            <a-select
              v-model:value="searchForm.category"
              placeholder="选择分类"
              allow-clear
              style="width: 120px"
              :options="categoryOptions"
            />
          </a-form-item>
          <a-form-item>
            <a-space>
              <a-button type="primary" @click="handleSearch">
                <template #icon>
                  <SearchOutline />
                </template>
                搜索
              </a-button>
              <a-button @click="handleReset">
                <template #icon>
                  <RefreshOutline />
                </template>
                重置
              </a-button>
            </a-space>
          </a-form-item>
        </a-form>
      </a-card>

      <a-card :bordered="false">
        <BasicTable
          ref="actionRef"
          :columns="columns"
          :request="loadDataTable"
          :row-key="(row) => row.id"
          :actionColumn="actionColumn"
          :scroll-x="1400"
        >
          <template #tableTitle>
            <a-button type="primary" @click="handleAdd">
              <template #icon>
                <AddOutline />
              </template>
              新建预设
            </a-button>
          </template>
        </BasicTable>
      </a-card>
    </div>

    <!-- 添加/编辑变量预设弹窗 -->
    <a-modal v-model:open="showAddModal" :mask-closable="false" :title="editingPreset ? '编辑变量预设' : '添加变量预设'" width="600px">
      <a-form ref="formRef" :model="formData" :rules="formRules" layout="vertical">
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="预设名称" name="name">
              <a-input
                v-model:value="formData.name"
                placeholder="请输入预设名称"
                :maxlength="50"
                show-count
              />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="显示名称" name="displayName">
              <a-input
                v-model:value="formData.displayName"
                placeholder="请输入显示名称"
                :maxlength="100"
                show-count
              />
            </a-form-item>
          </a-col>
        </a-row>

        <a-form-item label="描述" name="description">
          <a-textarea
            v-model:value="formData.description"
            placeholder="请输入预设描述"
            :maxlength="500"
            show-count
            :rows="3"
          />
        </a-form-item>

        <a-row :gutter="16">
          <a-col :span="8">
            <a-form-item label="分类" name="category">
              <a-select
                v-model:value="formData.category"
                placeholder="选择分类"
                :options="categoryOptions"
              />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="图标" name="icon">
              <a-input v-model:value="formData.icon" placeholder="图标名称或URL" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="排序权重" name="sort">
              <a-input-number
                v-model:value="formData.sort"
                placeholder="数值越大越靠前"
                :min="0"
                :max="999"
                style="width: 100%"
              />
            </a-form-item>
          </a-col>
        </a-row>

        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="版本" name="version">
              <a-input v-model:value="formData.version" placeholder="如: 1.0" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="状态" name="isEnabled">
              <a-switch v-model:checked="formData.isEnabled" />
              <span style="margin-left: 8px; color: #666">{{
                formData.isEnabled ? '启用' : '禁用'
              }}</span>
            </a-form-item>
          </a-col>
        </a-row>

        <a-form-item label="默认数据" name="defaultDataJson">
          <a-textarea
            v-model:value="formData.defaultDataJson"
            placeholder="可选，提供默认数据值..."
            :rows="4"
          />
        </a-form-item>
      </a-form>

      <template #footer>
        <div class="modal-footer">
          <a-button @click="closeModal">取消</a-button>
          <a-button type="primary" @click="handleSubmit" :loading="submitting">
            {{ editingPreset ? '更新' : '添加' }}
          </a-button>
        </div>
      </template>
    </a-modal>

    <!-- 预览弹窗 -->
    <a-modal v-model:open="showPreviewModal" :mask-closable="true" title="预设详情" width="80%" :style="{ maxWidth: '800px' }">
      <div v-if="previewData" class="preview-content">
        <div class="preview-info">
          <a-descriptions :column="2" bordered>
            <a-descriptions-item label="预设名称">{{ previewData.name }}</a-descriptions-item>
            <a-descriptions-item label="显示名称">{{
              previewData.displayName
            }}</a-descriptions-item>
            <a-descriptions-item label="分类">
              <a-tag :color="previewData.category === 'system' ? 'blue' : 'green'">
                {{ previewData.category === 'system' ? '系统' : '自定义' }}
              </a-tag>
            </a-descriptions-item>
            <a-descriptions-item label="版本">{{ previewData.version }}</a-descriptions-item>
            <a-descriptions-item label="状态">
              <a-tag :color="previewData.isEnabled === 1 ? 'green' : 'red'">
                {{ previewData.isEnabled === 1 ? '启用' : '禁用' }}
              </a-tag>
            </a-descriptions-item>
            <a-descriptions-item label="排序权重">{{ previewData.sort }}</a-descriptions-item>
            <a-descriptions-item label="描述" :span="2">{{
              previewData.description || '无'
            }}</a-descriptions-item>
          </a-descriptions>
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
    </a-modal>

    <!-- 删除确认弹窗 -->
    <a-modal v-model:open="showDeleteModal" :mask-closable="false" title="确认删除" width="400px">
      <div class="delete-content">
        <div class="delete-icon">
          <TrashOutline style="font-size: 48px; color: #d03050" />
        </div>
        <p class="delete-message">
          确定要删除变量预设
          <strong>"{{ deletingPreset?.displayName || deletingPreset?.name }}"</strong> 吗？
        </p>
        <p class="delete-warning"> 删除后将解除与所有模板的关联关系，此操作不可撤销。 </p>
      </div>

      <template #footer>
        <div class="modal-footer">
          <a-button @click="showDeleteModal = false">取消</a-button>
          <a-button danger @click="confirmDelete" :loading="deleting"> 确认删除 </a-button>
        </div>
      </template>
    </a-modal>
  </div>
</template>

<script setup>
  import { ref, reactive, h } from 'vue';
  import { useRouter } from 'vue-router';
  import { BasicTable, TableAction } from '@/components/Table';
  import { message } from 'ant-design-vue';
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
  } from '@/icons/ionicons5';
  import {
    listVarPresets,
    addVarPreset,
    editVarPreset,
    deleteVarPreset,
    toggleVarPreset,
    getVarPresetDetail,
  } from '@/api/varPreset';
  import { columns as baseColumns } from './columns';

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
    formRef.value?.resetFields();
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
