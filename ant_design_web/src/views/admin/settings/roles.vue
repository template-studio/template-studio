<template>
  <div class="role-management">
    <div class="page-header">
      <h3>角色管理</h3>
      <a-button type="primary" @click="openCreate">新增角色</a-button>
    </div>

    <a-table
      :columns="columns"
      :data-source="roles"
      :loading="loading"
      :pagination="false"
      bordered
      row-key="id"
    />

    <!-- 新增/编辑弹窗 -->
    <a-modal
      v-model:open="showModal"
      :title="editingRole ? '编辑角色' : '新增角色'"
      width="500px"
    >
      <a-form :model="formData" layout="horizontal" :label-col="{ span: 5 }" :wrapper-col="{ span: 19 }">
        <a-form-item v-if="!editingRole" label="角色标识" name="name">
          <a-input v-model:value="formData.name" placeholder="如 admin, viewer" />
        </a-form-item>
        <a-form-item label="角色名称" name="display_name">
          <a-input v-model:value="formData.display_name" placeholder="如 管理员" />
        </a-form-item>
        <a-form-item label="描述" name="description">
          <a-textarea v-model:value="formData.description" placeholder="角色描述" />
        </a-form-item>
      </a-form>
      <template #footer>
        <a-button @click="showModal = false">取消</a-button>
        <a-button type="primary" :loading="submitting" @click="handleSubmit">确定</a-button>
      </template>
    </a-modal>

    <!-- 权限分配弹窗 -->
    <a-modal
      v-model:open="showPermModal"
      title="分配权限"
      width="500px"
    >
      <a-checkbox-group v-model:value="selectedPermissions">
        <a-space direction="vertical">
          <a-checkbox
            v-for="perm in permissions"
            :key="perm.id"
            :value="perm.id"
          >
            {{ perm.display_name }}
          </a-checkbox>
        </a-space>
      </a-checkbox-group>
      <template #footer>
        <a-button @click="showPermModal = false">取消</a-button>
        <a-button type="primary" :loading="submitting" @click="handleAssignPerms">确定</a-button>
      </template>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, h } from 'vue';
import { Button, Space, Tag, message } from 'ant-design-vue';
import { getRoleList, createRole, updateRole, deleteRole, getRolePermissions, assignPermissions } from '@/api/system/role';
import { getPermissionList } from '@/api/system/permission';

const loading = ref(false);
const roles = ref([]);
const permissions = ref<any[]>([]);
const showModal = ref(false);
const showPermModal = ref(false);
const editingRole = ref<any>(null);
const currentRoleId = ref<number>(0);
const selectedPermissions = ref<number[]>([]);
const submitting = ref(false);

const formData = reactive({
  name: '',
  display_name: '',
  description: '',
});

const columns = [
  { title: 'ID', dataIndex: 'id', key: 'id', width: 60 },
  { title: '角色标识', dataIndex: 'name', key: 'name', width: 120 },
  { title: '角色名称', dataIndex: 'display_name', key: 'display_name', width: 120 },
  { title: '描述', dataIndex: 'description', key: 'description', ellipsis: true },
  {
    title: '状态',
    dataIndex: 'status',
    key: 'status',
    width: 80,
    customRender: ({ record }: any) =>
      h(Tag, { color: record.status === 1 ? 'green' : 'red' }, () =>
        record.status === 1 ? '启用' : '禁用'
      ),
  },
  {
    title: '操作',
    dataIndex: 'actions',
    key: 'actions',
    width: 200,
    customRender: ({ record }: any) =>
      h(Space, null, () => [
        h(Button, { type: 'link', size: 'small', onClick: () => handleEdit(record) }, () => '编辑'),
        h(
          Button,
          { type: 'link', size: 'small', onClick: () => handleOpenPerms(record) },
          () => '权限'
        ),
        h(
          Button,
          { type: 'link', size: 'small', danger: true, onClick: () => handleDelete(record) },
          () => '删除'
        ),
      ]),
  },
];

async function fetchRoles() {
  loading.value = true;
  try {
    const res = await getRoleList();
    roles.value = res?.data?.data?.list || [];
  } catch (e) {
    message.error('获取角色列表失败');
  } finally {
    loading.value = false;
  }
}

async function fetchPermissions() {
  try {
    const res = await getPermissionList();
    permissions.value = res?.data?.data?.list || [];
  } catch (e) {
    // ignore
  }
}

function openCreate() {
  editingRole.value = null;
  formData.name = '';
  formData.display_name = '';
  formData.description = '';
  showModal.value = true;
}

function handleEdit(row: any) {
  editingRole.value = row;
  formData.name = row.name;
  formData.display_name = row.display_name;
  formData.description = row.description || '';
  showModal.value = true;
}

async function handleOpenPerms(row: any) {
  currentRoleId.value = row.id;
  try {
    const res = await getRolePermissions(row.id);
    selectedPermissions.value = res?.data?.data?.permission_ids || [];
    showPermModal.value = true;
  } catch (e) {
    message.error('获取角色权限失败');
  }
}

async function handleAssignPerms() {
  submitting.value = true;
  try {
    await assignPermissions(currentRoleId.value, selectedPermissions.value);
    message.success('权限分配成功');
    showPermModal.value = false;
  } catch (e) {
    message.error('权限分配失败');
  } finally {
    submitting.value = false;
  }
}

async function handleSubmit() {
  submitting.value = true;
  try {
    if (editingRole.value) {
      await updateRole({
        id: editingRole.value.id,
        display_name: formData.display_name,
        description: formData.description,
      });
      message.success('更新成功');
    } else {
      if (!formData.name || !formData.display_name) {
        message.warning('角色标识和名称不能为空');
        return;
      }
      await createRole({
        name: formData.name,
        display_name: formData.display_name,
        description: formData.description,
      });
      message.success('创建成功');
    }
    showModal.value = false;
    await fetchRoles();
  } catch (e: any) {
    message.error(e?.message || '操作失败');
  } finally {
    submitting.value = false;
  }
}

async function handleDelete(row: any) {
  try {
    await deleteRole(row.id);
    message.success('删除成功');
    await fetchRoles();
  } catch (e) {
    message.error('删除失败');
  }
}

onMounted(() => {
  fetchRoles();
  fetchPermissions();
});
</script>

<style scoped>
.role-management {
  padding: 0;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.page-header h3 {
  margin: 0;
  font-size: 16px;
}
</style>
