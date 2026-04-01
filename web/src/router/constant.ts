export const RedirectName = 'Redirect';

export const ErrorPage = () => import('@/views/exception/404.vue');

// 后台管理布局（侧边栏 + 顶部栏 + 内容）
export const Layout = () => import('@/layout/index.vue');

// 前台展示布局（导航栏 + 内容 + 页脚）
export const ClientLayout = () => import('@/layout/client/index.vue');

export const ParentLayout = () => import('@/layout/parentLayout.vue');
