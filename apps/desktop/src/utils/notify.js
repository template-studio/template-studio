import { message } from 'ant-design-vue'
import { useNotificationStore } from '@/stores/notification'

/**
 * 统一通知：显示 toast + 记录到通知中心
 * @param {'success'|'error'|'warning'|'info'} type
 * @param {string} title - toast 和通知标题
 * @param {string} [content] - 通知详情（可选）
 */
export function notify({ type = 'info', title, content = '' }) {
  message[type](title)
  const store = useNotificationStore()
  store.addNotification({ type, title, content })
}
