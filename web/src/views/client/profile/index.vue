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

            <!-- 头像 -->
            <div class="avatar-section">
              <div class="avatar-preview" @click="triggerAvatarUpload" style="cursor: pointer">
                <img v-if="avatarUrl" :src="avatarUrl" alt="avatar" class="avatar-img" />
                <div v-else class="avatar-placeholder">
                  <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" stroke-width="1.5"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                </div>
                <div class="avatar-overlay">
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#fff" stroke-width="2"><path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/><circle cx="12" cy="13" r="4"/></svg>
                </div>
              </div>
              <input ref="avatarInput" type="file" accept="image/*" style="display: none" @change="handleAvatarChange" />
              <div class="avatar-hint">点击更换头像，支持 JPG/PNG，不超过 2MB</div>
            </div>

            <!-- 个人简介 -->
            <div class="bio-section">
              <label class="bio-label">个人简介</label>
              <n-input
                v-model:value="bioText"
                type="textarea"
                placeholder="介绍一下自己吧..."
                :maxlength="200"
                show-count
                :rows="3"
              />
              <n-button type="primary" size="small" style="margin-top: 12px" @click="handleSaveBio" :loading="bioSaving">保存简介</n-button>
            </div>

            <n-divider />

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

          <!-- 令牌管理 -->
          <div v-if="activeTab === 'tokens'" class="content-panel">
            <div class="panel-header">
              <div class="panel-header-row">
                <div>
                  <h2>令牌管理</h2>
                  <p class="panel-desc">用于 CLI、桌面端等第三方工具对接 API</p>
                </div>
                <n-button type="primary" @click="showCreateToken = true">
                  创建令牌
                </n-button>
              </div>
            </div>

            <div v-if="tokenList.length === 0" class="empty-state">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#cbd5e1" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
              <p>还没有令牌</p>
              <span>创建一个令牌来让其他工具访问你的 API</span>
            </div>

            <div v-else class="token-list">
              <div v-for="token in tokenList" :key="token.id" class="token-item">
                <div class="token-info">
                  <div class="token-name-row">
                    <span class="token-name">{{ token.name }}</span>
                    <span class="token-prefix">{{ token.token_prefix }}</span>
                  </div>
                  <div class="token-meta">
                    <span>创建于 {{ formatDate(token.created_at) }}</span>
                    <span v-if="token.last_used_at"> · 最近使用 {{ formatDate(token.last_used_at) }}</span>
                    <span v-if="token.expires_at" class="token-expiry"> · {{ isExpired(token.expires_at) ? '已过期' : '过期于 ' + formatDate(token.expires_at) }}</span>
                  </div>
                  <div v-if="token.scopes && parseScopes(token.scopes).length" class="token-scopes">
                    <span v-for="scope in parseScopes(token.scopes)" :key="scope" class="scope-tag">{{ scopeLabelMap[scope] || scope }}</span>
                  </div>
                </div>
                <n-button size="small" type="error" ghost @click="handleDeleteToken(token.id)">删除</n-button>
              </div>
            </div>

            <!-- 创建令牌弹窗 -->
            <n-modal v-model:show="showCreateToken" preset="dialog" title="创建新令牌" :show-icon="false" style="width: 480px">
              <n-form label-placement="top">
                <n-form-item label="令牌名称">
                  <n-input v-model:value="newTokenName" placeholder="如：CLI 工具、VS Code 插件" />
                </n-form-item>
                <n-form-item label="过期时间">
                  <n-select
                    v-model:value="newTokenExpiry"
                    :options="expiryOptions"
                    placeholder="选择过期时间"
                  />
                </n-form-item>
                <n-form-item label="权限范围">
                  <n-checkbox-group v-model:value="newTokenScopes">
                    <n-space item-style="display: flex;">
                      <n-checkbox v-for="opt in scopeOptions" :key="opt.value" :value="opt.value" :label="opt.label" />
                    </n-space>
                  </n-checkbox-group>
                </n-form-item>
              </n-form>
              <div v-if="createdToken" class="token-created">
                <div class="token-created-warning">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
                  <span>请立即复制令牌，关闭后将无法再次查看</span>
                </div>
                <div class="token-created-value">
                  <code>{{ createdToken }}</code>
                  <n-button size="tiny" quaternary @click="copyToken">复制</n-button>
                </div>
              </div>
              <template #action>
                <n-button v-if="!createdToken" @click="showCreateToken = false">取消</n-button>
                <n-button v-if="!createdToken" type="primary" @click="handleCreateToken" :loading="createTokenLoading" :disabled="!newTokenName.trim()">创建</n-button>
                <n-button v-else @click="closeCreateModal">完成</n-button>
              </template>
            </n-modal>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
  import { ref, computed, reactive, onMounted, watch } from 'vue';
  import { useMessage } from 'naive-ui';
  import { useUserStore } from '@/store/modules/user';
  import { changePassword, createPat, listPats, deletePat, updateProfile, uploadAvatar } from '@/api/system/user';
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

  // 头像 & 简介
  const avatarInput = ref(null);
  const rawAvatar = ref(userStore.getUserInfo?.avatar || '');
  const bioText = ref(userStore.getUserInfo?.bio || '');
  const bioSaving = ref(false);

  const getApiBase = () => (import.meta.env.VITE_API_URL || '').replace(/\/+$/, '');
  const avatarUrl = computed(() => {
    const a = rawAvatar.value;
    if (!a) return '';
    if (a.startsWith('http')) return a;
    return `${getApiBase()}${a}`;
  });

  watch(() => userStore.getUserInfo, (info) => {
    if (info) {
      rawAvatar.value = info.avatar || '';
      bioText.value = info.bio || '';
    }
  }, { immediate: true });

  function triggerAvatarUpload() {
    avatarInput.value?.click();
  }

  async function handleAvatarChange(e) {
    const file = e.target.files?.[0];
    if (!file) return;
    if (file.size > 2 * 1024 * 1024) {
      message.error('头像文件不能超过2MB');
      return;
    }
    try {
      const res = await uploadAvatar(file);
      if (res.data?.code === 0) {
        rawAvatar.value = res.data?.data?.avatar;
        message.success('头像更新成功');
        await userStore.getInfo();
      } else {
        message.error(res.data?.message || '上传失败');
      }
    } catch {
      message.error('上传失败');
    }
    e.target.value = '';
  }

  async function handleSaveBio() {
    bioSaving.value = true;
    try {
      const res = await updateProfile({ bio: bioText.value });
      if (res.data?.code === 0) {
        message.success('简介已保存');
        await userStore.getInfo();
      } else {
        message.error(res.data?.message || '保存失败');
      }
    } catch {
      message.error('保存失败');
    } finally {
      bioSaving.value = false;
    }
  }

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
    {
      key: 'tokens',
      label: '令牌管理',
      icon: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>',
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

  // ===== 令牌管理 =====
  const tokenList = ref([]);
  const showCreateToken = ref(false);
  const newTokenName = ref('');
  const newTokenExpiry = ref(null);
  const newTokenScopes = ref(['template:read', 'generate:use']);
  const createTokenLoading = ref(false);
  const createdToken = ref('');

  const expiryOptions = [
    { label: '永不过期', value: null },
    { label: '30 天', value: 30 },
    { label: '90 天', value: 90 },
    { label: '180 天', value: 180 },
    { label: '365 天', value: 365 },
  ];

  const scopeOptions = [
    { label: '查看模板', value: 'template:read' },
    { label: '创建/编辑模板', value: 'template:write' },
    { label: '删除模板', value: 'template:delete' },
    { label: '提交审核/发布', value: 'template:publish' },
    { label: '使用模板生成代码', value: 'generate:use' },
    { label: '创建发布版本', value: 'release:create' },
    { label: '回滚版本', value: 'release:rollback' },
  ];

  const scopeLabelMap = {
    'template:read': '查看',
    'template:write': '编辑',
    'template:delete': '删除',
    'template:publish': '发布',
    'generate:use': '生成',
    'release:create': '版本',
    'release:rollback': '回滚',
  };

  async function loadTokens() {
    try {
      const result = await listPats();
      tokenList.value = result || [];
    } catch {}
  }

  watch(activeTab, (v) => { if (v === 'tokens') loadTokens(); });

  async function handleCreateToken() {
    createTokenLoading.value = true;
    try {
      const result = await createPat({
        name: newTokenName.value,
        expires_in_days: newTokenExpiry.value,
        scopes: newTokenScopes.value,
      });
      createdToken.value = result?.token || '';
      message.success('令牌创建成功');
      await loadTokens();
    } catch {
      message.error('创建失败');
    } finally {
      createTokenLoading.value = false;
    }
  }

  async function handleDeleteToken(id) {
    try {
      await deletePat(id);
      message.success('令牌已删除');
      await loadTokens();
    } catch {
      message.error('删除失败');
    }
  }

  function closeCreateModal() {
    showCreateToken.value = false;
    newTokenName.value = '';
    newTokenExpiry.value = null;
    newTokenScopes.value = ['template:read', 'generate:use'];
    createdToken.value = '';
    loadTokens();
  }

  function copyToken() {
    navigator.clipboard.writeText(createdToken.value);
    message.success('已复制到剪贴板');
  }

  function parseScopes(scopes) {
    if (Array.isArray(scopes)) return scopes;
    try { return JSON.parse(scopes); } catch { return []; }
  }

  function formatDate(d) {
    if (!d) return '-';
    return new Date(d).toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
  }

  function isExpired(d) {
    return new Date(d) < new Date();
  }
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

  /* ===== Avatar & Bio ===== */
  .avatar-section {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 24px;
  }

  .avatar-preview {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    overflow: hidden;
    position: relative;
    background: #f1f5f9;
    border: 2px solid #e2e8f0;
    flex-shrink: 0;
  }

  .avatar-preview:hover .avatar-overlay {
    opacity: 1;
  }

  .avatar-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .avatar-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .avatar-hint {
    font-size: 13px;
    color: #94a3b8;
  }

  .bio-section {
    margin-bottom: 8px;
  }

  .bio-label {
    display: block;
    font-size: 14px;
    font-weight: 600;
    color: #334155;
    margin-bottom: 8px;
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

  /* ===== Token Management ===== */
  .panel-header-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 64px 20px;
    color: #94a3b8;
  }

  .empty-state p {
    font-size: 15px;
    font-weight: 500;
    color: #64748b;
    margin: 16px 0 4px;
  }

  .empty-state span {
    font-size: 13px;
  }

  .token-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .token-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-radius: 10px;
    background: #f8fafc;
    border: 1px solid #f1f5f9;
    transition: border-color 0.15s ease;
  }

  .token-item:hover {
    border-color: #e2e8f0;
  }

  .token-name-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 4px;
  }

  .token-name {
    font-size: 14px;
    font-weight: 600;
    color: #0f172a;
  }

  .token-prefix {
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
    color: #94a3b8;
    background: #f1f5f9;
    padding: 2px 8px;
    border-radius: 4px;
  }

  .token-meta {
    font-size: 12px;
    color: #94a3b8;
  }

  .token-expiry {
    color: #f59e0b;
  }

  .token-scopes {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 6px;
  }

  .scope-tag {
    display: inline-block;
    padding: 1px 8px;
    font-size: 11px;
    color: #6366f1;
    background: #eef2ff;
    border-radius: 10px;
    line-height: 20px;
  }

  .token-created {
    margin-top: 16px;
    padding: 16px;
    background: #fffbeb;
    border: 1px solid #fde68a;
    border-radius: 8px;
  }

  .token-created-warning {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #92400e;
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 12px;
  }

  .token-created-value {
    display: flex;
    align-items: center;
    gap: 8px;
    background: #fff;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    padding: 8px 12px;
  }

  .token-created-value code {
    flex: 1;
    font-size: 13px;
    font-family: 'JetBrains Mono', monospace;
    color: #0f172a;
    word-break: break-all;
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
