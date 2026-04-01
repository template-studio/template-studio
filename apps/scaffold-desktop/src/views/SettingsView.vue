<template>
  <div class="settings-container">
    <!-- 一级菜单 -->
    <SettingsSidebar
      :current-main-tab="currentMainTab"
      @main-tab-change="switchMainTab"
    />

    <div class="settings-content">
      <div class="settings-layout">
        <!-- 二级菜单 -->
        <SettingsSubSidebar
          :current-main-tab="currentMainTab"
          :current-sub-tab="currentSubTab"
          :show-sub-sidebar="showSubSidebar"
          @sub-tab-change="switchSubTab"
        />

        <!-- 三级菜单 -->
        <SettingsThirdSidebar
          :current-sub-tab="currentSubTab"
          :current-third-tab="currentThirdTab"
          :show-third-sidebar="showThirdSidebar"
          @third-tab-change="switchThirdTab"
        />

        <!-- 内容区域 -->
        <SettingsContent
          :current-main-tab="currentMainTab"
          :current-sub-tab="currentSubTab"
          :current-third-tab="currentThirdTab"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { onMounted } from 'vue'
import { useSettingsNavigation } from '@/composables/useSettingsNavigation'

// 导入新的组件
import {
  SettingsSidebar,
  SettingsSubSidebar,
  SettingsThirdSidebar,
  SettingsContent
} from '@/components/settings'

// 使用设置导航组合函数
const {
  currentMainTab,
  currentSubTab,
  currentThirdTab,
  showSubSidebar,
  showThirdSidebar,
  switchMainTab,
  switchSubTab,
  switchThirdTab,
  initializeNavigation
} = useSettingsNavigation()

// 组件挂载时初始化导航状态
onMounted(() => {
  initializeNavigation()
})
</script>

<style scoped>
.settings-container {
  display: flex;
  flex: 1;
  height: 100%;
  overflow: hidden;
  background: var(--color-background);
  color: var(--color-text);
}

.settings-content {
  display: flex;
  flex: 1;
  height: 100%;
  overflow: hidden;
}

.settings-layout {
  display: flex;
  width: 100%;
  height: 100%;
}
</style>