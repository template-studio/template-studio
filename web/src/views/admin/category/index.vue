<template>
  <div class="categories-manage">
    <n-flex vertical>
      <n-card :bordered="false">
        <n-form inline :label-width="80" :model="searchForm">
          <n-form-item label="关键词">
            <n-input
              v-model:value="searchForm.name"
              placeholder="输入分类名称或描述进行搜索"
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
              新建分类
            </n-button>
          </template>
        </BasicTable>
      </n-card>
    </n-flex>

    <!-- 添加/编辑分类弹窗 -->
    <n-modal v-model:show="showAddModal" :mask-closable="false">
      <n-card
        style="width: 500px"
        :title="editingCategory ? '编辑分类' : '添加分类'"
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
          <n-form-item label="分类名称" path="name">
            <n-input
              v-model:value="formData.name"
              placeholder="请输入分类名称"
              :maxlength="50"
              show-count
            />
          </n-form-item>

          <n-form-item label="分类描述" path="description">
            <n-input
              v-model:value="formData.description"
              type="textarea"
              placeholder="请输入分类描述（可选）"
              :maxlength="200"
              show-count
              :rows="3"
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

          <n-form-item label="状态" path="status">
            <n-switch v-model:value="formData.status" :checked-value="1" :unchecked-value="0">
              <template #checked>启用</template>
              <template #unchecked>禁用</template>
            </n-switch>
          </n-form-item>
        </n-form>

        <template #footer>
          <div class="modal-footer">
            <n-button @click="closeModal">取消</n-button>
            <n-button type="primary" @click="handleSubmit" :loading="submitting">
              {{ editingCategory ? '更新' : '添加' }}
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
            确定要删除分类 <strong>"{{ deletingCategory?.name }}"</strong> 吗？
          </p>
          <p class="delete-warning"> 此操作不可撤销，删除后相关模板的分类信息可能会受到影响。 </p>
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
  import { listCategories, addCategory, editCategory, deleteCategory } from '@/api/categories';
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
    formRef.value?.restoreValidation();
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
