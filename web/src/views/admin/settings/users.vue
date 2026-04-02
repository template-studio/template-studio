<template>
  <div class="user-management">
    <div class="page-header">
      <h3>用户管理</h3>
      <n-button type="primary" @click="showCreateModal = true">新增用户</n-button>
    </div>

    <n-data-table
      :columns="columns"
      :data="users"
      :loading="loading"
      :pagination="false"
      bordered
    />

    <!-- 新增/编辑弹窗 -->
    <n-modal
      v-model:show="showModal"
      :title="editingUser ? '编辑用户' : '新增用户'"
      preset="card"
      style="width: 500px"
    >
      <n-form ref="formRef" :model="formData" label-placement="left" label-width="80">
        <n-form-item v-if="!editingUser" label="用户名" path="username">
          <n-input v-model:value="formData.username" placeholder="请输入用户名" />
        </n-form-item>
        <n-form-item v-if="!editingUser" label="密码" path="password">
          <n-input v-model:value="formData.password" type="password" placeholder="请输入密码" />
        </n-form-item>
        <n-form-item label="邮箱" path="email">
          <n-input v-model:value="formData.email" placeholder="请输入邮箱" />
        </n-form-item>
        <n-form-item label="状态" path="status">
          <n-switch v-model:value="formData.statusBool" :checked-value="1" :unchecked-value="0">
            <template #checked>启用</template>
            <template #unchecked>禁用</template>
          </n-switch>
        </n-form-item>
        <n-form-item label="角色" path="role_ids">
          <n-select
            v-model:value="formData.role_ids"
            :options="roleOptions"
            multiple
            placeholder="请选择角色"
          />
        </n-form-item>
      </n-form>
      <template #action>
        <n-button @click="showModal = false">取消</n-button>
        <n-button type="primary" :loading="submitting" @click="handleSubmit">确定</n-button>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, h, watch } from 'vue';
import { NButton, NSpace, NTag, useMessage } from 'naive-ui';
import { getUserList, createUser, updateUser, deleteUser } from '@/api/admin/user';
import { getRoleList } from '@/api/system/role';

const message = useMessage();
const loading = ref(false);
const users = ref([]);
const roles = ref([]);
const showModal = ref(false);
const showCreateModal = ref(false);
const editingUser = ref<any>(null);
const submitting = ref(false);

const formData = reactive({
  username: '',
  password: '',
  email: '',
  statusBool: 1,
  role_ids: [] as number[],
});

const roleOptions = ref<{ label: string; value: number }[]>([]);

const columns = [
  { title: 'ID', key: 'id', width: 60 },
  { title: '用户名', key: 'username', width: 120 },
  { title: '邮箱', key: 'email', width: 200 },
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
    title: '最后登录',
    key: 'last_login_at',
    width: 180,
    render: (row: any) => row.last_login_at || '从未登录',
  },
  {
    title: '操作',
    key: 'actions',
    width: 160,
    render: (row: any) =>
      h(NSpace, null, () => [
        h(NButton, { text: true, type: 'primary', onClick: () => handleEdit(row) }, () => '编辑'),
        h(
          NButton,
          { text: true, type: 'error', onClick: () => handleDelete(row) },
          () => '删除'
        ),
      ]),
  },
];

async function fetchUsers() {
  loading.value = true;
  try {
    const res = await getUserList();
    users.value = res?.data?.data?.list || [];
  } catch (e) {
    message.error('获取用户列表失败');
  } finally {
    loading.value = false;
  }
}

async function fetchRoles() {
  try {
    const res = await getRoleList();
    const list = res?.data?.data?.list || [];
    roles.value = list;
    roleOptions.value = list.map((r: any) => ({ label: r.display_name, value: r.id }));
  } catch (e) {
    // ignore
  }
}

function handleEdit(row: any) {
  editingUser.value = row;
  formData.username = '';
  formData.password = '';
  formData.email = row.email || '';
  formData.statusBool = row.status;
  formData.role_ids = [];
  showModal.value = true;
}

async function handleDelete(row: any) {
  try {
    await deleteUser(row.id);
    message.success('删除成功');
    await fetchUsers();
  } catch (e) {
    message.error('删除失败');
  }
}

async function handleSubmit() {
  submitting.value = true;
  try {
    if (editingUser.value) {
      await updateUser({
        id: editingUser.value.id,
        email: formData.email,
        status: formData.statusBool,
      });
      message.success('更新成功');
    } else {
      if (!formData.username || !formData.password) {
        message.warning('用户名和密码不能为空');
        return;
      }
      await createUser({
        username: formData.username,
        password: formData.password,
        email: formData.email,
        role_ids: formData.role_ids,
      });
      message.success('创建成功');
    }
    showModal.value = false;
    showCreateModal.value = false;
    await fetchUsers();
  } catch (e: any) {
    message.error(e?.message || '操作失败');
  } finally {
    submitting.value = false;
  }
}

watch(
  () => showCreateModal.value,
  (val) => {
    if (val) {
      editingUser.value = null;
      formData.username = '';
      formData.password = '';
      formData.email = '';
      formData.statusBool = 1;
      formData.role_ids = [];
      showModal.value = true;
      showCreateModal.value = false;
    }
  }
);

onMounted(() => {
  fetchUsers();
  fetchRoles();
});
</script>

<style scoped>
.user-management {
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
