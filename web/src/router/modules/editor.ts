import { RouteRecordRaw } from 'vue-router';
import { CreateOutline } from '@vicons/ionicons5';
import { renderIcon } from '@/utils/index';

// 编辑器路由（独立页面，不使用布局）
const routes: Array<RouteRecordRaw> = [
  // 前台用户编辑路由（无需权限）
  {
    path: '/editor/:id',
    name: 'template-editor',
    meta: {
      title: '模板编辑器',
      hidden: true, // 不在菜单中显示
      ignoreAuth: true, // 前台编辑器无需权限验证
    },
    component: () => import('@/views/editor/index.vue'),
    props: { mode: 'user' },
  },
  // 后台管理员编辑路由（需要权限）
  {
    path: '/admin/editor/:id',
    name: 'admin-template-editor',
    meta: {
      title: '模板管理编辑',
      hidden: true,
      // 后台编辑器需要权限验证
    },
    component: () => import('@/views/editor/index.vue'),
    props: { mode: 'admin' },
  },
];

export default routes;
