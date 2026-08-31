<template>
  <!-- 登录态身份锚点：配置 API Token 后显示；点击弹出用户信息与凭据管理 -->
  <a-popover v-model:open="menuOpen" trigger="click" placement="rightBottom" :overlay-class-name="'user-card-popover'">
    <button class="user-chip" :class="{ error: isError, collapsed: collapsed }">
      <img
        v-if="avatarUrl"
        :src="avatarUrl"
        class="avatar"
        alt=""
        @error="avatarUrl = ''"
        @load="onAvatarLoad"
      />
      <span v-else class="avatar avatar-fallback">{{ initial }}</span>
      <template v-if="!collapsed">
        <span class="name">{{ displayName }}</span>
        <span class="dot" :class="isError ? 'dot-error' : 'dot-ok'" :title="isError ? 'Token 无效' : '已连接'"></span>
      </template>
    </button>

    <template #content>
      <div class="user-menu">
        <div class="user-menu-head">
          <img v-if="avatarUrl" :src="avatarUrl" class="menu-avatar" alt="" />
          <span v-else class="menu-avatar avatar-fallback">{{ initial }}</span>
          <div class="menu-head-info">
            <div class="menu-name">{{ username || '未连接' }}</div>
            <div v-if="email" class="menu-email">{{ email }}</div>
          </div>
          <span class="dot dot-ok menu-dot" v-if="!isError" title="已连接"></span>
        </div>
        <div v-if="roles.length" class="menu-roles">
          <span v-for="r in roles.slice(0, 3)" :key="r" class="role-tag">{{ r }}</span>
        </div>
        <div class="menu-divider"></div>
        <button class="menu-item" @click="goTokenSettings">
          <KeyOutlined class="menu-item-ic" />
          <span>API Token 管理</span>
        </button>
        <button v-if="isError" class="menu-item menu-item-warn" @click="goTokenSettings">
          <WarningOutlined class="menu-item-ic" />
          <span>Token 已失效，点击重新配置</span>
        </button>
      </div>
    </template>
  </a-popover>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { KeyOutlined, WarningOutlined } from '@ant-design/icons-vue'
import apiRequest from '@/utils/apiRequest'
import { useConfigStore } from '@/stores/config'
import { useLayoutStore } from '@/stores/layout'

const router = useRouter()
const configStore = useConfigStore()
const layoutStore = useLayoutStore()

const username = ref('')
const email = ref('')
const roles = ref([])
const avatarUrl = ref('')
const isError = ref(false)
const menuOpen = ref(false)

const collapsed = computed(() => layoutStore.sidebarCollapsed)

const displayName = computed(() => {
  if (isError.value) return 'Token 无效'
  return username.value || '已连接'
})

const initial = computed(() => (username.value ? username.value[0].toUpperCase() : '·'))

const goTokenSettings = () => {
  menuOpen.value = false
  router.push('/settings/web-server')
}

// 防御无效头像：加载"成功"但自然尺寸过小（如 1×1 占位图）视为无效，回退首字母
const onAvatarLoad = (e) => {
  if (e.target.naturalWidth <= 2 || e.target.naturalHeight <= 2) {
    avatarUrl.value = ''
  }
}

onMounted(async () => {
  if (!configStore.hasApiKey) return
  try {
    const res = await apiRequest.get('/api/v1/admin/auth/info')
    const user = res?.data?.data
    if (!user?.username) throw new Error('empty user')
    username.value = user.username
    email.value = user.email || ''
    roles.value = (user.roles || []).map(r => (typeof r === 'string' ? r : r?.name || r?.roleName || '')).filter(Boolean)
    if (user.avatar) {
      avatarUrl.value = `${configStore.baseURL.replace(/\/$/, '')}${user.avatar}`
    }
  } catch {
    // Token 无效/服务不可达：显示警示态引导去设置页
    isError.value = true
  }
})
</script>

<style scoped>
.user-chip {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  height: 32px;
  margin-top: 4px;
  padding: 0 8px;
  border: none;
  border-radius: 7px;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 13px;
  text-align: left;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease;
}

.user-chip:hover {
  background: var(--color-nav-hover);
  color: var(--color-text);
}

.avatar {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  object-fit: cover;
  flex: none;
}

.avatar-fallback {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--color-active);
  color: var(--color-text);
  font-size: 11px;
  font-weight: 600;
}

.name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  position: relative;
  top: 0.5px;
}

.user-chip.error .name {
  color: var(--color-warning);
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex: none;
}

.dot-ok {
  background: var(--color-brand);
}

.dot-error {
  background: var(--color-warning);
}

.user-chip.collapsed {
  justify-content: center;
  padding: 0;
}

/* ---------- 弹层内容 ---------- */
.user-menu {
  min-width: 220px;
  margin: -4px -8px;
}

.user-menu-head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
}

.menu-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  object-fit: cover;
  flex: none;
}

.menu-head-info {
  flex: 1;
  min-width: 0;
}

.menu-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.menu-email {
  font-size: 11.5px;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.menu-dot {
  width: 7px;
  height: 7px;
}

.menu-roles {
  display: flex;
  gap: 4px;
  padding: 0 12px 10px;
  flex-wrap: wrap;
}

.role-tag {
  font-size: 10.5px;
  padding: 1px 7px;
  border-radius: 4px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  color: var(--color-text-secondary);
}

.menu-divider {
  height: 1px;
  background: var(--color-border-light);
  margin: 2px 0;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  height: 30px;
  padding: 0 12px;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 12.5px;
  text-align: left;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease;
}

.menu-item:hover {
  background: var(--color-nav-hover);
  color: var(--color-text);
}

.menu-item-ic {
  font-size: 13px;
  color: var(--color-text-muted);
}

.menu-item:hover .menu-item-ic {
  color: var(--color-text);
}

.menu-item-warn {
  color: var(--color-warning);
}
</style>
