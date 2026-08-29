import { RouteRecordRaw } from 'vue-router';
import { Layout } from '@/router/constant';
import { ExclamationCircleOutlined } from '@ant-design/icons-vue';
import { renderIcon } from '@/utils/index';

const routes: Array<RouteRecordRaw> = [
  {
    path: '/exception',
    name: 'Exception',
    redirect: '/exception/403',
    component: Layout,
    meta: {
      title: '异常页面',
      icon: renderIcon(ExclamationCircleOutlined),
      sort: 3,
      hidden: true, // 隐藏菜单项，不显示在导航中
    },
    children: [
      {
        path: '403',
        name: 'exception-403',
        meta: {
          title: '403',
          hidden: true, // 隐藏菜单项
        },
        component: () => import('@/views/exception/403.vue'),
      },
      {
        path: '404',
        name: 'exception-404',
        meta: {
          title: '404',
          hidden: true, // 隐藏菜单项
        },
        component: () => import('@/views/exception/404.vue'),
      },
      {
        path: '500',
        name: 'exception-500',
        meta: {
          title: '500',
          hidden: true, // 隐藏菜单项
        },
        component: () => import('@/views/exception/500.vue'),
      },
    ],
  },
];

export default routes;
