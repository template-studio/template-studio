<template>
  <div class="profile-page">
    <!-- 内容区域 -->
    <div class="profile-body">
      <div class="body-inner">
        <!-- 左侧导航 -->
        <div class="side-nav">
          <div
            v-for="tab in tabs"
            :key="tab.key"
            class="nav-item"
            :class="{ active: activeTab === tab.key }"
            @click="activeTab = tab.key"
          >
            <div class="nav-icon" v-html="tab.icon"></div>
            <span>{{ tab.label }}</span>
          </div>
        </div>

        <!-- 右侧内容 -->
        <div class="main-content">
          <!-- 基本信息 -->
          <div v-if="activeTab === 'info'" class="content-panel">
            <div class="panel-header">
              <h2>基本信息</h2>
              <p class="panel-desc">管理您的个人账号信息</p>
            </div>
            <div class="info-grid">
              <div class="info-card">
                <div class="info-card-icon">
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                </div>
                <div class="info-card-body">
                  <span class="info-label">用户名</span>
                  <span class="info-value">{{ userStore.getNickname || '-' }}</span>
                </div>
              </div>
              <div class="info-card">
                <div class="info-card-icon">
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/></svg>
                </div>
                <div class="info-card-body">
                  <span class="info-label">邮箱</span>
                  <span class="info-value">{{ userStore.getUserInfo?.email || '-' }}</span>
                </div>
              </div>
              <div class="info-card">
                <div class="info-card-icon">
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10"/><path d="m9 12 2 2 4-4"/></svg>
                </div>
                <div class="info-card-body">
                  <span class="info-label">角色</span>
                  <span class="info-value">{{ userRoles.join(', ') || '-' }}</span>
                </div>
              </div>
              <div class="info-card">
                <div class="info-card-icon">
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                </div>
                <div class="info-card-body">
                  <span class="info-label">注册时间</span>
                  <span class="info-value">-</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 修改密码 -->
          <div v-if="activeTab === 'password'" class="content-panel">
            <div class="panel-header">
              <h2>修改密码</h2>
              <p class="panel-desc">定期更换密码有助于保护账号安全</p>
            </div>
            <n-form ref="passwordFormRef" :model="passwordForm" :rules="passwordRules" label-placement="top" class="password-form">
              <div class="form-grid">
                <n-form-item label="当前密码" path="oldPassword">
                  <n-input v-model:value="passwordForm.oldPassword" type="password" showPasswordOn="click" placeholder="请输入当前密码" />
                </n-form-item>
                <n-form-item label="新密码" path="newPassword">
                  <n-input v-model:value="passwordForm.newPassword" type="password" showPasswordOn="click" placeholder="请输入新密码（至少6位）" />
                </n-form-item>
                <n-form-item label="确认新密码" path="confirmPassword">
                  <n-input v-model:value="passwordForm.confirmPassword" type="password" showPasswordOn="click" placeholder="请再次输入新密码" />
                </n-form-item>
              </div>
              <div class="form-actions">
                <n-button type="primary" @click="handleChangePassword" :loading="passwordLoading">
                  保存修改
                </n-button>
              </div>
            </n-form>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
  import { ref, computed, reactive, onMounted } from 'vue';
  import { useMessage } from 'naive-ui';
  import { useUserStore } from '@/store/modules/user';
  import { changePassword } from '@/api/system/user';

  const userStore = useUserStore();
  const message = useMessage();

  const activeTab = ref('info');

  onMounted(async () => {
    if (userStore.getToken && !userStore.getNickname) {
      try {
        await userStore.getInfo();
      } catch (e) {
        console.warn('获取用户信息失败:', e);
      }
    }
  });

  const tabs = [
    {
      key: 'info',
      label: '基本信息',
      icon: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>',
    },
    {
      key: 'password',
      label: '修改密码',
      icon: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>',
    },
  ];

  const userRoles = computed(() => {
    const roles = userStore.getRoles || [];
    if (roles.length === 0) return ['普通用户'];
    return roles.map((r) => {
      const map = { super_admin: '超级管理员', admin: '管理员', viewer: '观察者' };
      return map[r] || r;
    });
  });

  // 修改密码
  const passwordFormRef = ref();
  const passwordLoading = ref(false);
  const passwordForm = reactive({
    oldPassword: '',
    newPassword: '',
    confirmPassword: '',
  });

  const passwordRules = {
    oldPassword: { required: true, message: '请输入当前密码', trigger: 'blur' },
    newPassword: [
      { required: true, message: '请输入新密码', trigger: 'blur' },
      { min: 6, message: '密码至少6位', trigger: 'blur' },
    ],
    confirmPassword: {
      required: true,
      message: '请确认新密码',
      trigger: 'blur',
      validator: (_rule, value) => {
        if (value && value !== passwordForm.newPassword) {
          return new Error('两次输入的密码不一致');
        }
        return true;
      },
    },
  };

  const handleChangePassword = () => {
    passwordFormRef.value?.validate(async (errors) => {
      if (errors) return;
      passwordLoading.value = true;
      try {
        await changePassword({
          oldPassword: passwordForm.oldPassword,
          newPassword: passwordForm.newPassword,
        });
        message.success('密码修改成功');
        passwordForm.oldPassword = '';
        passwordForm.newPassword = '';
        passwordForm.confirmPassword = '';
      } catch (e) {
        message.error('密码修改失败');
      } finally {
        passwordLoading.value = false;
      }
    });
  };
