<template>
  <!-- 登录态身份锚点：配置 API Token 后显示（头像 + 用户名 + 连接点）；
       Token 失效转警示态；点击跳转凭据设置页 -->
  <a-tooltip v-if="collapsed" :title="tooltipText" placement="right">
    <button class="user-chip collapsed-item" :class="{ error: isError }" @click="goSettings">
      <img v-if="avatarUrl" :src="avatarUrl" class="avatar" alt="" @error="avatarUrl = ''" />
      <span v-else class="avatar avatar-fallback">{{ initial }}</span>
    </button>
  </a-tooltip>
  <button v-else class="user-chip" :class="{ error: isError }" @click="goSettings">
    <img v-if="avatarUrl" :src="avatarUrl" class="avatar" alt="" @error="avatarUrl = ''" />
    <span v-else class="avatar avatar-fallback">{{ initial }}</span>
    <span class="name">{{ displayName }}</span>
    <span class="dot" :class="isError ? 'dot-error' : 'dot-ok'" :title="isError ? 'Token 无效' : '已连接'"></span>
  </button>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import apiRequest from '@/utils/apiRequest'
import { useConfigStore } from '@/stores/config'
import { useLayoutStore } from '@/stores/layout'

const router = useRouter()
const configStore = useConfigStore()
const layoutStore = useLayoutStore()

const username = ref('')
const avatarUrl = ref('')
const isError = ref(false)

const collapsed = computed(() => layoutStore.sidebarCollapsed)

const displayName = computed(() => {
  if (isError.value) return 'Token 无效'
  return username.value || '已连接'
})

const tooltipText = computed(() => (isError.value ? 'Token 无效，点击配置' : displayName.value))

const initial = computed(() => (username.value ? username.value[0].toUpperCase() : '·'))

const goSettings = () => {
  router.push('/settings/web-server')
}

onMounted(async () => {
  if (!configStore.hasApiKey) return
  try {
    const res = await apiRequest.get('/api/v1/admin/auth/info')
    const user = res?.data?.data
    if (!user?.username) throw new Error('empty user')
    username.value = user.username
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

.collapsed-item {
  justify-content: center;
  padding: 0;
}
</style>
