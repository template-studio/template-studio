import { RouteRecordRaw } from 'vue-router';
import { Layout, ParentLayout } from '@/router/constant';
import {
  GridOutline,
  DocumentTextOutline,
  LanguageOutline,
  OptionsOutline,
  ServerOutline,
  LayersOutline,
} from '@vicons/ionicons5';
import { renderIcon } from '@/utils/index';

const routeName = 'admin';

// 后台管理路由配置
const routes: Array<RouteRecordRaw> = [
  // /admin 重定向到首页
  {
    path: '/admin',
    redirect: '/admin/dashboard',
    meta: {
      hidden: true, // 隐藏菜单
    },
  },

  // 首页
  {
    path: '/admin/dashboard',
    name: 'admin-dashboard',
    component: Layout,
    meta: {
      title: '首页',
      icon: renderIcon(GridOutline),
      affix: true,
      sort: 0,
    },
    children: [
      {
        path: '',
        name: 'admin-dashboard-index',
        meta: {
          title: '首页',
          affix: true,
        },
        component: () => import('@/views/admin/dashboard/index.vue'),
      },
    ],
  },

  // 模板管理
  {
    path: '/admin/templates',
    name: 'admin-templates-list',
    component: Layout,
    meta: {
      title: '模板管理',
      icon: renderIcon(DocumentTextOutline),
      sort: 1,
    },
    children: [
      {
        path: '',
        name: 'admin-templates-index',
        meta: {
          title: '模板管理',
        },
        component: () => import('@/views/admin/template/index.vue'),
      },
    ],
  },

  // 基础数据（带子菜单）
  {
    path: '/admin/basic-data',
    name: 'admin-basic-data',
    component: ParentLayout,
    redirect: '/admin/basic-data/categories',
    meta: {
      title: '基础数据',
      icon: renderIcon(ServerOutline),
      sort: 2,
    },
    children: [
      {
        path: 'categories',
        name: 'admin-categories',
        component: Layout,
        meta: {
          title: '分类管理',
          icon: renderIcon(LayersOutline),
        },
        children: [
          {
            path: '',
            name: 'admin-categories-index',
            meta: {
              title: '分类管理',
            },
            component: () => import('@/views/admin/category/index.vue'),
          },
        ],
      },
      {
        path: 'languages',
        name: 'admin-languages',
        component: Layout,
        meta: {
          title: '语言管理',
          icon: renderIcon(LanguageOutline),
        },
        children: [
          {
            path: '',
            name: 'admin-languages-index',
            meta: {
              title: '语言管理',
            },
            component: () => import('@/views/admin/language/index.vue'),
          },
        ],
      },
      {
        path: 'var-presets',
        name: 'admin-var-presets',
        component: Layout,
        meta: {
          title: '变量预设',
          icon: renderIcon(OptionsOutline),
        },
        children: [
          {
            path: '',
            name: 'admin-var-presets-index',
            meta: {
              title: '变量预设',
            },
            component: () => import('@/views/admin/var-preset/index.vue'),
          },
        ],
      },
    ],
  },

  // 变量预设设计页面（全屏编辑器，无布局）
  {
    path: '/admin/var-presets/:id/design',
    name: 'var-preset-design',
    component: () => import('@/views/admin/var-preset/design.vue'),
    meta: {
      title: '变量预设设计',
      hidden: true, // 隐藏菜单
    },
  },
];

export default routes;
