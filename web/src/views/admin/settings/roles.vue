<template>
  <div class="role-management">
    <div class="page-header">
      <h3>角色管理</h3>
      <n-button type="primary" @click="openCreate">新增角色</n-button>
    </div>

    <n-data-table
      :columns="columns"
      :data="roles"
      :loading="loading"
      :pagination="false"
      bordered
    />

    <!-- 新增/编辑弹窗 -->
    <n-modal
      v-model:show="showModal"
      :title="editingRole ? '编辑角色' : '新增角色'"
      preset="card"
      style="width: 500px"
    >
      <n-form :model="formData" label-placement="left" label-width="80">
        <n-form-item v-if="!editingRole" label="角色标识" path="name">
          <n-input v-model:value="formData.name" placeholder="如 admin, viewer" />
        </n-form-item>
        <n-form-item label="角色名称" path="display_name">
          <n-input v-model:value="formData.display_name" placeholder="如 管理员" />
        </n-form-item>
        <n-form-item label="描述" path="description">
          <n-input v-model:value="formData.description" type="textarea" placeholder="角色描述" />
        </n-form-item>
      </n-form>
      <template #action>
        <n-button @click="showModal = false">取消</n-button>
        <n-button type="primary" :loading="submitting" @click="handleSubmit">确定</n-button>
      </template>
    </n-modal>

    <!-- 权限分配弹窗 -->
    <n-modal
      v-model:show="showPermModal"
      title="分配权限"
      preset="card"
      style="width: 500px"
    >
      <n-checkbox-group v-model:value="selectedPermissions">
        <n-space vertical>
          <n-checkbox
            v-for="perm in permissions"
            :key="perm.id"
            :value="perm.id"
            :label="perm.display_name"
          />
        </n-space>
      </n-checkbox-group>
      <template #action>
        <n-button @click="showPermModal = false">取消</n-button>
        <n-button type="primary" :loading="submitting" @click="handleAssignPerms">确定</n-button>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, h } from 'vue';
import { NButton, NSpace, NTag, useMessage } from 'naive-ui';
import { getRoleList, createRole, updateRole, deleteRole, getRolePermissions, assignPermissions } from '@/api/system/role';
import { getPermissionList } from '@/api/system/permission';

const message = useMessage();
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
  { title: 'ID', key: 'id', width: 60 },
  { title: '角色标识', key: 'name', width: 120 },
  { title: '角色名称', key: 'display_name', width: 120 },
  { title: '描述', key: 'description', ellipsis: true },
  {
    title: '状态',
    key: 'status',
    width: 80,
    render: (row: any) =>
      h(NTag, { type: row.status === 1 ? 'success' : 'error', size: 'small' }, () =>
        row.status === 1 ? '启用' : '禁用'
      ),
  },
  {
    title: '操作',
    key: 'actions',
    width: 200,
    render: (row: any) =>
      h(NSpace, null, () => [
        h(NButton, { text: true, type: 'primary', onClick: () => handleEdit(row) }, () => '编辑'),
        h(
          NButton,
          { text: true, type: 'info', onClick: () => handleOpenPerms(row) },
          () => '权限'
        ),
        h(
          NButton,
          { text: true, type: 'error', onClick: () => handleDelete(row) },
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
