<template>
  <a-spin :spinning="loading">
    <a-alert type="info" style="margin-bottom: 20px" message="配置 SMTP 邮件服务后，用户可以通过'忘记密码'功能自助重置密码。" show-icon />
    <a-form layout="horizontal" :label-col="{ span: 4 }" :wrapper-col="{ span: 20 }">
      <a-form-item label="SMTP 服务器">
        <a-input v-model:value="form.host" placeholder="smtp.example.com" />
      </a-form-item>
      <a-form-item label="端口">
        <a-input-number v-model:value="form.port" :min="1" :max="65535" placeholder="465" style="width: 200px" />
      </a-form-item>
      <a-form-item label="使用 TLS">
        <a-switch v-model:checked="form.useTls" />
      </a-form-item>
      <a-form-item label="用户名">
        <a-input v-model:value="form.username" placeholder="noreply@example.com" />
      </a-form-item>
      <a-form-item label="密码">
        <a-input-password v-model:value="form.password" placeholder="SMTP 授权码或密码" />
      </a-form-item>
      <a-form-item label="发件人地址">
        <a-input v-model:value="form.sender" placeholder="Template Studio <noreply@example.com>" />
      </a-form-item>

      <a-divider />
      <a-form-item label="测试收件邮箱">
        <div style="display: flex; gap: 8px; width: 100%">
          <a-input v-model:value="testEmail" placeholder="输入邮箱测试 SMTP 连通性" style="flex: 1" />
          <a-button :loading="testing" @click="handleTest" :disabled="!form.host">发送测试</a-button>
        </div>
      </a-form-item>
      <a-divider />

      <a-form-item :wrapper-col="{ offset: 4, span: 20 }">
        <a-button type="primary" :loading="saving" @click="handleSave">保存设置</a-button>
      </a-form-item>
    </a-form>
  </a-spin>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { getSettings, batchUpdateSettings } from '@/api/system/settings';
import { message } from 'ant-design-vue';
import request from '@/utils/request';
const loading = ref(false);
const saving = ref(false);
const testing = ref(false);
const testEmail = ref('');

const form = reactive({
  host: '',
  port: 465,
  useTls: true,
  username: '',
  password: '',
  sender: '',
});

async function loadSettings() {
  loading.value = true;
  try {
    const res = await getSettings({ group: 'smtp' });
    const items = res.data?.data || [];
    for (const item of items) {
      const val = item.value || '';
      switch (item.key) {
        case 'smtp_host': form.host = val; break;
        case 'smtp_port': form.port = parseInt(val) || 465; break;
        case 'smtp_tls': form.useTls = val !== 'false'; break;
        case 'smtp_username': form.username = val; break;
        case 'smtp_password': form.password = val; break;
        case 'smtp_sender': form.sender = val; break;
      }
    }
  } catch {
    message.error('加载设置失败');
  } finally {
    loading.value = false;
  }
}

async function handleSave() {
  saving.value = true;
  try {
    await batchUpdateSettings({
      group: 'smtp',
      items: [
        { key: 'smtp_host', value: form.host },
        { key: 'smtp_port', value: String(form.port) },
        { key: 'smtp_tls', value: String(form.useTls) },
        { key: 'smtp_username', value: form.username },
        { key: 'smtp_password', value: form.password },
        { key: 'smtp_sender', value: form.sender },
      ],
    });
    message.success('保存成功');
  } catch {
    message.error('保存失败');
  } finally {
    saving.value = false;
  }
}

async function handleTest() {
  if (!testEmail.value) {
    message.warning('请输入测试邮箱');
    return;
  }
  testing.value = true;
  try {
    await handleSave();
    const res = await request.post('/api/v1/admin/email/test', { email: testEmail.value });
    if (res.data?.code === 0) {
      message.success('测试邮件已发送，请检查收件箱');
    } else {
      message.error(res.data?.message || '发送失败');
    }
  } catch (e) {
    message.error(e?.response?.data?.message || '发送失败，请检查 SMTP 配置');
  } finally {
    testing.value = false;
  }
}

onMounted(() => loadSettings());
</script>
