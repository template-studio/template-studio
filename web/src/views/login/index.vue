<template>
  <div class="auth-page">
    <!-- 左侧品牌区域 -->
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
            <rect width="32" height="32" rx="6" fill="url(#loginGrad)" />
            <rect x="8" y="6" width="12" height="16" rx="1" fill="#fff" />
            <path d="M18 6 L18 10 L22 10 Z" fill="#e6f7ff" />
            <rect x="10" y="10" width="6" height="1" fill="#52c41a" />
            <rect x="10" y="12" width="4" height="1" fill="#1890ff" />
            <rect x="10" y="14" width="5" height="1" fill="#722ed1" />
            <circle cx="11" cy="17" r="0.5" fill="#ff4d4f" />
            <circle cx="13" cy="17" r="0.5" fill="#ff4d4f" />
            <rect x="14.5" y="16.5" width="2" height="1" fill="#ff4d4f" />
            <path d="M22 20 L26 24 L22 28" stroke="#52c41a" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none" />
            <defs>
              <linearGradient id="loginGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style="stop-color:#1890ff;stop-opacity:1" />
                <stop offset="100%" style="stop-color:#18a058;stop-opacity:1" />
              </linearGradient>
            </defs>
          </svg>
          <span class="brand-name">Template <span class="accent">Studio</span></span>
        </div>
        <h1 class="brand-title">从模板到代码<br />一键生成项目</h1>
        <p class="brand-desc">选择精心设计的项目模板，通过变量配置快速生成完整的项目结构，让开发效率提升 10 倍。</p>
        <div class="brand-features">
          <div class="feature-item">
            <div class="feature-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"></polyline><polyline points="8 6 2 12 8 18"></polyline></svg>
            </div>
            <span>模板引擎驱动</span>
          </div>
          <div class="feature-item">
            <div class="feature-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"></path></svg>
            </div>
            <span>秒级项目生成</span>
          </div>
          <div class="feature-item">
            <div class="feature-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg>
            </div>
            <span>Git 版本管理</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧表单区域 -->
    <div class="auth-form-side">
      <div class="form-container">
        <!-- 返回首页 -->
        <div class="back-home" @click="router.push('/')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5"/><polyline points="12 19 5 12 12 5"/></svg>
          <span>返回首页</span>
        </div>

        <!-- 模式切换 Tab -->
        <div class="auth-tabs">
          <button
            :class="['auth-tab', { active: !isRegister }]"
            @click="switchMode(false)"
            type="button"
          >
            登录
          </button>
          <button
            :class="['auth-tab', { active: isRegister }]"
            @click="switchMode(true)"
            type="button"
          >
            注册
          </button>
          <div class="tab-indicator" :class="{ right: isRegister }"></div>
        </div>

        <div class="form-header">
          <h2 class="form-title">{{ isRegister ? '创建账号' : '欢迎回来' }}</h2>
          <p class="form-subtitle">{{ isRegister ? '填写以下信息，开始使用 Template Studio' : '登录您的 Template Studio 账号' }}</p>
        </div>

        <n-form
          ref="formRef"
          label-placement="left"
          size="large"
          :model="formInline"
          :rules="currentRules"
          class="auth-form"
        >
          <n-form-item path="username">
            <n-input
              v-model:value="formInline.username"
              placeholder="用户名"
            >
              <template #prefix>
                <n-icon size="18" color="#94a3b8">
                  <PersonOutline />
                </n-icon>
              </template>
            </n-input>
          </n-form-item>

          <n-form-item v-if="isRegister" path="email">
            <n-input
              v-model:value="formInline.email"
              placeholder="邮箱（选填）"
            >
              <template #prefix>
                <n-icon size="18" color="#94a3b8">
                  <MailOutline />
                </n-icon>
              </template>
            </n-input>
          </n-form-item>

          <n-form-item path="password">
            <n-input
              v-model:value="formInline.password"
              type="password"
              showPasswordOn="click"
              placeholder="密码"
            >
              <template #prefix>
                <n-icon size="18" color="#94a3b8">
                  <LockClosedOutline />
                </n-icon>
              </template>
            </n-input>
          </n-form-item>

          <n-form-item v-if="isRegister" path="confirmPassword">
            <n-input
              v-model:value="formInline.confirmPassword"
              type="password"
              showPasswordOn="click"
              placeholder="确认密码"
            >
              <template #prefix>
                <n-icon size="18" color="#94a3b8">
                  <LockClosedOutline />
                </n-icon>
              </template>
            </n-input>
          </n-form-item>

          <div v-if="!isRegister" class="form-options">
            <n-checkbox v-model:checked="autoLogin">记住登录</n-checkbox>
            <a href="javascript:" class="forgot-link">忘记密码</a>
          </div>

          <n-button
            type="primary"
            @click="handleSubmit"
            size="large"
            :loading="loading"
            block
            class="submit-btn"
          >
            {{ isRegister ? '创建账号' : '登录' }}
          </n-button>
        </n-form>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { reactive, ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useUserStore } from '@/store/modules/user';
