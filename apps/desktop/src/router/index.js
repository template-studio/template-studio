import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '@/views/home/index.vue'
import { useLayoutStore } from '@/stores/layout'

const routes = [
  {
    path: '/',
    redirect: '/home'
  },
  {
    path: '/home',
    name: 'Home',
    component: HomeView
  },
  {
    path: '/templates',
    name: 'Templates',
    component: () => import('@/views/templates/index.vue')
  },
  {
    path: '/datasource',
    name: 'DataSource',
    component: () => import('@/views/datasource/index.vue')
  },
  {
    path: '/datasource/:id/browse',
    name: 'DatabaseBrowser',
    component: () => import('@/views/database-browser/index.vue')
  },
  {
    path: '/projects',
    name: 'Projects',
    component: () => import('@/views/projects/index.vue')
  },
  {
    path: '/mappings',
    name: 'Mappings',
    component: () => import('@/views/mappings/index.vue')
  },
  {
    path: '/languages',
    name: 'Languages',
    component: () => import('@/views/languages/index.vue')
  },
  {
    path: '/project/:id',
    name: 'ProjectWorkspace',
    component: () => import('@/views/project/index.vue')
  },
  {
    path: '/project/:id/tables',
    name: 'ProjectTables',
    component: () => import('@/views/project/tables/index.vue')
  },
  {
    path: '/project/:id/preferences',
    name: 'ProjectPreferences',
    component: () => import('@/views/project/preferences/index.vue')
  },
  {
    path: '/project/:id/mappings',
    name: 'ProjectMappings',
    component: () => import('@/views/project/mappings/index.vue')
  },
  {
    path: '/template-render',
    name: 'TemplateRender',
    component: () => import('@/views/template-render/index.vue')
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/views/settings/index.vue')
  },
  // 添加所有深层链接的通配符路由
  {
    path: '/settings/:mainTab?',
    name: 'SettingsWithMainTab',
    component: () => import('@/views/settings/index.vue')
  },
  {
    path: '/settings/:mainTab/:subTab?',
    name: 'SettingsWithSubTab',
    component: () => import('@/views/settings/index.vue')
  },
  {
    path: '/settings/:mainTab/:subTab/:thirdTab?',
    name: 'SettingsWithThirdTab',
    component: () => import('@/views/settings/index.vue')
  },
  {
    path: '/help',
    name: 'Help',
    component: () => import('@/views/HelpView.vue')
  },
  // 404 通配符路由（必须放在最后）
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    component: () => import('@/views/NotFoundView.vue')
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

// 路由守卫：控制 footer 显示/隐藏
router.beforeEach((to) => {
  const layoutStore = useLayoutStore()
  layoutStore.onRouteChange(to.path)
})

export default router