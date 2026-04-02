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

          <!-- 外观设置 -->
          <div v-if="activeTab === 'appearance'" class="content-panel">
            <div class="panel-header">
              <h2>外观设置</h2>
              <p class="panel-desc">个性化你的浏览体验，所有设置保存在本地浏览器中</p>
            </div>

            <!-- 主题色 -->
            <div class="theme-section">
              <h3 class="section-label">主题色</h3>
              <div class="color-grid">
                <div
                  v-for="color in themeColors"
                  :key="color.value"
                  class="color-swatch"
                  :class="{ active: currentTheme === color.value }"
                  :style="{ '--swatch-color': color.value }"
                  :title="color.name"
                  @click="handleThemeChange(color.value)"
                >
                  <div class="swatch-circle"></div>
                  <span class="swatch-name">{{ color.name }}</span>
                </div>
              </div>
            </div>

            <!-- 英雄区风格 -->
            <div class="theme-section">
              <h3 class="section-label">英雄区风格</h3>
              <div class="preset-grid">
                <div
                  v-for="preset in heroPresets"
                  :key="preset.name"
                  class="preset-card"
                  :class="{ active: currentHero.name === preset.name }"
                  @click="handleHeroChange(preset)"
                >
                  <div
                    class="preset-preview hero-preview"
                    :style="{ background: `linear-gradient(135deg, ${preset.from}, ${preset.to})` }"
                  >
                    <div class="hero-preview-text">Aa</div>
                  </div>
                  <span class="preset-name">{{ preset.name }}</span>
                </div>
              </div>
            </div>

            <!-- 卡片风格 -->
            <div class="theme-section">
              <h3 class="section-label">卡片风格</h3>
              <div class="preset-grid">
                <div
                  v-for="style in cardStyles"
                  :key="style.name"
                  class="preset-card"
                  :class="{ active: currentCardStyle.name === style.name }"
                  @click="handleCardChange(style)"
                >
                  <div
                    class="preset-preview card-preview"
                    :style="{
                      background: style.bg,
                      border: style.border,
                      boxShadow: style.shadow,
                      borderRadius: style.radius
                    }"
                  >
                    <div class="card-preview-line w70"></div>
                    <div class="card-preview-line w50"></div>
                    <div class="card-preview-line w90"></div>
                  </div>
                  <span class="preset-name">{{ style.name }}</span>
                </div>
              </div>
            </div>

            <!-- 预览 -->
            <div class="theme-preview">
              <h3 class="section-label">预览效果</h3>
              <div class="preview-row">
                <div class="preview-btn" :style="{ background: currentTheme }">主要按钮</div>
                <div class="preview-btn ghost" :style="{ color: currentTheme, borderColor: currentTheme }">幽灵按钮</div>
                <div class="preview-tag" :style="{ background: currentTheme }">标签</div>
              </div>
            </div>
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
  import {
    applyClientTheme, getClientTheme,
    heroPresets, applyHeroPreset, getHeroPreset,
    cardStyles, applyCardStyle, getCardStyle,
  } from '@/utils/clientTheme';

  const userStore = useUserStore();
  const message = useMessage();

  const activeTab = ref('info');
  const currentTheme = ref(getClientTheme());
  const currentHero = ref(getHeroPreset());
  const currentCardStyle = ref(getCardStyle());

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
    {
      key: 'appearance',
      label: '外观设置',
      icon: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/><path d="M2 12h20"/></svg>',
    },
  ];

  const themeColors = [
    { name: '经典蓝', value: '#2d8cf0' },
    { name: '翡翠绿', value: '#22c55e' },
    { name: '青碧', value: '#009688' },
    { name: '天青', value: '#00C1D4' },
    { name: '靛蓝', value: '#536dfe' },
    { name: '深海蓝', value: '#0960bd' },
    { name: '海洋', value: '#0084f4' },
    { name: '碧波', value: '#0096c7' },
    { name: '薰衣草', value: '#9c27b0' },
    { name: '珊瑚粉', value: '#ff5c93' },
    { name: '赤焰', value: '#FF3D68' },
    { name: '暖橙', value: '#ff9800' },
    { name: '烈焰', value: '#ee4f12' },
    { name: '琥珀', value: '#FB9300' },
    { name: '薄荷', value: '#71EFA3' },
    { name: '青瓷', value: '#78DEC7' },
    { name: '藏蓝', value: '#1768AC' },
    { name: '炭墨', value: '#171010' },
  ];

  function handleThemeChange(color) {
    currentTheme.value = color;
    applyClientTheme(color);
    message.success('主题色已更新');
  }

  function handleHeroChange(preset) {
    currentHero.value = preset;
    applyHeroPreset(preset);
    message.success('英雄区风格已更新');
  }

  function handleCardChange(style) {
    currentCardStyle.value = style;
    applyCardStyle(style);
    message.success('卡片风格已更新');
  }

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
    background: var(--client-theme-bg-light);
    color: var(--client-theme-color);
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
    color: var(--client-theme-color);
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

  /* ===== Appearance ===== */
  .section-label {
    font-size: 14px;
    font-weight: 600;
    color: #334155;
    margin: 0 0 16px;
  }

  .theme-section {
    margin-bottom: 32px;
  }

  .color-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
    gap: 12px;
  }

  .color-swatch {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px 8px;
    border-radius: 10px;
    border: 2px solid transparent;
    cursor: pointer;
    transition: all 0.15s ease;
    background: #f8fafc;
  }

  .color-swatch:hover {
    border-color: #e2e8f0;
    background: #fff;
  }

  .color-swatch.active {
    border-color: var(--swatch-color);
    background: #fff;
  }

  .swatch-circle {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--swatch-color);
    transition: transform 0.15s ease;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .color-swatch:hover .swatch-circle {
    transform: scale(1.1);
  }

  .color-swatch.active .swatch-circle {
    transform: scale(1.1);
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.8), 0 0 0 5px var(--swatch-color);
  }

  .swatch-name {
    font-size: 12px;
    color: #64748b;
    font-weight: 500;
    white-space: nowrap;
  }

  .color-swatch.active .swatch-name {
    color: #0f172a;
    font-weight: 600;
  }

  /* ===== Preset Cards (Hero & Card Style) ===== */
  .preset-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 12px;
  }

  .preset-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px;
    border-radius: 10px;
    border: 2px solid #e2e8f0;
    cursor: pointer;
    transition: all 0.15s ease;
    background: #f8fafc;
  }

  .preset-card:hover {
    border-color: #cbd5e1;
    background: #fff;
  }

  .preset-card.active {
    border-color: var(--client-theme-color);
    background: #fff;
  }

  .preset-preview {
    width: 100%;
    height: 64px;
    border-radius: 8px;
    overflow: hidden;
    position: relative;
  }

  .hero-preview-text {
    position: absolute;
    bottom: 8px;
    left: 10px;
    color: #f8fafc;
    font-size: 18px;
    font-weight: 700;
    font-family: 'JetBrains Mono', monospace;
    opacity: 0.7;
  }

  .card-preview {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px 12px;
  }

  .card-preview-line {
    height: 6px;
    border-radius: 3px;
    background: #e2e8f0;
  }

  .card-preview-line.w50 { width: 50%; }
  .card-preview-line.w70 { width: 70%; }
  .card-preview-line.w90 { width: 90%; }

  .preset-card.active .card-preview-line {
    background: rgba(var(--client-theme-rgb), 0.15);
  }

  .preset-name {
    font-size: 12px;
    color: #64748b;
    font-weight: 500;
    white-space: nowrap;
  }

  .preset-card.active .preset-name {
    color: #0f172a;
    font-weight: 600;
  }

  .theme-preview {
    padding-top: 24px;
    border-top: 1px solid #f1f5f9;
  }

  .preview-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .preview-btn {
    padding: 8px 20px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: #fff;
    cursor: default;
  }

  .preview-btn.ghost {
    background: transparent;
    border: 1px solid;
  }

  .preview-tag {
    padding: 4px 12px;
    border-radius: 4px;
    font-size: 13px;
    color: #fff;
    font-weight: 500;
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

    .color-grid {
      grid-template-columns: repeat(auto-fill, minmax(72px, 1fr));
      gap: 8px;
    }
  }
</style>
