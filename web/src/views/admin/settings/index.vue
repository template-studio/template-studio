<template>
  <div class="settings-page">
    <n-card title="系统设置" :bordered="false">
      <div class="settings-layout">
        <div class="settings-sidebar">
          <div
            v-for="tab in visibleTabs"
            :key="tab.key"
            :class="['sidebar-item', { active: activeTab === tab.key }]"
            @click="handleTabClick(tab.key)"
          >
            <n-icon size="18"><component :is="tab.icon" /></n-icon>
            <span>{{ tab.label }}</span>
          </div>
        </div>
        <div class="settings-content">
          <FooterSettings v-if="activeTab === 'footer'" />
          <UserManagement v-if="activeTab === 'users'" />
          <RoleManagement v-if="activeTab === 'roles'" />
        </div>
      </div>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { BookmarkOutline, PeopleOutline, ShieldOutline } from '@vicons/ionicons5';
import FooterSettings from './footer.vue';
import UserManagement from './users.vue';
import RoleManagement from './roles.vue';
import { useUser } from '@/store/modules/user';

const activeTab = ref('footer');
const userStore = useUser();

const allTabs = [
  { key: 'footer', label: 'Footer 设置', icon: BookmarkOutline, permission: 'settings' },
  { key: 'users', label: '用户管理', icon: PeopleOutline, permission: 'user_management' },
  { key: 'roles', label: '角色管理', icon: ShieldOutline, permission: 'role_management' },
];

const visibleTabs = computed(() => {
  const permissions = userStore.getPermissions?.map((p: any) => p.value) || [];
  return allTabs.filter((tab) => permissions.includes(tab.permission));
});

function handleTabClick(key: string) {
  activeTab.value = key;
}
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
