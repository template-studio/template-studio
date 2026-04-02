<template>
  <n-layout-header bordered class="n-layout-header header-no-padding">
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
          <n-menu
            class="menu-center"
            mode="horizontal"
            :options="menuOptions"
            :value="activeKey"
            @update:value="handleUpdateValue"
          />
          <div class="search-inline" ref="searchRef">
            <n-input
              v-model:value="searchKeyword"
              round
              placeholder="搜索模板..."
              class="search-input"
              clearable
              @keyup.enter="handleSearch"
              @update:value="handleSearchChange"
              @clear="handleClear"
            >
              <template #prefix>
                <n-icon size="16" color="#94a3b8">
                  <SearchOutline />
                </n-icon>
              </template>
            </n-input>
          </div>
        </div>
        <div class="nav-right">
          <template v-if="isLoggedIn">
            <n-dropdown :options="userMenuOptions" @select="handleUserMenu">
              <div class="user-trigger">
                <n-avatar round size="small" style="background: linear-gradient(135deg, #0f172a, var(--client-theme-color))">
                  {{ userStore.getNickname?.charAt(0)?.toUpperCase() || 'U' }}
                </n-avatar>
                <span class="user-name">{{ userStore.getNickname }}</span>
              </div>
            </n-dropdown>
          </template>
          <template v-else>
            <div class="auth-buttons">
              <n-button text @click="goLogin">登录</n-button>
              <n-button type="primary" size="small" @click="goRegister">注册</n-button>
            </div>
          </template>
        </div>
      </div>
    </div>
  </n-layout-header>
</template>

<script setup>
  import { ref, computed, onMounted, watch } from 'vue';
  import { useRoute, useRouter } from 'vue-router';
  import { NLayoutHeader, NMenu, NInput, NButton, NIcon, NDropdown, NAvatar } from 'naive-ui';
  import { SearchOutline, SettingsOutline, PersonOutline, LogOutOutline } from '@vicons/ionicons5';
  import { useUser } from '@/store/modules/user';
  import { renderIcon } from '@/utils/index';
  import { storage } from '@/utils/Storage';
  import { ACCESS_TOKEN } from '@/store/mutation-types';
  import { applyClientTheme, getClientTheme, applyHeroPreset, getHeroPreset, applyCardStyle, getCardStyle } from '@/utils/clientTheme';

  const route = useRoute();
  const router = useRouter();
  const userStore = useUser();

  const isLoggedIn = computed(() => !!userStore.getToken);

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
    return [
      { label: '首页', key: 'home' },
      { label: '模板', key: 'templates' },
    ];
  });

  const activeKey = ref(route.name || 'home');

  function handleUpdateValue(key) {
    activeKey.value = key;
    if (key === 'home') router.push('/');
    else if (key === 'templates') router.push('/templates');
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
  .n-layout-header {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 200;
    background: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
    border-bottom: 1px solid #f1f5f9;
  }

  :deep(.n-layout-header__content) {
    padding: 0 !important;
  }

  :deep(.menu-center .n-menu--horizontal .n-menu__content) {
    justify-content: flex-start !important;
  }

  /* ===== Dropdown ===== */
  :deep(.n-dropdown-option .n-dropdown-option-body) {
    padding: 10px 16px !important;
  }

  :deep(.n-dropdown-option .n-dropdown-option-body__prefix) {
    margin-right: 8px !important;
  }

  :deep(.n-dropdown-option .n-dropdown-option-body__label) {
    font-size: 14px !important;
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
