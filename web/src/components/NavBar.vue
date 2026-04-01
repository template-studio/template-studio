<template>
  <n-layout-header bordered class="n-layout-header header-no-padding">
    <div class="nav-bar-center">
      <div class="nav-bar-container">
        <div class="nav-left">
          <div class="brand" @click="goHome" style="cursor: pointer">
            <div class="logo-icon">
              <svg
                width="32"
                height="32"
                viewBox="0 0 32 32"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
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
                  stroke="#52c41a"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  fill="none"
                />
                <defs>
                  <linearGradient id="brandGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" style="stop-color: #1890ff; stop-opacity: 1" />
                    <stop offset="100%" style="stop-color: #18a058; stop-opacity: 1" />
                  </linearGradient>
                </defs>
              </svg>
            </div>
            <div class="logo-text">
              <span class="logo-main">Template <span class="brand-accent">Studio</span></span>
              <span class="logo-shadow">Template <span class="brand-accent">Studio</span></span>
            </div>
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
                <n-icon>
                  <SearchOutline />
                </n-icon>
              </template>
            </n-input>
          </div>
        </div>
        <div class="nav-right">
          <!-- 后台管理链接已移动到首页英雄区域 -->
        </div>
      </div>
    </div>
  </n-layout-header>
</template>

<script setup>
  import { ref, computed, onMounted, watch } from 'vue';
  import { useRoute, useRouter } from 'vue-router';
  import { NLayoutHeader, NMenu, NInput, NButton, NIcon } from 'naive-ui';
  import { SearchOutline, SettingsOutline } from '@vicons/ionicons5';

  const route = useRoute();
  const router = useRouter();

  const menuOptions = computed(() => {
    const baseOptions = [
      { label: '首页', key: 'home' },
      { label: '模板', key: 'templates' },
    ];

    return baseOptions;
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

  // 搜索相关
  const searchKeyword = ref('');
  const searchRef = ref(null);

  // 初始化搜索关键词 - 从路由参数中读取
  const initSearchKeyword = () => {
    const routeSearch = route.query.search || '';
    searchKeyword.value = routeSearch;
  };

  function handleSearchChange(value) {
    searchKeyword.value = value;

    // 如果搜索框被清空且当前在模板页面，则清空URL中的搜索参数
    if (!value.trim() && route.path === '/templates' && route.query.search) {
      router.push({
        path: '/templates',
        query: {},
      });
    }
  }

  function handleSearch() {
    if (searchKeyword.value.trim()) {
      router.push({
        path: '/templates',
        query: { search: searchKeyword.value.trim() },
      });
    }
  }

  function handleClear() {
    searchKeyword.value = '';

    // 如果当前在模板页面且有搜索参数，则清空搜索参数
    if (route.path === '/templates' && route.query.search) {
      router.push({
        path: '/templates',
        query: {},
      });
    }
  }

  // 监听路由变化，同步搜索框状态
  watch(
    () => route.query.search,
    (newSearch) => {
      const currentSearch = searchKeyword.value;
      if (newSearch !== currentSearch) {
        searchKeyword.value = newSearch || '';
      }
    }
  );

  // 组件挂载时初始化搜索关键词
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
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: #fff;
    box-sizing: border-box;
    padding: 0 32px;
    position: relative;
  }

  .nav-left {
    display: flex;
    align-items: center;
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

  .user-menu {
    display: flex;
    align-items: center;
  }

  .user-trigger {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border-radius: 6px;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .user-trigger:hover {
    background-color: rgba(0, 0, 0, 0.04);
  }

  .user-name {
    font-size: 14px;
    color: #333;
  }

  .brand {
    display: flex;
    align-items: center;
    margin-right: 36px;
    margin-left: 8px;
    position: relative;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .logo-icon {
    margin-right: 12px;
    display: flex;
    align-items: center;
    transition: all 0.3s ease;
  }

  .brand:hover .logo-icon {
    transform: scale(1.1) rotate(5deg);
  }

  .brand:hover {
    transform: scale(1.05) rotate(-1deg);
  }

  .brand:hover .logo-main {
    text-shadow: 0 0 20px rgba(24, 160, 88, 0.8);
    transform: translateY(-2px) scale(1.1);
  }

  .brand:hover .logo-shadow {
    opacity: 0.8;
    transform: translateY(4px) scale(1.1);
  }

  .logo-text {
    position: relative;
    display: flex;
    align-items: center;
  }

  .logo-main {
    font-size: 1.7rem;
    font-weight: 800;
    letter-spacing: 1.5px;
    color: #333;
    font-family: 'Fira Code', 'Lato', 'Segoe UI', 'Arial', sans-serif;
    background: linear-gradient(90deg, #18a058 0%, #2196f3 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    position: relative;
    z-index: 2;
    animation: float 3s ease-in-out infinite;
    transition: all 0.3s ease;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    white-space: nowrap;
  }

  .logo-shadow {
    font-size: 1.7rem;
    font-weight: 800;
    letter-spacing: 1.5px;
    color: rgba(51, 51, 51, 0.2);
    font-family: 'Fira Code', 'Lato', 'Segoe UI', 'Arial', sans-serif;
    position: absolute;
    top: 2px;
    left: 0;
    right: 0;
    z-index: 1;
    animation: float-shadow 3s ease-in-out infinite;
    transition: all 0.3s ease;
    filter: blur(1px);
    white-space: nowrap;
  }

  .brand-accent {
    color: #18a058;
    -webkit-text-fill-color: #18a058;
    background: none;
    font-weight: 900;
  }

  @keyframes float {
    0%,
    100% {
      transform: translateY(0px);
    }

    50% {
      transform: translateY(-3px);
    }
  }

  @keyframes float-shadow {
    0%,
    100% {
      transform: translateY(2px);
      opacity: 0.2;
    }

    50% {
      transform: translateY(5px);
      opacity: 0.3;
    }
  }

  .brand::before {
    content: '';
    position: absolute;
    top: -20px;
    left: -20px;
    right: -20px;
    bottom: -20px;
    background: radial-gradient(circle, rgba(24, 160, 88, 0.1) 0%, transparent 70%);
    opacity: 0;
    transition: opacity 0.3s ease;
    pointer-events: none;
  }

  .brand:hover::before {
    opacity: 1;
  }

  .menu-center {
    min-width: 200px;
  }

  .search-inline {
    margin-left: 32px;
    display: flex;
    align-items: center;
    position: relative;
  }

  .search-input {
    width: 260px;
    background: #f5f6fa;
  }

  .nav-right {
    display: flex;
    align-items: center;
    gap: 18px;
  }

  /* 后台管理链接样式 */
  .admin-link {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    border-radius: 6px;
    background: linear-gradient(135deg, #18a058 0%, #36ad6a 100%);
    color: white;
    text-decoration: none;
    font-weight: 400;
    font-size: 12px;
    transition: all 0.2s ease;
    box-shadow: 0 1px 4px rgba(24, 160, 88, 0.2);
    border: 1px solid transparent;
  }

  .admin-link:hover {
    background: linear-gradient(135deg, #16b878 0%, #40c477 100%);
    transform: translateY(-1px);
    box-shadow: 0 2px 6px rgba(24, 160, 88, 0.3);
    color: white;
    text-decoration: none;
  }

  .admin-link:active {
    transform: translateY(0);
    box-shadow: 0 1px 3px rgba(24, 160, 88, 0.2);
  }

  .admin-icon {
    font-size: 13px;
    transition: transform 0.2s ease;
  }

  .admin-link:hover .admin-icon {
    transform: rotate(90deg);
  }

  .admin-text {
    letter-spacing: 0.2px;
  }

  :deep(.menu-center .n-menu--horizontal .n-menu__content) {
    justify-content: flex-start !important;
  }

  :deep(.n-layout-header__content) {
    padding: 0 !important;
  }

  .n-layout-header {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 200;
    background: #fff;
    box-shadow: 0 2px 8px 0 rgba(60, 60, 60, 0.04);
  }

  /* 下拉菜单样式优化 */
  :deep(.n-dropdown-option .n-dropdown-option-body) {
    padding: 10px 16px !important;
  }

  :deep(.n-dropdown-option .n-dropdown-option-body__prefix) {
    margin-right: 8px !important;
  }

  :deep(.n-dropdown-option .n-dropdown-option-body__label) {
    font-size: 14px !important;
  }
</style>
