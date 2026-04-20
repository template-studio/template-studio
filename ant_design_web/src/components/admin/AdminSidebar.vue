<template>
  <a-layout-sider
    :collapsed="collapsed"
    :collapsed-width="64"
    :width="240"
    collapsible
    :trigger="null"
    @collapse="(val) => $emit('update:collapsed', val)"
    class="admin-sidebar"
  >
    <!-- Logo区域 -->
    <div class="sidebar-logo" @click="goHome">
      <div v-if="!collapsed" class="logo-full">
        <div class="logo-full-container">
          <svg
            width="24"
            height="24"
            viewBox="0 0 32 32"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            class="logo-full-icon"
          >
            <rect width="32" height="32" rx="6" fill="url(#adminFullGradient)" />
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
              <linearGradient id="adminFullGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style="stop-color: #18a058; stop-opacity: 1" />
                <stop offset="100%" style="stop-color: #1890ff; stop-opacity: 1" />
              </linearGradient>
            </defs>
          </svg>
          <div class="logo-text-container">
            <span class="logo-text">Template <span class="logo-accent">Studio</span></span>
            <span class="logo-shadow">Template <span class="logo-accent">Studio</span></span>
          </div>
        </div>
        <span class="logo-subtitle">管理后台</span>
      </div>
      <div v-else class="logo-collapsed">
        <svg
          width="32"
          height="32"
          viewBox="0 0 32 32"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          class="logo-icon-svg"
        >
          <rect width="32" height="32" rx="6" fill="url(#adminGradient)" />
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
            <linearGradient id="adminGradient" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" style="stop-color: #18a058; stop-opacity: 1" />
              <stop offset="100%" style="stop-color: #1890ff; stop-opacity: 1" />
            </linearGradient>
          </defs>
        </svg>
      </div>
    </div>

    <!-- 菜单 -->
    <a-menu
      mode="inline"
      :inline-collapsed="collapsed"
      :items="menuOptions"
      :selectedKeys="[activeKey]"
      :openKeys="openKeys"
      @update:openKeys="openKeys = $event"
      @click="handleMenuClick"
      class="sidebar-menu"
    />
  </a-layout-sider>
</template>

<script setup>
  import { ref, computed, h } from 'vue';
  import { useRoute, useRouter } from 'vue-router';
  import {
    GridOutline,
    LayersOutline,
    DocumentTextOutline,
    LanguageOutline,
    ServerOutline,
    OptionsOutline,
  } from '@/icons/ionicons5';

  const props = defineProps({
    collapsed: {
      type: Boolean,
      default: false,
    },
  });

  const emit = defineEmits(['update:collapsed']);

  const route = useRoute();
  const router = useRouter();

  const activeKey = ref(route.name || 'admin-dashboard');
  const openKeys = ref(['admin-basic-data']);

  function renderIcon(icon) {
    return () => h(icon);
  }

  const menuOptions = [
    {
      label: '首页',
      key: 'admin-dashboard',
      icon: renderIcon(GridOutline),
    },
    {
      label: '模板管理',
      key: 'admin-templates-list',
      icon: renderIcon(DocumentTextOutline),
    },
    {
      label: '基础数据',
      key: 'admin-basic-data',
      icon: renderIcon(ServerOutline),
      children: [
        {
          label: '分类管理',
          key: 'admin-categories',
          icon: renderIcon(LayersOutline),
        },
        {
          label: '语言管理',
          key: 'admin-languages',
          icon: renderIcon(LanguageOutline),
        },
        {
          label: '变量预设',
          key: 'admin-var-presets',
          icon: renderIcon(OptionsOutline),
        },
      ],
    },
  ];

  function handleMenuClick({ key }) {
    activeKey.value = key;
    router.push({ name: key });
  }

  function goHome() {
    router.push('/');
  }
</script>

