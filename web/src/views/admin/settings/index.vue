<template>
  <div class="settings-page">
    <n-card title="系统设置" :bordered="false">
      <div class="settings-layout">
        <div class="settings-sidebar">
          <div
            v-for="tab in tabs"
            :key="tab.key"
            :class="['sidebar-item', { active: activeTab === tab.key }]"
            @click="activeTab = tab.key"
          >
            <n-icon size="18"><component :is="tab.icon" /></n-icon>
            <span>{{ tab.label }}</span>
          </div>
        </div>
        <div class="settings-content">
          <FooterSettings v-if="activeTab === 'footer'" />
        </div>
      </div>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { BookmarkOutline } from '@vicons/ionicons5';
import FooterSettings from './footer.vue';

const activeTab = ref('footer');

const tabs = [
  { key: 'footer', label: 'Footer 设置', icon: BookmarkOutline },
];
</script>

<style scoped>
.settings-page {
  padding: 16px;
}

.settings-layout {
  display: flex;
  gap: 24px;
  min-height: 400px;
}

.settings-sidebar {
  width: 180px;
  flex-shrink: 0;
  border-right: 1px solid #efeff5;
  padding-right: 16px;
}

.sidebar-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 4px;
  cursor: pointer;
  color: #666;
  font-size: 14px;
  transition: all 0.2s;
  margin-bottom: 4px;
}

.sidebar-item:hover {
  background: #f5f5f5;
  color: #333;
}

.sidebar-item.active {
  background: #e8f5e9;
  color: #18a058;
  font-weight: 500;
}

.settings-content {
  flex: 1;
  min-width: 0;
}
</style>