import { useMessage } from 'naive-ui';
import { ResultEnum } from '@/enums/httpEnum';
import {
  PersonOutline,
  LockClosedOutline,
  MailOutline,
} from '@vicons/ionicons5';
import { PageEnum } from '@/enums/pageEnum';
import { register } from '@/api/system/user';

interface FormState {
  username: string;
  password: string;
}

const formRef = ref();
const message = useMessage();
const loading = ref(false);
const autoLogin = ref(true);
const isRegister = ref(false);
const LOGIN_NAME = PageEnum.BASE_LOGIN_NAME;

const router = useRouter();
const route = useRoute();
const userStore = useUserStore();

const formInline = reactive({
  username: 'admin',
  password: 'admin123',
  email: '',
  confirmPassword: '',
});

const loginRules = {
  username: { required: true, message: '请输入用户名', trigger: 'blur' },
  password: { required: true, message: '请输入密码', trigger: 'blur' },
};

const registerRules = {
  username: { required: true, message: '请输入用户名', trigger: 'blur' },
  password: { required: true, message: '请输入密码', trigger: 'blur' },
  confirmPassword: {
    required: true,
    message: '请确认密码',
    trigger: 'blur',
    validator: (_rule, value) => {
      if (value !== formInline.password) {
        return new Error('两次输入的密码不一致');
      }
      return true;
    },
  },
};

const currentRules = computed(() => isRegister.value ? registerRules : loginRules);

function switchMode(register: boolean) {
  isRegister.value = register;
  formRef.value?.restoreValidation();
}

onMounted(() => {
  if (route.query.mode === 'register') {
    isRegister.value = true;
  }
  setTimeout(() => {
    const input = document.querySelector('.form-container input');
    if (input) (input as HTMLElement).focus();
  }, 300);
});

const handleSubmit = (e) => {
  e.preventDefault();
  formRef.value.validate(async (errors) => {
    if (errors) {
      message.error('请填写完整信息');
      return;
    }

    const { username, password, email } = formInline;

    if (isRegister.value) {
      loading.value = true;
      try {
        const { code, message: msg } = await register({ username, password, email: email || undefined });
        if (code == ResultEnum.SUCCESS) {
          message.success('注册成功');
          await userStore.login({ username, password });
          router.replace('/');
        } else {
          message.info(msg || '注册失败');
        }
      } finally {
        loading.value = false;
      }
      return;
    }

    loading.value = true;
    try {
      const { code, message: msg, result } = await userStore.login({ username, password });
      message.destroyAll();
      if (code == ResultEnum.SUCCESS) {
        const roles: string[] = result?.roles || [];
        const isAdmin = roles.some((r) => ['super_admin', 'admin'].includes(r));
        const defaultPath = isAdmin ? '/admin/dashboard' : '/';
        const toPath = decodeURIComponent((route.query?.redirect || defaultPath) as string);
        message.success('登录成功');
        router.replace(toPath);
      } else {
        message.info(msg || '登录失败');
      }
    } finally {
      loading.value = false;
    }
  });
};
</script>

<style scoped>
.auth-page {
  display: flex;
  height: 100vh;
  overflow: hidden;
  background: #0f172a;
}

/* ===== 左侧品牌区域 ===== */
.auth-brand {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f172a 100%);
}

.brand-bg {
  position: absolute;
  inset: 0;
  overflow: hidden;
}

.grid-pattern {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(24, 160, 88, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(24, 160, 88, 0.03) 1px, transparent 1px);
  background-size: 60px 60px;
}

.glow {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  pointer-events: none;
}

.glow-1 {
  width: 400px;
  height: 400px;
  background: rgba(24, 160, 88, 0.15);
  top: -10%;
  right: -5%;
  animation: drift 20s ease-in-out infinite;
}

.glow-2 {
  width: 300px;
  height: 300px;
  background: rgba(45, 140, 240, 0.12);
  bottom: -5%;
  left: 5%;
  animation: drift 25s ease-in-out infinite reverse;
}

