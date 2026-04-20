import { App } from 'vue';
import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';
import { RedirectRoute } from '@/router/base';
import { PageEnum } from '@/enums/pageEnum';
import { createRouterGuards } from './guards';
import type { IModuleType } from './types';

const modules = import.meta.glob<IModuleType>('./modules/**/*.ts', { eager: true });

const routeModuleList: RouteRecordRaw[] = Object.keys(modules).reduce((list, key) => {
  const mod = modules[key].default ?? {};
  const modList = Array.isArray(mod) ? [...mod] : [mod];
  return [...list, ...modList];
}, []);

function sortRoute(a, b) {
  return (a.meta?.sort ?? 0) - (b.meta?.sort ?? 0);
}

routeModuleList.sort(sortRoute);

export const LoginRoute: RouteRecordRaw = {
  path: '/login',
  name: 'Login',
  component: () => import('@/views/login/index.vue'),
  meta: {
    title: '登录',
  },
};

export const ResetPasswordRoute: RouteRecordRaw = {
  path: '/reset-password',
  name: 'ResetPassword',
  component: () => import('@/views/client/reset-password/index.vue'),
  meta: {
    title: '重置密码',
    ignoreAuth: true,
  },
};

// 分离需要权限的路由和公开路由
export const publicRoutes: RouteRecordRaw[] = routeModuleList.filter(
  (route) => route.meta?.ignoreAuth === true
);

export const asyncRoutes = routeModuleList.filter((route) => route.meta?.ignoreAuth !== true);

//普通路由 无需验证权限（包含公开路由）
export const constantRouter: RouteRecordRaw[] = [
  LoginRoute,
  ResetPasswordRoute,
  ...publicRoutes, // 公开路由直接添加，无需动态加载
  RedirectRoute,
];

const router = createRouter({
  history: createWebHistory(),
  routes: constantRouter,
  strict: true,
  scrollBehavior: () => ({ left: 0, top: 0 }),
});

export function setupRouter(app: App) {
  app.use(router);
  // 创建路由守卫
  createRouterGuards(router);
}

export default router;
