<template>
  <div class="header titlebar-drag-region">
    <div class="header-left">
      <a-button
        type="text"
        class="sidebar-toggle titlebar-no-drag"
        @click="emit('toggle-sidebar')"
      >
        <template #icon>
          <MenuFoldOutlined v-if="!sidebarCollapsed" />
          <MenuUnfoldOutlined v-else />
        </template>
      </a-button>
      <a-breadcrumb>
        <a-breadcrumb-item>
          <a @click.prevent="emit('go-back')" class="titlebar-no-drag">项目列表</a>
        </a-breadcrumb-item>
        <a-breadcrumb-item>{{ projectName }}</a-breadcrumb-item>
        <a-breadcrumb-item>{{ currentPageTitle }}</a-breadcrumb-item>
      </a-breadcrumb>
    </div>
    <div class="header-right">
      <div class="project-info-display">
        <FolderOutlined class="info-icon project-icon" />
        <span class="project-name-text">{{ projectName }}</span>
        <a-divider type="vertical" />
        <DatabaseOutlined class="info-icon" />
        <span class="info-text">{{ databaseType }}</span>
        <a-divider type="vertical" />
        <TableOutlined class="info-icon" />
        <span class="info-text">{{ tableCount }} 张表</span>
      </div>

      <a-button
        type="text"
        size="small"
        class="window-control back-btn"
        @click="emit('go-back')"
      >
        <template #icon>
          <ArrowLeftOutlined />
        </template>
      </a-button>

      <div class="window-controls titlebar-no-drag">
        <a-button
          type="text"
          size="small"
          class="window-control"
          @click="emit('minimize')"
        >
          <template #icon>
            <MinusOutlined />
          </template>
        </a-button>
        <a-button
          type="text"
          size="small"
          class="window-control"
          @click="emit('maximize')"
        >
          <template #icon>
            <BorderOutlined />
          </template>
        </a-button>
        <a-button
          type="text"
          size="small"
          class="window-control close"
          @click="emit('close')"
        >
          <template #icon>
            <CloseOutlined />
          </template>
        </a-button>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  TableOutlined,
  DatabaseOutlined,
  FolderOutlined,
  ArrowLeftOutlined,
  MinusOutlined,
  BorderOutlined,
  CloseOutlined,
  MenuFoldOutlined,
  MenuUnfoldOutlined
} from '@ant-design/icons-vue'

defineProps({
  sidebarCollapsed: { type: Boolean, required: true },
  projectName: { type: String, required: true },
  databaseType: { type: String, required: true },
  tableCount: { type: [Number, String], required: true },
  currentPageTitle: { type: String, required: true }
})

const emit = defineEmits(['toggle-sidebar', 'go-back', 'minimize', 'maximize', 'close'])
</script>

<style scoped>
.header {
  background: var(--color-navbar);
  border-bottom: 1px solid var(--color-border);
  padding: 0 var(--spacing-lg);
  display: flex;
  align-items: center;
  justify-content: space-between;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
  height: var(--navbar-height);
  line-height: var(--navbar-height);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.sidebar-toggle {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
}

.sidebar-toggle:hover {
  color: var(--color-primary);
  background: var(--color-hover);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.project-info-display {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 16px;
  background: transparent;
  height: 32px;
  line-height: 20px;
}

.header-right :deep(.ant-btn) {
  margin: 0;
  cursor: pointer !important;
}

.header-right :deep(.ant-btn:hover) {
  cursor: pointer !important;
}

.header-right :deep(.ant-btn .anticon) {
  cursor: pointer !important;
}

.project-icon {
  font-size: 16px;
  color: var(--color-primary);
}

.project-name-text {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  line-height: 20px;
}

.info-icon {
  font-size: 14px;
  color: var(--color-text-secondary);
}

.info-text {
  font-size: 13px;
  color: var(--color-text);
  font-weight: 500;
  line-height: 20px;
}

.titlebar-drag-region {
  -webkit-app-region: drag;
  user-select: none;
}

.titlebar-no-drag {
  -webkit-app-region: no-drag;
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 12px;
}

.window-control {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: all 0.2s;
}

.window-control:hover {
  color: var(--color-primary);
  background: var(--color-hover);
}

.window-control.close:hover {
  background: #ff4757;
  color: white;
}
</style>
