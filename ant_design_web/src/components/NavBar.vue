<template>
  <a-layout-header class="ant-layout-header header-no-padding">
    <div class="nav-bar-center">
      <div class="nav-bar-container">
        <div class="nav-left">
          <div class="brand" @click="goHome" style="cursor: pointer">
            <div class="logo-icon">
              <svg
                width="28"
                height="28"
                viewBox="0 0 32 32"
                fill="none"
              >
                <rect width="32" height="32" rx="6" fill="url(#brandGradient)" />
                <rect x="8" y="6" width="12" height="16" rx="1" fill="#ffffff" />
                <path d="M18 6 L18 10 L22 10 Z" fill="#e6f7ff" />
                <rect x="10" y="10" width="6" height="1" fill="#52c41a" />
                <rect x="10" y="12" width="4" height="1" fill="#1890ff" />
                <rect x="10" y="14" width="5" height="1" fill="#722ed1" />
                <circle cx="11" cy="17" r="0.5" fill="#ff4d4f" />
                <circle cx="13" cy="17" r="0.5" fill="#ff4d4f" />
                <rect x="14.5" y="16.5" width="2" height="1" fill="#ff4d4f" />
                <path
                  d="M22 20 L26 24 L22 28"
                  stroke="var(--client-theme-color)"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  fill="none"
                />
                <defs>
                  <linearGradient id="brandGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" style="stop-color: #0f172a; stop-opacity: 1" />
                    <stop offset="100%" style="stop-color: #1e293b; stop-opacity: 1" />
                  </linearGradient>
                </defs>
              </svg>
            </div>
            <span class="logo-text">Template <span class="brand-accent">Studio</span></span>
          </div>
          <a-menu
            class="menu-center"
            mode="horizontal"
            :items="menuOptions"
            :selectedKeys="[activeKey]"
            @click="({key}) => handleUpdateValue(key)"
          />
          <div class="search-inline" ref="searchRef">
            <a-input
              v-model:value="searchKeyword"
              placeholder="搜索模板..."
              class="search-input"
              allow-clear
              @keyup.enter="handleSearch"
              @change="handleSearchInputChange"
              @clear="handleClear"
            >
              <template #prefix>
                <SearchOutline style="font-size: 16px; color: #94a3b8" />
              </template>
            </a-input>
          </div>
        </div>
        <div class="nav-right">
          <template v-if="isLoggedIn">
            <a-dropdown>
              <div class="user-trigger">
                <a-avatar v-if="userAvatar" :size="32" :src="userAvatar" />
                <a-avatar v-else :size="32" style="background: linear-gradient(135deg, #0f172a, var(--client-theme-color))">
                  {{ userStore.getNickname?.charAt(0)?.toUpperCase() || 'U' }}
                </a-avatar>
                <span class="user-name">{{ userStore.getNickname }}</span>
              </div>
              <template #overlay>
                <a-menu @click="({key}) => handleUserMenu(key)">
                  <template v-for="opt in userMenuOptions" :key="opt.key">
                    <a-menu-divider v-if="opt.type === 'divider'" />
                    <a-menu-item v-else :key="opt.key">
                      <component v-if="opt.icon" :is="opt.icon" style="margin-right: 8px" />
                      {{ opt.label }}
                    </a-menu-item>
                  </template>
                </a-menu>
              </template>
            </a-dropdown>
          </template>
          <template v-else>
            <div class="auth-buttons">
              <a-button type="text" @click="goLogin">登录</a-button>
              <a-button type="primary" size="small" @click="goRegister">注册</a-button>
            </div>
          </template>
        </div>
      </div>
    </div>
  </a-layout-header>
</template>

