<template>
  <div class="categories-manage">
    <div style="display: flex; flex-direction: column; gap: 16px">
      <a-card :bordered="false">
        <a-form layout="inline" :model="searchForm">
          <a-form-item label="关键词">
            <a-input
              v-model:value="searchForm.name"
              placeholder="输入分类名称或描述进行搜索"
              allow-clear
              style="width: 240px"
              @keyup.enter="handleSearch"
            >
              <template #prefix>
                <SearchOutline />
              </template>
            </a-input>
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
          :scroll-x="1200"
        >
          <template #tableTitle>
            <a-button type="primary" @click="handleAdd">
              <template #icon>
                <AddOutline />
              </template>
              新建分类
            </a-button>
          </template>
        </BasicTable>
      </a-card>
    </div>

    <!-- 添加/编辑分类弹窗 -->
    <a-modal v-model:open="showAddModal" :mask-closable="false" :title="editingCategory ? '编辑分类' : '添加分类'" width="500px">
      <a-form ref="formRef" :model="formData" :rules="formRules" layout="vertical">
        <a-form-item label="分类名称" name="name">
          <a-input
            v-model:value="formData.name"
            placeholder="请输入分类名称"
            :maxlength="50"
            show-count
          />
        </a-form-item>

        <a-form-item label="分类描述" name="description">
          <a-textarea
            v-model:value="formData.description"
            placeholder="请输入分类描述（可选）"
            :maxlength="200"
            show-count
            :rows="3"
          />
        </a-form-item>

        <a-form-item label="排序" name="sort">
          <a-input-number
            v-model:value="formData.sort"
            placeholder="排序值，数字越小越靠前"
            :min="0"
            :max="9999"
            style="width: 100%"
          />
        </a-form-item>

        <a-form-item label="状态" name="status">
          <a-switch v-model:checked="formData.status" :checked-value="1" :unchecked-value="0" checked-children="启用" un-checked-children="禁用" />
        </a-form-item>
      </a-form>

      <template #footer>
        <div class="modal-footer">
          <a-button @click="closeModal">取消</a-button>
          <a-button type="primary" @click="handleSubmit" :loading="submitting">
            {{ editingCategory ? '更新' : '添加' }}
          </a-button>
        </div>
      </template>
    </a-modal>

    <!-- 删除确认弹窗 -->
    <a-modal v-model:open="showDeleteModal" :mask-closable="false" title="确认删除" width="400px">
      <div class="delete-content">
        <div class="delete-icon">
          <TrashOutline style="font-size: 48px; color: #d03050" />
        </div>
        <p class="delete-message">
          确定要删除分类 <strong>"{{ deletingCategory?.name }}"</strong> 吗？
        </p>
        <p class="delete-warning"> 此操作不可撤销，删除后相关模板的分类信息可能会受到影响。 </p>
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
  import { BasicTable, TableAction } from '@/components/Table';
  import { message } from 'ant-design-vue';
  import {
    AddOutline,
    CloseOutline,
    TrashOutline,
    CreateOutline,
    SearchOutline,
    RefreshOutline,
  } from '@/icons/ionicons5';
  import { listCategories, addCategory, editCategory, deleteCategory } from '@/api/categories';
  import { columns as baseColumns } from './columns';

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
  const editingCategory = ref(null);
  const deletingCategory = ref(null);

  // 表单数据
  const formRef = ref(null);
  const formData = reactive({
    name: '',
    description: '',
    sort: 0,
    status: 1,
  });

  // 表单验证规则
  const formRules = {
    name: {
      required: true,
      message: '请输入分类名称',
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

      const response = await listCategories(params);
      console.log('API响应:', response);
      console.log('响应数据:', response.data);
      console.log('列表数据:', response.data.data.categoriesList);

      // BasicTable期望返回 { list: [], itemCount: number } 格式
      const result = {
        list: response.data.data.categoriesList || [],
        itemCount: response.data.data.total || 0,
      };
      console.log('返回给Table的数据:', result);
      return result;
    } catch (error) {
      console.error('获取分类列表失败:', error);
      message.error('获取分类列表失败');
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

  // 多选回调
  function onCheckedRow(rowKeys) {
    console.log('选中的行:', rowKeys);
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
    editingCategory.value = null;
    resetForm();
    showAddModal.value = true;
  };

  // 编辑
  const handleEdit = (category) => {
    editingCategory.value = category;
    formData.name = category.name;
    formData.description = category.description || '';
    formData.sort = category.sort || 0;
    formData.status = category.status || 1;
    showAddModal.value = true;
  };

  // 删除
  const handleDelete = (category) => {
    deletingCategory.value = category;
    showDeleteModal.value = true;
  };

  // 关闭弹窗
  const closeModal = () => {
    showAddModal.value = false;
    editingCategory.value = null;
    resetForm();
  };

  // 重置表单
  const resetForm = () => {
    formData.name = '';
    formData.description = '';
    formData.sort = 0;
    formData.status = 1;
    formRef.value?.resetFields();
  };

  // 提交表单
  const handleSubmit = async () => {
    try {
      await formRef.value?.validate();
      submitting.value = true;

      const data = {
        name: formData.name,
        description: formData.description,
        sort: formData.sort,
        status: formData.status,
      };

      if (editingCategory.value) {
        // 编辑
        await editCategory({ ...data, id: editingCategory.value.id });
        message.success('分类更新成功');
      } else {
        // 添加
        await addCategory(data);
        message.success('分类添加成功');
      }

      closeModal();
      reloadTable();
    } catch (error) {
      console.error('操作失败:', error);
      message.error(editingCategory.value ? '更新分类失败' : '添加分类失败');
    } finally {
      submitting.value = false;
    }
  };

  // 确认删除
  const confirmDelete = async () => {
    try {
      deleting.value = true;
      await deleteCategory({ id: deletingCategory.value.id });
      message.success('分类删除成功');
      showDeleteModal.value = false;
      deletingCategory.value = null;
      reloadTable();
    } catch (error) {
      console.error('删除分类失败:', error);
      message.error('删除分类失败');
    } finally {
      deleting.value = false;
    }
  };
</script>

<style scoped>
  .categories-manage {
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

  .category-name {
    font-weight: 500;
    color: #333;
  }

  .text-placeholder {
    color: #999;
    font-style: italic;
  }
</style>
