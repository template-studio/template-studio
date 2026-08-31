<template>
  <n-layout has-sider style="height: 100vh;">
    <!-- 侧边栏 -->
    <n-layout-sider
      bordered
      show-trigger="arrow-circle"
      collapse-mode="width"
      :collapsed-width="64"
      :width="240"
      :native-scrollbar="false"
    >
      <div class="sidebar">
        <div class="app-title">
          <div class="brand-logo">
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
              <rect x="10" y="12" width="4" height="1" fill="#16a34a" />
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
                  <stop offset="0%" style="stop-color: #16a34a; stop-opacity: 1" />
                  <stop offset="100%" style="stop-color: #18a058; stop-opacity: 1" />
                </linearGradient>
              </defs>
            </svg>
          </div>
          <transition name="fade">
            <span v-if="!collapsed" class="title-text">
              Template <span class="brand-accent">Studio</span>
            </span>
          </transition>
        </div>

        <n-menu
          v-model:value="activeKey"
          :collapsed="collapsed"
          :collapsed-width="64"
          :collapsed-icon-size="22"
          :options="menuOptions"
          @update:value="handleMenuSelect"
        />
      </div>
    </n-layout-sider>

    <!-- 主内容区 -->
    <n-layout>
      <!-- 内容区 -->
      <n-layout-content
        :native-scrollbar="false"
        :style="contentStyle"
      >
        <div class="main-content-wrapper">
          <template-config
            :templates="templates"
            @update:template="handleTemplateChange"
            @update:variables="handleVariablesChange"
          />
        </div>
      </n-layout-content>

      <!-- 右侧预览面板 - 选择模板后自动显示 -->
      <n-layout-sider
        v-if="currentTemplate"
        bordered
        :width="420"
        :native-scrollbar="false"
      >
        <preview-pane
          :template-id="currentTemplate.id"
          :variables="variables"
        />
      </n-layout-sider>
    </n-layout>
  </n-layout>
</template>

<script setup>
import { ref, computed, h, watch, onMounted } from 'vue';
import {
  NLayout, NLayoutSider, NLayoutContent,
  NMenu, NIcon
} from 'naive-ui';
import { FolderOpen } from '@vicons/ionicons5';
import { invoke } from '@tauri-apps/api/core';
import TemplateConfig from './TemplateConfig.vue';
import PreviewPane from './PreviewPane.vue';

const activeKey = ref('templates');
const collapsed = ref(false);
const currentTemplate = ref(null);
const projectName = ref('');
const variables = ref({});
const templates = ref([]);

// 菜单选项 - 移除预览菜单项
const menuOptions = [
  {
    label: '模板库',
    key: 'templates',
    icon: () => h(NIcon, null, { default: () => h(FolderOpen) })
  }
];

const contentStyle = computed(() => ({
  padding: '0',
  backgroundColor: '#ffffff'
}));

// 处理菜单选择
const handleMenuSelect = (key) => {
  activeKey.value = key;
};

// 处理模板变化
const handleTemplateChange = (template) => {
  currentTemplate.value = template;
};

// 处理变量变化
const handleVariablesChange = (vars) => {
  variables.value = vars;
  projectName.value = vars.project_name || vars.ProjectName || '';
};

// 生成项目
const generateProject = async () => {
  try {
    await invoke('generate_project', {
      templateId: currentTemplate.value.id,
      variables: variables.value,
      outputPath: '~/projects'
    });

    // 成功提示
    window.$message?.success('项目生成成功！');
  } catch (error) {
    console.error('生成项目失败:', error);
    window.$message?.error(`生成失败: ${error}`);
  }
};

// 加载模板列表
const loadTemplates = async () => {
  try {
    const result = await invoke('list_templates');
    templates.value = result || [];
  } catch (error) {
    console.error('加载模板列表失败:', error);
    window.$message?.error('加载模板列表失败: ' + error);
  }
};

// 组件挂载时加载模板列表
onMounted(() => {
  loadTemplates();
});
</script>

<style scoped>
.sidebar {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #ffffff;
}

.app-title {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 24px 20px;
  font-size: 18px;
  font-weight: 700;
  border-bottom: 1px solid #f0f0f0;
  background: #ffffff;
  position: relative;
}

.brand-logo {
  display: flex;
  align-items: center;
  transition: all 0.3s ease;
  cursor: pointer;
}

.brand-logo:hover {
  transform: scale(1.05);
}

.brand-logo svg {
  filter: drop-shadow(0 2px 4px rgba(24, 160, 88, 0.2));
}

.title-text {
  flex: 1;
  font-family: 'Fira Code', 'Segoe UI', 'Arial', sans-serif;
  letter-spacing: 0.5px;
  background: linear-gradient(90deg, #18a058 0%, #2196f3 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  font-weight: 800;
  font-size: 16px;
}

.brand-accent {
  font-weight: 900;
  -webkit-text-fill-color: #18a058;
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-enter-from, .fade-leave-to {
  opacity: 0;
  transform: translateX(-10px);
}

.content-area {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.main-content-wrapper {
  height: 100%;
  overflow-y: auto;
  padding: 0;
}

:deep(.n-layout-sider) {
  background: #ffffff;
  border-right: 1px solid #f0f0f0;
}

:deep(.n-menu) {
  border-right: none;
  background: transparent;
  padding: 8px;
}

:deep(.n-menu-item) {
  border-radius: 8px;
  margin: 2px 0;
  padding: 8px 12px;
  transition: all 0.2s ease;
  font-weight: 500;
}

:deep(.n-menu-item:hover) {
  background: #f5f7fa;
  color: #18a058;
}

:deep(.n-menu-item.n-menu-item--selected) {
  background: linear-gradient(90deg, rgba(24, 160, 88, 0.1) 0%, rgba(24, 160, 88, 0.05) 100%);
  color: #18a058;
  font-weight: 600;
}
</style>