.glow-3 {
  width: 200px;
  height: 200px;
  background: rgba(139, 92, 246, 0.1);
  top: 40%;
  left: 30%;
  animation: drift 18s ease-in-out infinite 3s;
}

@keyframes drift {
  0%, 100% { transform: translate(0, 0); }
  25% { transform: translate(30px, -20px); }
  50% { transform: translate(-20px, 30px); }
  75% { transform: translate(20px, 20px); }
}

.brand-content {
  position: relative;
  z-index: 1;
  max-width: 480px;
  padding: 40px;
  color: #f8fafc;
}

.brand-logo {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 48px;
}

.brand-name {
  font-size: 20px;
  font-weight: 700;
  color: #f8fafc;
  letter-spacing: -0.5px;
  font-family: 'Fira Code', 'Segoe UI', system-ui, sans-serif;
}

.brand-name .accent {
  color: #18a058;
}

.brand-title {
  font-size: 36px;
  font-weight: 700;
  line-height: 1.3;
  margin: 0 0 16px 0;
  color: #f8fafc;
  letter-spacing: -0.5px;
}

.brand-desc {
  font-size: 16px;
  line-height: 1.7;
  color: #94a3b8;
  margin: 0 0 40px 0;
}

.brand-features {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.feature-item {
  display: flex;
  align-items: center;
  gap: 12px;
  color: #cbd5e1;
  font-size: 14px;
}

.feature-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: rgba(24, 160, 88, 0.1);
  border: 1px solid rgba(24, 160, 88, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #18a058;
  flex-shrink: 0;
}

/* ===== 右侧表单区域 ===== */
.auth-form-side {
  width: 520px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #ffffff;
  position: relative;
}

.form-container {
  width: 100%;
  max-width: 400px;
  padding: 0 24px;
}

/* ===== 返回首页 ===== */
.back-home {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #94a3b8;
  font-size: 13px;
  cursor: pointer;
  margin-bottom: 32px;
  transition: color 0.2s;
}

.back-home:hover {
  color: #18a058;
}

/* ===== Tab 切换 ===== */
.auth-tabs {
  display: flex;
  position: relative;
  background: #f1f5f9;
  border-radius: 12px;
  padding: 4px;
  margin-bottom: 36px;
}

.auth-tab {
  flex: 1;
  padding: 11px 0;
  border: none;
  background: transparent;
  font-size: 14px;
  font-weight: 600;
  color: #64748b;
  cursor: pointer;
  position: relative;
  z-index: 1;
  border-radius: 9px;
  transition: color 0.2s;
  letter-spacing: 0.3px;
}

.auth-tab.active {
  color: #0f172a;
}

.tab-indicator {
  position: absolute;
  top: 4px;
  left: 4px;
  width: calc(50% - 4px);
  height: calc(100% - 8px);
  background: #ffffff;
  border-radius: 9px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.tab-indicator.right {
  transform: translateX(100%);
}

/* ===== 表单头部 ===== */
.form-header {
  margin-bottom: 32px;
}

.form-title {
  font-size: 26px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 10px 0;
  letter-spacing: -0.4px;
}

.form-subtitle {
  font-size: 14px;
  color: #94a3b8;
  margin: 0;
  line-height: 1.6;
}

/* ===== 表单样式 ===== */

.form-options {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 4px;
  margin-bottom: 28px;
  font-size: 13px;
}

.forgot-link {
  color: #18a058;
  text-decoration: none;
  font-size: 13px;
  font-weight: 500;
  transition: color 0.2s;
  cursor: pointer;
}

.forgot-link:hover {
  color: #36ad6a;
}

.submit-btn {
  height: 50px;
  font-size: 15px;
  font-weight: 600;
  border-radius: 10px;
  letter-spacing: 0.5px;
  margin-top: 4px;
  transition: all 0.2s;
}

/* ===== 响应式 ===== */
@media (max-width: 960px) {
  .auth-brand {
    display: none;
  }

  .auth-form-side {
    width: 100%;
  }
}

@media (max-width: 520px) {
  .form-container {
    max-width: 100%;
    padding: 0 20px;
  }

  .form-title {
    font-size: 22px;
  }

  .auth-tabs {
    margin-bottom: 28px;
  }

  .auth-form :deep(.n-input__input-el) {
    height: 46px;
  }

  .submit-btn {
    height: 48px;
  }
}

/* ===== prefers-reduced-motion ===== */
@media (prefers-reduced-motion: reduce) {
  .glow { animation: none; }
  .tab-indicator { transition: none; }
  .submit-btn { transition: none; }
}
</style>