</script>

<style scoped>
  .profile-page {
    min-height: calc(100vh - 64px);
    background: #f1f5f9;
  }

  /* ===== Body ===== */
  .profile-body {
    max-width: 1200px;
    margin: 0 auto;
    padding: 32px 40px 64px;
  }

  .body-inner {
    display: grid;
    grid-template-columns: 220px 1fr;
    gap: 24px;
    align-items: start;
  }

  /* ===== Side Nav ===== */
  .side-nav {
    background: #fff;
    border-radius: 12px;
    border: 1px solid #e2e8f0;
    padding: 8px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    border-radius: 8px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    color: #64748b;
    transition: all 0.15s ease;
  }

  .nav-item:hover {
    background: #f8fafc;
    color: #334155;
  }

  .nav-item.active {
    background: #f0fdf4;
    color: #22c55e;
  }

  .nav-icon {
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  /* ===== Main Content ===== */
  .content-panel {
    background: #fff;
    border-radius: 12px;
    border: 1px solid #e2e8f0;
    padding: 32px;
  }

  .panel-header {
    margin-bottom: 28px;
    padding-bottom: 20px;
    border-bottom: 1px solid #f1f5f9;
  }

  .panel-header h2 {
    font-size: 18px;
    font-weight: 600;
    color: #0f172a;
    margin: 0 0 4px;
  }

  .panel-desc {
    font-size: 13px;
    color: #94a3b8;
    margin: 0;
  }

  /* ===== Info Grid ===== */
  .info-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .info-card {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 20px;
    border-radius: 10px;
    background: #f8fafc;
    border: 1px solid #f1f5f9;
    transition: border-color 0.15s ease;
  }

  .info-card:hover {
    border-color: #e2e8f0;
  }

  .info-card-icon {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    background: #fff;
    border: 1px solid #e2e8f0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #22c55e;
    flex-shrink: 0;
  }

  .info-card-body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .info-label {
    font-size: 12px;
    color: #94a3b8;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .info-value {
    font-size: 14px;
    color: #0f172a;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* ===== Password Form ===== */
  .password-form {
    max-width: 560px;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 4px;
  }

  .form-actions {
    margin-top: 24px;
    padding-top: 20px;
    border-top: 1px solid #f1f5f9;
  }

  /* ===== Responsive ===== */
  @media (max-width: 768px) {
    .profile-body {
      padding: 20px;
    }

    .body-inner {
      grid-template-columns: 1fr;
    }

    .side-nav {
      position: static;
      display: flex;
      gap: 4px;
      overflow-x: auto;
      padding: 6px;
    }

    .nav-item {
      white-space: nowrap;
      padding: 10px 14px;
    }

    .info-grid {
      grid-template-columns: 1fr;
    }

    .content-panel {
      padding: 20px;
    }
  }
</style>
