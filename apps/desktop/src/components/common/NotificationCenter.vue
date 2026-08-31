<template>
  <div class="notification-wrapper titlebar-no-drag">
  <a-popover
    v-model:open="popoverVisible"
    trigger="click"
    placement="bottomRight"
    overlay-class-name="notification-popover"
    :arrow="false"
  >
    <template #content>
      <div class="notification-panel">
        <!-- 头部 -->
        <div class="panel-header">
          <span class="panel-title">通知中心</span>
          <div class="panel-actions">
            <a-button type="link" size="small" @click="handleMarkAllRead" :disabled="store.unreadCount === 0">
              全部已读
            </a-button>
            <a-button type="link" size="small" @click="handleClearAll" :disabled="store.notifications.length === 0">
              清空
            </a-button>
          </div>
        </div>

        <!-- 通知列表 -->
        <div class="panel-body" v-if="store.notifications.length > 0">
          <div
            v-for="item in store.notifications"
            :key="item.id"
            class="notification-item"
            :class="{ unread: !item.read }"
            @click="store.markAsRead(item.id)"
          >
            <div class="item-icon" :class="'type-' + item.type">
              <CheckCircleOutlined v-if="item.type === 'success'" />
              <CloseCircleOutlined v-else-if="item.type === 'error'" />
              <ExclamationCircleOutlined v-else-if="item.type === 'warning'" />
              <InfoCircleOutlined v-else />
            </div>
            <div class="item-content">
              <div class="item-title">{{ item.title }}</div>
              <div v-if="item.content" class="item-desc">{{ item.content }}</div>
              <div class="item-time">{{ formatTime(item.time) }}</div>
            </div>
            <div v-if="!item.read" class="unread-dot"></div>
          </div>
        </div>

        <!-- 空状态 -->
        <div class="panel-empty" v-else>
          <BellOutlined />
          <span>暂无通知</span>
        </div>
      </div>
    </template>

    <!-- 铃铛图标 -->
    <a-button type="text" class="notification-trigger titlebar-no-drag">
      <template #icon>
        <a-badge :count="store.unreadCount" :offset="[-4, 4]" :overflow-count="99" class="bell-badge">
          <BellOutlined class="bell-icon" />
        </a-badge>
      </template>
    </a-button>
  </a-popover>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useNotificationStore } from '@/stores/notification'
import {
  BellOutlined, CheckCircleOutlined, CloseCircleOutlined,
  ExclamationCircleOutlined, InfoCircleOutlined
} from '@ant-design/icons-vue'

const store = useNotificationStore()
const popoverVisible = ref(false)

function handleMarkAllRead() {
  store.markAllAsRead()
}

function handleClearAll() {
  store.clearAll()
}

function formatTime(time) {
  const now = new Date()
  const diff = now - time
  const seconds = Math.floor(diff / 1000)
  if (seconds < 60) return '刚刚'
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes} 分钟前`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} 小时前`
  const month = time.getMonth() + 1
  const day = time.getDate()
  const h = String(time.getHours()).padStart(2, '0')
  const m = String(time.getMinutes()).padStart(2, '0')
  return `${month}/${day} ${h}:${m}`
}
</script>

<style scoped>
.notification-wrapper {
  -webkit-app-region: no-drag;
  display: flex;
  align-items: center;
}

.notification-trigger {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
  cursor: pointer;
  -webkit-app-region: no-drag;
}

.notification-trigger:hover {
  color: var(--color-primary);
  background: var(--color-hover);
}

.bell-icon {
  font-size: 16px;
  cursor: pointer;
}

.notification-trigger :deep(.ant-btn-icon) .bell-icon,
.notification-trigger .bell-icon {
  font-size: 16px !important;
}

.bell-badge {
  font-size: 16px;
  display: flex;
  /* AntD badge 外层自带 scale(1.143) zoom 动画基准，会把 16px 图标撑到 18px，与顶栏其它图标不齐 */
  transform: none !important;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  -webkit-app-region: no-drag;
}

/* 面板 */
.notification-panel {
  width: 340px;
  max-height: 400px;
  display: flex;
  flex-direction: column;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 0 10px;
  border-bottom: 1px solid var(--color-border);
}

.panel-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
}

.panel-actions {
  display: flex;
  gap: 4px;
}

.panel-body {
  overflow-y: auto;
  max-height: 340px;
  margin: 0 -12px;
  padding: 4px 0;
}

/* 通知项 */
.notification-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  cursor: pointer;
  transition: background var(--transition-fast);
  position: relative;
}

.notification-item:hover {
  background: var(--color-hover);
}

.notification-item.unread {
  background: var(--color-primary-bg);
}

.item-icon {
  font-size: 16px;
  flex-shrink: 0;
  margin-top: 2px;
}

.item-icon.type-success { color: var(--color-success, #52c41a); }
.item-icon.type-error { color: var(--color-error, #ff4d4f); }
.item-icon.type-warning { color: var(--color-warning, #faad14); }
.item-icon.type-info { color: var(--color-primary); }

.item-content {
  flex: 1;
  min-width: 0;
}

.item-title {
  font-size: 13px;
  color: var(--color-text);
  line-height: 1.4;
}

.item-desc {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 2px;
  line-height: 1.4;
}

.item-time {
  font-size: 11px;
  color: var(--color-text-muted);
  margin-top: 4px;
}

.unread-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--color-primary);
  flex-shrink: 0;
  margin-top: 6px;
}

/* 空状态 */
.panel-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 32px 0;
  color: var(--color-text-muted);
  font-size: 13px;
}

.panel-empty .anticon {
  font-size: 24px;
}
</style>

<style>
/* 全局样式：popover 渲染在 body 下，scoped 无法覆盖 */
.notification-popover .ant-popover-inner {
  padding: 12px;
}
</style>
