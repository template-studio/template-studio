<template>
  <div class="languages-manage">
    <n-flex vertical>
      <n-card :bordered="false">
        <n-form inline :label-width="80" :model="searchForm">
          <n-form-item label="关键词">
            <n-input
              v-model:value="searchForm.name"
              placeholder="输入语言名称或代码进行搜索"
              clearable
              style="width: 240px"
              @keyup.enter="handleSearch"
            >
              <template #prefix>
                <n-icon>
                  <SearchOutline />
                </n-icon>
              </template>
            </n-input>
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
          :scroll-x="1200"
        >
          <template #tableTitle>
            <n-button type="primary" @click="handleAdd">
              <template #icon>
                <n-icon>
                  <AddOutline />
                </n-icon>
              </template>
              新建语言
            </n-button>
          </template>
        </BasicTable>
      </n-card>
    </n-flex>

    <!-- 添加/编辑语言弹窗 -->
    <n-modal v-model:show="showAddModal" :mask-closable="false">
      <n-card
        style="width: 500px"
        :title="editingLanguage ? '编辑语言' : '添加语言'"
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
          <n-form-item label="语言名称" path="name">
            <n-input
              v-model:value="formData.name"
              placeholder="请输入语言名称（如：JavaScript）"
              :maxlength="50"
              show-count
            />
          </n-form-item>

          <n-form-item label="显示名称" path="displayName">
            <n-input
              v-model:value="formData.displayName"
              placeholder="请输入显示名称（如：JavaScript）"
              :maxlength="50"
              show-count
            />
          </n-form-item>

          <n-form-item label="语言代码" path="code">
            <n-input
              v-model:value="formData.code"
              placeholder="请输入语言代码（如：js）"
              :maxlength="20"
              show-count
            />
          </n-form-item>

          <n-form-item label="排序" path="sort">
            <n-input-number
              v-model:value="formData.sort"
              placeholder="排序值，数字越小越靠前"
              :min="0"
              :max="9999"
              style="width: 100%"
            />
          </n-form-item>

          <n-form-item label="热门语言" path="isPopular">
            <n-switch v-model:value="formData.isPopular" :checked-value="1" :unchecked-value="0">
              <template #checked>是</template>
              <template #unchecked>否</template>
            </n-switch>
          </n-form-item>
        </n-form>

        <template #footer>
          <div class="modal-footer">
            <n-button @click="closeModal">取消</n-button>
            <n-button type="primary" @click="handleSubmit" :loading="submitting">
              {{ editingLanguage ? '更新' : '添加' }}
            </n-button>
          </div>
        </template>
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
            确定要删除语言 <strong>"{{ deletingLanguage?.name }}"</strong> 吗？
          </p>
          <p class="delete-warning"> 此操作不可撤销，删除后相关模板的语言信息可能会受到影响。 </p>
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
  import { BasicTable, TableAction } from '@/components/Table';
  import { NButton, NIcon, useMessage } from 'naive-ui';
  import {
    AddOutline,
    CloseOutline,
    TrashOutline,
    CreateOutline,
    SearchOutline,
    RefreshOutline,
  } from '@vicons/ionicons5';
  import { listLanguages, addLanguage, editLanguage, deleteLanguage } from '@/api/languages';
  import { columns as baseColumns } from './columns';

  const message = useMessage();
  const actionRef = ref();

  // 数据状态
  const submitting = ref(false);
  const deleting = ref(false);

  // 搜索表单
  const searchForm = reactive({
    name: '',
  });

  // 弹窗状态
  const showAddModal = ref(false);
  const showDeleteModal = ref(false);
  const editingLanguage = ref(null);
  const deletingLanguage = ref(null);

  // 表单数据
  const formRef = ref(null);
  const formData = reactive({
    name: '',
    displayName: '',
    code: '',
    sort: 0,
    isPopular: 0,
  });

  // 表单验证规则
  const formRules = {
    name: {
      required: true,
      message: '请输入语言名称',
      trigger: ['input', 'blur'],
    },
    displayName: {
      required: true,
      message: '请输入显示名称',
      trigger: ['input', 'blur'],
    },
    code: {
      required: true,
      message: '请输入语言代码',
      trigger: ['input', 'blur'],
    },
  };

  // 使用基础列配置
  const columns = baseColumns;

  // 操作列
  const actionColumn = reactive({
    width: 200,
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
        label: '编辑',
        icon: CreateOutline,
        onClick: handleEdit.bind(null, record),
      },
      {
        label: '删除',
        icon: TrashOutline,
        onClick: handleDelete.bind(null, record),
      },
    ];
  }

  // 加载数据
  const loadDataTable = async (res) => {
    try {
      const params = {
        page: res.page,
        pageSize: res.pageSize,
      };

      // 添加搜索关键词
      if (searchForm.name) {
        params.name = searchForm.name;
      }

      const response = await listLanguages(params);

      // BasicTable期望返回 { list: [], itemCount: number } 格式
      const result = {
        list: response.data.data.languagesList || [],
        itemCount: response.data.data.total || 0,
      };
      return result;
    } catch (error) {
      console.error('获取语言列表失败:', error);
      message.error('获取语言列表失败');
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
    reloadTable();
  }

  // 添加
  const handleAdd = () => {
    editingLanguage.value = null;
    resetForm();
    showAddModal.value = true;
  };

  // 编辑
  const handleEdit = (language) => {
    editingLanguage.value = language;
    formData.name = language.name;
    formData.displayName = language.displayName || language.display_name || language.name;
    formData.code = language.code || '';
    formData.sort = language.sort || 0;
    formData.isPopular = language.isPopular || language.is_popular || 0;
    showAddModal.value = true;
  };

  // 删除
  const handleDelete = (language) => {
    deletingLanguage.value = language;
    showDeleteModal.value = true;
  };

  // 关闭弹窗
  const closeModal = () => {
    showAddModal.value = false;
    editingLanguage.value = null;
    resetForm();
  };

  // 重置表单
  const resetForm = () => {
    formData.name = '';
    formData.displayName = '';
    formData.code = '';
    formData.sort = 0;
    formData.isPopular = 0;
    formRef.value?.restoreValidation();
  };

  // 提交表单
  const handleSubmit = async () => {
    try {
      await formRef.value?.validate();
      submitting.value = true;

      const data = {
        name: formData.name,
        display_name: formData.displayName,
        code: formData.code,
        sort: formData.sort,
        isPopular: formData.isPopular,
      };

      if (editingLanguage.value) {
        // 编辑
        await editLanguage({ ...data, id: editingLanguage.value.id });
        message.success('语言更新成功');
      } else {
        // 添加
        await addLanguage(data);
        message.success('语言添加成功');
      }

      closeModal();
      reloadTable();
    } catch (error) {
      console.error('操作失败:', error);
      message.error(editingLanguage.value ? '更新语言失败' : '添加语言失败');
    } finally {
      submitting.value = false;
    }
  };

  // 确认删除
  const confirmDelete = async () => {
    try {
      deleting.value = true;
      await deleteLanguage({ id: deletingLanguage.value.id });
      message.success('语言删除成功');
      showDeleteModal.value = false;
      deletingLanguage.value = null;
      reloadTable();
    } catch (error) {
      console.error('删除语言失败:', error);
      message.error('删除语言失败');
    } finally {
      deleting.value = false;
    }
  };
</script>

<style scoped>
  .languages-manage {
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

  .language-name {
    font-weight: 500;
    color: #333;
  }

  .language-display-name {
    color: #666;
  }
</style>