<style scoped>
  .admin-sidebar {
    height: 100vh;
    position: fixed;
    left: 0;
    top: 0;
    z-index: 100;
    overflow: hidden !important;
  }

  :deep(.ant-layout-sider) {
    overflow: hidden !important;
  }

  :deep(.ant-layout-sider-children) {
    overflow: hidden !important;
    height: 100vh;
  }

  .sidebar-logo {
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-bottom: 1px solid #f0f0f0;
    cursor: pointer;
    transition: all 0.3s ease;
    background: #fff;
    position: relative;
  }

  .sidebar-logo:hover {
    background: #fafafa;
    transform: scale(1.02);
  }

  .sidebar-logo:hover .logo-text {
    text-shadow: 0 0 15px rgba(24, 160, 88, 0.6);
    transform: translateY(-1px) scale(1.05);
  }

  .sidebar-logo:hover .logo-shadow {
    opacity: 0.6;
    transform: translateY(2px) scale(1.05);
  }

  .sidebar-logo:hover .logo-icon {
    transform: scale(1.1) rotate(5deg);
    box-shadow: 0 4px 12px rgba(24, 160, 88, 0.3);
  }

  .sidebar-logo::before {
    content: '';
    position: absolute;
    top: -10px;
    left: -10px;
    right: -10px;
    bottom: -10px;
    background: radial-gradient(circle, rgba(24, 160, 88, 0.08) 0%, transparent 70%);
    opacity: 0;
    transition: opacity 0.3s ease;
    pointer-events: none;
  }

  .sidebar-logo:hover::before {
    opacity: 1;
  }

  .logo-full {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .logo-full-container {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .logo-full-icon {
    width: 24px;
    height: 24px;
    transition: all 0.3s ease;
  }

  .logo-text-container {
    position: relative;
    display: flex;
    align-items: center;
  }

  .logo-text {
    font-size: 16px;
    font-weight: 700;
    color: #333;
    line-height: 1.2;
    font-family: 'Fira Code', 'Lato', 'Segoe UI', 'Arial', sans-serif;
    background: linear-gradient(90deg, #18a058 0%, #2196f3 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    position: relative;
    z-index: 2;
    animation: float-admin 3s ease-in-out infinite;
    transition: all 0.3s ease;
    white-space: nowrap;
  }

  .logo-shadow {
    font-size: 16px;
    font-weight: 700;
    color: rgba(51, 51, 51, 0.15);
    font-family: 'Fira Code', 'Lato', 'Segoe UI', 'Arial', sans-serif;
    position: absolute;
    top: 1px;
    left: 0;
    right: 0;
    z-index: 1;
    animation: float-shadow-admin 3s ease-in-out infinite;
    transition: all 0.3s ease;
    filter: blur(0.5px);
    white-space: nowrap;
  }

  .logo-accent {
    color: #18a058;
    -webkit-text-fill-color: #18a058;
    background: none;
    font-weight: 800;
  }

  .logo-subtitle {
    font-size: 12px;
    color: #666;
    margin-top: 4px;
    transition: all 0.3s ease;
  }

  @keyframes float-admin {
    0%,
    100% {
      transform: translateY(0px);
    }

    50% {
      transform: translateY(-2px);
    }
  }

  @keyframes float-shadow-admin {
    0%,
    100% {
      transform: translateY(1px);
      opacity: 0.15;
    }

    50% {
      transform: translateY(3px);
      opacity: 0.25;
    }
  }

  .logo-collapsed {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .logo-icon {
    width: 32px;
    height: 32px;
    background: linear-gradient(135deg, #18a058, #36c);
    color: white;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    font-size: 14px;
    transition: all 0.3s ease;
    animation: pulse-admin 2s ease-in-out infinite;
  }

  .logo-icon-svg {
    width: 32px;
    height: 32px;
    transition: all 0.3s ease;
    animation: pulse-admin 2s ease-in-out infinite;
  }

  @keyframes pulse-admin {
    0%,
    100% {
      transform: scale(1);
      box-shadow: 0 2px 8px rgba(24, 160, 88, 0.2);
    }

    50% {
      transform: scale(1.05);
      box-shadow: 0 4px 12px rgba(24, 160, 88, 0.3);
    }
  }

  .sidebar-menu {
    padding: 8px 0;
  }

  :deep(.ant-menu-item) {
    margin: 0 8px 4px 8px;
    border-radius: 6px;
  }

  :deep(.ant-menu-item:hover) {
    background: rgba(24, 160, 88, 0.1);
  }

  :deep(.ant-menu-item-selected) {
    background: rgba(24, 160, 88, 0.15);
    color: #18a058;
    font-weight: 600;
  }

  :deep(.ant-menu-item-selected .ant-menu-item-icon) {
    color: #18a058;
  }
</style>
