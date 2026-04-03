<template>
  <div class="auth-page">
    <div class="auth-brand">
      <div class="brand-bg">
        <div class="grid-pattern"></div>
        <div class="glow glow-1"></div>
        <div class="glow glow-2"></div>
        <div class="glow glow-3"></div>
      </div>
      <div class="brand-content">
        <div class="brand-logo">
          <svg width="48" height="48" viewBox="0 0 32 32" fill="none">
            <rect width="32" height="32" rx="6" fill="url(#rpGrad)" />
            <rect x="8" y="6" width="12" height="16" rx="1" fill="#fff" />
            <path d="M18 6 L18 10 L22 10 Z" fill="#e6f7ff" />
            <defs>
              <linearGradient id="rpGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style="stop-color:#0f172a;stop-opacity:1" />
                <stop offset="100%" style="stop-color:#1e293b;stop-opacity:1" />
              </linearGradient>
            </defs>
          </svg>
          <span class="brand-name">Template <span class="accent">Studio</span></span>
        </div>
        <h1 class="brand-title">重置密码</h1>
        <p class="brand-desc">设置您的新密码</p>
      </div>
    </div>

    <div class="auth-form-side">
      <div class="form-container">
        <div class="back-home" @click="router.push('/login')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5"/><polyline points="12 19 5 12 12 5"/></svg>
          <span>返回登录</span>
        </div>

        <div v-if="error" class="reset-error">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#ef4444" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          <h3 style="margin: 16px 0 8px; color: #0f172a">链接无效</h3>
          <p style="color: #64748b; font-size: 14px">{{ error }}</p>
          <n-button type="primary" style="margin-top: 20px" @click="router.push('/login')">返回登录</n-button>
        </div>

        <div v-else-if="success" class="reset-success">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="1.5"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          <h3 style="margin: 16px 0 8px; color: #0f172a">密码重置成功</h3>
          <p style="color: #64748b; font-size: 14px">请使用新密码登录</p>
          <n-button type="primary" style="margin-top: 20px" @click="router.push('/login')">去登录</n-button>
        </div>

        <div v-else>
          <div class="form-header">
            <h2 class="form-title">设置新密码</h2>
            <p class="form-subtitle">请输入您的新密码</p>
          </div>
          <n-form ref="formRef" :model="form" :rules="rules" label-placement="left" size="large">
            <n-form-item path="password">
              <n-input v-model:value="form.password" type="password" show-password-on="click" placeholder="新密码（至少6位）">
                <template #prefix>
                  <n-icon size="18" color="#94a3b8"><LockClosedOutline /></n-icon>
                </template>
              </n-input>
            </n-form-item>
            <n-form-item path="confirmPassword">
              <n-input v-model:value="form.confirmPassword" type="password" show-password-on="click" placeholder="确认新密码">
                <template #prefix>
                  <n-icon size="18" color="#94a3b8"><LockClosedOutline /></n-icon>
                </template>
              </n-input>
            </n-form-item>
            <n-button type="primary" block size="large" :loading="loading" @click="handleReset">重置密码</n-button>
          </n-form>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useMessage } from 'naive-ui';
import { LockClosedOutline } from '@vicons/ionicons5';
import { resetPassword } from '@/api/system/password';

const route = useRoute();
const router = useRouter();
const message = useMessage();
const formRef = ref();
const loading = ref(false);
const error = ref('');
const success = ref(false);
const token = ref('');

const form = reactive({ password: '', confirmPassword: '' });
const rules = {
  password: { required: true, min: 6, message: '密码至少6位', trigger: 'blur' },
  confirmPassword: {
    required: true,
    message: '请确认密码',
    trigger: 'blur',
    validator: (_rule, value) => {
      if (value !== form.password) return new Error('两次输入的密码不一致');
      return true;
    },
  },
};

onMounted(() => {
  token.value = route.query.token || '';
  if (!token.value) {
    error.value = '缺少重置令牌，请重新申请密码重置邮件。';
  }
});

async function handleReset() {
  try { await formRef.value?.validate(); } catch { return; }
  loading.value = true;
  try {
    const res = await resetPassword(token.value, form.password);
    if (res.code === 200) {
      success.value = true;
    } else {
      error.value = res.message || '重置失败';
    }
  } catch (e) {
    message.error('重置失败，请稍后重试');
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.auth-page { display: flex; height: 100vh; overflow: hidden; background: #0f172a; }
.auth-brand { flex: 1; display: flex; align-items: center; justify-content: center; position: relative; overflow: hidden; background: linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f172a 100%); }
.brand-bg { position: absolute; inset: 0; overflow: hidden; }
.grid-pattern { position: absolute; inset: 0; background-image: linear-gradient(rgba(34,197,94,0.03) 1px, transparent 1px), linear-gradient(90deg, rgba(34,197,94,0.03) 1px, transparent 1px); background-size: 60px 60px; }
.glow { position: absolute; border-radius: 50%; filter: blur(80px); pointer-events: none; }
.glow-1 { width: 400px; height: 400px; top: -100px; right: -100px; background: rgba(34,197,94,0.15); }
.glow-2 { width: 300px; height: 300px; bottom: -50px; left: -50px; background: rgba(59,130,246,0.1); }
.glow-3 { width: 200px; height: 200px; top: 50%; left: 50%; transform: translate(-50%,-50%); background: rgba(168,85,247,0.08); }
.brand-content { position: relative; z-index: 1; text-align: center; color: #fff; padding: 0 60px; }
.brand-logo { display: flex; align-items: center; justify-content: center; gap: 12px; margin-bottom: 32px; }
.brand-name { font-size: 20px; font-weight: 700; font-family: 'JetBrains Mono', monospace; color: #fff; }
.brand-name .accent { color: #22c55e; }
.brand-title { font-size: 32px; font-weight: 700; margin: 0 0 12px; letter-spacing: -0.5px; }
.brand-desc { font-size: 15px; color: #94a3b8; margin: 0; }
.auth-form-side { width: 480px; flex-shrink: 0; display: flex; align-items: center; justify-content: center; background: #fff; }
.form-container { width: 100%; max-width: 360px; padding: 0 20px; }
.back-home { display: flex; align-items: center; gap: 6px; color: #64748b; font-size: 14px; cursor: pointer; margin-bottom: 32px; transition: color 0.2s; }
.back-home:hover { color: #0f172a; }
.form-header { margin-bottom: 28px; }
.form-title { font-size: 22px; font-weight: 700; color: #0f172a; margin: 0 0 8px; }
.form-subtitle { font-size: 14px; color: #94a3b8; margin: 0; }
.reset-error, .reset-success { text-align: center; padding: 40px 0; }
</style>
