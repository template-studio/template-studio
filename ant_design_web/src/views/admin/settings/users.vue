<template>
  <div class="user-management">
    <div class="page-header">
      <h3>用户管理</h3>
      <a-button type="primary" @click="showCreateModal = true">新增用户</a-button>
    </div>

    <a-table
      :columns="columns"
      :data-source="users"
      :loading="loading"
      :pagination="false"
      bordered
      row-key="id"
    />

    <!-- 新增/编辑弹窗 -->
    <a-modal
      v-model:open="showModal"
      :title="editingUser ? '编辑用户' : '新增用户'"
      width="500px"
    >
      <a-form :model="formData" layout="horizontal" :label-col="{ span: 5 }" :wrapper-col="{ span: 19 }">
        <a-form-item v-if="!editingUser" label="用户名" name="username">
          <a-input v-model:value="formData.username" placeholder="请输入用户名" />
        </a-form-item>
        <a-form-item v-if="!editingUser" label="密码" name="password">
          <a-input-password v-model:value="formData.password" placeholder="请输入密码" />
        </a-form-item>
        <a-form-item label="邮箱" name="email">
          <a-input v-model:value="formData.email" placeholder="请输入邮箱" />
        </a-form-item>
        <a-form-item label="状态" name="status">
          <a-switch v-model:checked="formData.statusBool" :checked-value="1" :unchecked-value="0" checked-children="启用" un-checked-children="禁用" />
        </a-form-item>
        <a-form-item label="角色" name="role_ids">
          <a-select
            v-model:value="formData.role_ids"
            :options="roleOptions"
            mode="multiple"
            placeholder="请选择角色"
          />
        </a-form-item>
      </a-form>
      <template #footer>
        <a-button @click="showModal = false">取消</a-button>
        <a-button type="primary" :loading="submitting" @click="handleSubmit">确定</a-button>
      </template>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, h, watch } from 'vue';
import { Button, Space, Tag, message } from 'ant-design-vue';
import { getUserList, createUser, updateUser, deleteUser } from '@/api/admin/user';
import { getRoleList } from '@/api/system/role';

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
  { title: 'ID', dataIndex: 'id', key: 'id', width: 60 },
  { title: '用户名', dataIndex: 'username', key: 'username', width: 120 },
  { title: '邮箱', dataIndex: 'email', key: 'email', width: 200 },
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
    title: '最后登录',
    dataIndex: 'last_login_at',
    key: 'last_login_at',
    width: 180,
    customRender: ({ record }: any) => record.last_login_at || '从未登录',
  },
  {
    title: '操作',
    dataIndex: 'actions',
    key: 'actions',
    width: 160,
    customRender: ({ record }: any) =>
      h(Space, null, () => [
        h(Button, { type: 'link', size: 'small', onClick: () => handleEdit(record) }, () => '编辑'),
        h(
          Button,
          { type: 'link', size: 'small', danger: true, onClick: () => handleDelete(record) },
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
