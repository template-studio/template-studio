import { message, Modal, notification } from 'ant-design-vue';

/**
 * 挂载 Ant Design Vue 脱离上下文的 API
 * Ant Design Vue 的 message、Modal、notification 支持静态方法调用，无需 Provider 包裹
 * 为保持与原 Naive UI 代码的兼容性，将其挂载到 window 上
 */
export function setupAntdDiscreteApi() {
  window['$message'] = message;
  window['$dialog'] = Modal;
  window['$notification'] = notification;
  // $loading 用 message.loading 替代，或使用 NProgress
  window['$loading'] = {
    start: () => {},
    finish: () => {},
    error: () => {},
  };
}
