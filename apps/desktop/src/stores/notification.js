import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useNotificationStore = defineStore('notification', () => {
  const notifications = ref([])
  let nextId = 1

  const MAX_NOTIFICATIONS = 100

  const unreadCount = computed(() => notifications.value.filter(n => !n.read).length)

  function addNotification({ type = 'info', title, content = '' }) {
    notifications.value.unshift({
      id: nextId++,
      type,
      title,
      content,
      time: new Date(),
      read: false
    })
    // 超出上限移除最旧的
    if (notifications.value.length > MAX_NOTIFICATIONS) {
      notifications.value = notifications.value.slice(0, MAX_NOTIFICATIONS)
    }
  }

  function markAsRead(id) {
    const n = notifications.value.find(item => item.id === id)
    if (n) n.read = true
  }

  function markAllAsRead() {
    notifications.value.forEach(n => { n.read = true })
  }

  function clearAll() {
    notifications.value = []
  }

  return {
    notifications,
    unreadCount,
    addNotification,
    markAsRead,
    markAllAsRead,
    clearAll
  }
})