<script setup>
  import { ref, computed, onMounted, watch, h } from 'vue';
  import { useRoute, useRouter } from 'vue-router';
  import { SearchOutline, SettingsOutline, PersonOutline, LogOutOutline } from '@/icons/ionicons5';
  import { useUser } from '@/store/modules/user';
  import { storage } from '@/utils/Storage';
  import { ACCESS_TOKEN } from '@/store/mutation-types';
  import { applyClientTheme, getClientTheme, applyHeroPreset, getHeroPreset, applyCardStyle, getCardStyle } from '@/utils/clientTheme';

  function renderIcon(icon) {
    return () => h(icon);
  }

  const route = useRoute();
  const router = useRouter();
  const userStore = useUser();

  const isLoggedIn = computed(() => !!userStore.getToken);

  const userAvatar = computed(() => {
    const avatar = userStore.getUserInfo?.avatar;
    if (!avatar) return '';
    if (avatar.startsWith('http')) return avatar;
    const base = (import.meta.env.VITE_API_URL || '').replace(/\/+$/, '');
    return `${base}${avatar}`;
  });

  applyClientTheme(getClientTheme());
  applyHeroPreset(getHeroPreset());
  applyCardStyle(getCardStyle());

  const userMenuOptions = computed(() => {
    const options = [
      { label: '个人中心', key: 'profile', icon: renderIcon(PersonOutline) },
    ];
    if (userStore.isAdmin) {
      options.push({ label: '后台管理', key: 'admin', icon: renderIcon(SettingsOutline) });
    }
    options.push({ type: 'divider', key: 'd1' });
    options.push({ label: '退出登录', key: 'logout', icon: renderIcon(LogOutOutline) });
    return options;
  });

  function handleUserMenu(key) {
    if (key === 'admin') {
      router.push('/admin/dashboard');
    } else if (key === 'profile') {
      router.push('/profile');
    } else if (key === 'logout') {
      userStore.logout();
      window.location.href = '/';
    }
  }

  function goLogin() {
    router.push('/login');
  }

  function goRegister() {
    router.push({ path: '/login', query: { mode: 'register' } });
  }

  const menuOptions = computed(() => {
    const options = [
      { label: '首页', key: 'home' },
      { label: '模板', key: 'templates' },
    ];
    if (userStore.getToken) {
      options.push({ label: '我的模板', key: 'my-templates' });
    }
    return options;
  });

  const activeKey = ref(route.name || 'home');

  function handleUpdateValue(key) {
    activeKey.value = key;
    if (key === 'home') router.push('/');
    else if (key === 'templates') router.push('/templates');
    else if (key === 'my-templates') router.push('/my-templates');
  }

  function goHome() {
    router.push('/');
  }

  const searchKeyword = ref('');
  const searchRef = ref(null);

  const initSearchKeyword = () => {
    const routeSearch = route.query.search || '';
    searchKeyword.value = routeSearch;
  };

  function handleSearchInputChange(e) {
    const value = e?.target?.value ?? e;
    searchKeyword.value = value;
    if (!String(value).trim() && route.path === '/templates' && route.query.search) {
      router.push({ path: '/templates', query: {} });
    }
  }

  function handleSearchChange(value) {
    searchKeyword.value = value;
    if (!value.trim() && route.path === '/templates' && route.query.search) {
      router.push({ path: '/templates', query: {} });
    }
  }

  function handleSearch() {
    if (searchKeyword.value.trim()) {
      router.push({ path: '/templates', query: { search: searchKeyword.value.trim() } });
    }
  }

  function handleClear() {
    searchKeyword.value = '';
    if (route.path === '/templates' && route.query.search) {
      router.push({ path: '/templates', query: {} });
    }
  }

  watch(
    () => route.query.search,
    (newSearch) => {
      const currentSearch = searchKeyword.value;
      if (newSearch !== currentSearch) {
        searchKeyword.value = newSearch || '';
      }
    }
  );

  onMounted(() => {
    initSearchKeyword();
  });
</script>

<style scoped>
  .header-no-padding {
    padding: 0 !important;
  }

  .nav-bar-center {
    width: 80vw;
    max-width: 1280px;
    margin: 0 auto;
    display: flex;
    justify-content: center;
  }

  .nav-bar-container {
    width: 100%;
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: #fff;
    box-sizing: border-box;
    padding: 0 24px;
    position: relative;
  }

  .nav-left {
    display: flex;
    align-items: center;
    gap: 32px;
  }

  .nav-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .auth-buttons {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .user-trigger {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border-radius: 8px;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .user-trigger:hover {
    background-color: #f1f5f9;
  }

  .user-name {
    font-size: 14px;
    color: #334155;
    font-weight: 500;
  }

  /* ===== Brand ===== */
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .brand:hover {
    opacity: 0.85;
  }

  .logo-icon {
    display: flex;
    align-items: center;
  }

  .logo-text {
    font-size: 16px;
    font-weight: 700;
    color: #0f172a;
    font-family: 'JetBrains Mono', 'Fira Code', 'Segoe UI', sans-serif;
    letter-spacing: -0.3px;
    white-space: nowrap;
  }

  .brand-accent {
    color: var(--client-theme-color);
  }

  /* ===== Menu ===== */
  .menu-center {
    min-width: 160px;
  }

  .search-inline {
    display: flex;
    align-items: center;
  }

  .search-input {
    width: 220px;
    background: #f8fafc;
    border: 1px solid #e2e8f0;
  }

  /* ===== Fixed Header ===== */
  .ant-layout-header {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 200;
    background: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
    border-bottom: 1px solid #f1f5f9;
    height: auto;
    line-height: normal;
    padding: 0;
  }

  :deep(.menu-center.ant-menu-horizontal) {
    justify-content: flex-start !important;
    border-bottom: none;
  }

  /* ===== Responsive ===== */
  @media (max-width: 768px) {
    .search-input {
      width: 160px;
    }
    .nav-left {
      gap: 16px;
    }
  }
</style>
