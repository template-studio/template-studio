import { RouteRecordRaw } from 'vue-router';
import { ClientLayout } from '@/router/constant';
import { HomeOutline, GridOutline } from '@vicons/ionicons5';
import { renderIcon } from '@/utils/index';

const routeName = 'client';

const routes: Array<RouteRecordRaw> = [
  // 前台主路由
  {
    path: '/',
    component: ClientLayout,
    meta: {
      title: '前台',
      sort: -1, // 设置最高优先级，确保排在其他路由之前
      ignoreAuth: true, // 前台页面无需权限验证
    },
    children: [
      {
        path: '',
        name: 'home',
        meta: {
          title: '首页',
          affix: true,
        },
        component: () => import('@/views/client/home/index.vue'),
      },
      {
        path: 'templates',
        name: 'templates',
        meta: {
          title: '模板广场',
        },
        component: () => import('@/views/client/templates-public/index.vue'),
      },
      {
        path: 'profile',
        name: 'profile',
        meta: {
          title: '个人中心',
        },
        component: () => import('@/views/client/profile/index.vue'),
      },
      {
        path: 'my-templates',
        name: 'my-templates',
        meta: {
          title: '我的模板',
        },
        component: () => import('@/views/client/my-templates/index.vue'),
      },
    ],
  },

  // 模板生成器 - 独立全屏路由
  {
    path: '/template-generator',
    name: 'template-generator',
    meta: {
      title: '模板生成器',
      ignoreAuth: true,
    },
    component: () => import('@/views/client/template-generator/index.vue'),
  },
  {
    path: '/template-generator/:id',
    name: 'template-generator-detail',
    meta: {
      title: '模板生成器',
      ignoreAuth: true,
      hidden: true,
    },
    component: () => import('@/views/client/template-generator/index.vue'),
  },
];

export default routes;
