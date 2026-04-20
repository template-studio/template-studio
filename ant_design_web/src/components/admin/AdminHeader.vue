<template>
  <a-layout-header class="admin-header">
    <div class="header-content">
      <!-- 左侧：侧边栏切换按钮和面包屑 -->
      <div class="header-left">
        <a-button type="text" @click="$emit('toggle-sidebar')" class="sidebar-toggle">
          <template #icon>
            <MenuOutline style="font-size: 18px" />
          </template>
        </a-button>

        <!-- 面包屑 -->
        <div class="breadcrumb-wrapper">
          <AdminBreadcrumb />
        </div>
      </div>

      <!-- 右侧：操作按钮 -->
      <div class="header-right">
        <!-- 返回前台 -->
        <a-tooltip title="返回前台">
          <a-button type="text" @click="goFrontend" class="header-action">
            <template #icon>
              <ExitOutline style="font-size: 18px" />
            </template>
          </a-button>
        </a-tooltip>

        <!-- 全屏切换 -->
        <a-tooltip title="全屏">
          <a-button type="text" @click="toggleFullscreen" class="header-action">
            <template #icon>
              <ScanOutline style="font-size: 18px" />
            </template>
          </a-button>
        </a-tooltip>
      </div>
    </div>
  </a-layout-header>
</template>

<script setup>
  import { useRouter } from 'vue-router';
  // ant-design-vue components are globally registered
  import AdminBreadcrumb from './AdminBreadcrumb.vue';
  import { MenuOutline, ScanOutline, ExitOutline } from '@/icons/ionicons5';

  const props = defineProps({
    sidebarCollapsed: {
      type: Boolean,
      default: false,
    },
  });

  const emit = defineEmits(['toggle-sidebar']);

  const router = useRouter();

  function toggleFullscreen() {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen();
    } else {
      if (document.exitFullscreen) {
        document.exitFullscreen();
      }
    }
  }

  function goFrontend() {
    router.push('/');
  }
</script>

<style scoped>
  .admin-header {
    height: 56px;
    background: #fff;
    position: sticky;
    top: 0;
    z-index: 99;
    border-bottom: 1px solid #f0f0f0;
  }

  .header-content {
    height: 100%;
    padding: 0 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px; /* 按钮和面包屑之间的间距 */
  }

  .sidebar-toggle {
    padding: 8px;
    border-radius: 6px;
    flex-shrink: 0; /* 防止按钮收缩 */
  }

  .sidebar-toggle:hover {
    background: #f5f5f5;
  }

  .breadcrumb-wrapper {
    flex: 1;
    overflow: hidden; /* 防止面包屑溢出 */
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0; /* 防止右侧元素收缩 */
  }

  .header-action {
    padding: 8px;
    border-radius: 6px;
    color: #666;
  }

  .header-action:hover {
    background: #f5f5f5;
    color: #333;
  }

  /* 面包屑样式调整 */
  .breadcrumb-wrapper :deep(.breadcrumb-container) {
    background: transparent;
    padding: 0;
    border: none;
  }

  .breadcrumb-wrapper :deep(.admin-breadcrumb) {
    font-size: 13px;
  }

  .breadcrumb-wrapper :deep(.ant-breadcrumb-link) {
    font-size: 13px;
  }

  /* 确保面包屑不会换行 */
  .breadcrumb-wrapper :deep(.ant-breadcrumb) {
    white-space: nowrap;
  }
</style>
